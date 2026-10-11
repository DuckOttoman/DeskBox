//! Windows ABI adapters. Native behavior is reconstructed from imports and frontend contracts.
use crate::model::HitRect;
use std::{ffi::c_void, ptr, sync::{Mutex, atomic::{AtomicBool, Ordering}}};

pub type Hwnd = isize;
#[repr(C)] #[derive(Default, Clone, Copy)]
pub struct Rect { pub left: i32, pub top: i32, pub right: i32, pub bottom: i32 }
#[repr(C)] #[derive(Default)] pub struct Point { pub x: i32, pub y: i32 }
static ICON_LOCK: Mutex<Option<(Hwnd, u32)>> = Mutex::new(None);
static ICON_STOPPED: AtomicBool = AtomicBool::new(false);
static HID_BY_US: AtomicBool = AtomicBool::new(false);

#[link(name = "user32")]
unsafe extern "system" {
    fn FindWindowW(class: *const u16, name: *const u16) -> Hwnd;
    fn FindWindowExW(parent: Hwnd, after: Hwnd, class: *const u16, name: *const u16) -> Hwnd;
    fn EnumWindows(callback: unsafe extern "system" fn(Hwnd, isize) -> i32, arg: isize) -> i32;
    fn EnumChildWindows(parent: Hwnd, callback: unsafe extern "system" fn(Hwnd, isize) -> i32, arg: isize) -> i32;
    fn GetClassNameW(window: Hwnd, class: *mut u16, size: i32) -> i32;
    fn SendMessageTimeoutW(window: Hwnd, message: u32, wparam: usize, lparam: isize, flags: u32, timeout: u32, result: *mut usize) -> isize;
    fn GetParent(window: Hwnd) -> Hwnd;
    fn SetParent(child: Hwnd, parent: Hwnd) -> Hwnd;
    fn GetWindowLongPtrW(window: Hwnd, index: i32) -> isize;
    fn SetWindowLongPtrW(window: Hwnd, index: i32, value: isize) -> isize;
    fn SetWindowPos(window: Hwnd, after: Hwnd, x: i32, y: i32, w: i32, h: i32, flags: u32) -> i32;
    fn GetWindowRect(window: Hwnd, rect: *mut Rect) -> i32;
    fn SetWindowRgn(window: Hwnd, region: isize, redraw: i32) -> i32;
    fn IsWindowVisible(window: Hwnd) -> i32;
    fn ShowWindow(window: Hwnd, command: i32) -> i32;
    fn GetAsyncKeyState(key: i32) -> i16;
    fn GetCursorPos(point: *mut Point) -> i32;
    fn WindowFromPoint(point: Point) -> Hwnd;
    fn GetAncestor(window: Hwnd, flags: u32) -> Hwnd;
    fn GetForegroundWindow() -> Hwnd;
    fn SetForegroundWindow(window: Hwnd) -> i32;
    fn GetFocus() -> Hwnd;
    fn SetFocus(window: Hwnd) -> Hwnd;
    fn IsChild(parent: Hwnd, child: Hwnd) -> i32;
    fn GetWindowThreadProcessId(window: Hwnd, process: *mut u32) -> u32;
    fn GetDC(window: Hwnd) -> isize;
    fn ReleaseDC(window: Hwnd, dc: isize) -> i32;
    fn DrawIconEx(dc: isize, x: i32, y: i32, icon: isize, w: i32, h: i32, step: u32, brush: isize, flags: u32) -> i32;
    fn DestroyIcon(icon: isize) -> i32;
    fn SystemParametersInfoW(action: u32, param: u32, data: *mut c_void, flags: u32) -> i32;
}
#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateRectRgn(left: i32, top: i32, right: i32, bottom: i32) -> isize;
    fn CreateRoundRectRgn(left: i32, top: i32, right: i32, bottom: i32, ew: i32, eh: i32) -> isize;
    fn CreateEllipticRgn(left: i32, top: i32, right: i32, bottom: i32) -> isize;
    fn CombineRgn(destination: isize, a: isize, b: isize, mode: i32) -> i32;
    fn DeleteObject(object: isize) -> i32;
    fn CreateCompatibleDC(dc: isize) -> isize;
    fn DeleteDC(dc: isize) -> i32;
    fn SelectObject(dc: isize, object: isize) -> isize;
    fn CreateDIBSection(dc: isize, info: *const BitmapInfo, usage: u32, bits: *mut *mut c_void, section: isize, offset: u32) -> isize;
    fn GetObjectW(object: isize, size: i32, data: *mut c_void) -> i32;
    fn GetDIBits(dc: isize, bitmap: isize, start: u32, lines: u32, bits: *mut c_void, info: *mut BitmapInfo, usage: u32) -> i32;
}

