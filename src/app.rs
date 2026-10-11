use crate::{classify, commands, config, model::*, rename, scanner, state, sysmon, win};
use std::{collections::BTreeSet, fs::{self, OpenOptions}, io::Write, path::PathBuf,
    sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, time::{Duration, Instant}};
use tauri::{Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

pub struct AppState {
    pub snapshot: Mutex<Snapshot>,
    pub directory: PathBuf,
    pub watch_outside: AtomicBool,
    pub stopping: AtomicBool,
    pub smoke: bool,
    pub desktop_check: bool,
    pub settings_window: Mutex<()>,
    pub filesystem_operation: Mutex<()>,
    pub diagnostic_scan_started: AtomicBool,
    pub diagnostic_scan_finished: AtomicBool,
    pub smoke_ready: Mutex<BTreeSet<String>>,
    pub smoke_error: AtomicBool,
    pub smoke_contract_passed: AtomicBool,
    pub smoke_rename_menu_passed: AtomicBool,
}
impl AppState {
    fn scan(&self) -> Result<Vec<Item>, String> {
        let cache = self.directory.join("icons-v2");
        if self.smoke { scanner::scan_paths(&cache, &[self.directory.join("rename-fixtures")], false) }
        else { scanner::scan(&cache) }
    }
    pub fn log(&self, text: impl AsRef<str>) {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(self.directory.join("runtime.log")) {
            let _ = writeln!(file, "{}", text.as_ref());
        }
    }
    pub fn snapshot(&self) -> Result<Snapshot, String> {
        self.snapshot.lock().map(|s| {
            let mut snapshot = s.clone();
            snapshot.config.icons_hidden_by_us = win::icons_hidden_by_us();
            snapshot
        }).map_err(|_| "State lock poisoned".into())
    }
    pub fn mutate<T>(&self, change: impl FnOnce(&mut Config) -> Result<T, String>) -> Result<T, String> {
        let mut data = self.snapshot.lock().map_err(|_| "State lock poisoned")?;
        let mut next = data.config.clone();
        let result = change(&mut next)?;
        config::save(&self.directory.join("config.json"), &next)?;
        data.config = next;
        Ok(result)
    }
    pub fn publish(&self, app: &tauri::AppHandle) -> Result<(), String> {
        app.emit("state", self.snapshot()?).map_err(|e| e.to_string())
    }
    pub fn location(&self, origin: Option<[f64; 2]>) -> Result<(String, [f64; 2], [f64; 2]), String> {
        let data = self.snapshot()?;
        let primary = data.monitors.iter().find(|m| m.primary).or(data.monitors.first()).ok_or("No monitor available")?;
        let point = origin.unwrap_or([primary.x as f64 + 40. * primary.scale, primary.y as f64 + 40. * primary.scale]);
        state::locate_monitor(&data.monitors, point)
    }
    pub fn refresh(&self, app: &tauri::AppHandle, full: bool, requested: bool) -> Result<(), String> {
        self.refresh_with_scan(app, full, requested, || self.scan())
    }
    pub fn refresh_with_scan(&self, app: &tauri::AppHandle, full: bool, requested: bool,
        scan: impl FnOnce() -> Result<Vec<Item>, String>) -> Result<(), String> {
        // Serialize file transactions while allowing state reads/edits during Shell work.
        let _operation = self.filesystem_operation.lock().map_err(|_| "File operation lock poisoned")?;
        let items = scan()?;
        let mut data = self.snapshot.lock().map_err(|_| "State lock poisoned")?;
        let mut next = data.config.clone();
        state::adopt_legacy_ids(&mut next, &items);
        if !requested && !full && data.ready && items == data.items { return Ok(()); }
        if let Some(monitor) = data.monitors.iter().find(|m| m.primary).or(data.monitors.first()) {
            if full || requested || next.settings.auto_organize || !next.initialized {
                state::organize(&mut next, &items, &monitor.name, [monitor.w as f64 / monitor.scale, monitor.h as f64 / monitor.scale], full);
            } else {
                state::collect_inbox(&mut next, &items, &monitor.name, [monitor.w as f64 / monitor.scale, monitor.h as f64 / monitor.scale]);
            }
        }
        config::save(&self.directory.join("config.json"), &next)?;
        data.config = next; data.items = items; data.ready = true;
        drop(data);
        self.publish(app)
    }

    pub fn rename_item(&self, app: &tauri::AppHandle, id: &str, name: &str) -> Result<(), String> {
        // Share the file-operation gate with refresh; a stale scan cannot undo remapping.
        let _operation = self.filesystem_operation.lock().map_err(|_| "File operation lock poisoned")?;
        let old = self.snapshot()?.items.into_iter().find(|i| i.id == id).ok_or("文件已不存在，请刷新后重试")?;
        let destination = rename::rename_file(&old, name)?;
        let original = std::path::Path::new(&old.path);
        if destination == original { return Ok(()); }
        let result = (|| {
            let items = self.scan()?;
            let new_id = scanner::stable_id(&destination.to_string_lossy());
            let new = items.iter().find(|i| i.id == new_id).cloned().ok_or("重命名后的文件未被扫描到")?;
            let mut data = self.snapshot.lock().map_err(|_| "State lock poisoned")?;
            let mut next = data.config.clone();
            state::remap_item(&mut next, &old, &new);
            config::save(&self.directory.join("config.json"), &next)?;
            data.config = next;
            data.items = items;
            Ok::<(), String>(())
        })();
        if let Err(error) = result {
            return Err(match rename::move_path(&destination, original) {
                Ok(()) => format!("{error}；已恢复原文件名"),
                Err(rollback) => format!("{error}；恢复原文件名失败：{rollback}。当前文件：{}", destination.display()),
            });
        }
        self.publish(app)
    }
}

fn monitors(app: &tauri::AppHandle) -> Result<Vec<Monitor>, String> {
    let primary = app.primary_monitor().map_err(|e| e.to_string())?;
    let all = app.available_monitors().map_err(|e| e.to_string())?;
    Ok(all.into_iter().enumerate().map(|(i, m)| {
        let position = m.position(); let size = m.size(); let scale = m.scale_factor();
        let work = win::work_area(position.x + size.width as i32 / 2, position.y + size.height as i32 / 2);
        Monitor { name: m.name().cloned().unwrap_or_else(|| format!("display-{i}")), label: format!("overlay-{i}"),
            x: position.x, y: position.y, w: size.width, h: size.height, scale,
            work: [(work.left - position.x) as f64 / scale, (work.top - position.y) as f64 / scale,
                (work.right - work.left) as f64 / scale, (work.bottom - work.top) as f64 / scale],
            primary: primary.as_ref().is_some_and(|p| p.position() == position) }
    }).collect())
}

pub fn smoke_script(enabled: bool) -> String {
    if !enabled { return String::new(); }
    if std::env::args().any(|x|x == "--fence-merge-check") {
        // Optional diagnostic driver override only in isolated smoke mode.
        if let Some(path)=std::env::var_os("DESKBOX_MERGE_CHECK_SCRIPT") {
            if let Ok(script)=fs::read_to_string(path) {return script;}
        }
        return include_str!("../tests/fence-merge-webview.js").into();
    }
    if std::env::args().any(|x|x == "--drag-session") {
        return r#"(() => {
          const invoke=(command,args)=>window.__TAURI_INTERNALS__.invoke(command,args);
          window.addEventListener('error',e=>invoke('frontend_error',{message:String(e.message)}));
          window.addEventListener('unhandledrejection',e=>invoke('frontend_error',{message:String(e.reason)}));
          for(const type of ['pointerdown','pointermove','pointerup','pointercancel','lostpointercapture','blur','focus']) {
            window.addEventListener(type,e=>invoke('frontend_checks_passed',{checks:['drag_event:'+JSON.stringify({type,id:e.pointerId,x:e.clientX,y:e.clientY,target:e.target?.tagName,dragging:window.DeskBoxFenceMerge?.dragging})]}),true);
          }
          setTimeout(async()=>{
            const s=await invoke('get_state');
            await invoke('frontend_checks_passed',{checks:['drag_snapshot:'+JSON.stringify(s.monitors),'drag_nodes:'+document.querySelectorAll('*').length]});
          },3000);
        })();"#.into();
    }
    if std::env::args().any(|x| x == "--scan-responsiveness-check") {
        return include_str!("../tests/scan-responsiveness-webview.js").into();
    }
    if std::env::args().any(|x| x == "--ball-settings-check" || x == "--tray-settings-check") {
        return format!("window.__DeskBoxTrayFirst={};\n{}", std::env::args().any(|x| x == "--tray-settings-check"), include_str!("../tests/ball-settings-webview.js"));
    }
    // An initialization script only; recovered frontend assets remain byte-identical.
    r#"(() => {
      const invoke = (command, args) => window.__TAURI_INTERNALS__.invoke(command, args);
      let releaseMenu;
      const menuReady = new Promise(resolve => { releaseMenu = resolve; });
      invoke('plugin:event|listen', {event:'smoke-menu-complete',target:{kind:'Any'},
        handler:window.__TAURI_INTERNALS__.transformCallback(() => releaseMenu())}).catch(e => invoke('frontend_error',{message:String(e)}));
      async function contract() {
        await menuReady;
        const checks = [];
        const check = (name, value) => { if (!value) throw new Error('Contract check failed: ' + name); checks.push(name); };
        const state = () => invoke('get_state');
        await window.DeskBoxLearningCheck(invoke);
        const initial = await state();
        check('snapshot', initial.ready && Array.isArray(initial.items) && initial.categories.length === 14);
        const fixture = initial.items.find(i => i.name === 'rename-fixture.txt');
        check('rename_fixture', !!fixture);
        const holder = initial.config.fences.find(f => f.items.includes(fixture.id));
        const slot = holder.items.indexOf(fixture.id);
        await window.DeskBoxRename.open(fixture);
        document.getElementById('deskbox-rename-input').value = '重命名测试.txt';
        document.getElementById('deskbox-rename-form').requestSubmit();
        for (let n = 0; n < 80 && document.getElementById('deskbox-rename-dialog'); n++) await new Promise(r => setTimeout(r, 100));
        const renamedState = await state();
        const renamed = renamedState.items.find(i => i.name === '重命名测试.txt');
        check('rename_item', !!renamed && !document.getElementById('deskbox-rename-dialog')
          && renamedState.config.fences.find(f => f.id === holder.id).items[slot] === renamed.id);
        await invoke('rename_item', {id: renamed.id, name: 'rename-fixture.txt'});
        const m = initial.monitors.find(m => m.primary) || initial.monitors[0];
        const origin = [m.x + 70 * m.scale, m.y + 70 * m.scale];
        await invoke('organize_now', {full: false});
        const fenceId = await invoke('create_fence', {origin});
        let current = await state();
        let fence = current.config.fences.find(f => f.id === fenceId);
        check('create_fence', !!fence && fence.monitor === m.name);
        fence.title = '重建接口测试'; fence.category = 'document';
        await invoke('update_fence', {fence});
        check('update_fence', (await state()).config.fences.find(f => f.id === fenceId).title === '重建接口测试');
        if (initial.items.length) {
          const item = initial.items[0].id;
          await invoke('move_item', {item, to: fenceId, index: 0});
          current = await state();
          check('move_item', current.config.fences.filter(f => f.items.includes(item)).length === 1
              && current.config.fences.find(f => f.id === fenceId).items[0] === item);
        }
        await invoke('delete_fence', {id: fenceId});
        check('delete_fence', !(await state()).config.fences.some(f => f.id === fenceId));
        const widgetId = await invoke('create_widget', {kind: 'clock', origin});
        let widget = (await state()).config.widgets.find(w => w.id === widgetId);
        check('create_widget', !!widget);
        widget.x += 15;
        await invoke('update_widget', {widget});
        check('update_widget', (await state()).config.widgets.find(w => w.id === widgetId).x === widget.x);
        await invoke('delete_widget', {id: widgetId});
        check('delete_widget', !(await state()).config.widgets.some(w => w.id === widgetId));
        await invoke('set_theme', {theme: 'forest'});
        check('set_theme', (await state()).config.theme === 'forest');
        await invoke('set_theme', {theme: initial.config.theme});
        await invoke('update_settings', {settings: initial.config.settings});
        check('update_settings', (await state()).config.settings.iconSize === initial.config.settings.iconSize);
        await invoke('toggle_fences', {origin});
        check('toggle_fences', (await state()).config.fencesVisible !== initial.config.fencesVisible);
        await invoke('toggle_fences', {origin});
        await invoke('save_ball', {x: origin[0], y: origin[1]});
        check('save_ball', (await state()).config.ball[0] === origin[0]);
        await invoke('reset_learned');
        check('reset_learned', Object.keys((await state()).config.learned).length === 0);
        await invoke('clear_fence_image', {id: (await state()).config.fences[0].id});
        await invoke('watch_outside_click', {enable: true});
        await invoke('watch_outside_click', {enable: false});
        const monitorId = await invoke('create_widget', {kind: 'sysmon', origin});
        await new Promise(resolve => setTimeout(resolve, 4500));
        await invoke('delete_widget', {id: monitorId});
        await invoke('frontend_checks_passed', {checks});
      }
      window.addEventListener('error', e => invoke('frontend_error', {message: String(e.message)}));
      window.addEventListener('unhandledrejection', e => invoke('frontend_error', {message: String(e.reason)}));
      async function menuCheck() {
        for (let n = 0; n < 80 && !document.querySelector('.item'); n++) await new Promise(r => setTimeout(r, 100));
        const menuInitial = await invoke('get_state');
        const menuFixture = menuInitial.items.find(i => i.name === 'rename-fixture.txt');
        let item = [...document.querySelectorAll('.item')].find(e => e.dataset.item === menuFixture?.id);
        if (!item) throw new Error('No fixture item rendered in overlay');
        item.dispatchEvent(new MouseEvent('contextmenu', {bubbles: true, clientX: 100, clientY: Math.max(10, innerHeight - 381)}));
        await new Promise(r => setTimeout(r, 100));
        const entryFor = label => [...document.querySelectorAll('.pm > .pm-item')].find(e => e.textContent.trim() === label);
        const baseLabels = ['打开','打开所在位置','属性','移动到','删除到回收站'];
        for (const label of baseLabels) if (!entryFor(label)) throw new Error('Original menu missing: '+label);
        const move = entryFor('移动到');
        move.click();
        await new Promise(r => setTimeout(r, 400));
        const expanded = move.nextElementSibling;
        if (!expanded?.classList.contains('show')) throw new Error('Move submenu did not expand');
        const stateNow = await invoke('get_state');
        const holderNow = item.closest('[data-fence]').dataset.fence;
        const originalSlot = stateNow.config.fences.find(f => f.id === holderNow).items.indexOf(menuFixture.id);
        const destinations = stateNow.config.fences.filter(f => f.id !== holderNow);
        const subLabels = [...expanded.querySelectorAll('.pm-label')].map(e => e.textContent.trim());
        if (JSON.stringify(subLabels) !== JSON.stringify(destinations.map(f => f.title))) throw new Error('Move destinations differ from configuration');
        const rect = document.querySelector('.pm').getBoundingClientRect();
        await invoke('frontend_checks_passed', {checks:['original_menu_entries','move_submenu_expands','move_destination_labels',
          'menu_geometry:'+JSON.stringify({top:rect.top,bottom:rect.bottom,height:rect.height,viewport:innerHeight})]});
        if (rect.top < 0 || rect.bottom > innerHeight) throw new Error('Expanded move submenu extends outside viewport');
        if (destinations.length) {
          const panel = document.querySelector('.pm');
          panel.scrollTop = panel.scrollHeight;
          const targetRow = [...expanded.querySelectorAll('.pm-item.sub')].at(-1);
          const lastRect = targetRow.getBoundingClientRect();
          const panelRect = panel.getBoundingClientRect();
          if (lastRect.bottom > panelRect.bottom + 1 || lastRect.top < panelRect.top) throw new Error('Last move destination not reachable by scrolling');
          targetRow.click();
          const destination = destinations.at(-1).id;
          let moved;
          for (let n=0;n<50;n++) {
            moved = await invoke('get_state');
            if (moved.config.fences.find(f => f.id === destination)?.items.includes(menuFixture.id)) break;
            await new Promise(r => setTimeout(r,100));
          }
          if (!moved.config.fences.find(f => f.id === destination)?.items.includes(menuFixture.id) || document.querySelector('.pm')) throw new Error('Move leaf did not dispatch and close menu');
          await invoke('move_item',{item:menuFixture.id,to:holderNow,index:originalSlot});
          await new Promise(r => setTimeout(r,150));
          item = [...document.querySelectorAll('.item')].find(e => e.dataset.item === menuFixture.id);
          item.dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:innerWidth-2,clientY:innerHeight-2}));
          await new Promise(r => setTimeout(r,400));
          const right = document.querySelector('.pm').getBoundingClientRect();
          if (right.left < 0 || right.right > innerWidth || right.top < 0 || right.bottom > innerHeight) throw new Error('Bottom-right menu clipped');
          await invoke('frontend_checks_passed',{checks:['move_leaf_dispatch_and_close','move_last_row_reachable','bottom_right_menu_bounds']});
        } else { move.click(); }
        const remove = entryFor('删除到回收站');
        remove.click();
        await new Promise(r => setTimeout(r, 400));
        if (!remove.nextElementSibling?.classList.contains('show') || remove.nextElementSibling.textContent.trim() !== '确认删除') throw new Error('Delete confirmation submenu missing');
        if (!(await invoke('get_state')).items.some(i => i.id === menuFixture.id)) throw new Error('Delete parent executed without confirmation');
        remove.click();
        await invoke('frontend_checks_passed', {checks:['delete_confirmation_submenu','delete_parent_does_not_execute']});
        const entry = entryFor('重命名');
        if (!entry || entry.classList.contains('disabled')) throw new Error('Rename context menu entry missing');
        entry.click();
        await new Promise(r => setTimeout(r, 150));
        if (!document.getElementById('deskbox-rename-input')) throw new Error('Rename menu did not open dialog');
        document.getElementById('deskbox-rename-cancel').click();
        await invoke('frontend_checks_passed', {checks: ['rename_menu']});
        await window.DeskBoxOwnMenuCheck(invoke);
        await invoke('plugin:event|emit',{event:'smoke-menu-complete',payload:null});
      }
      window.addEventListener('DOMContentLoaded', () => {
        // Native-parented hidden WebViews may throttle animation frames. Test final
        // menu layout without measuring an in-flight spring animation.
        const stableMenus = document.createElement('style');
        stableMenus.textContent = '.pm,.pm-item{animation:none!important}.pm-sub{transition:none!important}';
        document.head.append(stableMenus);
        let attempts = 0;
        const timer = setInterval(() => {
          const nodes = document.querySelector('#app')?.children.length || 0;
          if (nodes > 0) {
            clearInterval(timer);
            const role = document.documentElement.dataset.role || '';
            invoke('frontend_ready', {nodes, role});
            if (role === 'settings') contract().catch(e => invoke('frontend_error', {message: String(e)}));
            if (role === 'overlay' && window.__TAURI_INTERNALS__.metadata.currentWindow.label === 'overlay-0')
              menuCheck().catch(e => invoke('frontend_error', {message: String(e)}));
          }
          else if (++attempts > 150) { clearInterval(timer); invoke('frontend_error', {message: 'Frontend did not mount within 15 seconds'}); }
        }, 100);
      });
    })();"#.to_string() + "\n" + include_str!("../tests/own-menus-webview.js") + "\n" + include_str!("../tests/learning-webview.js")
}

