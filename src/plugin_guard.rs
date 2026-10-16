#[cfg(test)] mod tests {
 use super::*;
 #[test] fn permits_only_initial_documents(){
  assert!(allowed_frame_uri("about:blank"));assert!(allowed_frame_uri("about:srcdoc"));
  for uri in ["https://example.com/", "http://127.0.0.1/", "data:text/html,hello", "file:///C:/Windows/win.ini", "http://tauri.localhost/", "about:blank?https://example.com"] {assert!(!allowed_frame_uri(uri),"{uri}");}
 }
}

fn allowed_frame_uri(uri:&str)->bool {matches!(uri,"about:blank"|"about:srcdoc")}

// Chromium does not enforce navigate-to. Cancel subframe navigation natively,
// before handing any installed script to the host page.
#[cfg(feature="gui")]
pub async fn ensure(window:&tauri::WebviewWindow)->Result<(),String>{
 use std::{collections::HashSet,sync::Mutex};
 use tauri::Emitter;
 use webview2_com::{NavigationStartingEventHandler,take_pwstr};
 use windows::core::PWSTR;
 static GUARDED:Mutex<Option<HashSet<isize>>>=Mutex::new(None);
 let key=window.hwnd().map_err(|e|e.to_string())?.0 as isize;
 let owner=window.clone();
 let (tx,rx)=std::sync::mpsc::channel();
 window.with_webview(move|view|{
  let result=(||->Result<(),String>{
   let mut guarded=GUARDED.lock().map_err(|_|"Plugin guard lock")?;
   let set=guarded.get_or_insert_with(HashSet::new);
   if set.contains(&key){return Ok(())}
   let emitter=owner.clone();
   unsafe {
    let core=view.controller().CoreWebView2().map_err(|e|e.to_string())?;
    let mut token=0;
    core.add_FrameNavigationStarting(&NavigationStartingEventHandler::create(Box::new(move|_,args|{
     if let Some(args)=args {
      let mut raw=PWSTR::null();
      let ok=args.Uri(&mut raw).is_ok();
      let uri=take_pwstr(raw);
      if !ok||!allowed_frame_uri(&uri){args.SetCancel(true)?;let _=emitter.emit("plugin-navigation-blocked",());}
     }
     Ok(())
    })),&mut token).map_err(|e|e.to_string())?;
   }
   set.insert(key);
   owner.on_window_event(move|event|{if matches!(event,tauri::WindowEvent::Destroyed){if let Ok(mut cache)=GUARDED.lock(){if let Some(set)=cache.as_mut(){set.remove(&key);}}}});
   Ok(())
  })();
  let _=tx.send(result);
 }).map_err(|e|e.to_string())?;
 tauri::async_runtime::spawn_blocking(move||rx.recv_timeout(std::time::Duration::from_secs(10)).map_err(|e|e.to_string())?).await.map_err(|e|e.to_string())?
}
