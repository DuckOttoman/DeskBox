use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::collections::BTreeMap;

pub type Style = Map<String, serde_json::Value>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub opacity: f64,
    pub frost: bool,
    pub icon_size: u32,
    pub auto_organize: bool,
    pub hide_desktop_icons: bool,
    pub sysmon_interval_ms: u64,
    pub launch_at_startup: bool,
    pub accent: Option<String>,
    pub anim_speed: f64,
}
impl Default for Settings {
    fn default() -> Self {
        Self { opacity: 1., frost: true, icon_size: 48, auto_organize: true,
            hide_desktop_icons: false, sysmon_interval_ms: 2000,
            launch_at_startup: false, accent: None, anim_speed: 1. }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fence {
    pub id: String,
    pub title: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    #[serde(default)] pub style: Style,
    pub category: Option<String>,
    pub monitor: String,
    #[serde(default)] pub collapsed: bool,
    #[serde(default)] pub hidden: bool,
    #[serde(default)] pub items: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Widget {
    pub id: String,
    pub kind: String,
    pub x: f64,
    pub y: f64,
    pub monitor: String,
    #[serde(default)] pub style: Style,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    pub version: u32,
    pub theme: String,
    pub settings: Settings,
    pub fences: Vec<Fence>,
    pub fence_groups: Vec<FenceGroup>,
    pub merge_restores: Vec<MergeRestore>,
    pub widgets: Vec<Widget>,
    pub learned: BTreeMap<String, String>,
    pub ball: Option<[f64; 2]>,
    pub fences_visible: bool,
    pub icons_hidden_by_us: bool,
    pub initialized: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self { version: 1, theme: "glass-dark".into(), settings: Settings::default(), fences: vec![],
            fence_groups: vec![], merge_restores: vec![], widgets: vec![], learned: BTreeMap::new(), ball: None, fences_visible: true,
            icons_hidden_by_us: false, initialized: false }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FenceGroup {
    pub id: String,
    pub members: Vec<String>,
    pub active: String,
    pub x: f64,
    pub y: f64,
    pub monitor: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeRestore {
    pub id: String,
    pub source: Fence,
    pub target: String,
    pub target_items: Vec<String>,
    #[serde(default)] pub target_title: String,
    pub created_at: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    #[serde(default, skip_serializing)]
    pub learning_key: Option<String>,
    pub id: String,
    pub name: String,
    pub path: String,
    pub kind: String,
    pub ext: String,
    pub category: String,
    pub icon: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Monitor {
    pub name: String,
    pub label: String,
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub scale: f64,
    pub work: [f64; 4],
    pub primary: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub order: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wallpaper {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub style: String,
    pub color: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Snapshot {
    pub config: Config,
    pub items: Vec<Item>,
    pub monitors: Vec<Monitor>,
    pub categories: Vec<Category>,
    pub wallpaper: Option<Wallpaper>,
    pub ready: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HitRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    #[serde(default)] pub r: i32,
    #[serde(default)] pub ellipse: bool,
}
