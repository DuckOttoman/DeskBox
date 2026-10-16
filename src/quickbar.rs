//! Reference-only launch policy. Never changes source files or monitors other applications.
use crate::model::*;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;
#[derive(Clone,Debug,Default,Serialize,Deserialize)]
#[serde(default,rename_all="camelCase")]
pub struct QuickbarConfig {pub pins:Vec<String>,pub excluded:Vec<String>,pub usage:BTreeMap<String,Usage>}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Usage {pub path:String,pub score:f64,pub updated:u64}
#[derive(Clone,Debug)]
pub struct Removed {pub path:String,pub pin_index:Option<usize>,pub excluded_before:bool}
pub fn now()->u64{std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()}
pub fn key(path:&str)->String{path.replace('/',"\\").trim_end_matches('\\').to_lowercase()}
pub fn application_target(path:&str)->bool{let lower=path.to_lowercase();(local_path(path)&&[".exe",".com",".bat",".cmd",".msc",".appref-ms"].iter().any(|e|lower.ends_with(e)))||["steam:","com.epicgames.launcher:","uplay:","battlenet:","origin:","shell:appsfolder"].iter().any(|e|lower.starts_with(e))}
pub fn limit(width:f64)->usize{((width.max(0.)*0.68/72.).floor() as usize).max(12)}
pub fn valid_count(v:&serde_json::Value)->bool{v.as_str()==Some("MAX")||v.as_u64().is_some_and(|n|(10..=1000).contains(&n))}
pub fn slots(v:&serde_json::Value,width:f64)->usize{if v.as_str()==Some("MAX"){limit(width)}else{(v.as_u64().unwrap_or(12) as usize).max(10).min(limit(width))}}
fn local_path(p:&str)->bool{let b=p.as_bytes();!p.contains('\0')&&p.len()<=32767&&((b.len()>2&&b[1]==b':'&&(b[2]==b'\\'||b[2]==b'/'))||p.starts_with("\\\\"))}
pub fn pin(q:&mut QuickbarConfig,path:&str)->Result<(),String>{
 if !local_path(path){return Err("只能固定本地文件、文件夹或应用快捷方式".into());}
 let k=key(path);if !q.pins.iter().any(|p|key(p)==k){q.pins.push(path.into());}q.excluded.retain(|p|key(p)!=k);Ok(())
}
pub fn remove(q:&mut QuickbarConfig,path:&str)->Removed{
 let k=key(path);let pin_index=q.pins.iter().position(|p|key(p)==k);let path=pin_index.map(|i|q.pins[i].clone()).unwrap_or_else(||path.into());
 q.pins.retain(|p|key(p)!=k);let excluded_before=q.excluded.iter().any(|p|key(p)==k);if !excluded_before{q.excluded.push(k);}Removed{path,pin_index,excluded_before}
}
pub fn restore_removed(q:&mut QuickbarConfig,r:Removed){let k=key(&r.path);if !r.excluded_before{q.excluded.retain(|p|key(p)!=k);}if let Some(i)=r.pin_index{if !q.pins.iter().any(|p|key(p)==k){q.pins.insert(i.min(q.pins.len()),r.path);}}}
pub fn record_launch(q:&mut QuickbarConfig,path:&str,now:u64){
 if !local_path(path){return;}let k=key(path);let u=q.usage.entry(k).or_insert(Usage{path:path.into(),score:0.,updated:now});
 u.score=score(u,now)+1.;u.updated=now;
 // Bound aggregate storage; no raw activity log is retained.
 if q.usage.len()>2048{let victim=q.usage.iter().min_by(|a,b|score(a.1,now).total_cmp(&score(b.1,now))).map(|(k,_)|k.clone());if let Some(k)=victim{q.usage.remove(&k);}}
}
fn score(u:&Usage,now:u64)->f64{if !u.score.is_finite(){return 0.;}u.score.max(0.)*2f64.powf(-(now.saturating_sub(u.updated) as f64)/(14.*86400.))}
pub fn ranked(q:&QuickbarConfig,now:u64)->Vec<String>{
 let mut result=q.pins.clone();let mut usage:Vec<_>=q.usage.values().filter(|u|!q.excluded.iter().any(|p|key(p)==key(&u.path))).collect();
 usage.sort_by(|a,b|score(b,now).total_cmp(&score(a,now)).then_with(||key(&a.path).cmp(&key(&b.path))));
 for u in usage{if !result.iter().any(|p|key(p)==key(&u.path)){result.push(u.path.clone());}}result
}
pub fn remap(q:&mut QuickbarConfig,old:&str,new:&str){let k=key(old);for p in &mut q.pins{if key(p)==k{*p=new.into();}}for p in &mut q.excluded{if key(p)==k{*p=key(new);}}if let Some(mut u)=q.usage.remove(&k){u.path=new.into();q.usage.insert(key(new),u);}}
fn fit(x:&mut f64,y:&mut f64,w:&mut f64,h:&mut f64,m:&Monitor){let [l,t,mw,mh]=m.work;*w=w.min(mw.max(1.));*h=h.min(mh.max(1.));*x=x.clamp(l,(l+mw-*w).max(l));*y=y.clamp(t,(t+mh-*h).max(t));}
pub fn protect(c:&mut Config,monitors:&[Monitor]){
 if !c.settings.hide_taskbar{return;}
 let find=|name:&str|monitors.iter().find(|m|m.name==name).or_else(||monitors.iter().find(|m|m.primary));
 for f in &mut c.fences{if let Some(m)=find(&f.monitor){let mut h=if f.collapsed{38.}else{f.h};fit(&mut f.x,&mut f.y,&mut f.w,&mut h,m);if !f.collapsed{f.h=h;}}}
 for g in &mut c.fence_groups{if let Some(m)=find(&g.monitor){let(mut w,mut h)=(0f64,0f64);for f in c.fences.iter_mut().filter(|f|g.members.contains(&f.id)){f.monitor=g.monitor.clone();let mut visible_h=if f.collapsed{38.}else{f.h};fit(&mut f.x,&mut f.y,&mut f.w,&mut visible_h,m);if !f.collapsed{f.h=visible_h;}w=w.max(f.w);h=h.max(visible_h);}fit(&mut g.x,&mut g.y,&mut w,&mut h,m);}}
 for w in &mut c.widgets{if let Some(m)=find(&w.monitor){let mut width=w.style.get("widgetWidth").and_then(|v|v.as_f64()).unwrap_or(280.);let mut height=w.style.get("widgetHeight").and_then(|v|v.as_f64()).unwrap_or(180.);fit(&mut w.x,&mut w.y,&mut width,&mut height,m);w.style.insert("widgetWidth".into(),width.into());w.style.insert("widgetHeight".into(),height.into());}}
}

