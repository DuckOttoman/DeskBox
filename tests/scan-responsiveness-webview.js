(() => {
 const invoke=(command,args)=>window.__TAURI_INTERNALS__.invoke(command,args);
 const wait=ms=>new Promise(r=>setTimeout(r,ms));
 const until=async test=>{for(let n=0;n<100;n++){if(await test())return;await wait(50);}throw Error('Slow scan timeout');};
 addEventListener('DOMContentLoaded',async()=>{try {
  const current=window.__TAURI_INTERNALS__.metadata.currentWindow.label;
  await until(async()=>current==='ball'?!!document.querySelector('.ball'):current==='settings'?!!document.querySelector('.brand'):!!document.querySelector('.stage'));
  await invoke('frontend_ready',{nodes:1,role:current});if(current!=='ball')return;
  const initial=await invoke('get_state');
  await invoke('settings_test_probe',{action:'slow_scan'});
  await until(async()=>(await invoke('settings_test_probe')).scanStarted);
  const started=performance.now();await invoke('get_state');const elapsed=performance.now()-started;
  if(elapsed>1000)throw Error('get_state blocked behind Shell scan for '+Math.round(elapsed)+'ms');
  const saved=initial.config.fences[0];
  await invoke('update_fence',{fence:{...saved,title:'扫描期间编辑保留'}});
  const fixture=initial.items.find(i=>i.name==='rename-fixture.txt');
  const renaming=invoke('rename_item',{id:fixture.id,name:'scan-rename-fixture.txt'});
  await Promise.all([renaming,until(async()=>(await invoke('settings_test_probe')).scanFinished)]);
  if((await invoke('get_state')).config.fences.find(f=>f.id===saved.id).title!=='扫描期间编辑保留')throw Error('Scan overwrote concurrent fence edit');
  const after=await invoke('get_state');const renamed=after.items.find(i=>i.name==='scan-rename-fixture.txt');
  if(!renamed||after.items.some(i=>i.id===fixture.id)||!after.config.fences.some(f=>f.items.includes(renamed.id)))throw Error('Scan/rename serialization lost item mapping');
  await invoke('rename_item',{id:renamed.id,name:'rename-fixture.txt'});
  await invoke('update_fence',{fence:saved});
  await invoke('frontend_checks_passed',{checks:['scan_state_response_ms:'+Math.round(elapsed),'scan_preserves_concurrent_edit','scan_rename_serialization','scan_responsiveness']});
 }catch(e){await invoke('frontend_error',{message:String(e)});}});
})();
