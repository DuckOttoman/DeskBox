use crate::{app::AppState,model::*,quickbar,scanner,taskbar,win};
use std::{collections::{BTreeMap,BTreeSet},sync::{Arc,atomic::{AtomicBool,Ordering}},time::{Duration,Instant}};
use tauri::{Emitter,Manager,PhysicalPosition,WebviewUrl,WebviewWindow,WebviewWindowBuilder};
#[derive(Default)]
pub struct Runtime {pub controller:Option<taskbar::Controller>,pub docks:Vec<Monitor>,pub ready:BTreeSet<String>,pub dom_verified:BTreeSet<String>,pub undo:BTreeMap<String,quickbar::Removed>,pub last_tick:Option<Instant>,pub last_topology:Option<Instant>}
pub static CHANGING:AtomicBool=AtomicBool::new(false);
struct ChangeGuard;impl Drop for ChangeGuard{fn drop(&mut self){CHANGING.store(false,Ordering::Release);}}
pub async fn set_enabled(app:&tauri::AppHandle,data:&Arc<AppState>,enabled:bool)->Result<(),String>{
 if CHANGING.swap(true,Ordering::AcqRel){return Err("快捷栏正在切换，请稍后重试".into());}let _changing=ChangeGuard;
 let current=data.quickbar.lock().map_err(|_|"Quickbar lock")?.controller.is_some();if enabled==current{return Ok(());}
 if !enabled{
  let labels=data.quickbar.lock().map_err(|_|"Quickbar lock")?.docks.iter().map(|m|m.label.clone()).collect::<Vec<_>>();
  for label in labels{let _=app.emit_to(&label,"quickbar-hide",());}
  let ms=(250./data.snapshot()?.config.settings.anim_speed.max(0.25)) as u64;
  tauri::async_runtime::spawn_blocking(move||std::thread::sleep(Duration::from_millis(ms))).await.map_err(|e|e.to_string())?;
  return disable(app,data);
 }
 if data.smoke&&!std::env::args().any(|a|a=="--quickbar-native-check"){return Err("隔离测试不接管系统任务栏".into());}
 let mut docks=taskbar::taskbar_monitors(&data.snapshot()?.monitors);
 if docks.is_empty(){return Err("未找到可接管的 Windows 任务栏".into());}
 for (i,m) in docks.iter_mut().enumerate(){m.label=format!("quickbar-{i}");}
 {let mut r=data.quickbar.lock().map_err(|_|"Quickbar lock")?;r.docks=docks.clone();r.ready.clear();r.dom_verified.clear();}
 let result=async{
  let mut handles=vec![];
  for m in &docks{
   let window=WebviewWindowBuilder::new(app,&m.label,WebviewUrl::App("quickbar.html".into())).title("DeskBox 快捷访问栏")
    .inner_size(m.w as f64/m.scale,220.).decorations(false).transparent(true).shadow(false).always_on_top(true).skip_taskbar(true).resizable(false)
    .focused(false).visible(false).data_directory(data.directory.join("webview")).build().map_err(|e|e.to_string())?;
   window.set_position(PhysicalPosition::new(m.x,m.y+m.h as i32-(220.*m.scale).round() as i32)).map_err(|e|e.to_string())?;
   let hwnd=window.hwnd().map_err(|e|e.to_string())?.0 as isize;win::focusable(hwnd,false);handles.push((hwnd,m.clone()));
  }
  let copy=data.clone();tauri::async_runtime::spawn_blocking(move||{let deadline=Instant::now()+Duration::from_secs(20);while Instant::now()<deadline{if copy.quickbar.lock().map(|r|r.docks.iter().all(|m|r.ready.contains(&m.label))).unwrap_or(false){return Ok(());}std::thread::sleep(Duration::from_millis(40));}Err::<(),String>("快捷访问栏未就绪，已保留系统任务栏".into())}).await.map_err(|e|e.to_string())??;
  let dir=data.directory.clone();let controller=tauri::async_runtime::spawn_blocking(move||taskbar::Controller::enable(&dir,&handles)).await.map_err(|e|e.to_string())??;
  data.quickbar.lock().map_err(|_|"Quickbar lock")?.controller=Some(controller);
  // Shell shrinks rcWork and moves ordinary popup windows out of its new band.
  // The dock belongs in that band; anchor after reservation and after showing it.
  for m in &docks{let w=app.get_webview_window(&m.label).ok_or("快捷栏窗口已关闭")?;anchor(&w,m)?;win::show_window_no_activate(w.hwnd().map_err(|e|e.to_string())?.0 as isize);anchor(&w,m)?;}
  refresh_work(data)?;Ok::<(),String>(())
 }.await;
 if result.is_err(){let _=disable(app,data);}result
}
fn anchor(window:&WebviewWindow,monitor:&Monitor)->Result<(),String>{
 let size=window.outer_size().map_err(|e|e.to_string())?;
 let target=PhysicalPosition::new(monitor.x,monitor.y+monitor.h as i32-size.height as i32);
 if window.outer_position().map_err(|e|e.to_string())?!=target{window.set_position(target).map_err(|e|e.to_string())?;}
 Ok(())
}
pub fn refresh_work(data:&AppState)->Result<(),String>{
 let mut s=data.snapshot.lock().map_err(|_|"State lock")?;
 for m in &mut s.monitors{let r=win::work_area(m.x+m.w as i32/2,m.y+m.h as i32/2);m.work=[(r.left-m.x) as f64/m.scale,(r.top-m.y) as f64/m.scale,(r.right-r.left) as f64/m.scale,(r.bottom-r.top) as f64/m.scale];}
 Ok(())
}
pub fn disable(app:&tauri::AppHandle,data:&AppState)->Result<(),String>{
 let mut r=data.quickbar.lock().map_err(|_|"Quickbar lock")?;
 if let Some(mut c)=r.controller.take(){c.disable();}
 for m in &r.docks{if let Some(w)=app.get_webview_window(&m.label){let _=w.close();}}r.docks.clear();r.ready.clear();drop(r);refresh_work(data)
}
pub fn tick(app:&tauri::AppHandle,data:&Arc<AppState>){
 if CHANGING.load(Ordering::Acquire){return;}
 let topology={let Ok(mut r)=data.quickbar.lock()else{return;};if r.controller.is_none()||r.last_tick.is_some_and(|t|t.elapsed()<Duration::from_millis(300)){return;}r.last_tick=Some(Instant::now());let due=r.last_topology.is_none_or(|t|t.elapsed()>Duration::from_secs(2));if due{r.last_topology=Some(Instant::now());}due};
 if topology&&topology_tick(app,data){return;}
 let result=(||{
  let (states,docks)={let mut r=data.quickbar.lock().map_err(|_|"Quickbar lock")?;let states=match r.controller.as_mut(){Some(c)=>c.tick()?,None=>vec![]};(states,r.docks.clone())};
  // Do not hold Runtime's lock while dispatching window operations to the UI.
  for(label,visible)in states{if let Some(w)=app.get_webview_window(&label){let hwnd=w.hwnd().map_err(|e|e.to_string())?.0 as isize;if visible{if let Some(m)=docks.iter().find(|m|m.label==label){anchor(&w,m)?;}win::show_window_no_activate(hwnd);}else{win::hide_window(hwnd);}}}
  Ok::<(),String>(())})();if let Err(e)=result{if topology_tick(app,data){return;}data.log(format!("Quickbar: {e}"));let _=disable(app,data);let _=data.mutate(|c|{c.settings.hide_taskbar=false;Ok(())});let _=data.publish(app);}
}
/// Recreate reservations and dock windows after display/DPI/taskbar topology changes.
pub fn topology_tick(app:&tauri::AppHandle,data:&Arc<AppState>)->bool{
 if CHANGING.load(Ordering::Acquire){return false;}
 let active=data.quickbar.lock().map(|r|r.controller.is_some()).unwrap_or(false);if !active{return false;}
 let Ok(monitors)=crate::app::monitors(app)else{return false;};
 let same=|a:&Monitor,b:&Monitor|a.name==b.name&&a.x==b.x&&a.y==b.y&&a.w==b.w&&a.h==b.h&&(a.scale-b.scale).abs()<0.001;
 let targets=taskbar::taskbar_monitors(&monitors);
 // Explorer recreates all taskbar HWNDs briefly. Let Controller hide/wait;
 // treating this interval as a removed display would disable the feature.
 if targets.is_empty(){return false;}
 let changed=data.quickbar.lock().map(|r|r.docks.len()!=targets.len()||r.docks.iter().any(|a|!targets.iter().any(|b|same(a,b)))).unwrap_or(false);
 if changed{let a=app.clone();let d=data.clone();tauri::async_runtime::spawn(async move{
  if let Err(e)=set_enabled(&a,&d,false).await{d.log(e);return;}
  if let Err(e)=crate::app::sync_monitors(&a,&d){d.log(e);return;}
  if let Err(e)=set_enabled(&a,&d,true).await{d.log(e);let _=d.mutate(|c|{c.settings.hide_taskbar=false;Ok(())});}
  else{let _=d.mutate(|_|Ok(()));}let _=d.publish(&a);
 });}
 changed
}
fn own_monitor(data:&AppState,label:&str)->Result<Monitor,String>{data.quickbar.lock().map_err(|_|"Quickbar lock")?.docks.iter().find(|m|m.label==label).cloned().ok_or("快捷栏显示器不可用".into())}
#[derive(serde::Serialize)]#[serde(rename_all="camelCase")]
pub struct Entry{path:String,name:String,icon:Option<String>,missing:bool,pinned:bool}
#[tauri::command]
pub async fn quickbar_state(window:WebviewWindow,data:tauri::State<'_,Arc<AppState>>)->Result<serde_json::Value,String>{
 let monitor=own_monitor(&data,window.label())?;let s=data.snapshot()?;let dir=data.directory.clone();let diagnostic=data.smoke;
 tauri::async_runtime::spawn_blocking(move||{
  let mut entries=vec![];for path in quickbar::ranked(&s.config.quickbar,quickbar::now()){
   let p=std::path::Path::new(&path);let missing=!p.exists();let pinned=s.config.quickbar.pins.iter().any(|x|quickbar::key(x)==quickbar::key(&path));if missing&&!pinned{continue;}
   let name=p.file_stem().unwrap_or_default().to_string_lossy().into_owned();let item=Item{learning_key:None,id:scanner::stable_id(&path),name:name.clone(),path:path.clone(),kind:if p.is_dir(){"directory"}else{"file"}.into(),ext:String::new(),category:String::new(),icon:None};
   let icon=if missing{None}else{scanner::cached_icon(&item,&dir.join("icons-v2"))};entries.push(Entry{path,name,icon,missing,pinned});
  }serde_json::json!({"config":s.config,"monitor":monitor,"entries":entries,"diagnostic":diagnostic})
 }).await.map_err(|e|e.to_string())
}
#[tauri::command]
pub fn quickbar_ready(window:WebviewWindow,data:tauri::State<'_,Arc<AppState>>)->Result<(),String>{own_monitor(&data,window.label())?;data.quickbar.lock().map_err(|_|"Quickbar lock")?.ready.insert(window.label().into());if data.smoke{window.eval(include_str!("../tests/quickbar-native-dom.js")).map_err(|e|e.to_string())?;}Ok(())}
#[tauri::command]
pub fn quickbar_dom_report(window:WebviewWindow,data:tauri::State<'_,Arc<AppState>>,ok:bool,message:String)->Result<(),String>{if !data.smoke{return Err("Diagnostic mode required".into());}if !ok{data.log(format!("FRONTEND_ERROR quickbar DOM {message}"));return Err(message);}data.log(message);data.quickbar.lock().map_err(|_|"Quickbar lock")?.dom_verified.insert(window.label().into());Ok(())}
#[tauri::command]
pub fn quickbar_test_status(data:tauri::State<'_,Arc<AppState>>)->Result<bool,String>{if !data.smoke{return Err("Diagnostic mode required".into());}let r=data.quickbar.lock().map_err(|_|"Quickbar lock")?;Ok(!r.docks.is_empty()&&r.docks.iter().all(|m|r.dom_verified.contains(&m.label)))}
#[derive(serde::Deserialize)]pub struct Region{x:f64,y:f64,width:f64,height:f64,#[serde(default)]radius:Option<f64>}
#[tauri::command]
pub fn quickbar_regions(window:WebviewWindow,regions:Vec<Region>)->Result<(),String>{
 if !window.label().starts_with("quickbar-"){return Err("Invalid quickbar window".into());}let scale=window.scale_factor().map_err(|e|e.to_string())?;
 let rects=regions.into_iter().filter(|r|[r.x,r.y,r.width,r.height].iter().all(|v|v.is_finite())).map(|r|HitRect{x:(r.x*scale).floor() as i32,y:(r.y*scale).floor() as i32,w:(r.width*scale).ceil() as i32,h:(r.height*scale).ceil() as i32,r:(r.radius.unwrap_or(14.).clamp(0.,100.)*scale) as i32,ellipse:false}).collect();win::set_region(window.hwnd().map_err(|e|e.to_string())?.0 as isize,Some(rects))
}
#[tauri::command]
pub fn quickbar_pin(app:tauri::AppHandle,data:tauri::State<'_,Arc<AppState>>,paths:Vec<String>)->Result<(),String>{
 if paths.is_empty()||paths.len()>256{return Err("一次最多固定 256 项".into());}
 let paths:Vec<String>=paths.iter().map(|p|std::fs::canonicalize(p).map(|p|{let p=p.to_string_lossy();if let Some(unc)=p.strip_prefix("\\\\?\\UNC\\"){format!("\\\\{unc}")}else{p.strip_prefix("\\\\?\\").unwrap_or(&p).to_string()}}).map_err(|e|format!("无法固定文件：{e}"))).collect::<Result<_,_>>()?;
 data.mutate(|c|{for p in paths{quickbar::pin(&mut c.quickbar,&p)?;}Ok(())})?;data.publish(&app)?;app.emit("quickbar-pins-changed",()).map_err(|e|e.to_string())
}
/// Draw the drag image in the dock's own topmost WebView, not its desktop sibling.
/// This command never changes pins, source items, or source files.
#[tauri::command]
pub fn quickbar_drag_preview(app:tauri::AppHandle,window:WebviewWindow,data:tauri::State<'_,Arc<AppState>>,id:Option<String>,source:Option<String>,point:Option<[f64;2]>)->Result<(),String>{
 let s=data.snapshot()?;let origin=s.monitors.iter().find(|m|m.label==window.label()).ok_or("Invalid drag source window")?;
 let preview=if let (Some(id),Some(source),Some(point))=(id,source,point){
  if !point.iter().all(|n|n.is_finite()){return Err("Invalid drag point".into());}
  let f=s.config.fences.iter().find(|f|f.id==source&&f.items.contains(&id)).ok_or("Drag item is not in source frame")?;
  if f.monitor!=origin.name{return Err("Drag source display mismatch".into());}
  let item=s.items.iter().find(|i|i.id==id).ok_or("Drag item missing")?;
  Some((origin.x as f64+point[0]*origin.scale,origin.y as f64+point[1]*origin.scale,item.icon.clone(),item.name.clone()))
 }else{None};
 let docks=data.quickbar.lock().map_err(|_|"Quickbar lock")?.docks.clone();
 for m in docks{if let Some(w)=app.get_webview_window(&m.label){
  let payload=if let Some((x,y,ref icon,ref name))=preview{
   let pos=w.outer_position().map_err(|e|e.to_string())?;let size=w.outer_size().map_err(|e|e.to_string())?;
   if x>=m.x as f64&&x<(m.x as f64+m.w as f64)&&y>=pos.y as f64-32.*m.scale&&y<m.y as f64+m.h as f64&&size.height>0{
    serde_json::json!({"x":(x-pos.x as f64)/m.scale,"y":(y-pos.y as f64)/m.scale,"icon":icon,"name":name})
   }else{serde_json::Value::Null}
  }else{serde_json::Value::Null};
  app.emit_to(&m.label,"quickbar-drag-preview",payload).map_err(|e|e.to_string())?;
 }}Ok(())
}
#[tauri::command]
pub fn quickbar_remove(app:tauri::AppHandle,data:tauri::State<'_,Arc<AppState>>,path:String)->Result<(),String>{
 let r=data.mutate(|c|Ok(quickbar::remove(&mut c.quickbar,&path)))?;data.quickbar.lock().map_err(|_|"Quickbar lock")?.undo.insert(quickbar::key(&path),r);data.publish(&app)
}
#[tauri::command]
pub fn quickbar_undo(app:tauri::AppHandle,data:tauri::State<'_,Arc<AppState>>,path:String)->Result<(),String>{
 let r=data.quickbar.lock().map_err(|_|"Quickbar lock")?.undo.remove(&quickbar::key(&path)).ok_or("撤销记录已过期")?;data.mutate(|c|{quickbar::restore_removed(&mut c.quickbar,r);Ok(())})?;data.publish(&app)
}
#[tauri::command]
pub fn quickbar_reorder(app:tauri::AppHandle,data:tauri::State<'_,Arc<AppState>>,paths:Vec<String>)->Result<(),String>{
 data.mutate(|c|{let old:BTreeSet<_>=c.quickbar.pins.iter().map(|p|quickbar::key(p)).collect();let new:BTreeSet<_>=paths.iter().map(|p|quickbar::key(p)).collect();if old!=new||paths.len()!=old.len(){return Err("固定项目已改变，请重试".into());}c.quickbar.pins=paths;Ok(())})?;data.publish(&app)
}
#[tauri::command]
pub async fn quickbar_open(app:tauri::AppHandle,window:WebviewWindow,data:tauri::State<'_,Arc<AppState>>,path:String)->Result<(),String>{
 let s=data.snapshot()?;if !quickbar::ranked(&s.config.quickbar,quickbar::now()).iter().any(|p|quickbar::key(p)==quickbar::key(&path)){return Err("快捷项目已不存在".into());}
 if !std::path::Path::new(&path).exists(){return Err("文件已移动或不存在，请重新固定".into());}
 let copy=path.clone();let hwnd=window.hwnd().map_err(|e|e.to_string())?.0 as isize;
 let desktop_item=s.items.iter().find(|i|quickbar::key(&i.path)==quickbar::key(&path)).cloned();
 let eligible=tauri::async_runtime::spawn_blocking(move||{let eligible=desktop_item.as_ref().is_some_and(scanner::recommendation_allowed);scanner::action(&Item{learning_key:None,id:String::new(),name:String::new(),path:copy,kind:"file".into(),ext:String::new(),category:String::new(),icon:None},"open",hwnd)?;Ok::<bool,String>(eligible)}).await.map_err(|e|e.to_string())??;
 if eligible{data.mutate(|c|{quickbar::record_launch(&mut c.quickbar,&path,quickbar::now());Ok(())})?;data.publish(&app)?;}Ok(())
}
