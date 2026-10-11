use crate::{learning, classify, model::*, win};
use std::{collections::BTreeSet, fs, path::{Path, PathBuf}};
use windows::{core::{Interface, PCWSTR}, Win32::{Foundation::HWND, System::Com::*, UI::Shell::*}};

struct Apartment(bool);
impl Apartment {
    fn enter() -> Result<Self, String> {
        let status = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if status.is_ok() { Ok(Self(true)) }
        else if status.0 == -2147417850 { Ok(Self(false)) } // already initialized in another mode
        else { Err(status.to_string()) }
    }
}
impl Drop for Apartment { fn drop(&mut self) { if self.0 { unsafe { CoUninitialize(); } } } }

pub fn stable_id(path: &str) -> String {
    crate::state::item_id(path)
}

pub fn desktop_paths() -> Result<Vec<PathBuf>, String> {
    unsafe {
        let mut paths = vec![];
        for folder in [FOLDERID_Desktop, FOLDERID_PublicDesktop] {
            let text = SHGetKnownFolderPath(&folder, KF_FLAG_DEFAULT, None).map_err(|e| e.to_string())?;
            let path = text.to_string().map_err(|e| e.to_string());
            CoTaskMemFree(Some(text.0.cast()));
            paths.push(PathBuf::from(path?));
        }
        Ok(paths)
    }
}

fn shortcut_target(path: &Path) -> Option<String> {
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let file: IPersistFile = link.cast().ok()?;
        file.Load(PCWSTR(win::wide(&path.to_string_lossy()).as_ptr()), STGM_READ).ok()?;
        let mut buffer = vec![0u16; 32768];
        link.GetPath(&mut buffer, std::ptr::null_mut(), SLGP_RAWPATH.0 as u32).ok()?;
        let size = buffer.iter().position(|x| *x == 0)?;
        Some(String::from_utf16_lossy(&buffer[..size]))
    }
}

fn cached_icon(item: &Item, directory: &Path) -> Option<String> {
    let path = directory.join(format!("{}.png", crate::state::legacy_item_id(&item.path)));
    if !path.exists() {
        let image = win::shell_image(&item.path, item.kind == "system").ok()?;
        image.save(&path).ok()?;
    }
    Some(path.to_string_lossy().into_owned())
}

pub fn scan(cache: &Path) -> Result<Vec<Item>, String> {
    scan_paths(cache, &desktop_paths()?, true)
}

