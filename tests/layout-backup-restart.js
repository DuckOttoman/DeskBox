window.addEventListener('DOMContentLoaded',()=>{if(window.__TAURI_INTERNALS__.metadata.currentWindow.label!=='overlay-0')return;(async()=>{
 const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a);try{
 await new Promise(r=>setTimeout(r,1800));const result=await invoke('layout_backup_action',{action:'list',id:null,name:null});
 if(result.backups.length!==3||!result.backups.every(b=>b.createdAt>0))throw Error('Backup list/time persistence failed');
 const s=await invoke('get_state');if(!s.config.widgets.some(w=>w.style.widgetData?.note==='latest survives restore'))throw Error('Live notes not persistent');
 await invoke('request_layout_backup',{action:'manage'});await new Promise(r=>setTimeout(r,400));if(document.querySelectorAll('[data-backup] time').length!==3)throw Error('Restart manager timestamp rows');
 await invoke('frontend_checks_passed',{checks:['LAYOUT_BACKUP_RESTART_SUCCESS']});
 }catch(e){await invoke('frontend_error',{message:String(e)});}await invoke('quit_app');})();});