//! Public Win32 ownership with durable rollback and a same-executable watchdog.
use std::{path::{Path,PathBuf}, fs, time::{Duration,Instant}, process::Command};
use serde::{Serialize,Deserialize};
use crate::model::Monitor;
pub fn reserved_bottom(m:[i32;4],h:i32)->[i32;4]{[m[0],m[3]-h,m[2],m[3]]}
pub fn reservation_is_safe(m:[i32;4],r:[i32;4],h:i32)->bool{r==reserved_bottom(m,h)}
pub fn covers_monitor(w:[i32;4],m:[i32;4])->bool{w[0]<=m[0]+1&&w[1]<=m[1]+1&&w[2]>=m[2]-1&&w[3]>=m[3]-1}
pub fn session_matches(expected:&str,current:&str)->bool{!expected.is_empty()&&expected==current}
pub fn session_ready_path(journal:&Path,token:&str)->PathBuf{journal.with_extension(format!("{token}.ready"))}
pub fn same_instance(p:u32,t:u64,q:u32,u:u64)->bool{p==q&&t!=0&&t==u}
#[repr(C)] #[derive(Default,Clone,Copy)] struct Rect{l:i32,t:i32,r:i32,b:i32}
impl Rect{fn array(self)->[i32;4]{[self.l,self.t,self.r,self.b]} fn from(a:[i32;4])->Self{Self{l:a[0],t:a[1],r:a[2],b:a[3]}}}
#[repr(C)] struct Bar{size:u32,hwnd:isize,callback:u32,edge:u32,rect:Rect,param:isize}
fn bar(hwnd:isize,rect:Rect)->Bar{Bar{size:std::mem::size_of::<Bar>() as u32,hwnd,callback:0,edge:3,rect,param:0}}
// Local repr(C) structs use the same Win32 ABI as equivalent declarations elsewhere.
#[allow(clashing_extern_declarations)]
#[link(name="user32")] unsafe extern "system"{
 fn EnumWindows(cb:unsafe extern "system" fn(isize,isize)->i32,p:isize)->i32;
 fn GetClassNameW(h:isize,s:*mut u16,n:i32)->i32; fn GetWindowRect(h:isize,r:*mut Rect)->i32;
 fn IsWindowVisible(h:isize)->i32; fn ShowWindow(h:isize,c:i32)->i32; fn GetTopWindow(h:isize)->isize; fn GetWindow(h:isize,command:u32)->isize; fn GetWindowLongPtrW(h:isize,index:i32)->isize;
 fn MonitorFromWindow(h:isize,flags:u32)->isize; fn GetMonitorInfoW(h:isize,info:*mut MonitorInfo)->i32;
 fn IsIconic(h:isize)->i32; fn GetWindowThreadProcessId(h:isize,p:*mut u32)->u32;
}
#[repr(C)] struct MonitorInfo{size:u32,monitor:Rect,work:Rect,flags:u32}
#[link(name="shell32")] unsafe extern "system"{fn SHAppBarMessage(m:u32,b:*mut Bar)->usize;}
#[repr(C)] #[derive(Default)] struct Time{lo:u32,hi:u32}
#[allow(clashing_extern_declarations)]
#[link(name="kernel32")] unsafe extern "system"{
 fn OpenProcess(access:u32,inherit:i32,pid:u32)->*mut std::ffi::c_void; fn CloseHandle(h:*mut std::ffi::c_void)->i32;
 fn GetProcessTimes(h:*mut std::ffi::c_void,c:*mut Time,e:*mut Time,k:*mut Time,u:*mut Time)->i32;
 fn WaitForSingleObject(h:*mut std::ffi::c_void,ms:u32)->u32;
}
fn process_time(pid:u32)->Option<u64>{unsafe{let h=OpenProcess(0x1000,0,pid);if h.is_null(){return None}let(mut c,mut e,mut k,mut u)=(Time::default(),Time::default(),Time::default(),Time::default());let ok=GetProcessTimes(h,&mut c,&mut e,&mut k,&mut u);CloseHandle(h);(ok!=0).then_some(((c.hi as u64)<<32)|c.lo as u64)}}
#[derive(Serialize,Deserialize,Clone)] struct SavedTask{class:String,rect:[i32;4],visible:bool}
#[derive(Serialize,Deserialize)] struct Journal{pid:u32,created:u64,token:String,state:usize,tasks:Vec<SavedTask>}
unsafe extern "system" fn enumerate(h:isize,p:isize)->i32{let mut s=[0u16;128];let n=GetClassNameW(h,s.as_mut_ptr(),128);let class=String::from_utf16_lossy(&s[..n.max(0)as usize]);if class=="Shell_TrayWnd"||class=="Shell_SecondaryTrayWnd"{let mut r=Rect::default();if GetWindowRect(h,&mut r)!=0{(*(p as *mut Vec<(isize,SavedTask)>)).push((h,SavedTask{class,rect:window_monitor(h).unwrap_or(r.array()),visible:IsWindowVisible(h)!=0}));}}1}
fn window_monitor(h:isize)->Option<[i32;4]>{unsafe{let mut info=MonitorInfo{size:std::mem::size_of::<MonitorInfo>()as u32,monitor:Rect::default(),work:Rect::default(),flags:0};(GetMonitorInfoW(MonitorFromWindow(h,2),&mut info)!=0).then_some(info.monitor.array())}}
fn shell_identity()->(isize,u32){let h=tasks().into_iter().find(|(_,t)|t.class=="Shell_TrayWnd").map(|(h,_)|h).unwrap_or(0);let mut p=0;unsafe{GetWindowThreadProcessId(h,&mut p);}(h,p)}
fn tasks()->Vec<(isize,SavedTask)>{let mut v=Vec::new();unsafe{EnumWindows(enumerate,&mut v as *mut _ as isize);}v}
fn monitor_rect(m:&Monitor)->[i32;4]{[m.x,m.y,m.x.saturating_add(m.w as i32),m.y.saturating_add(m.h as i32)]}
fn overlaps(a:[i32;4],b:[i32;4])->bool{a[0]<b[2]&&a[2]>b[0]&&a[1]<b[3]&&a[3]>b[1]}
pub fn taskbar_monitors(monitors:&[Monitor])->Vec<Monitor>{let ts=tasks();monitors.iter().filter(|m|ts.iter().any(|(_,t)|overlaps(t.rect,monitor_rect(m)))).cloned().collect()}
/// Input windows are meaningful visible windows in front-to-back order.
pub fn fullscreen_from_z_order(windows:&[[i32;4]], monitor:[i32;4])->bool {
 windows.iter().find(|r|overlaps(**r,monitor)).is_some_and(|r|covers_monitor(*r,monitor))
}
// Resolve DWM dynamically: the minimal GNU toolchain need not ship dwmapi.lib.
#[allow(clashing_extern_declarations)]
#[link(name="kernel32")] unsafe extern "system" {
 fn LoadLibraryW(name:*const u16)->*mut std::ffi::c_void;
 fn GetProcAddress(module:*mut std::ffi::c_void,name:*const u8)->*mut std::ffi::c_void;
}
type DwmAttribute=unsafe extern "system" fn(isize,u32,*mut std::ffi::c_void,u32)->i32;
fn is_cloaked(hwnd:isize)->bool {
 static DWM:std::sync::OnceLock<Option<DwmAttribute>>=std::sync::OnceLock::new();
 let api=DWM.get_or_init(||unsafe {
  let name:Vec<u16>="dwmapi.dll".encode_utf16().chain(Some(0)).collect();
  let module=LoadLibraryW(name.as_ptr());if module.is_null(){return None}
  // Retain one module reference for this process-lifetime function pointer.
  let address=GetProcAddress(module,b"DwmGetWindowAttribute\0".as_ptr());
  if address.is_null(){None}else{Some(std::mem::transmute::<*mut std::ffi::c_void,DwmAttribute>(address))}
 });
 let mut value=0u32;
 api.is_some_and(|f|unsafe{f(hwnd,14,&mut value as *mut _ as *mut std::ffi::c_void,4)>=0&&value!=0})
}
pub fn fullscreen_on(m:&Monitor)->bool {
 let bounds=monitor_rect(m);
 unsafe {
  let mut h=GetTopWindow(0);
  // Walk actual Z order, independently of which display owns keyboard focus.
  for _ in 0..10000 {
   if h==0 {break}
   let current=h; h=GetWindow(h,2); // GW_HWNDNEXT
   if IsWindowVisible(current)==0||IsIconic(current)!=0 {continue}
   let mut pid=0;GetWindowThreadProcessId(current,&mut pid);
   if pid==std::process::id()||GetWindowLongPtrW(current,-20)&0x80!=0 {continue}
   if is_cloaked(current) {continue}
   let mut name=[0u16;128];let n=GetClassNameW(current,name.as_mut_ptr(),128);
   let class=String::from_utf16_lossy(&name[..n.max(0)as usize]);
   if matches!(class.as_str(),"Progman"|"WorkerW"|"Shell_TrayWnd"|"Shell_SecondaryTrayWnd"|"#32768"|"tooltips_class32") {continue}
   let mut r=Rect::default();
   if GetWindowRect(current,&mut r)!=0&&overlaps(r.array(),bounds) {
    return fullscreen_from_z_order(&[r.array()],bounds)
   }
  }
 }
 false
}
fn restore(j:&Journal){unsafe{let mut b=bar(0,Rect::default());b.param=j.state as isize;SHAppBarMessage(10,&mut b);for(h,t)in tasks(){if let Some(old)=j.tasks.iter().find(|old|old.class==t.class&&overlaps(old.rect,t.rect)){ShowWindow(h,if old.visible{4}else{0});}}}}
fn journal_path(d:&Path)->PathBuf{d.join("quickbar-recovery.json")}
/// Restores only dead owners; PID reuse is checked against process creation time.
pub fn recover_stale(d:&Path)->Result<(),String>{let p=journal_path(d);let _ownership=recovery_lock(&p)?;if !p.exists(){return Ok(())}let j:Journal=serde_json::from_slice(&fs::read(&p).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;if process_time(j.pid).is_some_and(|t|same_instance(j.pid,j.created,j.pid,t)){return Err("Another live process owns taskbar recovery".into())}restore(&j);fs::remove_file(p).map_err(|e|e.to_string())}
/// Call before Tauri/single-instance initialization. None means ordinary startup.
pub fn watchdog_main(args:&[String])->Option<i32>{let i=args.iter().position(|s|s=="--deskbox-taskbar-watchdog")?;Some((||->Result<(),String>{let p=PathBuf::from(args.get(i+1).ok_or("Missing journal")?);let token=args.get(i+2).ok_or("Missing token")?;let j:Journal=serde_json::from_slice(&fs::read(&p).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;if &j.token!=token||process_time(j.pid)!=Some(j.created){return Err("Owner mismatch".into())}let h=unsafe{OpenProcess(0x00100000|0x1000,0,j.pid)};if h.is_null(){return Err("Cannot observe owner".into())}if let Err(e)=fs::write(session_ready_path(&p,token),token){unsafe{CloseHandle(h);}return Err(e.to_string())}loop{let current=match fs::read(&p).ok().and_then(|bytes|serde_json::from_slice::<Journal>(&bytes).ok()){Some(j)=>j,None=>break};if !session_matches(token,&current.token){break}let wait=unsafe{WaitForSingleObject(h,250)};if wait==0{let _ownership=match recovery_lock(&p){Ok(lock)=>lock,Err(_)=>{std::thread::sleep(Duration::from_millis(250));continue}};if let Ok(bytes)=fs::read(&p){if let Ok(current)=serde_json::from_slice::<Journal>(&bytes){if session_matches(token,&current.token){restore(&current);let _=fs::remove_file(&p);}}}break}if wait==u32::MAX{break}}unsafe{CloseHandle(h);}let _=fs::remove_file(session_ready_path(&p,token));Ok(())})().map_or(1,|_|0))}
pub struct Controller{path:PathBuf,journal:Journal,docks:Vec<(isize,Monitor)>,reservations:Vec<(isize,Monitor)>,active:bool,shell:(isize,u32)}
impl Controller{
 pub fn enable(directory:&Path,docks:&[(isize,Monitor)])->Result<Self,String>{
  recover_stale(directory)?;if docks.is_empty(){return Err("No taskbar displays".into())}fs::create_dir_all(directory).map_err(|e|e.to_string())?;
  let original=tasks();if original.is_empty(){return Err("Explorer taskbar unavailable".into())}let mut b=bar(0,Rect::default());let state=unsafe{SHAppBarMessage(4,&mut b)};
  let j=Journal{pid:std::process::id(),created:process_time(std::process::id()).ok_or("Cannot identify owner")?,token:uuid::Uuid::new_v4().to_string(),state,tasks:original.iter().map(|(_,t)|t.clone()).collect()};
  let path=journal_path(directory);let bytes=serde_json::to_vec(&j).map_err(|e|e.to_string())?;{let _ownership=recovery_lock(&path)?;use std::io::Write;let mut f=fs::OpenOptions::new().write(true).create_new(true).open(&path).map_err(|e|e.to_string())?;f.write_all(&bytes).and_then(|_|f.sync_all()).map_err(|e|e.to_string())?;}
  let mut cmd=Command::new(std::env::current_exe().map_err(|e|e.to_string())?);cmd.arg("--deskbox-taskbar-watchdog").arg(&path).arg(&j.token);#[cfg(windows)]{use std::os::windows::process::CommandExt;cmd.creation_flags(0x08000000);}if let Err(e)=cmd.spawn(){let _=fs::remove_file(&path);return Err(e.to_string())}
  let start=Instant::now();while fs::read_to_string(session_ready_path(&path,&j.token)).ok().as_deref()!=Some(j.token.as_str()){if start.elapsed()>Duration::from_secs(4){let _=fs::remove_file(&path);return Err("Recovery watchdog did not become ready".into())}std::thread::sleep(Duration::from_millis(25));}
  let mut c=Self{path,journal:j,docks:docks.to_vec(),reservations:Vec::new(),active:true,shell:shell_identity()};
  unsafe{b.param=(state|1)as isize;SHAppBarMessage(10,&mut b);} // Auto-hide releases Explorer's normal work-area reservation.
  for(_,m)in docks{let h=match reservation_window(m){Ok(h)=>h,Err(e)=>{c.disable();return Err(e)}};c.reservations.push((h,m.clone()));let mut a=bar(h,Rect::default());a.callback=0x8000+431;if unsafe{SHAppBarMessage(0,&mut a)}==0{c.disable();return Err("Cannot register shortcut AppBar".into())}}
  // Let Explorer process the state change before negotiating a replacement reservation.
  std::thread::sleep(Duration::from_millis(150));
  if let Err(e)=c.reserve(){c.disable();return Err(e)}for(h,_)in original{unsafe{ShowWindow(h,0);}}
  let begin=Instant::now();let mut stable=0;
  while begin.elapsed()<Duration::from_secs(3){
   std::thread::sleep(Duration::from_millis(100));
   if c.verify_work().is_ok(){stable+=1;if stable>=4{return Ok(c)}}else{stable=0;let _=c.reserve();}
  }
  let diagnostics=c.reservations.iter().map(|(h,m)|format!("{} monitor={:?} work={:?}",m.label,monitor_rect(m),monitor_work(*h))).collect::<Vec<_>>().join("\n");
  let _=fs::write(directory.join("quickbar-native-diagnostic.txt"),&diagnostics);
  c.disable();Err(format!("Windows did not retain the AppBar work area: {diagnostics}"))
 }
 fn reserve(&self)->Result<(),String>{for(h,m)in &self.reservations{let bounds=monitor_rect(m);let height=(96.0*m.scale).round()as i32;let mut b=bar(*h,Rect::from(bounds));unsafe{SHAppBarMessage(2,&mut b);}let proposed=reserved_bottom(b.rect.array(),height);if !reservation_is_safe(bounds,proposed,height){return Err("Explorer retained a work-area reservation; taskbar takeover was rolled back".into())}b.rect=Rect::from(proposed);unsafe{SHAppBarMessage(3,&mut b);}if !reservation_is_safe(bounds,b.rect.array(),height){return Err("AppBar reservation does not match safe monitor bounds".into())}unsafe{if SetWindowPos(*h,0,b.rect.l,b.rect.t,b.rect.r-b.rect.l,b.rect.b-b.rect.t,0x14)==0{return Err("Cannot position reservation window".into())}SHAppBarMessage(9,&mut b);}}Ok(())}
 fn verify_work(&self)->Result<(),String>{for(h,m)in &self.reservations{let r=monitor_rect(m);let expected=[r[0],r[1],r[2],r[3]-(96.0*m.scale).round()as i32];if monitor_work(*h)!=Some(expected){return Err(format!("Work-area reservation wrong: {} {:?} expected {:?}",m.label,monitor_work(*h),expected))}}Ok(())}
 pub fn tick(&mut self)->Result<Vec<(String,bool)>,String>{if !self.active{return Ok(Vec::new())}let current=shell_identity();if current.0==0{return Ok(self.docks.iter().map(|(_,m)|(m.label.clone(),false)).collect())}let restarted=current!=self.shell;if restarted{let mut state=bar(0,Rect::default());state.param=(self.journal.state|1)as isize;unsafe{SHAppBarMessage(10,&mut state);}for(h,_)in &self.reservations{let mut b=bar(*h,Rect::default());b.callback=0x8000+431;unsafe{SHAppBarMessage(0,&mut b);}}self.shell=current;}if restarted{for(h,t)in tasks(){if t.visible&&self.journal.tasks.iter().any(|old|old.class==t.class&&overlaps(old.rect,t.rect)){unsafe{ShowWindow(h,0);}}}let start=Instant::now();let mut stable=0;while start.elapsed()<Duration::from_secs(2){if self.reserve().is_ok()&&self.verify_work().is_ok(){stable+=1;if stable>=3{break}}else{stable=0}std::thread::sleep(Duration::from_millis(100));}if stable<3{return Err("Explorer restart AppBar work area did not stabilize".into())}}self.reserve()?;for(h,t)in tasks(){if t.visible&&self.journal.tasks.iter().any(|old|old.class==t.class&&overlaps(old.rect,t.rect)){unsafe{ShowWindow(h,0);}}}self.verify_work()?;Ok(self.docks.iter().map(|(_,m)|(m.label.clone(),!fullscreen_on(m))).collect())}
 pub fn disable(&mut self){if !self.active{return}let Ok(_ownership)=recovery_lock(&self.path)else{return};for(h,_)in &self.reservations{let mut b=bar(*h,Rect::default());unsafe{SHAppBarMessage(1,&mut b);PostMessageW(*h,0x10,0,0);}}restore(&self.journal);self.active=false;let _=fs::remove_file(&self.path);let _=fs::remove_file(session_ready_path(&self.path,&self.journal.token));}
}
impl Drop for Controller{fn drop(&mut self){self.disable()}}
#[repr(C)] struct WindowClass { style:u32,proc:unsafe extern "system" fn(isize,u32,usize,isize)->isize,cls_extra:i32,wnd_extra:i32,instance:isize,icon:isize,cursor:isize,background:isize,menu:*const u16,name:*const u16 }
#[repr(C)] #[derive(Default)] struct Message { hwnd:isize,message:u32,wparam:usize,lparam:isize,time:u32,point:[i32;2],private:u32 }
#[allow(clashing_extern_declarations)]
#[link(name="user32")] unsafe extern "system" {
 fn RegisterClassW(class:*const WindowClass)->u16;
 fn CreateWindowExW(ex:u32,class:*const u16,title:*const u16,style:u32,x:i32,y:i32,w:i32,h:i32,parent:isize,menu:isize,instance:isize,param:*mut std::ffi::c_void)->isize;
 fn DefWindowProcW(h:isize,m:u32,w:usize,l:isize)->isize;
 fn GetMessageW(message:*mut Message,h:isize,min:u32,max:u32)->i32;
 fn TranslateMessage(message:*const Message)->i32; fn DispatchMessageW(message:*const Message)->isize;
 fn DestroyWindow(h:isize)->i32; fn PostQuitMessage(code:i32);
 fn PostMessageW(h:isize,m:u32,w:usize,l:isize)->i32;
 fn SetLayeredWindowAttributes(h:isize,color:u32,alpha:u8,flags:u32)->i32;
 fn SetWindowPos(h:isize,after:isize,x:i32,y:i32,w:i32,height:i32,flags:u32)->i32;
}
#[allow(clashing_extern_declarations)]
#[link(name="kernel32")] unsafe extern "system" { fn GetModuleHandleW(name:*const u16)->isize; }
unsafe extern "system" fn reservation_proc(h:isize,m:u32,w:usize,l:isize)->isize {
 match m {
  0x47=>{let mut b=bar(h,Rect::default());SHAppBarMessage(9,&mut b);},
  6=>{let mut b=bar(h,Rect::default());b.param=(w&0xffff!=0)as isize;SHAppBarMessage(6,&mut b);},
  0x84=>return -1, // HTTRANSPARENT: reservation surface never accepts input.
  0x10=>{DestroyWindow(h);return 0},
  2=>{PostQuitMessage(0);return 0},
  _=>{}
 }
 DefWindowProcW(h,m,w,l)
}
fn reservation_window(m:&Monitor)->Result<isize,String> {
 let rect=reserved_bottom(monitor_rect(m),(96.0*m.scale).round()as i32);
 let (send,receive)=std::sync::mpsc::sync_channel(1);
 std::thread::spawn(move||unsafe {
  let class:Vec<u16>="DeskBoxQuickbarReservation".encode_utf16().chain(Some(0)).collect();
  let instance=GetModuleHandleW(std::ptr::null());
  let wc=WindowClass{style:0,proc:reservation_proc,cls_extra:0,wnd_extra:0,instance,icon:0,cursor:0,background:0,menu:std::ptr::null(),name:class.as_ptr()};
  RegisterClassW(&wc);
  // Layered alpha-zero, transparent, tool, noactivate; independent of WebView geometry.
  let h=CreateWindowExW(0x080800A0,class.as_ptr(),class.as_ptr(),0x80000000,rect[0],rect[1],rect[2]-rect[0],rect[3]-rect[1],0,0,instance,std::ptr::null_mut());
  if h==0 {let _=send.send(Err("Cannot create reservation window".to_string()));return}
  if SetLayeredWindowAttributes(h,0,0,2)==0 {DestroyWindow(h);let _=send.send(Err("Cannot make reservation window transparent".to_string()));return}
  ShowWindow(h,4);
  if send.send(Ok(h)).is_err(){DestroyWindow(h);return}
  let mut msg=Message::default();while GetMessageW(&mut msg,0,0,0)>0{TranslateMessage(&msg);DispatchMessageW(&msg);}
 });
 receive.recv_timeout(Duration::from_secs(2)).map_err(|e|e.to_string())?
}
fn monitor_work(h:isize)->Option<[i32;4]>{unsafe{let mut info=MonitorInfo{size:std::mem::size_of::<MonitorInfo>()as u32,monitor:Rect::default(),work:Rect::default(),flags:0};(GetMonitorInfoW(MonitorFromWindow(h,2),&mut info)!=0).then_some(info.work.array())}}
/// Stable cross-process name; paths are case/slash normalized before FNV-1a.
pub fn recovery_mutex_key(path:&str)->u64 {
 path.replace('/',"\\").to_lowercase().bytes().fold(0xcbf29ce484222325,|hash,b|(hash^(b as u64)).wrapping_mul(0x100000001b3))
}
#[allow(clashing_extern_declarations)]
#[link(name="kernel32")] unsafe extern "system" {
 fn CreateMutexW(attributes:*mut std::ffi::c_void,owned:i32,name:*const u16)->*mut std::ffi::c_void;
 fn ReleaseMutex(mutex:*mut std::ffi::c_void)->i32;
}
struct RecoveryLock(*mut std::ffi::c_void);
impl Drop for RecoveryLock{fn drop(&mut self){unsafe{ReleaseMutex(self.0);CloseHandle(self.0);}}}
fn recovery_lock(path:&Path)->Result<RecoveryLock,String>{
 let parent=path.parent().unwrap_or(Path::new("."));
 let absolute=fs::canonicalize(parent).unwrap_or_else(|_|if parent.is_absolute(){parent.to_path_buf()}else{std::env::current_dir().unwrap_or_default().join(parent)}).join(path.file_name().unwrap_or_default());
 let name=format!("Local\\DeskBoxQuickbarRecovery-{:016x}",recovery_mutex_key(&absolute.to_string_lossy()));
 let wide:Vec<u16>=name.encode_utf16().chain(Some(0)).collect();
 unsafe{let handle=CreateMutexW(std::ptr::null_mut(),0,wide.as_ptr());if handle.is_null(){return Err("Cannot create taskbar recovery mutex".into())}
 let status=WaitForSingleObject(handle,5000);
 if status==0||status==0x80{Ok(RecoveryLock(handle))}else{CloseHandle(handle);Err("Timed out acquiring taskbar recovery mutex".into())}}
}
