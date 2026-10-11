use crate::{model::*,state};
use serde::{Serialize,Deserialize};
use serde_json::{Value,json};
use std::{fs,io::Write,path::Path,sync::Mutex};
pub static OPERATIONS:Mutex<()>=Mutex::new(());
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Backup { pub id:String,pub name:String,pub created_at:u64,pub fences:Vec<Fence>,pub groups:Vec<FenceGroup>,pub widgets:Vec<Widget> }
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Summary {pub id:String,pub name:String,pub created_at:u64,pub fence_count:usize,pub widget_count:usize}
fn name(s:&str)->Result<String,String>{let s=s.trim();if s.is_empty()||s.chars().count()>100{return Err("备份名称应为1至100个字符".into());}Ok(s.into())}
fn settings(data:&Value)->Value{
 let mut d=data.as_object().cloned().unwrap_or_default();
 for key in ["note","todos","photos","albumDirectory","weatherCache","weatherLocationError"]{d.remove(key);}
 if let Some(t)=d.get_mut("timer").and_then(Value::as_object_mut){t.retain(|k,_|matches!(k.as_str(),"focusMinutes"|"breakMinutes"|"countMinutes"|"mode"));}
 Value::Object(d)
}
pub fn capture(c:&Config,title:&str,now:u64)->Result<Backup,String>{
 let mut fences=c.fences.clone();for f in &mut fences{f.items.clear();}
 let mut widgets=c.widgets.clone();for w in &mut widgets{if let Some(d)=w.style.get_mut("widgetData"){*d=settings(d);}}
 Ok(Backup{id:uuid::Uuid::new_v4().to_string(),name:name(title)?,created_at:now,fences,groups:c.fence_groups.clone(),widgets})
}
pub fn summaries(list:&[Backup])->Vec<Summary>{list.iter().rev().map(|b|Summary{id:b.id.clone(),name:b.name.clone(),created_at:b.created_at,fence_count:b.fences.len(),widget_count:b.widgets.len()}).collect()}
pub fn rename(list:&mut [Backup],id:&str,title:&str)->Result<(),String>{let title=name(title)?;list.iter_mut().find(|b|b.id==id).ok_or("备份不存在")?.name=title;Ok(())}
pub fn delete(list:&mut Vec<Backup>,id:&str)->Result<(),String>{let i=list.iter().position(|b|b.id==id).ok_or("备份不存在")?;list.remove(i);Ok(())}
pub fn load(dir:&Path)->Result<Vec<Backup>,String>{match fs::read(dir.join("layout-backups.json")){Ok(bytes)=>serde_json::from_slice(&bytes).map_err(|e|format!("读取布局备份失败：{e}")),Err(e)if e.kind()==std::io::ErrorKind::NotFound=>Ok(vec![]),Err(e)=>Err(e.to_string())}}
pub fn save(dir:&Path,list:&[Backup])->Result<(),String>{
 fs::create_dir_all(dir).map_err(|e|e.to_string())?;
 let tmp=dir.join("layout-backups.json.tmp");let mut f=fs::File::create(&tmp).map_err(|e|e.to_string())?;
 f.write_all(&serde_json::to_vec_pretty(list).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;f.sync_all().map_err(|e|e.to_string())?;drop(f);
 fs::rename(tmp,dir.join("layout-backups.json")).map_err(|e|format!("保存布局备份失败：{e}"))
}
fn fit(name:&mut String,x:&mut f64,y:&mut f64,w:&mut f64,h:&mut f64,monitors:&[Monitor])->Result<(),String>{
 if ![*x,*y,*w,*h].iter().all(|v|v.is_finite())||*w<=0.||*h<=0.{return Err("备份布局尺寸无效".into());}
 let m=monitors.iter().find(|m|m.name==*name).or_else(||monitors.iter().find(|m|m.primary)).or(monitors.first()).ok_or("没有可用显示器")?;
 *name=m.name.clone();let [left,top,width,height]=m.work;*w=w.min(width);*h=h.min(height);*x=x.clamp(left,(left+width-*w).max(left));*y=y.clamp(top,(top+height-*h).max(top));Ok(())
}
pub fn restore(c:&mut Config,b:&Backup,monitors:&[Monitor],items:&[Item])->Result<usize,String>{
 if monitors.is_empty(){return Err("没有可用显示器".into());}
 let mut next=c.clone();next.fences=b.fences.clone();next.fence_groups=b.groups.clone();next.widgets=b.widgets.clone();next.merge_restores.clear();let mut missing=0;
 for f in &mut next.fences{f.items.clear();fit(&mut f.monitor,&mut f.x,&mut f.y,&mut f.w,&mut f.h,monitors)?;
  let invalid=f.style.get("image").and_then(Value::as_str).is_some_and(|p|!Path::new(p).is_file())||f.style.get("media").is_some_and(|m|[m.get("original"),m.get("poster"),if m.get("mode").and_then(Value::as_str)==Some("eco"){m.get("eco")}else{None}].into_iter().flatten().filter_map(Value::as_str).any(|p|!Path::new(p).is_file()));
  if invalid{f.style.remove("image");f.style.remove("media");missing+=1;}
 }
 for g in &mut next.fence_groups{
  if g.members.len()<2||!g.members.contains(&g.active)||g.members.iter().any(|id|!next.fences.iter().any(|f|f.id==*id)){return Err("备份的合并组无效".into());}
  let m=monitors.iter().find(|m|m.name==g.monitor).or_else(||monitors.iter().find(|m|m.primary)).unwrap_or(&monitors[0]);g.monitor=m.name.clone();
  let(mut w,mut h)=(0.0_f64,0.0_f64);
  for f in next.fences.iter_mut().filter(|f|g.members.contains(&f.id)){f.monitor=g.monitor.clone();f.w=f.w.min(m.work[2]);f.h=f.h.min(m.work[3]);w=w.max(f.w);h=h.max(if f.collapsed{38.}else{f.h});}
  fit(&mut g.monitor,&mut g.x,&mut g.y,&mut w,&mut h,monitors)?;
 }
 for w in &mut next.widgets{
  if !crate::widgets::valid_kind(&w.kind){return Err("备份包含未知小组件".into());}
  let mut d=settings(w.style.get("widgetData").unwrap_or(&Value::Null));
  if let Some(old)=c.widgets.iter().find(|old|old.id==w.id&&old.kind==w.kind).and_then(|old|old.style.get("widgetData")){
   for key in ["note","todos","photos","albumDirectory"]{if let Some(v)=old.get(key){d[key]=v.clone();}}
   if old.get("weatherCity")==d.get("weatherCity"){if let Some(v)=old.get("weatherCache"){d["weatherCache"]=v.clone();}}
   if let Some(t)=old.get("timer").and_then(Value::as_object){if !d["timer"].is_object(){d["timer"]=json!({});}for key in ["running","phase","deadline","remaining"]{if let Some(v)=t.get(key){d["timer"][key]=v.clone();}}}
  }
  w.style.insert("widgetData".into(),d);
  let(mut width,mut height)=(w.style.get("widgetWidth").and_then(Value::as_f64).unwrap_or(if w.kind=="sysmon"{300.}else{320.}),w.style.get("widgetHeight").and_then(Value::as_f64).unwrap_or(if w.kind=="clock"{210.}else if w.kind=="sysmon"{260.}else{240.}));
  fit(&mut w.monitor,&mut w.x,&mut w.y,&mut width,&mut height,monitors)?;w.style.insert("widgetWidth".into(),json!(width));w.style.insert("widgetHeight".into(),json!(height));
 }
 // Reclassify current items without the organize() layout rearrangement.
 for item in items{
  let category=next.learned.get(&state::learning_key(item)).unwrap_or(&item.category);
  let dest=next.fences.iter().position(|f|f.category.as_deref()==Some(category.as_str()));
  let dest=dest.or_else(||next.fences.iter().position(|f|f.category.as_deref()==Some("inbox"))).unwrap_or_else(||{
   let m=monitors.iter().find(|m|m.primary).unwrap_or(&monitors[0]);let id=state::create_fence(&mut next,&m.name,[m.work[0],m.work[1]],"收纳框","inbox");next.fences.iter().position(|f|f.id==id).unwrap()
  });next.fences[dest].items.push(item.id.clone());
 }
 next.initialized=true;*c=next;Ok(missing)
}
