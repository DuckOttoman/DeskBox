use crate::model::{Config,Style,Widget};
pub fn valid_kind(kind:&str)->bool { ["clock","sysmon","timer","notes","album"].contains(&kind) }
pub fn validate_style(style:&Style)->Result<(),String>{
    for (key,min,max) in [("widgetWidth",180.,1200.),("widgetHeight",100.,1000.)] {
        if let Some(value)=style.get(key){let n=value.as_f64().ok_or("Invalid widget size")?;if !n.is_finite()||n<min||n>max{return Err("Invalid widget size".into());}}
    }
    if let Some(data)=style.get("widgetData") {if !data.is_object()||data.to_string().len()>512_000{return Err("Widget data too large or invalid".into());}}
    Ok(())
}
pub fn patch_data(config:&mut Config,id:&str,patch:Style)->Result<(),String>{
    let widget=config.widgets.iter_mut().find(|w|w.id==id).ok_or("Widget does not exist")?;
    if patch.contains_key("photos")||patch.contains_key("albumDirectory"){return Err("Use album import commands to update photos".into());}
    let mut style=widget.style.clone();
    let mut data=style.get("widgetData").and_then(|d|d.as_object()).cloned().unwrap_or_default();
    data.extend(patch);style.insert("widgetData".into(),data.into());validate_style(&style)?;widget.style=style;Ok(())
}
pub fn update_visual(config:&mut Config,mut widget:Widget)->Result<(),String>{
    let existing=config.widgets.iter_mut().find(|w|w.id==widget.id).ok_or("Widget does not exist")?;
    if existing.kind!=widget.kind{return Err("Widget kind cannot change".into());}
    // A resize/menu save can race a timer or weather update: retain latest data.
    widget.style.remove("widgetData");
    if let Some(value)=existing.style.get("widgetData"){widget.style.insert("widgetData".into(),value.clone());}
    *existing=widget;Ok(())
}
#[cfg(test)]mod tests{
 use super::*;use crate::model::Widget;use serde_json::json;
 #[test]fn geometry_bounds(){let mut s=Style::new();s.insert("widgetWidth".into(),json!(179));assert!(validate_style(&s).is_err());s.insert("widgetWidth".into(),json!(460));assert!(validate_style(&s).is_ok());}
 #[test]fn stale_resize_preserves_latest_timer(){let mut c=Config::default();let w=Widget{id:"w".into(),kind:"timer".into(),x:0.,y:0.,monitor:"m".into(),style:Style::new()};c.widgets.push(w.clone());patch_data(&mut c,"w",serde_json::from_value(json!({"timer":{"running":true,"deadline":12345}})).unwrap()).unwrap();let mut resized=w;resized.x=55.;update_visual(&mut c,resized).unwrap();assert_eq!(c.widgets[0].x,55.);assert_eq!(c.widgets[0].style["widgetData"]["timer"]["deadline"],12345);}
 #[test]fn patch_preserves_other_data(){let mut c=Config::default();c.widgets.push(Widget{id:"w".into(),kind:"notes".into(),x:0.,y:0.,monitor:"m".into(),style:serde_json::from_value(json!({"widgetData":{"note":"saved","todos":[]},"radius":20})).unwrap()});patch_data(&mut c,"w",serde_json::from_value(json!({"todos":[{"text":"test","done":true}]})).unwrap()).unwrap();assert_eq!(c.widgets[0].style["widgetData"]["note"],"saved");assert_eq!(c.widgets[0].style["radius"],20);assert!(patch_data(&mut c,"w",serde_json::from_value(json!({"photos":[]})).unwrap()).is_err());}
}