pub fn wide(value: &str) -> Vec<u16> { value.encode_utf16().chain(Some(0)).collect() }

fn class_name(window: Hwnd) -> String {
    let mut buffer = [0u16; 256];
    let length = unsafe { GetClassNameW(window, buffer.as_mut_ptr(), buffer.len() as i32) }.max(0) as usize;
    String::from_utf16_lossy(&buffer[..length])
}
unsafe extern "system" fn find_view(window: Hwnd, arg: isize) -> i32 {
    if class_name(window) == "SHELLDLL_DefView" {
        let icons = FindWindowExW(window, 0, wide("SysListView32").as_ptr(), ptr::null());
        if icons != 0 { *(arg as *mut Hwnd) = icons; return 0; }
    }
    1
}
unsafe extern "system" fn find_icons(window: Hwnd, arg: isize) -> i32 {
    // Do not traverse ordinary Explorer windows: they also contain folder views.
    if matches!(class_name(window).as_str(), "Progman" | "WorkerW") {
        EnumChildWindows(window, find_view, arg);
        if *(arg as *mut Hwnd) != 0 { return 0; }
    }
    1
}
fn icons_window() -> Hwnd {
    let mut result = 0;
    unsafe { EnumWindows(find_icons, &mut result as *mut Hwnd as isize); }
    result
}
pub fn desktop_icon_status() -> Result<(Hwnd, bool), String> {
    let window = icons_window();
    if window == 0 { return Err("无法找到 Explorer 桌面图标窗口".into()); }
    Ok((window, unsafe { IsWindowVisible(window) } != 0))
}
pub fn desktop_icons(hide: bool) -> Result<(), String> {
    let mut owner = ICON_LOCK.lock().map_err(|_| "Desktop icon lock poisoned")?;
    if hide && ICON_STOPPED.load(Ordering::SeqCst) { return Ok(()); }
    if !hide && !HID_BY_US.load(Ordering::SeqCst) { return Ok(()); }
    let (window, visible) = desktop_icon_status()?;
    let mut process = 0;
    unsafe { GetWindowThreadProcessId(window, &mut process); }
    if owner.is_some_and(|identity| identity != (window, process)) {
        *owner = None;
        HID_BY_US.store(false, Ordering::SeqCst);
    }
    unsafe {
        if hide && visible {
            ShowWindow(window, 0);
            if IsWindowVisible(window) != 0 { return Err("隐藏桌面图标失败".into()); }
            *owner = Some((window, process));
            HID_BY_US.store(true, Ordering::SeqCst);
        } else if !hide && HID_BY_US.load(Ordering::SeqCst) {
            ShowWindow(window, 5);
            if IsWindowVisible(window) == 0 { return Err("恢复桌面图标失败".into()); }
            *owner = None;
            HID_BY_US.store(false, Ordering::SeqCst);
        }
    }
    Ok(())
}
pub fn shutdown_desktop_icons() -> Result<(), String> {
    ICON_STOPPED.store(true, Ordering::SeqCst);
    desktop_icons(false)
}
pub fn icons_hidden_by_us() -> bool { HID_BY_US.load(Ordering::SeqCst) }

