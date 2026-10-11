use deskbox_reconstructed::{config, model::*, pixels, state};
use std::{fs, path::PathBuf};

#[test]
fn shell_pixels_preserve_straight_alpha_and_round_premultiplied_edges() {
    let mut straight = vec![30, 80, 220, 100, 0, 0, 0, 0];
    pixels::shell_bgra_to_rgba(&mut straight).unwrap();
    assert_eq!(straight, [220, 80, 30, 100, 0, 0, 0, 0]);
    let mut premult = vec![10, 25, 50, 100, 0, 0, 0, 0];
    pixels::shell_bgra_to_rgba(&mut premult).unwrap();
    assert_eq!(premult, [128, 64, 26, 100, 0, 0, 0, 0]);
    let mut opaque_thumbnail = vec![0, 0, 0, 0, 30, 80, 220, 0];
    pixels::shell_bgra_to_rgba(&mut opaque_thumbnail).unwrap();
    assert_eq!(opaque_thumbnail, [0,0,0,255,220,80,30,255]);
    assert!(pixels::shell_bgra_to_rgba(&mut [0, 1, 2]).is_err());
}

#[test]
fn item_ids_and_name_learning_match_original_configuration_encoding() {
    assert_eq!(state::item_id(r"C:\Users\Example\Desktop\中文.TXT"), r"c:\users\example\desktop\中文.txt");
    assert_eq!(state::item_id("::{645FF040-5081-101B-9F08-00AA002F954E}"), "::{645ff040-5081-101b-9f08-00aa002f954e}");
    let item = Item { learning_key: None, id: "id".into(), name: "报告.TXT".into(), path: "path".into(), kind:"file".into(), ext:"txt".into(), category:"document".into(), icon:None };
    assert_eq!(state::learning_key(&item), "n:报告.txt");
}

#[test]
fn upgrade_imports_original_preferences_and_backgrounds_without_touching_source() {
    let root = std::env::temp_dir().join(format!("deskbox-fidelity-{}", uuid::Uuid::new_v4()));
    let original=root.join("original"); let target=root.join("rebuilt"); fs::create_dir_all(&original).unwrap(); fs::create_dir_all(&target).unwrap();
    let background=original.join("photo.png"); fs::write(&background, b"background fixture").unwrap();
    let source=serde_json::json!({"version":1,"theme":"neon","settings":{"hideDesktopIcons":true,"sysmonIntervalMs":1000},
        "fences":[{"id":"fence-original","title":"原框","category":"document","monitor":"display","x":24.,"y":24.,"w":300.,"h":420.,
            "style":{"image":background.to_string_lossy()},"items":["c:\\desktop\\report.txt"]}],
        "widgets":[{"id":"clock-original","kind":"clock","monitor":"display","x":348.,"y":60.,"style":{}}],
        "learned":{"n:report.txt":"document"},"initialized":true});
    let source_bytes=serde_json::to_vec(&source).unwrap(); fs::write(original.join("config.json"), &source_bytes).unwrap();
    let legacy=br#"{"theme":"glass-dark","settings":{"hideDesktopIcons":false},"initialized":true}"#;
    fs::write(target.join("config.json"), legacy).unwrap();
    let imported = config::upgrade_legacy(&target, Some(&original)).unwrap();
    assert_eq!(imported.theme,"neon"); assert!(imported.settings.hide_desktop_icons); assert_eq!(imported.widgets.len(),1);
    assert_eq!(imported.fences[0].items,[r"c:\desktop\report.txt"]);
    let image=PathBuf::from(imported.fences[0].style["image"].as_str().unwrap());
    assert!(image.starts_with(&target)); assert_eq!(fs::read(image).unwrap(),b"background fixture");
    assert_eq!(fs::read(original.join("config.json")).unwrap(),source_bytes);
    assert_eq!(fs::read(target.join("config.before-native-fidelity.json")).unwrap(),legacy);
    let saved:serde_json::Value=serde_json::from_slice(&fs::read(target.join("config.json")).unwrap()).unwrap(); assert_eq!(saved["version"],1);
    assert_eq!(config::upgrade_legacy(&target,Some(&original)).unwrap().theme,"neon");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn legacy_upgrade_without_original_enables_requested_desktop_hiding() {
    let root=std::env::temp_dir().join(format!("deskbox-fidelity-{}",uuid::Uuid::new_v4())); fs::create_dir_all(&root).unwrap();
    fs::write(root.join("config.json"),br#"{"settings":{"hideDesktopIcons":false}}"#).unwrap();
    assert!(config::upgrade_legacy(&root,None).unwrap().settings.hide_desktop_icons);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn legacy_hash_assignments_and_learning_migrate_without_losing_order() {
    let item = Item { learning_key: None, id: state::item_id(r"C:\Desktop\Report.TXT"), name:"Report.TXT".into(), path:r"C:\Desktop\Report.TXT".into(), kind:"file".into(), ext:"txt".into(), category:"document".into(), icon:None };
    let mut c = Config::default();
    let fence = state::create_fence(&mut c,"display",[24.,24.],"Manual","tools");
    c.fences[0].items = vec!["previous".into(),state::legacy_item_id(&item.path),"next".into()];
    c.learned.insert("ext:txt".into(),"tools".into());
    state::adopt_legacy_ids(&mut c,&[item.clone()]);
    assert_eq!(c.fences[0].id,fence);
    assert_eq!(c.fences[0].items,vec!["previous".to_string(),item.id.clone(),"next".to_string()]);
    assert_eq!(c.learned["n:report.txt"],"tools");
    c.learned.insert("n:report.txt".into(),"document".into());
    state::adopt_legacy_ids(&mut c,&[item]);
    assert_eq!(c.learned["n:report.txt"],"document");
}