pub fn open_settings(app: &tauri::AppHandle, data: &AppState) -> Result<(), String> {
    let _creation = data.settings_window.lock().map_err(|_| "Settings creation lock poisoned")?;
    if data.smoke { data.log("SETTINGS_OPEN_REQUEST"); }
    if let Some(window) = app.get_webview_window("settings") {
        window.unminimize().and_then(|_| window.show()).and_then(|_| window.set_focus()).map_err(|e| e.to_string())?;
        return Ok(());
    }
    let window = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("DeskBox 重建版设置").inner_size(820., 640.).min_inner_size(700., 520.)
        .decorations(false).resizable(true).center().visible(!data.smoke || std::env::args().any(|x| x == "--ball-settings-check" || x == "--tray-settings-check"))
        .data_directory(data.directory.join("webview"))
        .initialization_script(smoke_script(data.smoke)).build().map_err(|e| e.to_string())?;
    if data.smoke { data.log("SETTINGS_CREATED"); }
    let copy = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event { api.prevent_close(); let _ = copy.hide(); }
    });
    Ok(())
}

pub async fn open_settings_async(app: tauri::AppHandle, data: Arc<AppState>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || open_settings(&app, &data)).await.map_err(|e| e.to_string())?
}

pub fn dispatch_tray_action(app: &tauri::AppHandle, action: &str) {
    let app = app.clone();
    let data = app.state::<Arc<AppState>>().inner().clone();
    let action = action.to_string();
    // Native event callbacks must not create WebViews or scan Shell entries inline.
    tauri::async_runtime::spawn_blocking(move || {
        let result = match action.as_str() {
            "settings" => open_settings(&app, &data),
            "organize" => data.refresh(&app, false, true),
            "toggle" => commands::toggle(&app, &data, None),
            "quit" => { app.exit(0); Ok(()) },
            _ => Ok(()),
        };
        if let Err(error) = result { data.log(error); }
    });
}