pub fn interact(config:&mut Config,id:&str,action:&str,value:serde_json::Value,now:u64)->Result<bool,String>{
 use serde_json::{json,Value};
 let w=config.widgets.iter().find(|w|w.id==id).ok_or("Widget does not exist")?;
 let kind=w.kind.clone();let mut d=w.style.get("widgetData").and_then(Value::as_object).cloned().unwrap_or_default();
 if action.starts_with("todo_") {
 if kind!="notes" {return Err("Invalid widget kind".into());}
 let mut todos=d.get("todos").and_then(Value::as_array).cloned().unwrap_or_default();
 let item_id=value.get("id").and_then(Value::as_str).unwrap_or("");
 match action {
 "todo_add"=>{let text=value.get("text").and_then(Value::as_str).unwrap_or("").trim();if text.is_empty()||text.len()>10000||todos.len()>=500{return Err("Invalid todo".into());}todos.push(json!({"id":uuid::Uuid::new_v4().to_string(),"text":text,"done":false}));},
 "todo_set"=>{for t in &mut todos {if t["id"]==item_id {t["done"]=json!(value.get("done").and_then(Value::as_bool).ok_or("Invalid done value")?);}}},
 "todo_remove"=>todos.retain(|t|t["id"]!=item_id),
 "todo_clear_done"=>todos.retain(|t|t["done"]!=true),_=>return Err("Unknown action".into())}
 d.insert("todos".into(),json!(todos));
 } else {
 if kind!="timer" {return Err("Invalid widget kind".into());}
 let mut t=json!({"focusMinutes":25,"breakMinutes":5,"countMinutes":10,"mode":"pomodoro","phase":"focus","running":false});
 if let Some(old)=d.get("timer").and_then(Value::as_object){t.as_object_mut().unwrap().extend(old.clone());}
 let duration=|t:&Value|->u64 {let key=if t["mode"]=="countdown" {"countMinutes"}else if t["phase"]=="break" {"breakMinutes"}else{"focusMinutes"};t[key].as_u64().unwrap_or(25)*60000};
 match action {
 "timer_start"=>{if t["running"]!=true {let n=t["remaining"].as_u64().filter(|n|*n>0).unwrap_or_else(||duration(&t));t["deadline"]=json!(now+n);t["running"]=json!(true);}},
 "timer_pause"=>{if t["running"]==true {t["remaining"]=json!(t["deadline"].as_u64().unwrap_or(now).saturating_sub(now));t["running"]=json!(false);}},
 "timer_mode"|"timer_reset"|"timer_settings"=>{
 if action=="timer_mode" {let mode=value.as_str().ok_or("Invalid mode")?;if !["countdown","pomodoro"].contains(&mode){return Err("Invalid mode".into());}t["mode"]=json!(mode);}
 if action=="timer_settings" {for key in ["focusMinutes","breakMinutes","countMinutes"] {let n=value[key].as_u64().filter(|n|(1..=1440).contains(n)).ok_or("Invalid duration")?;t[key]=json!(n);}}
 t["phase"]=json!("focus");t["running"]=json!(false);t["remaining"]=json!(duration(&t));},
 "timer_finish"=>{if t["running"]!=true||t["deadline"].as_u64().unwrap_or(u64::MAX)>now{return Ok(false);}t["running"]=json!(false);t["phase"]=json!(if t["phase"]=="break" {"focus"}else{"break"});t["remaining"]=json!(if t["mode"]=="countdown" {0}else{duration(&t)});},
 _=>return Err("Unknown action".into())}
 d.insert("timer".into(),t);
 }
 patch_data(config,id,d)?;Ok(true)
}

#[cfg(test)]mod interaction_tests {
 use super::*;use serde_json::json;
 fn config(kind:&str)->Config {let mut c=Config::default();c.widgets.push(Widget{id:"w".into(),kind:kind.into(),x:0.,y:0.,monitor:"m".into(),style:Style::new()});c}
 #[test]fn rapid_adds_preserve_both(){let mut c=config("notes");for text in ["first","second"]{interact(&mut c,"w","todo_add",json!({"text":text}),0).unwrap();}let list=c.widgets[0].style["widgetData"]["todos"].as_array().unwrap().clone();assert_eq!(list.len(),2);for t in &list{interact(&mut c,"w","todo_set",json!({"id":t["id"],"done":true}),0).unwrap();}assert!(c.widgets[0].style["widgetData"]["todos"].as_array().unwrap().iter().all(|t|t["done"]==true));}
 #[test]fn reset_prevents_stale_completion(){let mut c=config("timer");interact(&mut c,"w","timer_start",json!(null),1000).unwrap();interact(&mut c,"w","timer_pause",json!(null),2000).unwrap();assert_eq!(c.widgets[0].style["widgetData"]["timer"]["remaining"],1499000);interact(&mut c,"w","timer_reset",json!(null),3000).unwrap();assert!(!interact(&mut c,"w","timer_finish",json!(null),9999999).unwrap());}
}
