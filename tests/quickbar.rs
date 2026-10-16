use deskbox_reconstructed::{quickbar::*,model::*};
#[test]fn recommendation_only_applications(){assert!(application_target("C:\\app.exe"));assert!(application_target("steam://rungameid/123"));assert!(!application_target("C:\\report.pdf"));assert!(!application_target("https://example.com"));assert!(!application_target("C:\\Folder"));}
#[test] fn counts_and_legacy_defaults(){
 let c:Config=serde_json::from_str("{}").unwrap();assert!(!c.settings.hide_taskbar);assert_eq!(c.settings.quickbar_count,serde_json::json!(12));
 assert_eq!(limit(1920.),18);assert_eq!(limit(800.),12);assert_eq!(slots(&serde_json::json!("MAX"),1920.),18);assert_eq!(slots(&serde_json::json!(99),1920.),18);
}
#[test] fn pin_exclude_repin_and_undo(){
 let mut q=QuickbarConfig::default();pin(&mut q,"C:\\Apps\\One.exe").unwrap();pin(&mut q,"c:/apps/one.exe").unwrap();assert_eq!(q.pins.len(),1);
 record_launch(&mut q,"C:\\Apps\\One.exe",100);let undo=remove(&mut q,"c:/apps/one.exe");assert!(q.pins.is_empty());assert!(ranked(&q,100).is_empty());
 restore_removed(&mut q,undo);assert_eq!(q.pins.len(),1);remove(&mut q,"c:/apps/one.exe");pin(&mut q,"C:\\Apps\\One.exe").unwrap();assert!(q.excluded.is_empty());
 assert!(pin(&mut q,"https://example.com").is_err());
}
#[test] fn usage_decay_and_pinned_order(){
 let mut q=QuickbarConfig::default();record_launch(&mut q,"C:\\Old.exe",1);record_launch(&mut q,"C:\\New.exe",1+14*86400);assert_eq!(ranked(&q,1+14*86400),vec!["C:\\New.exe","C:\\Old.exe"]);
 pin(&mut q,"C:\\Old.exe").unwrap();assert_eq!(ranked(&q,1+14*86400),vec!["C:\\Old.exe","C:\\New.exe"]);
}
#[test] fn protection_clamps_group_members_and_widgets(){
 let mut c=Config::default();c.settings.hide_taskbar=true;
 c.fences.push(Fence{id:"f".into(),title:"F".into(),x:400.,y:720.,w:700.,h:500.,style:Style::new(),category:None,monitor:"m".into(),collapsed:false,hidden:false,items:vec![]});
 c.widgets.push(Widget{id:"w".into(),kind:"notes".into(),x:700.,y:700.,monitor:"m".into(),style:Style::new()});
 c.fence_groups.push(FenceGroup{id:"g".into(),members:vec!["f".into()],active:"f".into(),x:900.,y:750.,monitor:"m".into()});
 let m=Monitor{name:"m".into(),label:"overlay-0".into(),x:0,y:0,w:1000,h:800,scale:1.,work:[0.,0.,1000.,704.],primary:true};
 protect(&mut c,&[m]);assert!(c.fence_groups[0].y+c.fences[0].h<=704.);assert!(c.widgets[0].y+180.<=704.);
 let before=c.clone();c.settings.hide_taskbar=false;c.fences[0].y=780.;protect(&mut c,&[]);assert_eq!(c.fences[0].y,780.);assert!(before.fences[0].h>0.);
}
