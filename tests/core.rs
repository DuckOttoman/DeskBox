use deskbox_reconstructed::{classify, config, model::*, state};
use std::collections::BTreeMap;

fn file(id: &str, name: &str, kind: &str) -> Item {
    Item { learning_key: None, id: id.into(), name: name.into(), path: format!("C:\\Desktop\\{name}"),
        kind: kind.into(), ext: name.rsplit('.').next().unwrap_or("").to_lowercase(),
        category: classify::classify(name, name, kind).into(), icon: None }
}

#[test]
fn disabled_auto_organization_collects_new_files_then_manual_organization_places_them() {
    let mut c = Config::default();
    c.initialized = true;
    c.settings.auto_organize = false;
    let old = file("old", "old.md", "file");
    let new = file("new", "new.png", "file");
    state::create_fence(&mut c, "display", [24., 24.], "自定义", "");
    c.fences[0].items = vec!["old".into(), "deleted".into()];
    let items = vec![old, new];
    state::collect_inbox(&mut c, &items, "display", [1000., 800.]);
    state::collect_inbox(&mut c, &items, "display", [1000., 800.]);
    assert_eq!(c.fences[0].items, ["old"]);
    let inbox = c.fences.iter().find(|f| f.category.as_deref() == Some("inbox")).unwrap();
    assert_eq!(inbox.title, "新文件");
    assert_eq!(inbox.items, ["new"]);
    state::organize(&mut c, &items, "display", [1000., 800.], false);
    assert_eq!(c.fences[0].items, ["old"]);
    assert!(c.fences.iter().find(|f| f.category.as_deref() == Some("inbox")).unwrap().items.is_empty());
    assert_eq!(c.fences.iter().find(|f| f.category.as_deref() == Some("image")).unwrap().items, ["new"]);
}

#[test]
fn recovered_extensions_and_keyword_rules_classify_known_files() {
    assert_eq!(classify::classify("README.md", "", "file"), "document");
    assert_eq!(classify::classify("photo.HEIC", "", "file"), "image");
    assert_eq!(classify::classify("Visual Studio Code.lnk", "Code.exe", "shortcut"), "dev");
    assert_eq!(classify::classify("Steam.lnk", "steam.exe", "shortcut"), "game");
    assert_eq!(classify::classify("Microsoft Word.lnk", "WINWORD.EXE", "shortcut"), "office");
    assert_eq!(classify::classify("projects", "", "folder"), "folder");
    assert_eq!(classify::classify("此电脑", "", "system"), "system");
    assert_eq!(classify::classify("unrecognized.zzz", "", "file"), "other");
}

#[test]
fn move_item_keeps_single_assignment_and_learns_manual_category() {
    let mut c = Config::default();
    let item = file("one", "README.md", "file");
    let a = state::create_fence(&mut c, "display", [30., 30.], "文档", "document");
    let b = state::create_fence(&mut c, "display", [90., 90.], "开发", "dev");
    c.fences.iter_mut().find(|f| f.id == a).unwrap().items.push(item.id.clone());
    state::move_item(&mut c, &item, &b, Some(0)).unwrap();
    assert!(!c.fences.iter().find(|f| f.id == a).unwrap().items.contains(&item.id));
    assert_eq!(c.fences.iter().find(|f| f.id == b).unwrap().items, ["one"]);
    assert_eq!(c.learned.get(&state::learning_key(&item)), Some(&"dev".to_string()));
    let before = serde_json::to_value(&c).unwrap();
    assert!(state::move_item(&mut c, &item, "missing", None).is_err());
    assert_eq!(serde_json::to_value(&c).unwrap(), before);
}

#[test]
fn deleting_fence_rehomes_items_without_deleting_files() {
    let mut c = Config::default();
    let a = state::create_fence(&mut c, "display", [30., 30.], "文档", "document");
    c.fences[0].items = vec!["one".into(), "two".into()];
    state::delete_fence(&mut c, &a).unwrap();
    let other = c.fences.iter().find(|f| f.category.as_deref() == Some("other")).unwrap();
    assert_eq!(other.items, ["one", "two"]);
}

#[test]
fn organize_preserves_manual_assignments_unless_full_and_uses_learned_choices() {
    let mut c = Config::default();
    let item = file("one", "README.md", "file");
    let custom = state::create_fence(&mut c, "display", [0., 0.], "自定义", "");
    c.fences[0].items.push("one".into());
    c.learned.insert(state::learning_key(&item), "dev".into());
    state::organize(&mut c, &[item.clone()], "display", [1000., 800.], false);
    assert_eq!(c.fences.iter().find(|f| f.id == custom).unwrap().items, ["one"]);
    state::organize(&mut c, &[item], "display", [1000., 800.], true);
    assert!(c.fences.iter().find(|f| f.category.as_deref() == Some("dev")).unwrap().items.contains(&"one".into()));
}

#[test]
fn config_save_roundtrip_preserves_unicode_and_corrupt_file_is_reported() {
    let dir = std::env::temp_dir().join(format!("deskbox-core-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("config.json");
    let mut c = Config::default();
    state::create_fence(&mut c, "display", [40., 50.], "中文资料", "document");
    c.learned = BTreeMap::from([("md".into(), "document".into())]);
    config::save(&path, &c).unwrap();
    assert_eq!(serde_json::to_value(config::load(&path).unwrap()).unwrap(), serde_json::to_value(&c).unwrap());
    assert!(!path.with_extension("json.tmp").exists());
    std::fs::write(&path, "invalid").unwrap();
    assert!(config::load(&path).is_err());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "invalid");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn frontend_contract_serializes_camel_case_settings_and_state() {
    let c = serde_json::to_value(Config::default()).unwrap();
    assert_eq!(c["fencesVisible"], true);
    assert!(c["settings"]["hideDesktopIcons"].is_boolean());
    assert!(c["settings"]["sysmonIntervalMs"].is_number());
    assert!(c["fences"].is_array());
    assert!(c["widgets"].is_array());
    assert!(c["learned"].is_object());
}

#[test]
fn newly_detected_category_uses_free_space_instead_of_covering_an_existing_fence() {
    let mut c = Config::default();
    let document = file("one", "README.md", "file");
    state::organize(&mut c, &[document.clone()], "display", [1000., 800.], false);
    let before = c.fences[0].clone();
    let photo = file("two", "photo.png", "file");
    state::organize(&mut c, &[document, photo], "display", [1000., 800.], false);
    let image = c.fences.iter().find(|f| f.category.as_deref() == Some("image")).unwrap();
    assert!(image.x >= before.x + before.w || image.y >= before.y + before.h
        || image.x + image.w <= before.x || image.y + image.h <= before.y,
        "new category must not overlap the existing fence when free grid space exists");
    assert_eq!(c.fences[0].x, before.x);
    assert_eq!(c.fences[0].y, before.y);
}
