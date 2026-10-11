use deskbox_reconstructed::{fence_merge as fm, model::*, state};
use std::collections::HashSet;
fn setup() -> Config {
 let mut c=Config::default();
 for (id,x,items) in [("a",10.,vec!["a1","shared"]),("b",400.,vec!["b1","shared"]),("c",760.,vec!["c1"])] {
  c.fences.push(Fence{id:id.into(),title:format!("title-{id}"),x,y:20.,w:200.,h:150.,style:Style::new(),category:Some(id.into()),monitor:"display".into(),collapsed:false,hidden:false,items:items.into_iter().map(String::from).collect()});
 } c
}
fn valid() -> HashSet<String> { ["a1","shared","b1","c1","new","renamed"].into_iter().map(String::from).collect() }
#[test] fn grouping_preserves_members_and_detaches_only_dragged_member() {
 let mut c=setup(); let before=serde_json::to_value(&c.fences).unwrap();
 let g=fm::group(&mut c,"a","b").unwrap();
 assert_eq!(before,serde_json::to_value(&c.fences).unwrap());
 assert!(!fm::visible_fence(&c,&c.fences[0])); assert_eq!(fm::geometry(&c,&c.fences[0]),[400.,20.,200.,150.]);
 fm::select(&mut c,&g,"a").unwrap(); assert!(fm::visible_fence(&c,&c.fences[0]));
 fm::group(&mut c,"c","a").unwrap(); fm::detach(&mut c,"a",[-50.,80.]).unwrap();
 assert_eq!(c.fence_groups[0].members,["b","c"]); assert_eq!(c.fences[0].x,-50.);
 fm::detach(&mut c,"c",[30.,80.]).unwrap(); assert!(c.fence_groups.is_empty()); assert_eq!(c.fences[1].x,400.);
}
#[test] fn merge_restore_uses_live_properties_and_does_not_resurrect_moved_items() {
 let mut c=setup(); let r=fm::merge_contents(&mut c,"a","b").unwrap();
 assert_eq!(c.fences[0].items,["b1","shared","a1"]);
 c.fences[0].items.retain(|id|id!="a1"); c.fences[0].items.push("new".into());
 c.fences[1].items.push("a1".into()); c.fences[0].w=230.; c.fences[0].category=Some("edited".into());
 c.fences[0].title="changed".into();
 fm::restore(&mut c,&r,&valid(),[-100.,-100.,1000.,700.]).unwrap();
 let a=c.fences.iter().find(|f|f.id=="a").unwrap(); let b=c.fences.iter().find(|f|f.id=="b").unwrap();
 assert!(a.items.is_empty()); assert_eq!(b.items,["b1","shared","new"]); assert_eq!(a.title,"title-a");assert_eq!(b.title,"title-b");assert_eq!(a.w,230.);assert_eq!(a.category.as_deref(),Some("edited"));assert!(c.merge_restores.is_empty());
 assert!(a.x>=-100. && a.x+a.w<=900. && a.y>=-100. && a.y+a.h<=600.);
}
#[test] fn restore_is_lifo_atomic_and_survives_reload() {
 let mut c=setup();let first=fm::merge_contents(&mut c,"a","b").unwrap();let second=fm::merge_contents(&mut c,"c","b").unwrap();
 c=serde_json::from_value(serde_json::to_value(c).unwrap()).unwrap();let before=serde_json::to_value(&c).unwrap();
 assert!(fm::restore(&mut c,&first,&valid(),[0.,0.,1000.,700.]).is_err());assert_eq!(before,serde_json::to_value(&c).unwrap());
 fm::restore(&mut c,&second,&valid(),[0.,0.,1000.,700.]).unwrap();fm::restore(&mut c,&first,&valid(),[0.,0.,1000.,700.]).unwrap();assert_eq!(c.fences.len(),3);
}
#[test] fn rename_delete_and_scanning_keep_merge_metadata_consistent() {
 let mut c=setup();fm::group(&mut c,"a","b").unwrap();
 let items:Vec<Item>=valid().into_iter().map(|id|Item{learning_key:None,name:id.clone(),path:id.clone(),id,kind:"file".into(),ext:"".into(),category:"other".into(),icon:None}).collect();
 state::collect_inbox(&mut c,&items,"display",[1000.,700.]);assert!(c.fences[0].items.contains(&"a1".into()));
 fm::detach(&mut c,"a",[10.,20.]).unwrap();let r=fm::merge_contents(&mut c,"a","b").unwrap();let old=items.iter().find(|i|i.id=="a1").unwrap();let mut new=old.clone();new.id="renamed".into();state::remap_item(&mut c,old,&new);
 assert!(c.merge_restores[0].source.items.contains(&"renamed".into()));
 fm::restore(&mut c,&r,&valid(),[0.,0.,1000.,700.]).unwrap();fm::group(&mut c,"a","b").unwrap();state::delete_fence(&mut c,"a").unwrap();assert!(c.fence_groups.is_empty());
 fm::merge_contents(&mut c,"c","b").unwrap();state::delete_fence(&mut c,"b").unwrap();assert!(c.merge_restores.is_empty());
}
#[test] fn regular_organization_preserves_inactive_group_inbox_assignment() {
 let mut c=setup();c.initialized=true;c.fences[0].category=Some("inbox".into());fm::group(&mut c,"a","b").unwrap();
 let item=Item{learning_key:None,id:"a1".into(),name:"a1".into(),path:"a1".into(),kind:"file".into(),ext:"".into(),category:"other".into(),icon:None};
 state::organize(&mut c,&[item],"display",[1000.,700.],false);assert_eq!(c.fences[0].items,["a1"]);
}
#[test] fn crowded_restore_stays_inside_negative_work_area_and_avoids_neighbor() {
 let mut c=setup();c.fences[1].x=-100.;c.fences[1].y=-80.;c.fences[2].x=112.;c.fences[2].y=-80.;
 let r=fm::merge_contents(&mut c,"a","b").unwrap();fm::restore(&mut c,&r,&valid(),[-100.,-80.,650.,450.]).unwrap();
 for f in &c.fences {assert!(f.x>=-100.&&f.y>=-80.&&f.x+f.w<=550.&&f.y+f.h<=370.);}
 for (i,a) in c.fences.iter().enumerate(){for b in c.fences.iter().skip(i+1){assert!(a.x+a.w<=b.x||b.x+b.w<=a.x||a.y+a.h<=b.y||b.y+b.h<=a.y);}}
}
#[test] fn invalid_operations_leave_config_unchanged_and_legacy_defaults_work() {
 let mut c:Config=serde_json::from_str("{}").unwrap();assert!(c.fence_groups.is_empty()&&c.merge_restores.is_empty());c=setup();
 let before=serde_json::to_value(&c).unwrap();assert!(fm::group(&mut c,"a","a").is_err());assert!(fm::merge_contents(&mut c,"a","missing").is_err());assert!(fm::detach(&mut c,"a",[f64::NAN,0.]).is_err());assert_eq!(before,serde_json::to_value(c).unwrap());
}
#[test] fn group_to_group_moves_only_active_source_and_stays_flat() {
 let mut c=setup();let mut d=c.fences[2].clone();d.id="d".into();d.title="d".into();c.fences.push(d);
 let first=fm::group(&mut c,"a","b").unwrap();fm::select(&mut c,&first,"a").unwrap();
 let second=fm::group(&mut c,"c","d").unwrap();fm::group(&mut c,"a","d").unwrap();
 assert_eq!(c.fence_groups.len(),1);assert_eq!(c.fence_groups[0].id,second);assert_eq!(c.fence_groups[0].members,["d","c","a"]);assert!(fm::visible_fence(&c,&c.fences[1]));
 let before=serde_json::to_value(&c).unwrap();assert!(fm::group(&mut c,"c","d").is_err());assert_eq!(before,serde_json::to_value(&c).unwrap());
}
#[test] fn grouped_source_restores_independent_and_target_keeps_group() {
 let mut c=setup();let mut d=c.fences[2].clone();d.id="d".into();c.fences.push(d);
 let g=fm::group(&mut c,"a","b").unwrap();fm::select(&mut c,&g,"a").unwrap();let target=fm::group(&mut c,"c","d").unwrap();
 let record=fm::merge_contents(&mut c,"a","d").unwrap();fm::restore(&mut c,&record,&valid(),[0.,0.,1200.,800.]).unwrap();
 assert_eq!(c.fence_groups.len(),1);assert_eq!(c.fence_groups[0].id,target);assert_eq!(c.fence_groups[0].members,["d","c"]);
 assert!(fm::visible_fence(&c,c.fences.iter().find(|f|f.id=="a").unwrap()));
}
#[test] fn missing_files_are_not_restored_and_tiny_workarea_contains_every_fence() {
 let mut c=setup();let record=fm::merge_contents(&mut c,"a","b").unwrap();let valid=["b1".into()].into_iter().collect();
 fm::restore(&mut c,&record,&valid,[-30.,-40.,100.,80.]).unwrap();
 assert!(c.fences.iter().find(|f|f.id=="a").unwrap().items.is_empty());
 for f in &c.fences{assert!(f.x>=-30.&&f.y>=-40.&&f.x+f.w<=70.&&f.y+f.h<=40.);}
}
#[test] fn restore_recovers_original_target_order_before_new_additions() {
 let mut c=setup();let record=fm::merge_contents(&mut c,"a","b").unwrap();
 c.fences.iter_mut().find(|f|f.id=="b").unwrap().items=vec!["new".into(),"shared".into(),"a1".into(),"b1".into()];
 fm::restore(&mut c,&record,&valid(),[0.,0.,1000.,700.]).unwrap();
 assert_eq!(c.fences.iter().find(|f|f.id=="b").unwrap().items,["b1","shared","new"]);
 assert_eq!(c.fences.iter().find(|f|f.id=="a").unwrap().items,["a1"]);
}
#[test] fn restore_splits_toward_direction_with_most_remaining_space() {
 let mut c=setup();c.fences.retain(|f|f.id!="c");c.fences[1].x=700.;c.fences[1].y=50.;
 let record=fm::merge_contents(&mut c,"a","b").unwrap();fm::restore(&mut c,&record,&valid(),[0.,0.,1000.,400.]).unwrap();
 let a=c.fences.iter().find(|f|f.id=="a").unwrap();let b=c.fences.iter().find(|f|f.id=="b").unwrap();assert!(a.x+a.w<=b.x);
}
#[test] fn full_organization_places_a_group_once_and_preserves_inactive_geometry() {
 let mut c=setup();let g=fm::group(&mut c,"a","b").unwrap();
 let inactive=[c.fences[0].x,c.fences[0].y,c.fences[0].w,c.fences[0].h];
 state::organize(&mut c,&[],"display",[1000.,700.],true);
 assert_eq!(fm::geometry(&c,&c.fences[1]),[24.,24.,320.,260.]);
 assert_eq!([c.fences[0].x,c.fences[0].y,c.fences[0].w,c.fences[0].h],inactive);
 assert_eq!(c.fences[2].x,356.);assert_eq!(c.fence_groups[0].id,g);
}
#[test] fn restore_on_primary_includes_orphan_and_group_container_neighbors_only() {
 let mut c=setup();c.fences[1].monitor="disconnected".into();c.fences[1].x=24.;c.fences[1].y=24.;
 c.fences[2].x=-50.;c.fences[2].y=-80.;
 let mut d=c.fences[2].clone();d.id="d".into();d.monitor="secondary".into();c.fences.push(d);
 let record=fm::merge_contents(&mut c,"a","b").unwrap();
 let monitors=vec![Monitor{name:"display".into(),label:"".into(),x:0,y:0,w:1000,h:700,scale:1.,work:[0.,0.,1000.,700.],primary:true},Monitor{name:"secondary".into(),label:"".into(),x:1000,y:0,w:1000,h:700,scale:1.,work:[0.,0.,1000.,700.],primary:false}];
 fm::restore_on_display(&mut c,&record,&valid(),[0.,0.,1000.,700.],"display",&monitors).unwrap();
 let neighbor=c.fences.iter().find(|f|f.id=="c").unwrap();assert!(neighbor.x>=0.&&neighbor.y>=0.);
 let other=c.fences.iter().find(|f|f.id=="d").unwrap();assert_eq!([other.x,other.y],[-50.,-80.]);
}