fn start_workers(app: tauri::AppHandle, data: Arc<AppState>) {
    std::thread::spawn(move || {
        let mut sampler = sysmon::Sampler::new();
        let mut icons_at = Instant::now(); let mut icon_error = None;
        let mut sample_at = Instant::now(); let mut scan_at = Instant::now(); let mut previous_down = false;
        while !data.stopping.load(Ordering::Relaxed) {
            if data.watch_outside.load(Ordering::Relaxed) {
                if let Some(ball) = app.get_webview_window("ball") {
                    if let Ok(handle) = ball.hwnd() {
                        if win::click_outside(handle.0 as isize, &mut previous_down) {
                            let _ = ball.emit("outside-click", ());
                            data.watch_outside.store(false, Ordering::Relaxed);
                        }
                    }
                }
            } else { previous_down = false; }
            if sample_at.elapsed() >= Duration::from_millis(250) {
                if let Ok(snapshot) = data.snapshot() {
                    let interval = snapshot.config.settings.sysmon_interval_ms.clamp(1000, 10000);
                    if sample_at.elapsed() >= Duration::from_millis(interval) {
                        if snapshot.config.fences_visible && snapshot.config.widgets.iter().any(|w| w.kind == "sysmon")
                            && (data.smoke || !win::fullscreen_foreground()) {
                            let sample = sampler.sample();
                            let _ = app.emit("sysmon", &sample);
                            if data.smoke { data.log(format!("sysmon {}", serde_json::to_string(&sample).unwrap_or_default())); }
                        }
                        sample_at = Instant::now();
                    }
                }
            }
            if !data.smoke && icons_at.elapsed() >= Duration::from_secs(1) {
                if let Ok(snapshot) = data.snapshot() {
                    let error = win::desktop_icons(snapshot.config.settings.hide_desktop_icons && snapshot.config.fences_visible).err();
                    if error != icon_error { if let Some(message) = &error { data.log(format!("Desktop icons: {message}")); } }
                    icon_error = error;
                }
                icons_at = Instant::now();
            }
            if scan_at.elapsed() >= Duration::from_secs(5) {
                if let Err(error) = data.refresh(&app, false, false) { data.log(format!("Scan: {error}")); }
                scan_at = Instant::now();
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });
}

pub fn run() {
    if std::env::args().any(|x| x == "--native-check") {
        let result = native_check();
        if let Err(error) = &result {
            if let Some(directory) = std::env::var_os("DESKBOX_DATA_DIR") { let _ = fs::write(PathBuf::from(directory).join("native-error.txt"), error); }
        }
        std::process::exit(if result.is_ok() { 0 } else { 2 });
    }
    let drag_session = std::env::args().any(|x|x == "--drag-session");
    let desktop_check = std::env::args().any(|x| x == "--desktop-interaction-check");
    let ball_settings_check = std::env::args().any(|x| x == "--ball-settings-check" || x == "--tray-settings-check");
    let smoke = drag_session || std::env::args().any(|x| x == "--fence-merge-check" || x == "--scan-responsiveness-check") || ball_settings_check || desktop_check || std::env::args().any(|x| x == "--smoke-test");
    // Isolated diagnostics must not activate or interfere with a running user instance.
    let builder = tauri::Builder::default();
    let builder = if smoke { builder } else {
        builder.plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("settings").or_else(|| app.get_webview_window("ball")) {
                let _ = window.show(); let _ = window.set_focus();
            }
        }))
    };
    let application = builder.plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::Builder::new().args(["--autostart"]).build())
        .invoke_handler(tauri::generate_handler![
            commands::get_state, commands::rename_item, commands::set_region, commands::update_fence, commands::create_fence,
            commands::move_fence_group, commands::group_fences, commands::select_fence, commands::detach_fence, commands::merge_fence_contents, commands::restore_fence_merge,
            commands::delete_fence, commands::move_item, commands::item_action, commands::organize_now,
            commands::toggle_fences, commands::create_widget, commands::update_widget, commands::delete_widget,
            commands::patch_widget_data, commands::widget_album_action, commands::widget_interaction,
            commands::open_widget_weather_source,
            commands::set_theme, commands::update_settings, commands::save_ball, commands::open_settings_window,
            commands::request_layout_backup, commands::layout_backup_action, commands::apply_weather_location, commands::optimize_memory, commands::reset_learned, commands::quit_app, commands::keyboard_focus, commands::pick_fence_image,
            commands::import_fence_background, commands::set_fence_video_mode, commands::media_playback_allowed,
            commands::clear_fence_image, commands::watch_outside_click, commands::frontend_ready, commands::frontend_error,
            commands::frontend_checks_passed, commands::settings_test_probe,
        ])
        .setup(move |application| {
            let app = application.handle().clone();
            let directory = std::env::var_os("DESKBOX_DATA_DIR").map(PathBuf::from)
                .unwrap_or(app.path().app_local_data_dir()?);
            fs::create_dir_all(&directory)?;
            if smoke {
                let fixtures = directory.join("rename-fixtures");
                fs::create_dir_all(&fixtures)?;
                fs::write(fixtures.join("rename-fixture.txt"), "DeskBox rename fixture content")?;
            }
            let original_directory = app.path().app_local_data_dir()?.parent().map(|p| p.join("com.deskbox.app"));
            let mut config = if smoke { config::load(&directory.join("config.json"))? }
                else { config::upgrade_legacy(&directory, original_directory.as_deref())? };
            config.icons_hidden_by_us = false;
            if smoke { config.settings.hide_desktop_icons = false; config.settings.launch_at_startup = false; }
            let monitors = monitors(&app).map_err(std::io::Error::other)?;
            let wallpaper = scanner::wallpaper(&directory);
            let data = Arc::new(AppState { snapshot: Mutex::new(Snapshot { config, items: vec![], monitors: monitors.clone(),
                categories: classify::categories(), wallpaper, ready: false }), directory, smoke, desktop_check,
                settings_window: Mutex::new(()), filesystem_operation: Mutex::new(()), diagnostic_scan_started: AtomicBool::new(false), diagnostic_scan_finished: AtomicBool::new(false),
                watch_outside: AtomicBool::new(false), stopping: AtomicBool::new(false),
                smoke_ready: Mutex::new(BTreeSet::new()), smoke_error: AtomicBool::new(false), smoke_contract_passed: AtomicBool::new(false),
                smoke_rename_menu_passed: AtomicBool::new(false) });
            app.manage(data.clone());
            app.asset_protocol_scope().allow_directory(&data.directory, true)?;
            crate::widget_album::allow_saved(&app,&data);
            data.refresh(&app, false, false).map_err(std::io::Error::other)?;
            data.log("Reconstructed backend initialized");
            for monitor in &monitors {
                let window = WebviewWindowBuilder::new(&app, &monitor.label, WebviewUrl::App("index.html".into()))
                    .title(if drag_session {"DeskBox 拖动验证"} else {"DeskBox 桌面整理（重建版）"}).inner_size(monitor.w as f64 / monitor.scale, monitor.h as f64 / monitor.scale)
                    .position(monitor.x as f64 / monitor.scale, monitor.y as f64 / monitor.scale)
                    .decorations(false).transparent(true).shadow(false).resizable(false).skip_taskbar(true)
                    .focused(false).visible(false).data_directory(data.directory.join("webview"))
                    .initialization_script(smoke_script(smoke)).build()?;
                window.set_position(PhysicalPosition::new(monitor.x, monitor.y))?;
                if drag_session { window.show()?; }
                if !smoke || desktop_check {
                    // Tao.show rewrites its stored style; finish that before native parenting.
                    if !desktop_check { window.show()?; }
                    win::attach_overlay(window.hwnd()?.0 as isize, monitor.x, monitor.y, monitor.w, monitor.h).map_err(std::io::Error::other)?;
                    if desktop_check && !drag_session { win::hide_window(window.hwnd()?.0 as isize); }
                }
            }
            let primary = monitors.iter().find(|m| m.primary).or(monitors.first()).ok_or("No monitor")?;
            let center = data.snapshot().map_err(std::io::Error::other)?.config.ball.unwrap_or([
                primary.x as f64 + primary.w as f64 - 100. * primary.scale,
                primary.y as f64 + 130. * primary.scale]);
            let ball = WebviewWindowBuilder::new(&app, "ball", WebviewUrl::App("index.html".into()))
                .title("DeskBox Ball（重建版）").inner_size(600., 600.).decorations(false)
                .transparent(true).shadow(false).always_on_top(true).skip_taskbar(true).resizable(false)
                .focused(false).visible(!smoke).data_directory(data.directory.join("webview"))
                .initialization_script(smoke_script(smoke)).build()?;
            let size = ball.outer_size()?;
            ball.set_position(PhysicalPosition::new((center[0] - size.width as f64 / 2.) as i32, (center[1] - size.height as f64 / 2.) as i32))?;
            win::focusable(ball.hwnd()?.0 as isize, false);
            if !smoke {
                use tauri::{menu::{Menu, MenuItem}, tray::TrayIconBuilder};
                let toggle = MenuItem::with_id(&app, "toggle", "显示 / 隐藏全部框", true, None::<&str>)?;
                let organize = MenuItem::with_id(&app, "organize", "一键整理", true, None::<&str>)?;
                let settings = MenuItem::with_id(&app, "settings", "设置", true, None::<&str>)?;
                let quit = MenuItem::with_id(&app, "quit", "退出", true, None::<&str>)?;
                let menu = Menu::with_items(&app, &[&toggle, &organize, &settings, &quit])?;
                TrayIconBuilder::new().icon(app.default_window_icon().unwrap().clone()).tooltip("DeskBox 重建版")
                    .menu(&menu).on_menu_event(|app, event| {
                        dispatch_tray_action(app, event.id.as_ref());
                    }).build(&app)?;
                let config = data.snapshot().map_err(std::io::Error::other)?.config;
                if let Err(error) = win::desktop_icons(config.settings.hide_desktop_icons && config.fences_visible) { data.log(error); }
            } else if !drag_session {
                if !ball_settings_check { open_settings(&app, &data).map_err(std::io::Error::other)?; }
                let exit_app = app.clone(); let check = data.clone(); let expected = monitors.len() + 2;
                std::thread::spawn(move || {
                    let deadline = Instant::now() + Duration::from_secs(45);
                    loop {
                        let ready = check.smoke_ready.lock().map(|s| s.len()).unwrap_or(0);
                        let contract = check.smoke_contract_passed.load(Ordering::Relaxed) && check.smoke_rename_menu_passed.load(Ordering::Relaxed);
                        if (ready >= expected && contract) || Instant::now() >= deadline || check.smoke_error.load(Ordering::Relaxed) {
                            let success = ready >= expected && contract && !check.smoke_error.load(Ordering::Relaxed);
                            check.log(format!("SMOKE_RESULT success={success} ready={ready} expected={expected}"));
                            exit_app.exit(if success { 0 } else { 2 }); break;
                        }
                        std::thread::sleep(Duration::from_millis(100));
                    }
                });
            }
            start_workers(app, data);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Failed to initialize reconstructed DeskBox");
    application.run(|app, event| {
        if matches!(event, tauri::RunEvent::Exit | tauri::RunEvent::ExitRequested { .. }) {
            if let Some(data) = app.try_state::<Arc<AppState>>() {
                data.stopping.store(true, Ordering::Relaxed);
                if let Err(error) = win::shutdown_desktop_icons() { data.log(error); }
                if let Err(error) = data.mutate(|c| { c.icons_hidden_by_us = false; Ok(()) }) { data.log(error); }
            }
        }
    });
}

// Bounded diagnostic: uses the real Windows desktop, restores only visibility we own,
// and writes scans/import previews exclusively beneath the specified test directory.
fn native_check() -> Result<(), String> {
    let directory = PathBuf::from(std::env::var_os("DESKBOX_DATA_DIR").ok_or("DESKBOX_DATA_DIR required")?);
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    struct Restore;
    impl Drop for Restore { fn drop(&mut self) { let _ = win::desktop_icons(false); } }
    let _restore = Restore;
    let (_, before) = win::desktop_icon_status()?;
    win::desktop_icons(true)?;
    let (_, hidden) = win::desktop_icon_status()?;
    if hidden { return Err("Desktop icons remained visible".into()); }
    let owned = win::icons_hidden_by_us();
    win::desktop_icons(false)?;
    let (_, after) = win::desktop_icon_status()?;
    if before != after { return Err("Desktop visibility was not restored".into()); }
    let items = scanner::scan(&directory.join("icons-v2"))?;
    let original = std::env::var_os("DESKBOX_ORIGINAL_DATA_DIR").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default()).join("com.deskbox.app"));
    let imported = config::upgrade_legacy(&directory.join("import-preview"), Some(&original))?;
    let ids: BTreeSet<_> = items.iter().map(|i| i.id.as_str()).collect();
    let preserved = imported.fences.iter().flat_map(|f| &f.items).filter(|id| ids.contains(id.as_str())).count();
    let images: Vec<_> = items.iter().filter_map(|i| i.icon.as_ref()).filter_map(|p| image::image_dimensions(p).ok()).collect();
    if images.len() != items.len() || images.is_empty() { return Err("Some desktop images could not be extracted".into()); }
    if images.iter().any(|(w,h)| *w == 0 || *h == 0 || *w > 96 || *h > 96) { return Err("Shell images exceeded the original 96px fit".into()); }
    let result = serde_json::json!({"initial_visible": before, "hidden_visible": hidden, "owned_during_hide": owned,
        "restored_visible": after, "visibility_restored": before == after, "items": items.len(), "icons": images.len(),
        "dimensions": images, "imported_fences": imported.fences.len(), "imported_widgets": imported.widgets.len(),
        "imported_theme": imported.theme, "imported_hide_desktop_icons": imported.settings.hide_desktop_icons,
        "original_layout_items_matched": preserved});
    fs::write(directory.join("native-check.json"), serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
