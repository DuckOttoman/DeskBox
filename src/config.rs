use crate::model::Config;
use std::{fs::{self, File}, io::Write, path::Path};

pub fn upgrade_legacy(directory: &Path, original: Option<&Path>) -> Result<Config, String> {
    let path = directory.join("config.json");
    let existing = match fs::read(&path) {
        Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.to_string()),
    };
    if let Some(bytes) = &existing {
        let raw: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| format!("Invalid configuration: {e}"))?;
        if raw.get("version").and_then(|v| v.as_u64()).is_some_and(|v| v >= 1) { return load(&path); }
    }
    let original = original.filter(|p| p.join("config.json").exists());
    let mut config = if let Some(source) = original { load(&source.join("config.json"))? } else { load(&path)? };
    config.version = 1;
    config.icons_hidden_by_us = false;
    if original.is_some() {
        for fence in &mut config.fences {
            let Some(image) = fence.style.get("image").and_then(|v| v.as_str()).map(std::path::PathBuf::from) else { continue; };
            if !image.is_file() { fence.style.insert("image".into(), serde_json::Value::Null); continue; }
            let assets = directory.join("backgrounds").join("original-import");
            fs::create_dir_all(&assets).map_err(|e| e.to_string())?;
            let ext = image.extension().and_then(|s| s.to_str()).unwrap_or("png");
            let destination = assets.join(format!("{}.{}", uuid::Uuid::new_v4(), ext));
            fs::copy(&image, &destination).map_err(|e| format!("Cannot import original background: {e}"))?;
            fence.style.insert("image".into(), destination.to_string_lossy().into_owned().into());
        }
    } else {
        // The user requested the original desktop replacement behavior for this upgrade.
        config.settings.hide_desktop_icons = true;
    }
    if let Some(bytes) = existing {
        let backup = directory.join("config.before-native-fidelity.json");
        if !backup.exists() { fs::write(backup, bytes).map_err(|e| e.to_string())?; }
    }
    save(&path, &config)?;
    Ok(config)
}

pub fn load(path: &Path) -> Result<Config, String> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| format!("Invalid configuration {}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(format!("Cannot read {}: {e}", path.display())),
    }
}
pub fn save(path: &Path, config: &Config) -> Result<(), String> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    let temp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(config).map_err(|e| e.to_string())?;
    let mut file = File::create(&temp).map_err(|e| e.to_string())?;
    file.write_all(&bytes).and_then(|_| file.sync_all()).map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(&temp, path).map_err(|e| format!("Cannot replace configuration: {e}"))
}
