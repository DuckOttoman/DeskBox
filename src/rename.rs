use crate::model::Item;
use std::path::{Path, PathBuf};

fn validate(name: &str) -> Result<(), String> {
    if name.trim().is_empty() { return Err("名称不能为空".into()); }
    if name.encode_utf16().count() > 255 { return Err("名称过长，最多 255 个 UTF-16 字符".into()); }
    if name.ends_with(['.', ' ']) { return Err("名称不能以空格或句点结尾".into()); }
    if name.chars().any(|c| c.is_control() || "\\/:*?\"<>|".contains(c)) { return Err("名称包含 Windows 不允许的字符：\\ / : * ? \" < > |".into()); }
    let stem = name.split('.').next().unwrap_or("").trim_end().to_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$")
        || ["COM", "LPT"].iter().any(|prefix| stem.strip_prefix(prefix).is_some_and(|n| matches!(n, "1"|"2"|"3"|"4"|"5"|"6"|"7"|"8"|"9"|"¹"|"²"|"³")));
    if reserved { return Err("名称是 Windows 保留名称，请使用其他名称".into()); }
    Ok(())
}

pub fn destination(item: &Item, name: &str) -> Result<PathBuf, String> {
    if item.kind == "system" { return Err("系统图标不能重命名".into()); }
    validate(name)?;
    let source = Path::new(&item.path);
    let parent = source.parent().ok_or("无法确定文件所在目录")?;
    let mut name = name.to_string();
    if item.kind == "shortcut" {
        let ext = source.extension().and_then(|x| x.to_str()).ok_or("快捷方式扩展名无效")?;
        if !matches!(ext.to_ascii_lowercase().as_str(), "lnk" | "url") { return Err("不支持的快捷方式类型".into()); }
        let suffix = format!(".{ext}");
        if !name.to_lowercase().ends_with(&suffix.to_lowercase()) { name.push_str(&suffix); }
    }
    validate(&name)?;
    Ok(parent.join(name))
}

pub fn move_path(source: &Path, destination: &Path) -> Result<(), String> {
    #[cfg(windows)] {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "kernel32")]
        unsafe extern "system" { fn MoveFileW(old: *const u16, new: *const u16) -> i32; }
        let old: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
        let new: Vec<u16> = destination.as_os_str().encode_wide().chain(Some(0)).collect();
        // Unlike fs::rename, MoveFileW never replaces an existing destination.
        if unsafe { MoveFileW(old.as_ptr(), new.as_ptr()) } == 0 {
            let error = std::io::Error::last_os_error();
            return Err(match error.raw_os_error() {
                Some(80 | 183) => "同目录已有同名文件或文件夹，请使用其他名称".into(),
                Some(2 | 3) => "文件已不存在，请等待桌面刷新后重试".into(),
                Some(5 | 32) => "无法重命名：文件被占用或没有操作权限".into(),
                _ => format!("重命名失败：{error}"),
            });
        }
        Ok(())
    }
    #[cfg(not(windows))] { let _ = (source, destination); Err("重命名功能仅支持 Windows".into()) }
}

pub fn rename_file(item: &Item, name: &str) -> Result<PathBuf, String> {
    let destination = destination(item, name)?;
    std::fs::symlink_metadata(&item.path).map_err(|e| format!("无法读取原文件：{e}"))?;
    if Path::new(&item.path) != destination { move_path(Path::new(&item.path), &destination)?; }
    Ok(destination)
}
