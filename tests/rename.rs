use deskbox_reconstructed::{model::*, rename, state};
use std::{fs, path::PathBuf};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self { let p = std::env::temp_dir().join(format!("deskbox-rename-{}", uuid::Uuid::new_v4())); fs::create_dir_all(&p).unwrap(); Self(p) }
    fn item(&self, name: &str, kind: &str) -> Item {
        Item { learning_key: None, id: "old-id".into(), name: name.into(), path: self.0.join(name).to_string_lossy().into(), kind: kind.into(),
            ext: name.rsplit('.').next().unwrap_or("").into(), category: "document".into(), icon: None }
    }
}
impl Drop for Fixture { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

#[test]
fn rejects_invalid_windows_names_before_touching_files() {
    let fixture = Fixture::new(); let item = fixture.item("old.txt", "file"); fs::write(&item.path, "original").unwrap();
    for name in ["", " ", "../escape", "a/b", "a\\b", "a:b", "a*", "a?", "a\"", "a<", "a>", "a|", "a\n", "CON", "nul.txt", "COM1.log", "lpt9", "COM¹.txt", "last.", "last "] {
        assert!(rename::rename_file(&item, name).is_err(), "accepted {name:?}");
        assert_eq!(fs::read_to_string(&item.path).unwrap(), "original");
    }
}

#[test]
fn renames_unicode_file_without_changing_content_and_rejects_collision() {
    let fixture = Fixture::new(); let item = fixture.item("old.txt", "file"); fs::write(&item.path, "original").unwrap();
    fs::write(fixture.0.join("occupied.txt"), "other").unwrap();
    assert!(rename::rename_file(&item, "occupied.txt").is_err());
    assert_eq!(fs::read_to_string(fixture.0.join("occupied.txt")).unwrap(), "other");
    let path = rename::rename_file(&item, "新名称.txt").unwrap();
    assert!(!PathBuf::from(&item.path).exists());
    assert_eq!(fs::read_to_string(path).unwrap(), "original");
}

#[test]
fn folders_and_shortcuts_rename_with_shortcut_extension_preserved() {
    let fixture = Fixture::new(); let folder = fixture.item("folder", "folder"); fs::create_dir(&folder.path).unwrap();
    fs::write(PathBuf::from(&folder.path).join("inside.txt"), "inside").unwrap();
    let path = rename::rename_file(&folder, "文件夹").unwrap();
    assert_eq!(fs::read_to_string(path.join("inside.txt")).unwrap(), "inside");
    for ext in ["lnk", "url"] {
        let link = fixture.item(&format!("old.{ext}"), "shortcut"); fs::write(&link.path, "link-data").unwrap();
        let path = rename::rename_file(&link, "新快捷方式").unwrap();
        assert_eq!(path.extension().unwrap(), ext);
        assert_eq!(fs::read_to_string(path).unwrap(), "link-data");
    }
}

#[test]
fn supports_case_only_and_unchanged_names_but_rejects_system_icons() {
    let fixture = Fixture::new(); let item = fixture.item("sample.txt", "file"); fs::write(&item.path, "x").unwrap();
    assert_eq!(rename::rename_file(&item, "sample.txt").unwrap(), PathBuf::from(&item.path));
    let path = rename::rename_file(&item, "SAMPLE.txt").unwrap();
    assert_eq!(path.file_name().unwrap(), "SAMPLE.txt");
    let mut system = item; system.kind = "system".into();
    assert!(rename::rename_file(&system, "name").is_err());
}

#[test]
fn remapping_keeps_fence_order_and_transfers_learned_shortcut_name() {
    let fixture = Fixture::new(); let mut old = fixture.item("old.lnk", "shortcut"); old.name = "old".into();
    let mut new = old.clone(); new.id = "new-id".into(); new.name = "新快捷方式".into(); new.path = fixture.0.join("新快捷方式.lnk").to_string_lossy().into();
    let mut c = Config::default(); state::create_fence(&mut c, "display", [30., 30.], "手动分类", "dev");
    c.fences[0].items = vec!["before".into(), old.id.clone(), "after".into()];
    c.learned.insert(state::learning_key(&old), "dev".into());
    state::remap_item(&mut c, &old, &new);
    assert_eq!(c.fences[0].items, ["before", "new-id", "after"]);
    assert_eq!(c.learned.get(&state::learning_key(&new)).unwrap(), "dev");
    assert!(!c.learned.contains_key(&state::learning_key(&old)));
}

#[test]
fn remapping_folder_transfers_path_learning_without_changing_shared_extension_learning() {
    let fixture = Fixture::new();
    let old = fixture.item("folder", "folder");
    let mut new = old.clone(); new.id = "folder-new".into(); new.name = "renamed".into(); new.path = fixture.0.join("renamed").to_string_lossy().into();
    let mut c = Config::default(); c.learned.insert(state::learning_key(&old), "dev".into());
    state::remap_item(&mut c, &old, &new);
    assert_eq!(c.learned.get(&state::learning_key(&new)).unwrap(), "dev");
    let old = fixture.item("old.txt", "file");
    let mut new = old.clone(); new.ext = "md".into(); new.name = "new.md".into();
    c.learned.insert("ext:txt".into(), "office".into());
    state::remap_item(&mut c, &old, &new);
    assert_eq!(c.learned.get("ext:txt").unwrap(), "office");
    assert!(!c.learned.contains_key("ext:md"));
}

#[test]
fn extensionless_file_rename_transfers_per_name_learning() {
    let fixture = Fixture::new();
    let mut old = fixture.item("README", "file"); old.ext.clear();
    let mut new = old.clone(); new.id = "notes".into(); new.name = "NOTES".into(); new.path = fixture.0.join("NOTES").to_string_lossy().into();
    let mut c = Config::default(); c.learned.insert(state::learning_key(&old), "dev".into());
    state::remap_item(&mut c, &old, &new);
    assert_eq!(c.learned.get("n:notes"), Some(&"dev".to_string()));
    assert!(!c.learned.contains_key("n:readme"));
}
