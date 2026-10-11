use deskbox_reconstructed::{model::*,state};
fn monitor(name:&str,x:i32,scale:f64)->Monitor {Monitor{name:name.into(),label:name.into(),x,y:-200,w:1920,h:1080,scale,work:[0.,0.,1920./scale,1040./scale],primary:x==0}}
#[test]
fn orphan_monitor_fallback_preserves_assignment_but_unknown_new_assignment_is_rejected(){
 let monitors=vec![monitor("main",0,1.)];
 assert!(state::monitor_assignment_valid("unplugged",Some("unplugged"),&monitors));
 assert!(state::monitor_assignment_valid("main",Some("unplugged"),&monitors));
 assert!(!state::monitor_assignment_valid("unknown",Some("main"),&monitors));
 assert!(!state::monitor_assignment_valid("unknown",None,&monitors));
}
#[test]
fn physical_origin_maps_to_negative_monitor_local_logical_units(){
 let monitors=vec![monitor("main",0,1.),monitor("left",-1920,1.5)];
 let (name,point,size)=state::locate_monitor(&monitors,[-1770.,-50.]).unwrap();
 assert_eq!(name,"left");assert_eq!(point,[100.,100.]);assert_eq!(size,[1280.,720.]);
 let (name,_,_)=state::locate_monitor(&monitors,[9999.,9999.]).unwrap();assert_eq!(name,"main");
}
