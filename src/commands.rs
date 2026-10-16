use crate::{app::{self, AppState}, model::*, scanner, state, win};
use std::{collections::BTreeSet, sync::{atomic::Ordering, Arc}};
use tauri::{Emitter, Manager, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
type Shared<'a> = State<'a, Arc<AppState>>;

fn changed<T>(app: &tauri::AppHandle, data: &AppState, change: impl FnOnce(&mut Config) -> Result<T, String>) -> Result<T, String> {
    let result = data.mutate(change)?;
    data.publish(app)?;
    Ok(result)
}

#[tauri::command]
pub fn get_state(data: Shared<'_>) -> Result<Snapshot, String> { data.snapshot() }

#[tauri::command]
pub fn group_fences(app: tauri::AppHandle, window: WebviewWindow, data: Shared<'_>, source: String, target: String) -> Result<String, String> {
    let monitors=data.snapshot()?.monitors;
    changed(&app, &data, |c| { check_merge_window(c,&monitors,window.label(),&source,&target)?; crate::fence_merge::group(c, &source, &target) })
}
#[tauri::command]
pub fn select_fence(app: tauri::AppHandle, data: Shared<'_>, group: String, member: String) -> Result<(), String> {
    changed(&app, &data, |c| crate::fence_merge::select(c, &group, &member))
}
#[tauri::command]
pub fn detach_fence(app: tauri::AppHandle, data: Shared<'_>, member: String, point: [f64; 2]) -> Result<(), String> {
    changed(&app, &data, |c| crate::fence_merge::detach(c, &member, point))
}
#[tauri::command]
pub fn merge_fence_contents(app: tauri::AppHandle, window: WebviewWindow, data: Shared<'_>, source: String, target: String) -> Result<String, String> {
    let monitors=data.snapshot()?.monitors;
    changed(&app, &data, |c| { check_merge_window(c,&monitors,window.label(),&source,&target)?; crate::fence_merge::merge_contents(c, &source, &target) })
}
fn check_merge_window(c:&Config,monitors:&[Monitor],label:&str,source:&str,target:&str)->Result<(),String>{
    for id in [source,target]{
        let f=c.fences.iter().find(|f|f.id==id).ok_or("文件夹框不存在")?;
        let name=c.fence_groups.iter().find(|g|g.members.contains(&f.id)).map(|g|g.monitor.as_str()).unwrap_or(&f.monitor);
        let actual=monitors.iter().find(|m|m.name==name).or_else(||monitors.iter().find(|m|m.primary)).ok_or("显示器不可用")?;
        if actual.label!=label{return Err("请在同一显示器内合并文件夹框".into());}
    }
    Ok(())
}
#[tauri::command]
pub fn restore_fence_merge(app: tauri::AppHandle, data: Shared<'_>, record: String) -> Result<(), String> {
    let s = data.snapshot()?;
    let target = s.config.merge_restores.iter().find(|r|r.id == record).and_then(|r|s.config.fences.iter().find(|f|f.id == r.target)).ok_or("还原点或目标框已不存在")?;
    let name=s.config.fence_groups.iter().find(|g|g.members.contains(&target.id)).map(|g|g.monitor.as_str()).unwrap_or(&target.monitor);
    let monitor = s.monitors.iter().find(|m|m.name == name).or_else(||s.monitors.iter().find(|m|m.primary)).ok_or("显示器不可用")?;
    let valid = s.items.iter().map(|i|i.id.clone()).collect();
    changed(&app, &data, |c| crate::fence_merge::restore_on_display(c, &record, &valid, monitor.work, &monitor.name, &s.monitors))
}

#[tauri::command]
pub async fn rename_item(app: tauri::AppHandle, data: Shared<'_>, id: String, name: String) -> Result<(), String> {
    let data = data.inner().clone();
    tauri::async_runtime::spawn_blocking(move || data.rename_item(&app, &id, &name)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn set_region(window: WebviewWindow, rects: Option<Vec<HitRect>>) -> Result<(), String> {
    if window.label() == "settings" { return Ok(()); }
    win::set_region(window.hwnd().map_err(|e| e.to_string())?.0 as isize, rects)
}

#[tauri::command]
pub fn create_fence(app: tauri::AppHandle, data: Shared<'_>, origin: Option<[f64; 2]>) -> Result<String, String> {
    let (monitor, mut point, size) = data.location(origin)?;
    point[0] = point[0].min((size[0] - 320.).max(0.)).max(0.);
    point[1] = point[1].min((size[1] - 260.).max(0.)).max(0.);
    changed(&app, &data, |c| Ok(state::create_fence(c, &monitor, point, "新文件夹框", "")))
}

#[tauri::command]
pub fn update_fence(app: tauri::AppHandle, data: Shared<'_>, fence: Fence) -> Result<(), String> {
    if fence.title.trim().is_empty() || fence.title.len() > 1024 { return Err("Invalid fence title".into()); }
    if ![fence.x, fence.y, fence.w, fence.h].iter().all(|x| x.is_finite()) || fence.w < 140. || fence.h < 70. {
        return Err("Invalid fence geometry".into());
    }
    let snapshot = data.snapshot()?;
    let existing_monitor = snapshot.config.fences.iter().find(|f|f.id==fence.id).map(|f|f.monitor.as_str());
    if !state::monitor_assignment_valid(&fence.monitor, existing_monitor, &snapshot.monitors) { return Err("Monitor does not exist".into()); }
    let ids: BTreeSet<_> = snapshot.items.iter().map(|i| i.id.as_str()).collect();
    let mut used = BTreeSet::new();
    if fence.items.iter().any(|id| !ids.contains(id.as_str()) || !used.insert(id.clone())) { return Err("Invalid or duplicate item ID".into()); }
    changed(&app, &data, |c| {
        let index = c.fences.iter().position(|f| f.id == fence.id).ok_or("Fence does not exist")?;
        for (i, existing) in c.fences.iter_mut().enumerate() {
            if i != index { existing.items.retain(|id| !used.contains(id)); }
        }
        c.fences[index] = fence;
        Ok(())
    })
}

#[tauri::command]
pub fn delete_fence(app: tauri::AppHandle, data: Shared<'_>, id: String) -> Result<(), String> {
    changed(&app, &data, |c| state::delete_fence(c, &id))
}

#[tauri::command]
pub fn move_item(app: tauri::AppHandle, data: Shared<'_>, item: String, to: String, index: Option<usize>) -> Result<(), String> {
    let item = data.snapshot()?.items.into_iter().find(|i| i.id == item).ok_or("Item does not exist")?;
    data.mutate(|c| state::move_item(c, &item, &to, index))?;
    data.publish(&app)
}

#[tauri::command]
pub async fn item_action(app: tauri::AppHandle, window: WebviewWindow, data: Shared<'_>, id: String, action: String) -> Result<(), String> {
    let item = data.snapshot()?.items.into_iter().find(|i| i.id == id).ok_or("Item does not exist")?;
    let owner = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
    let recycled = action == "recycle";
    let opening = action == "open";
    let launch_path=item.path.clone();
    let launched=tauri::async_runtime::spawn_blocking(move || {let eligible=opening&&scanner::recommendation_allowed(&item);scanner::action(&item, &action, owner)?;Ok::<bool,String>(eligible)}).await.map_err(|e| e.to_string())??;
    if launched {data.mutate(|c|{crate::quickbar::record_launch(&mut c.quickbar,&launch_path,crate::quickbar::now());Ok(())})?;data.publish(&app)?;}
    if recycled {
        let data = data.inner().clone();
        tauri::async_runtime::spawn_blocking(move || data.refresh(&app, false, false)).await.map_err(|e| e.to_string())??;
    }
    Ok(())
}

#[tauri::command]
pub async fn organize_now(app: tauri::AppHandle, data: Shared<'_>, full: bool) -> Result<(), String> {
    let data = data.inner().clone();
    tauri::async_runtime::spawn_blocking(move || data.refresh(&app, full, true)).await.map_err(|e| e.to_string())?
}

pub fn toggle(app: &tauri::AppHandle, data: &AppState, origin: Option<[f64; 2]>) -> Result<(), String> {
    let snapshot = data.snapshot()?;
    let origin = origin.or(snapshot.config.ball).unwrap_or([100., 100.]);
    let visible = data.mutate(|c| { c.fences_visible = !c.fences_visible; Ok(c.fences_visible) })?;
    app.emit("fences-visible", serde_json::json!({"visible": visible, "origin": origin})).map_err(|e| e.to_string())?;
    if !data.smoke { win::desktop_icons(snapshot.config.settings.hide_desktop_icons && visible)?; }
    data.publish(app)
}
#[tauri::command]
pub fn toggle_fences(app: tauri::AppHandle, data: Shared<'_>, origin: Option<[f64; 2]>) -> Result<(), String> { toggle(&app, &data, origin) }

#[tauri::command]
pub fn create_widget(app: tauri::AppHandle, data: Shared<'_>, kind: String, origin: Option<[f64; 2]>) -> Result<String, String> {
    if !crate::widgets::valid_kind(&kind) { return Err("Unknown widget kind".into()); }
    let (monitor, mut point, size) = data.location(origin)?;
    point[0] = point[0].min((size[0] - 280.).max(0.)).max(0.);
    point[1] = point[1].min((size[1] - 180.).max(0.)).max(0.);
    let id = uuid::Uuid::new_v4().to_string();
    changed(&app, &data, |c| {
        let mut style=Style::new();if kind=="clock"{style.insert("widgetData".into(),serde_json::json!({"weatherAutoLocate":true}));}
        c.widgets.push(Widget { id: id.clone(), kind, x: point[0], y: point[1], monitor, style });
        Ok(id)
    })
}
#[tauri::command]
pub fn update_widget(app: tauri::AppHandle, data: Shared<'_>, widget: Widget) -> Result<(), String> {
    if !crate::widgets::valid_kind(&widget.kind) || !widget.x.is_finite() || !widget.y.is_finite() { return Err("Invalid widget".into()); }
    crate::widgets::validate_style(&widget.style)?;
    let snapshot = data.snapshot()?;
    let existing_monitor = snapshot.config.widgets.iter().find(|w|w.id==widget.id).map(|w|w.monitor.as_str());
    if !state::monitor_assignment_valid(&widget.monitor, existing_monitor, &snapshot.monitors) { return Err("Monitor does not exist".into()); }
    changed(&app, &data, |c| crate::widgets::update_visual(c,widget))
}

#[tauri::command]
pub fn patch_widget_data(app:tauri::AppHandle,data:Shared<'_>,id:String,patch:Style)->Result<(),String>{changed(&app,&data,|c|crate::widgets::patch_data(c,&id,patch))}

#[tauri::command]
pub async fn open_widget_weather_source()->Result<(),String>{
    tauri::async_runtime::spawn_blocking(||scanner::action(&Item{learning_key:None,id:String::new(),name:String::new(),path:"https://open-meteo.com/".into(),kind:"file".into(),ext:String::new(),category:String::new(),icon:None},"open",0)).await.map_err(|e|e.to_string())?
}

#[tauri::command]
pub async fn widget_album_action(app:tauri::AppHandle,data:Shared<'_>,id:String,action:String,index:Option<usize>,sources:Option<Vec<String>>,directory:Option<String>)->Result<(),String>{
    let data=data.inner().clone();
    tauri::async_runtime::spawn_blocking(move||crate::widget_album::action(&app,&data,&id,&action,index,sources,directory)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub fn delete_widget(app: tauri::AppHandle, data: Shared<'_>, id: String) -> Result<(), String> {
    changed(&app, &data, |c| {
        let index = c.widgets.iter().position(|w| w.id == id).ok_or("Widget does not exist")?;
        c.widgets.remove(index); Ok(())
    })
}

#[tauri::command]
pub fn set_theme(app: tauri::AppHandle, data: Shared<'_>, theme: String) -> Result<(), String> {
    if !["glass-dark", "glass-light", "minimal", "neon", "sakura", "forest"].contains(&theme.as_str()) { return Err("Unknown theme".into()); }
    changed(&app, &data, |c| { c.theme = theme; Ok(()) })
}
#[tauri::command]
pub async fn update_settings(app: tauri::AppHandle, data: Shared<'_>, settings: Settings) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    if !settings.opacity.is_finite() || !(0.1..=1.).contains(&settings.opacity)
        || !(28..=80).contains(&settings.icon_size) || !(1000..=10000).contains(&settings.sysmon_interval_ms)
        || !settings.anim_speed.is_finite() || !(0.2..=5.).contains(&settings.anim_speed)
        || !crate::quickbar::valid_count(&settings.quickbar_count) { return Err("Invalid settings values".into()); }
    let previous = data.snapshot()?.config;
    if !data.smoke && previous.settings.launch_at_startup != settings.launch_at_startup {
        if settings.launch_at_startup { app.autolaunch().enable() } else { app.autolaunch().disable() }.map_err(|e| e.to_string())?;
    }
    let hide = settings.hide_desktop_icons && previous.fences_visible;
    if let Err(e)=crate::quickbar_runtime::set_enabled(&app,data.inner(),settings.hide_taskbar).await {
        if !data.smoke && previous.settings.launch_at_startup != settings.launch_at_startup {let _=if previous.settings.launch_at_startup{app.autolaunch().enable()}else{app.autolaunch().disable()};}return Err(e);
    }
    if let Err(error) = data.mutate(|c| { c.settings = settings; Ok(()) }) {
        if !data.smoke { let _ = if previous.settings.launch_at_startup { app.autolaunch().enable() } else { app.autolaunch().disable() }; }
        let _=crate::quickbar_runtime::set_enabled(&app,data.inner(),previous.settings.hide_taskbar).await;
        return Err(error);
    }
    if !data.smoke { win::desktop_icons(hide)?; }
    data.publish(&app)
}

#[tauri::command]
pub fn save_ball(data: Shared<'_>, x: f64, y: f64) -> Result<(), String> {
    if !x.is_finite() || !y.is_finite() { return Err("Invalid ball coordinates".into()); }
    data.mutate(|c| { c.ball = Some([x, y]); Ok(()) })
}
#[tauri::command]
pub async fn open_settings_window(app: tauri::AppHandle, data: Shared<'_>) -> Result<(), String> {
    app::open_settings_async(app, data.inner().clone()).await
}
#[tauri::command]
pub async fn reset_learned(app: tauri::AppHandle, data: Shared<'_>) -> Result<(), String> {
    let data=data.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        data.mutate(|c| {c.learned.clear();Ok(())})?;
        // Rescan real shortcut targets/URLs rather than reclassifying only display names.
        data.refresh(&app,false,false)?;
        data.publish(&app)
    }).await.map_err(|e|e.to_string())?
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) { app.exit(0); }

#[tauri::command]
pub async fn keyboard_focus(window: WebviewWindow, data: Shared<'_>, on: bool) -> Result<(), String> {
    let handle = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
    let label = window.label().to_string();
    let data = data.inner().clone();
    let (send, receive) = std::sync::mpsc::channel();
    // with_webview runs on the UI thread where Win32 GetFocus/SetFocus are valid.
    window.with_webview(move |webview| {
        let result: Result<(), String> = (|| {
            win::focusable(handle, on);
            // Hidden DOM-only smoke mode cannot receive physical keyboard focus.
            if data.smoke && !data.desktop_check && !std::env::args().any(|x| x == "--drag-session") { return Ok(()); }
            if on {
                if data.desktop_check { win::show_window_no_activate(handle); }
                win::focus_child(handle)?;
                // WebView2 PROGRAMMATIC is the enum default (0); focus its input child.
                unsafe { webview.controller().MoveFocus(Default::default()).map_err(|e| e.to_string())?; }
                if !win::focus_owned(handle) { return Err("WebView 输入焦点未进入目标窗口".into()); }
                if data.desktop_check { data.log(format!("KEYBOARD_NATIVE_FOCUS label={label} owned=true")); }
            } else if data.desktop_check { win::hide_window(handle); }
            Ok(())
        })();
        if result.is_err() {
            win::focusable(handle, false);
            if data.desktop_check { win::hide_window(handle); }
        }
        let _ = send.send(result);
    }).map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn_blocking(move || receive.recv_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| format!("Keyboard focus dispatch failed: {e}"))).await.map_err(|e| e.to_string())??
}

fn import_background(app:&tauri::AppHandle,data:&AppState,id:&str,source:&std::path::Path,previous:Style)->Result<(),String>{
    let directory=data.directory.join("backgrounds");
    let (poster,media)=if crate::media::is_animated(source){
        let m=crate::media::prepare_background(source,&directory,&crate::media::ffmpeg_path(data.smoke)?)?;
        (m.poster.clone(),Some(serde_json::to_value(m).map_err(|e|e.to_string())?))
    }else{
        let image=image::open(source).map_err(|e|e.to_string())?;
        std::fs::create_dir_all(&directory).map_err(|e|e.to_string())?;
        let path=directory.join(format!("{}.png",uuid::Uuid::new_v4()));
        image.save(&path).map_err(|e|e.to_string())?;(path.to_string_lossy().into_owned(),None)
    };
    let result=data.mutate(|c|{
        let fence=c.fences.iter_mut().find(|f|f.id==id).ok_or("Fence does not exist")?;
        if fence.style.get("media")!=previous.get("media") || fence.style.get("image")!=previous.get("image"){return Err("Background changed while preparing media".into());}
        fence.style.insert("image".into(),poster.clone().into());
        if let Some(m)=media.clone(){fence.style.insert("media".into(),m);}else{fence.style.remove("media");}Ok(())
    });
    if result.is_err(){if media.is_some(){if let Some(parent)=std::path::Path::new(&poster).parent(){let _=std::fs::remove_dir_all(parent);}}else{let _=std::fs::remove_file(&poster);}}
    result?;data.publish(app)
}
#[tauri::command]
pub async fn pick_fence_image(app:tauri::AppHandle,data:Shared<'_>,id:String)->Result<(),String>{
    let previous=data.snapshot()?.config.fences.iter().find(|f|f.id==id).ok_or("Fence does not exist")?.style.clone();let data=data.inner().clone();
    tauri::async_runtime::spawn_blocking(move||{
        let picked=app.dialog().file().set_title("选择背景图片、GIF 或视频").add_filter("背景", &["png","jpg","jpeg","webp","bmp","ico","gif","mp4","webm","m4v","mov"]).blocking_pick_file();
        let Some(picked)=picked else{return Ok(());};import_background(&app,&data,&id,&picked.into_path().map_err(|e|e.to_string())?,previous)
    }).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn import_fence_background(app:tauri::AppHandle,data:Shared<'_>,id:String,source:String)->Result<(),String>{
    if !data.smoke{return Err("Diagnostic mode required".into());}
    let previous=data.snapshot()?.config.fences.iter().find(|f|f.id==id).ok_or("Fence does not exist")?.style.clone();let data=data.inner().clone();
    tauri::async_runtime::spawn_blocking(move||import_background(&app,&data,&id,std::path::Path::new(&source),previous)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub fn set_fence_video_mode(app:tauri::AppHandle,data:Shared<'_>,id:String,mode:String)->Result<(),String>{
    changed(&app,&data,|c|{let fence=c.fences.iter_mut().find(|f|f.id==id).ok_or("Fence does not exist")?;
        let mut m:crate::media::MediaBackground=serde_json::from_value(fence.style.get("media").cloned().ok_or("Video unavailable")?).map_err(|e|e.to_string())?;
        m.validate_mode(&mode,&data.directory.join("backgrounds"))?;m.mode=mode;
        fence.style.insert("media".into(),serde_json::to_value(m).map_err(|e|e.to_string())?);Ok(())})
}
#[tauri::command]
pub fn clear_fence_image(app:tauri::AppHandle,data:Shared<'_>,id:String)->Result<(),String>{
    changed(&app,&data,|c|{let f=c.fences.iter_mut().find(|f|f.id==id).ok_or("Fence does not exist")?;f.style.remove("image");f.style.remove("media");Ok(())})
}
#[tauri::command]
pub fn media_playback_allowed()->bool{win::media_playback_allowed()}
#[tauri::command]
pub fn watch_outside_click(data: Shared<'_>, enable: bool) { data.watch_outside.store(enable, Ordering::Relaxed); }

#[tauri::command]
pub fn frontend_ready(window: WebviewWindow, data: Shared<'_>, nodes: usize, role: String) {
    if data.smoke && nodes > 0 {
        data.log(format!("FRONTEND_READY label={} role={role} nodes={nodes}", window.label()));
        if let Ok(mut ready) = data.smoke_ready.lock() { ready.insert(window.label().into()); }
    }
}
#[tauri::command]
pub fn frontend_error(window: WebviewWindow, data: Shared<'_>, message: String) {
    data.log(format!("FRONTEND_ERROR label={} {message}", window.label()));
    data.smoke_error.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub fn frontend_checks_passed(app: tauri::AppHandle, data: Shared<'_>, checks: Vec<String>) {
    if data.smoke {
        if data.desktop_check {
            if let Ok(snapshot) = data.snapshot() {
                for monitor in snapshot.monitors {
                    if let Some(window) = app.get_webview_window(&monitor.label) {
                        if let Ok(hwnd) = window.hwnd() {
                            if let Err(error) = win::desktop_attachment_status(hwnd.0 as isize) {
                                data.log(format!("FRONTEND_ERROR {error}"));
                                data.smoke_error.store(true, Ordering::Relaxed);
                            }
                            win::hide_window(hwnd.0 as isize);
                        }
                    }
                }
            }
        }
        data.log(format!("FRONTEND_CONTRACT_CHECKS {}", checks.join(", ")));
        if checks.iter().any(|c| c == "rename_menu" || c == "ball_settings_lifecycle" || c == "scan_responsiveness" || c == "fence_merge_release") { data.smoke_rename_menu_passed.store(true, Ordering::Relaxed); }
        if checks.iter().any(|c| c == "rename_item" || c == "ball_settings_lifecycle" || c == "scan_responsiveness" || c == "fence_merge_release") { data.smoke_contract_passed.store(true, Ordering::Relaxed); }
    }
}

// Isolated diagnostic only; cannot operate settings in normal application mode.
#[tauri::command]
pub async fn settings_test_probe(app: tauri::AppHandle, data: Shared<'_>, action: Option<String>) -> Result<serde_json::Value, String> {
    if !data.smoke { return Err("Diagnostic mode required".into()); }
    if action.as_deref() == Some("tray_settings") {
        app::dispatch_tray_action(&app, "settings");
        return Ok(serde_json::json!({"dispatched":true}));
    }
    if action.as_deref() == Some("slow_scan") {
        let data = data.inner().clone(); let app = app.clone();
        let items = data.snapshot()?.items;
        tauri::async_runtime::spawn_blocking(move || {
            let result = data.refresh_with_scan(&app, false, true, || {
                data.diagnostic_scan_started.store(true, Ordering::Relaxed);
                data.log("SLOW_SCAN_ENTER");
                std::thread::sleep(std::time::Duration::from_secs(3));
                Ok(items)
            });
            if let Err(error) = result { data.log(error); data.smoke_error.store(true, Ordering::Relaxed); }
            data.diagnostic_scan_finished.store(true, Ordering::Relaxed);
        });
        return Ok(serde_json::json!({"dispatched":true}));
    }
    let Some(window) = app.get_webview_window("settings") else { return Ok(serde_json::json!({"exists":false})); };
    if let Some(action) = action {
        let script = match action.as_str() {
            "close" => "document.querySelector('.wbtn.close').click()",
            "minimize" => "document.querySelector('.wbtn:not(.close)').click()",
            _ => return Err("Unknown diagnostic action".into()),
        };
        window.eval(script).map_err(|e|e.to_string())?;
    }
    Ok(serde_json::json!({"exists":true,"visible":window.is_visible().map_err(|e|e.to_string())?,
        "minimized":window.is_minimized().map_err(|e|e.to_string())?,
        "scanStarted":data.diagnostic_scan_started.load(Ordering::Relaxed),
        "scanFinished":data.diagnostic_scan_finished.load(Ordering::Relaxed),
        "ready":data.smoke_ready.lock().map_err(|_|"State lock poisoned")?.contains("settings")}))
}

#[tauri::command]
pub fn widget_interaction(app:tauri::AppHandle,data:Shared<'_>,id:String,action:String,value:serde_json::Value)->Result<bool,String>{
 let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e|e.to_string())?.as_millis() as u64;
 changed(&app,&data,|c|crate::widgets::interact(c,&id,&action,value,now))
}

#[tauri::command]
pub async fn optimize_memory()->Result<serde_json::Value,String>{
 tauri::async_runtime::spawn_blocking(||crate::memory::optimize().map(|r|serde_json::to_value(r).unwrap())).await.map_err(|e|e.to_string())?
}

#[tauri::command]
pub fn apply_weather_location(app:tauri::AppHandle,data:Shared<'_>,id:String,city:serde_json::Value)->Result<bool,String>{
 let name=city["name"].as_str().filter(|n|!n.trim().is_empty()&&n.len()<=400).ok_or("Invalid city")?;
 let latitude=city["latitude"].as_f64().filter(|n|n.is_finite()&&n.abs()<=90.).ok_or("Invalid latitude")?;
 let longitude=city["longitude"].as_f64().filter(|n|n.is_finite()&&n.abs()<=180.).ok_or("Invalid longitude")?;
 changed(&app,&data,|c|{let w=c.widgets.iter().find(|w|w.id==id&&w.kind=="clock").ok_or("Clock does not exist")?;
 let d=w.style.get("widgetData");if d.and_then(|v|v.get("weatherCity")).is_some_and(|v|!v.is_null())||d.and_then(|v|v.get("weatherAutoLocate")).and_then(|v|v.as_bool())!=Some(true){return Ok(false);}
 crate::widgets::patch_data(c,&id,serde_json::from_value(serde_json::json!({"weatherCity":{"name":name,"latitude":latitude,"longitude":longitude,"source":"ip","provider":if city["provider"]=="ipinfo.io" {"ipinfo.io"}else{"ipapi.co"}},"weatherAutoLocate":false,"weatherLocationError":false})).unwrap())?;Ok(true)})
}

#[tauri::command]
pub fn move_fence_group(app:tauri::AppHandle,window:WebviewWindow,data:Shared<'_>,group:String,point:[f64;2])->Result<(),String>{
 let monitors=data.snapshot()?.monitors;
 changed(&app,&data,|c|{let g=c.fence_groups.iter().find(|g|g.id==group).ok_or("Group does not exist")?;
 let m=monitors.iter().find(|m|m.name==g.monitor).or_else(||monitors.iter().find(|m|m.primary)).ok_or("Monitor unavailable")?;
 if m.label!=window.label(){return Err("Group belongs to another display".into());}
 crate::fence_merge::move_group(c,&group,point)})
}


#[tauri::command]
pub fn request_layout_backup(app:tauri::AppHandle,data:Shared<'_>,action:String)->Result<(),String>{
 if !matches!(action.as_str(),"create"|"manage"){return Err("未知布局操作".into());}
 let s=data.snapshot()?;let m=s.monitors.iter().find(|m|m.primary).or(s.monitors.first()).ok_or("没有可用显示器")?;
 app.emit("layout-backup-panel",serde_json::json!({"label":m.label,"action":action})).map_err(|e|e.to_string())
}
#[tauri::command]
pub async fn layout_backup_action(app:tauri::AppHandle,data:Shared<'_>,action:String,id:Option<String>,name:Option<String>)->Result<serde_json::Value,String>{
 let data=data.inner().clone();
 tauri::async_runtime::spawn_blocking(move||{
  use crate::layout_backup as b;
  let _lock=b::OPERATIONS.lock().map_err(|_|"布局备份操作锁异常")?;
  let mut list=b::load(&data.directory)?;
  let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e|e.to_string())?.as_millis() as u64;
  let mut missing=0;
  match action.as_str(){
   "list"=>{},
   "create"=>{list.push(b::capture(&data.snapshot()?.config,name.as_deref().ok_or("缺少备份名称")?,now)?);b::save(&data.directory,&list)?;},
   "rename"=>{b::rename(&mut list,id.as_deref().ok_or("缺少备份编号")?,name.as_deref().ok_or("缺少备份名称")?)?;b::save(&data.directory,&list)?;},
   "delete"=>{b::delete(&mut list,id.as_deref().ok_or("缺少备份编号")?)?;b::save(&data.directory,&list)?;},
   "restore"=>{
    let backup=list.iter().find(|b|Some(b.id.as_str())==id.as_deref()).cloned().ok_or("备份不存在")?;
    data.refresh(&app,false,false)?;
    let _files=data.filesystem_operation.lock().map_err(|_|"文件操作锁异常")?;
    let s=data.snapshot()?;
    missing=data.mutate(|c|{
     let mut next=c.clone();let missing=b::restore(&mut next,&backup,&s.monitors,&s.items)?;
     list.push(b::capture(c,"还原前自动备份",now)?);b::save(&data.directory,&list)?;
     *c=next;Ok(missing)
    })?;data.publish(&app)?;
   },
   _=>return Err("未知布局备份操作".into())
  }
  Ok(serde_json::json!({"backups":b::summaries(&list),"missingBackgrounds":missing}))
 }).await.map_err(|e|e.to_string())?
}

#[tauri::command]
pub async fn enable_cpu_temperature()->Result<(),String>{
 tauri::async_runtime::spawn_blocking(||{
 let dir=std::env::current_exe().map_err(|e|e.to_string())?;
 crate::core_temperature::request_enable(dir.parent().is_some_and(|p|p.join("uninstall.exe").is_file()))
 }).await.map_err(|e|e.to_string())?
}
