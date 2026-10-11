(() => {
 const invoke=(command,args)=>window.__TAURI_INTERNALS__.invoke(command,args);
 const wait=ms=>new Promise(r=>setTimeout(r,ms));
 const until=async test=>{for(let n=0;n<100;n++){if(await test())return;await wait(100);}throw Error('Settings lifecycle timeout');};
 addEventListener('DOMContentLoaded',async()=>{
  try {
   const current=window.__TAURI_INTERNALS__.metadata.currentWindow.label;
   await until(async()=>current==='ball'?!!document.querySelector('.ball'):current==='settings'?!!document.querySelector('.brand'):!!document.querySelector('.stage'));
   await invoke('frontend_ready',{nodes:1,role:current});
   if(current!=='ball')return;
   const open=async()=>{
    document.querySelector('.ball').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:300,clientY:300}));
    await until(async()=>[...document.querySelectorAll('.pm > .pm-item')].some(e=>e.querySelector('.pm-label')?.textContent.trim()==='设置'));
    [...document.querySelectorAll('.pm > .pm-item')].find(e=>e.querySelector('.pm-label')?.textContent.trim()==='设置').click();
   };
   const probe=()=>invoke('settings_test_probe');
   await invoke('frontend_checks_passed',{checks:['ball_settings_first_open_start']});
   if(window.__DeskBoxTrayFirst) await invoke('settings_test_probe',{action:'tray_settings'}); else await open();
   await Promise.all([invoke('open_settings_window'),invoke('open_settings_window')]);
   await until(async()=>{const s=await probe();return s.exists&&s.ready&&s.visible;});
   if(document.querySelector('.pm'))throw Error('Ball settings leaf did not close menu');
   await invoke('frontend_checks_passed',{checks:['ball_settings_first_open','settings_concurrent_first_open']});
   await invoke('settings_test_probe',{action:'close'});await until(async()=>!(await probe()).visible);
   await open();await until(async()=>{const s=await probe();return s.visible&&!s.minimized;});
   await invoke('settings_test_probe',{action:'minimize'});await until(async()=>(await probe()).minimized);
   await open();await until(async()=>{const s=await probe();return s.visible&&!s.minimized;});
   await Promise.all([invoke('open_settings_window'),invoke('open_settings_window')]);
   await invoke('get_state');
   await invoke('frontend_checks_passed',{checks:['ball_settings_reopen','ball_settings_restore_minimized','ball_settings_repeated_open','ball_settings_lifecycle']});
  }catch(e){await invoke('frontend_error',{message:String(e)});}
 });
})();