unsafe extern "system" fn find_worker(window: Hwnd, arg: isize) -> i32 {
    // Use the visible desktop view host, not a later wallpaper/hidden WorkerW.
    if IsWindowVisible(window) != 0
        && matches!(class_name(window).as_str(), "WorkerW" | "Progman")
        && FindWindowExW(window, 0, wide("SHELLDLL_DefView").as_ptr(), ptr::null()) != 0 {
        *(arg as *mut Hwnd) = window;
        return 0;
    }
    1
}
pub fn attach_overlay(window: Hwnd, x: i32, y: i32, w: u32, h: u32) -> Result<(), String> {
    unsafe {
        let progman = FindWindowW(wide("Progman").as_ptr(), ptr::null());
        if progman == 0 { return Err("Explorer desktop window not found".into()); }
        let mut ignored = 0;
        SendMessageTimeoutW(progman, 0x052c, 0, 0, 2, 1000, &mut ignored);
        let mut worker = 0;
        EnumWindows(find_worker, &mut worker as *mut Hwnd as isize);
        if worker == 0 { return Err("Visible Explorer desktop view host not found".into()); }
        let parent = worker;
        let mut rect = Rect::default();
        GetWindowRect(parent, &mut rect);
        let style = GetWindowLongPtrW(window, -16);
        SetWindowLongPtrW(window, -16, (style & !(0x80000000u32 as isize)) | 0x40000000);
        let ex = GetWindowLongPtrW(window, -20);
        SetWindowLongPtrW(window, -20, ex | 0x08000080); // NOACTIVATE | TOOLWINDOW
        SetParent(window, parent);
        if GetParent(window) != parent { return Err("Cannot attach overlay to desktop".into()); }
        if SetWindowPos(window, 0, x - rect.left, y - rect.top, w as i32, h as i32, 0x0010) == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    Ok(())
}

pub fn focusable(window: Hwnd, enable: bool) {
    unsafe {
        if !enable {
            let focused = GetFocus();
            if focused != 0 && (focused == window || IsChild(window, focused) != 0) { SetFocus(0); }
        }
        let style = GetWindowLongPtrW(window, -20);
        SetWindowLongPtrW(window, -20, if enable { style & !0x08000000 } else { style | 0x08000000 });
    }
}

pub fn set_region(window: Hwnd, rects: Option<Vec<HitRect>>) -> Result<(), String> {
    unsafe {
        let Some(rects) = rects else {
            if SetWindowRgn(window, 0, 1) == 0 { return Err(std::io::Error::last_os_error().to_string()); }
            return Ok(());
        };
        if rects.len() > 4096 { return Err("Too many hit regions".into()); }
        let region = CreateRectRgn(0, 0, 0, 0);
        if region == 0 { return Err("Cannot create window region".into()); }
        for rect in rects {
            if rect.w <= 0 || rect.h <= 0 { continue; }
            let right = rect.x.saturating_add(rect.w);
            let bottom = rect.y.saturating_add(rect.h);
            let part = if rect.ellipse { CreateEllipticRgn(rect.x, rect.y, right, bottom) }
                else if rect.r > 0 { CreateRoundRectRgn(rect.x, rect.y, right, bottom, rect.r.saturating_mul(2), rect.r.saturating_mul(2)) }
                else { CreateRectRgn(rect.x, rect.y, right, bottom) };
            if part == 0 || CombineRgn(region, region, part, 2) == 0 {
                if part != 0 { DeleteObject(part); }
                DeleteObject(region);
                return Err("Cannot combine window region".into());
            }
            DeleteObject(part);
        }
        if SetWindowRgn(window, region, 1) == 0 {
            DeleteObject(region);
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    Ok(()) // Windows owns region after SetWindowRgn succeeds.
}

pub fn click_outside(window: Hwnd, previous_down: &mut bool) -> bool {
    unsafe {
        let down = GetAsyncKeyState(1) < 0 || GetAsyncKeyState(2) < 0;
        let edge = down && !*previous_down;
        *previous_down = down;
        if !edge { return false; }
        let mut point = Point::default();
        if GetCursorPos(&mut point) == 0 { return false; }
        GetAncestor(WindowFromPoint(point), 2) != window
    }
}
pub fn fullscreen_foreground() -> bool {
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground == 0 { return false; }
        let mut window = Rect::default();
        if GetWindowRect(foreground, &mut window) == 0 { return false; }
        let mut class = [0u16; 128];
        GetClassNameW(foreground, class.as_mut_ptr(), 128);
        let class = String::from_utf16_lossy(&class[..class.iter().position(|x| *x == 0).unwrap_or(128)]);
        if ["Progman", "WorkerW", "Shell_TrayWnd"].contains(&class.as_str()) { return false; }
        let monitor = MonitorFromWindow(foreground, 2);
        let mut info = MonitorInfo { size: std::mem::size_of::<MonitorInfo>() as u32, ..Default::default() };
        GetMonitorInfoW(monitor, &mut info) != 0 && window.left <= info.monitor.left
            && window.top <= info.monitor.top && window.right >= info.monitor.right && window.bottom >= info.monitor.bottom
    }
}

#[repr(C)] #[derive(Default)]
struct MonitorInfo { size: u32, monitor: Rect, work: Rect, flags: u32 }
#[link(name = "user32")]
unsafe extern "system" {
    fn MonitorFromWindow(window: Hwnd, flags: u32) -> isize;
    fn MonitorFromPoint(point: Point, flags: u32) -> isize;
    fn GetMonitorInfoW(monitor: isize, info: *mut MonitorInfo) -> i32;
}
pub fn work_area(x: i32, y: i32) -> Rect {
    unsafe {
        let monitor = MonitorFromPoint(Point { x, y }, 2);
        let mut info = MonitorInfo { size: std::mem::size_of::<MonitorInfo>() as u32, ..Default::default() };
        GetMonitorInfoW(monitor, &mut info);
        info.work
    }
}
pub fn wallpaper_path() -> Option<String> {
    let mut buffer = vec![0u16; 32768];
    if unsafe { SystemParametersInfoW(0x0073, buffer.len() as u32, buffer.as_mut_ptr().cast(), 0) } == 0 { return None; }
    let length = buffer.iter().position(|x| *x == 0)?;
    (length > 0).then(|| String::from_utf16_lossy(&buffer[..length]))
}

#[repr(C)] struct BitmapInfoHeader {
    size: u32, width: i32, height: i32, planes: u16, bit_count: u16,
    compression: u32, size_image: u32, xppm: i32, yppm: i32, used: u32, important: u32,
}
#[repr(C)] struct BitmapInfo { header: BitmapInfoHeader, colors: [u32; 1] }
#[repr(C)] struct ShellFileInfo { icon: isize, index: i32, attributes: u32, display: [u16; 260], type_name: [u16; 80] }
#[repr(C)] struct StockIconInfo { size: u32, icon: isize, index: i32, icon_index: i32, path: [u16; 260] }
#[link(name = "shell32")]
unsafe extern "system" {
    fn SHGetFileInfoW(path: *const u16, attributes: u32, info: *mut ShellFileInfo, size: u32, flags: u32) -> usize;
    fn SHGetStockIconInfo(id: i32, flags: u32, info: *mut StockIconInfo) -> i32;
}
pub fn icon_rgba(path: &str, system: bool) -> Result<Vec<u8>, String> {
    unsafe {
        let icon = if system {
            let mut info: StockIconInfo = std::mem::zeroed();
            info.size = std::mem::size_of::<StockIconInfo>() as u32;
            let id = if path.contains("645FF040") { 31 } else if path.contains("20D04FE0") { 15 } else { 4 };
            if SHGetStockIconInfo(id, 0x100, &mut info) < 0 { return Err("Stock icon lookup failed".into()); }
            info.icon
        } else {
            let mut info: ShellFileInfo = std::mem::zeroed();
            if SHGetFileInfoW(wide(path).as_ptr(), 0, &mut info, std::mem::size_of::<ShellFileInfo>() as u32, 0x100) == 0 {
                return Err("File icon lookup failed".into());
            }
            info.icon
        };
        let screen = GetDC(0);
        let dc = CreateCompatibleDC(screen);
        let info = BitmapInfo { header: BitmapInfoHeader { size: 40, width: 96, height: -96, planes: 1,
            bit_count: 32, compression: 0, size_image: 96 * 96 * 4, xppm: 0, yppm: 0, used: 0, important: 0 }, colors: [0] };
        let mut bits: *mut c_void = ptr::null_mut();
        let bitmap = CreateDIBSection(dc, &info, 0, &mut bits, 0, 0);
        if bitmap == 0 || bits.is_null() || dc == 0 {
            if bitmap != 0 { DeleteObject(bitmap); }
            if dc != 0 { DeleteDC(dc); }
            ReleaseDC(0, screen); DestroyIcon(icon);
            return Err("Icon bitmap allocation failed".into());
        }
        let old = SelectObject(dc, bitmap);
        ptr::write_bytes(bits, 0, 96 * 96 * 4);
        let drawn = DrawIconEx(dc, 0, 0, icon, 96, 96, 0, 0, 3);
        let mut rgba = std::slice::from_raw_parts(bits.cast::<u8>(), 96 * 96 * 4).to_vec();
        let legacy = !rgba.chunks_exact(4).any(|pixel| pixel[3] != 0);
        let mut masked = false;
        if legacy {
            ptr::write_bytes(bits, 255, 96 * 96 * 4);
            if DrawIconEx(dc, 0, 0, icon, 96, 96, 0, 0, 1) != 0 {
                masked = true;
                let mask = std::slice::from_raw_parts(bits.cast::<u8>(), 96 * 96 * 4);
                for (pixel, and_mask) in rgba.chunks_exact_mut(4).zip(mask.chunks_exact(4)) {
                    pixel[3] = if and_mask[..3].iter().all(|x| *x == 0) { 255 } else { 0 };
                }
            }
        }
        if masked {
            for pixel in rgba.chunks_exact_mut(4) { pixel.swap(0, 2); }
        } else { crate::pixels::shell_bgra_to_rgba(&mut rgba)?; }
        SelectObject(dc, old); DeleteObject(bitmap); DeleteDC(dc); ReleaseDC(0, screen); DestroyIcon(icon);
        if drawn == 0 { Err("Icon drawing failed".into()) } else { Ok(rgba) }
    }
}

/// Original extraction path: Shell thumbnails/icons, 96px fit, flags SIIGBF_RESIZETOFIT (0).
pub fn shell_image(path: &str, system: bool) -> Result<image::RgbaImage, String> {
    use windows::{core::PCWSTR, Win32::{Foundation::SIZE, UI::Shell::{IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_RESIZETOFIT}}};
    #[repr(C)] #[derive(Default)]
    struct Bitmap { kind: i32, width: i32, height: i32, stride: i32, planes: u16, depth: u16, bits: *mut c_void }
    struct OwnedBitmap(isize);
    impl Drop for OwnedBitmap { fn drop(&mut self) { unsafe { DeleteObject(self.0); } } }
    let result: Result<image::RgbaImage, String> = (|| unsafe {
        let factory: IShellItemImageFactory = SHCreateItemFromParsingName(PCWSTR(wide(path).as_ptr()), None).map_err(|e| e.to_string())?;
        let bitmap = OwnedBitmap(factory.GetImage(SIZE { cx: 96, cy: 96 }, SIIGBF_RESIZETOFIT).map_err(|e| e.to_string())?.0 as isize);
        let mut description = Bitmap::default();
        if GetObjectW(bitmap.0, std::mem::size_of::<Bitmap>() as i32, (&mut description as *mut Bitmap).cast()) == 0 { return Err("Shell bitmap dimensions unavailable".into()); }
        let w = description.width; let h = description.height.unsigned_abs();
        if w <= 0 || w > 4096 || h == 0 || h > 4096 { return Err("Invalid Shell bitmap dimensions".into()); }
        let mut rgba = vec![0u8; w as usize * h as usize * 4];
        let mut info = BitmapInfo { header: BitmapInfoHeader { size: 40, width: w, height: -(h as i32), planes: 1,
            bit_count: 32, compression: 0, size_image: rgba.len() as u32, xppm: 0, yppm: 0, used: 0, important: 0 }, colors: [0] };
        let dc = GetDC(0);
        if dc == 0 { return Err("Screen DC unavailable".into()); }
        let lines = GetDIBits(dc, bitmap.0, 0, h, rgba.as_mut_ptr().cast(), &mut info, 0);
        ReleaseDC(0, dc);
        if lines != h as i32 { return Err("Shell bitmap read failed".into()); }
        crate::pixels::shell_bgra_to_rgba(&mut rgba)?;
        image::RgbaImage::from_raw(w as u32, h, rgba).ok_or_else(|| "Invalid Shell image buffer".into())
    })();
    result.or_else(|_| image::RgbaImage::from_raw(96, 96, icon_rgba(path, system)?).ok_or_else(|| "Invalid fallback icon buffer".into()))
}

pub fn desktop_attachment_status(window: Hwnd) -> Result<(), String> {
    let style = unsafe { GetWindowLongPtrW(window, -16) } as u32;
    let parent = unsafe { GetParent(window) };
    let parent_class = class_name(parent);
    if style & 0x40000000 == 0 || style & 0x80000000 != 0 || unsafe { IsWindowVisible(parent) } == 0 || !matches!(parent_class.as_str(), "WorkerW" | "Progman") {
        return Err(format!("Desktop attachment lost: style={style:#x}, parent={parent_class}"));
    }
    Ok(())
}
pub fn hide_window(window: Hwnd) { unsafe { ShowWindow(window, 0); } }

pub fn focus_owned(window: Hwnd) -> bool {
    unsafe { let focused = GetFocus(); let root = GetAncestor(window, 2);
        focused != 0 && (focused == window || IsChild(window, focused) != 0)
            && root != 0 && GetAncestor(GetForegroundWindow(), 2) == root }
}
pub fn focus_child(window: Hwnd) -> Result<(), String> {
    unsafe {
        // Activate the real root directly; do not use Tao's synthetic Alt fallback.
        let root = GetAncestor(window, 2);
        if root != 0 { SetForegroundWindow(root); }
        SetFocus(window);
    }
    if focus_owned(window) { Ok(()) } else { Err(unsafe { format!("无法获取桌面编辑输入焦点: focus={:#x}, window={window:#x}, visible={}, parent_visible={}, root={:#x}, foreground={:#x}, style={:#x}, error={}", GetFocus(), IsWindowVisible(window), IsWindowVisible(GetParent(window)), GetAncestor(window,2), GetForegroundWindow(), GetWindowLongPtrW(window,-16), std::io::Error::last_os_error()) }) }
}
pub fn show_window_no_activate(window: Hwnd) { unsafe { ShowWindow(window, 8); } }

#[repr(C)]
#[derive(Default)]
struct SystemPowerStatus { ac: u8, flags: u8, percent: u8, saver: u8, life: u32, full_life: u32 }
#[link(name="kernel32")]
unsafe extern "system" { fn GetSystemPowerStatus(status: *mut SystemPowerStatus)->i32; }
#[link(name="user32")]
unsafe extern "system" {
    fn OpenInputDesktop(flags: u32, inherit: i32, access: u32)->isize;
    fn CloseDesktop(desktop: isize)->i32;
}
pub fn media_playback_allowed()->bool {
    // Our desktop overlay fills the monitor; focusing its menus is not a fullscreen application.
    let mut foreground_process=0;
    unsafe { GetWindowThreadProcessId(GetForegroundWindow(), &mut foreground_process); }
    if foreground_process!=std::process::id() && fullscreen_foreground(){return false;}
    unsafe {
        let mut status=SystemPowerStatus::default();
        if GetSystemPowerStatus(&mut status)!=0 && status.saver==1{return false;}
        // A locked or secure desktop cannot be opened with SWITCHDESKTOP access.
        let desktop=OpenInputDesktop(0,0,0x0100);
        if desktop==0{return false;}
        CloseDesktop(desktop);
    }
    true
}