pub fn scan_paths(cache: &Path, directories: &[PathBuf], include_system: bool) -> Result<Vec<Item>, String> {
    let _apartment = Apartment::enter()?;
    fs::create_dir_all(cache).map_err(|e| e.to_string())?;
    let mut items = vec![];
    let mut known = BTreeSet::new();
    for directory in directories {
        if !directory.exists() { continue; }
        let entries = fs::read_dir(&directory).map_err(|e| format!("Cannot scan {}: {e}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.eq_ignore_ascii_case("desktop.ini") || name.starts_with('.') { continue; }
            let metadata = entry.metadata().map_err(|e| e.to_string())?;
            let ext = path.extension().map(|x| x.to_string_lossy().to_lowercase()).unwrap_or_default();
            let kind = if metadata.is_dir() { "folder" } else if ext == "lnk" || ext == "url" { "shortcut" } else { "file" };
            let target = if ext == "lnk" { shortcut_target(&path).unwrap_or_default() } else { String::new() };
            let url = if ext == "url" { learning::internet_url(&fs::read_to_string(&path).unwrap_or_default()) } else { String::new() };
            let display = if kind == "shortcut" { path.file_stem().unwrap_or_default().to_string_lossy().into_owned() } else { name };
            let path = path.to_string_lossy().into_owned();
            let id = stable_id(&path);
            if !known.insert(id.clone()) { continue; }
            let mut item = Item { learning_key: Some(learning::key(&display, &target, &url)), id, name: display.clone(), path, kind: kind.into(), ext,
                category: classify::classify(&display, if target.is_empty() { &url } else { &target }, kind).into(), icon: None };
            item.icon = cached_icon(&item, cache);
            items.push(item);
        }
    }
    let key = windows_registry::CURRENT_USER.open(r"Software\Microsoft\Windows\CurrentVersion\Explorer\HideDesktopIcons\NewStartPanel").ok();
    for (guid, name, hidden) in [
        ("{20D04FE0-3AEA-1069-A2D8-08002B30309D}", "此电脑", true),
        ("{645FF040-5081-101B-9F08-00AA002F954E}", "回收站", false),
        ("{59031a47-3f72-44a7-89c5-5595fe6b30ee}", "个人文件夹", true),
        ("{F02C1A0D-BE21-4350-88B0-7367FC96EF3C}", "网络", true),
        ("{5399E694-6CE5-4D6C-8FCE-1D8870FDCBA0}", "控制面板", true),
    ] {
        if !include_system { continue; }
        if key.as_ref().and_then(|k| k.get_u32(guid).ok()).unwrap_or(hidden as u32) != 0 { continue; }
        let path = format!("::{guid}");
        let mut item = Item { learning_key: None, id: stable_id(&path), name: name.into(), path,
            kind: "system".into(), ext: String::new(), category: "system".into(), icon: None };
        item.icon = cached_icon(&item, cache);
        items.push(item);
    }
    items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(items)
}

pub fn action(item: &Item, action: &str, owner: win::Hwnd) -> Result<(), String> {
    let _apartment = Apartment::enter()?;
    unsafe {
        if action == "recycle" {
            if item.kind == "system" { return Err("System icons cannot be recycled".into()); }
            let operation: IFileOperation = CoCreateInstance(&FileOperation, None, CLSCTX_INPROC_SERVER).map_err(|e| e.to_string())?;
            operation.SetOwnerWindow(HWND(owner as *mut _)).map_err(|e| e.to_string())?;
            operation.SetOperationFlags(FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_NOERRORUI | FOFX_RECYCLEONDELETE | FOFX_EARLYFAILURE).map_err(|e| e.to_string())?;
            let shell_item: IShellItem = SHCreateItemFromParsingName(PCWSTR(win::wide(&item.path).as_ptr()), None).map_err(|e| e.to_string())?;
            operation.DeleteItem(&shell_item, None).map_err(|e| e.to_string())?;
            operation.PerformOperations().map_err(|e| e.to_string())?;
            if operation.GetAnyOperationsAborted().map_err(|e| e.to_string())?.as_bool() {
                return Err("Recycle operation was cancelled".into());
            }
            return Ok(());
        }
        let verb = win::wide(match action { "open" | "reveal" => "open", "properties" => "properties", _ => return Err("Unknown item action".into()) });
        let file = win::wide(if action == "reveal" { "explorer.exe" } else { &item.path });
        let parameters = win::wide(&format!("/select,\"{}\"", item.path));
        let mut info = SHELLEXECUTEINFOW { cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: if action == "properties" { 0x0c } else { 0 }, hwnd: HWND(owner as *mut _),
            lpVerb: PCWSTR(verb.as_ptr()), lpFile: PCWSTR(file.as_ptr()),
            lpParameters: if action == "reveal" { PCWSTR(parameters.as_ptr()) } else { PCWSTR::null() },
            nShow: 1, ..Default::default() };
        ShellExecuteExW(&mut info).map_err(|e| e.to_string())
    }
}

pub fn wallpaper(cache: &Path) -> Option<Wallpaper> {
    let source = PathBuf::from(win::wallpaper_path()?);
    let image = image::open(&source).ok()?;
    let path = cache.join("wallpaper.png");
    image.save(&path).ok()?;
    let style = windows_registry::CURRENT_USER.open(r"Control Panel\Desktop").ok()
        .and_then(|key| key.get_string("WallpaperStyle").ok()).unwrap_or_else(|| "10".into());
    Some(Wallpaper { path: path.to_string_lossy().into_owned(), width: image.width(), height: image.height(),
        style: match style.as_str() { "10" => "fill", "6" => "fit", "2" => "stretch", "22" => "span", _ => "center" }.into(),
        color: "#202020".into() })
}
