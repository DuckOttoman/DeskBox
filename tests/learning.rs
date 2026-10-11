use deskbox_reconstructed::learning;
#[test]
fn target_basename_wins_over_url_and_name(){
 assert_eq!(learning::key("Browser",r"C:/Apps/Chrome/CHROME.EXE","HTTPS://EXAMPLE.COM"),"t:chrome.exe");
 assert_eq!(learning::key("Folder",r"D:\My Folder",""),"t:my folder");
}
#[test]
fn url_value_is_lowercased_and_name_only_is_fallback(){
 let url=learning::internet_url("[InternetShortcut]\r\nURL=HTTPS://EXAMPLE.COM/Path?Q=A\r\nIconIndex=0");
 assert_eq!(url,"https://example.com/path?q=a");
 assert_eq!(learning::key("网站","",&url),"u:https://example.com/path?q=a");
 assert_eq!(learning::key("报告.TXT","",""),"n:报告.txt");
 assert_eq!(learning::internet_url("[InternetShortcut]\nIconIndex=1"),"");
}

#[test]
fn hidden_target_change_is_not_equal_and_moves_use_target_key_without_inbox_erasure(){
 use deskbox_reconstructed::{model::*,state};
 let mut item=Item{learning_key:Some("t:one.exe".into()),id:"link".into(),name:"Alias".into(),path:"alias.lnk".into(),kind:"shortcut".into(),ext:"lnk".into(),category:"other".into(),icon:None};
 let old=item.clone();item.learning_key=Some("t:two.exe".into());
 assert_eq!(serde_json::to_value(&old).unwrap(),serde_json::to_value(&item).unwrap());assert_ne!(old,item);
 let mut c=Config::default();let target=state::create_fence(&mut c,"main",[0.,0.],"文档","document");
 state::move_item(&mut c,&item,&target,None).unwrap();assert_eq!(c.learned.get("t:two.exe").map(String::as_str),Some("document"));
 assert!(!c.learned.contains_key("n:alias"));
 let inbox=state::create_fence(&mut c,"main",[0.,0.],"新文件","inbox");state::move_item(&mut c,&item,&inbox,None).unwrap();assert_eq!(c.learned.get("t:two.exe").map(String::as_str),Some("document"));
 let custom=state::create_fence(&mut c,"main",[0.,0.],"自定义","");state::move_item(&mut c,&item,&custom,None).unwrap();assert_eq!(c.learned.len(),1);
}

#[test]
fn organize_uses_shared_target_rule_and_clear_returns_to_scanned_category(){
 use deskbox_reconstructed::{model::*,state};
 let item=Item{learning_key:Some("t:notepad.exe".into()),id:"alias".into(),name:"任意名称".into(),path:"alias.lnk".into(),kind:"shortcut".into(),ext:"lnk".into(),category:"other".into(),icon:None};
 let mut c=Config::default();c.learned.insert("t:notepad.exe".into(),"media".into());
 state::organize(&mut c,&[item.clone()],"main",[1920.,1080.],true);
 assert!(c.fences.iter().any(|f|f.category.as_deref()==Some("media")&&f.items.contains(&item.id)));
 c.learned.clear();state::organize(&mut c,&[item.clone()],"main",[1920.,1080.],true);
 assert!(c.fences.iter().any(|f|f.category.as_deref()==Some("other")&&f.items.contains(&item.id)));
}

