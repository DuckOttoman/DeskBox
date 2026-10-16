pub use deskbox_reconstructed::model;
#[path = "../src/taskbar.rs"] mod taskbar;
#[test] fn reservation_replaces_instead_of_stacking() {
 assert_eq!(taskbar::reserved_bottom([0,0,1920,1080], 84), [0,996,1920,1080]);
 assert!(taskbar::reservation_is_safe([0,0,1920,1080], [0,956,1920,996],84)==false);
}
#[test] fn fullscreen_requires_same_monitor_and_all_edges() {
 assert!(taskbar::covers_monitor([1920,0,3840,1080],[1920,0,3840,1080]));
 assert!(!taskbar::covers_monitor([0,0,1920,1080],[1920,0,3840,1080]));
 assert!(!taskbar::covers_monitor([1920,20,3840,1080],[1920,0,3840,1080]));
}
#[test] fn pid_reuse_does_not_match_owner() {
 assert!(!taskbar::same_instance(10,123,10,124));
 assert!(taskbar::same_instance(10,123,10,123));
}

#[test]
fn another_monitors_foreground_does_not_uncover_fullscreen() {
 let a=[0,0,1920,1080];let b=[1920,0,3840,1080];
 assert!(taskbar::fullscreen_from_z_order(&[[2100,100,3000,700],a],a));
 assert!(!taskbar::fullscreen_from_z_order(&[[2100,100,3000,700],a],b));
 assert!(!taskbar::fullscreen_from_z_order(&[[100,100,800,700],a],a));
}
#[test]
fn watchdog_artifacts_are_owned_by_one_toggle_session() {
 assert!(taskbar::session_matches("first", "first"));
 assert!(!taskbar::session_matches("first", "second"));
 let journal=std::path::Path::new("config/quickbar-recovery.json");
 assert_ne!(taskbar::session_ready_path(journal,"first"),taskbar::session_ready_path(journal,"second"));
}
#[test]
fn recovery_mutex_identity_is_stable_and_directory_specific() {
 assert_eq!(taskbar::recovery_mutex_key("D:/Config/quickbar-recovery.json"), taskbar::recovery_mutex_key("d:\\config\\quickbar-recovery.json"));
 assert_ne!(taskbar::recovery_mutex_key("d:/one/quickbar-recovery.json"), taskbar::recovery_mutex_key("d:/two/quickbar-recovery.json"));
}
