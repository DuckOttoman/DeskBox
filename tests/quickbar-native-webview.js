window.addEventListener('DOMContentLoaded',()=>{
 if(window.__TAURI_INTERNALS__.metadata.currentWindow.label==='ball'){
  const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),wait=ms=>new Promise(r=>setTimeout(r,ms));
  (async()=>{const seen=new Set();for(let i=0;i<100&&seen.size<2;i++){await wait(300);const ball=document.querySelector('.ball');if(!ball||!window.DeskBoxQuickbar)continue;const hidden=(await invoke('get_state')).config.settings.hideTaskbar;if(seen.has(hidden))continue;ball.dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,cancelable:true,button:2}));await wait(70);const label=hidden?'恢复任务栏':'隐藏任务栏';const found=[...document.querySelectorAll('.pm:not(.db-ball-menu-ghost) .pm-item')].some(n=>n.textContent.trim()===label);window.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape'}));if(found){seen.add(hidden);await invoke('frontend_checks_passed',{checks:[hidden?'BALL_TASKBAR_LABEL_RESTORE_OK':'BALL_TASKBAR_LABEL_HIDE_OK']});}}})().catch(e=>invoke('frontend_error',{message:String(e.stack||e)}));return;
 }
 if(window.__TAURI_INTERNALS__.metadata.currentWindow.label!=='overlay-0')return;
 const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),wait=ms=>new Promise(r=>setTimeout(r,ms));
 async function until(test){for(let i=0;i<200;i++){if(await test())return;await wait(50);}throw Error('Quickbar timeout');}
 (async()=>{try{
  await until(()=>window.DeskBoxQuickbar&&window.DeskBoxFenceMerge.inspect().monitors?.length);await wait(1000);
  let s=await invoke('get_state');const m=s.monitors.find(m=>m.primary)||s.monitors[0],item=s.items.find(i=>i.path.endsWith('rename-fixture.txt'));
  if(!item)throw Error('Fixture unavailable');
  await invoke('quickbar_pin',{paths:[item.path,item.path]});s=await invoke('get_state');if(s.config.quickbar.pins.length!==1)throw Error('Pin dedupe failed');
  await invoke('quickbar_remove',{path:item.path});s=await invoke('get_state');if(s.config.quickbar.pins.length)throw Error('Remove failed');
  await invoke('quickbar_undo',{path:item.path});
  const settings={...s.config.settings,hideTaskbar:true,quickbarCount:12};await invoke('update_settings',{settings});
  await until(()=>invoke('quickbar_test_status'));
  await invoke('quickbar_remove',{path:item.path});await wait(150);
  const source=s.config.fences.find(f=>f.items.includes(item.id));if(!source)throw Error('Source frame not found');
  const cell=[...document.querySelectorAll('[data-fence]')].find(n=>n.dataset.fence===source.id)?.querySelectorAll('[data-item]');const sourceCell=[...(cell||[])].find(n=>n.dataset.item===item.id);if(!sourceCell)throw Error('Source frame icon absent');
  const rect=sourceCell.getBoundingClientRect(),x=rect.left+rect.width/2,y=rect.top+rect.height/2,dropX=m.w/m.scale/2,dropY=m.h/m.scale-40;
  const pointer=(type,px,py)=>new PointerEvent(type,{bubbles:true,pointerId:71,button:0,buttons:type==='pointerup'?0:1,clientX:px,clientY:py});
  sourceCell.dispatchEvent(pointer('pointerdown',x,y));window.dispatchEvent(pointer('pointermove',dropX,dropY));await wait(100);window.dispatchEvent(pointer('pointercancel',dropX,dropY));
  if((await invoke('get_state')).config.quickbar.pins.length)throw Error('Cancelled frame drag pinned');
  sourceCell.dispatchEvent(pointer('pointerdown',x,y));window.dispatchEvent(pointer('pointermove',dropX,dropY));await wait(500);await invoke('frontend_checks_passed',{checks:['QUICKBAR_DRAG_CAPTURE_READY']});await wait(2000);window.dispatchEvent(pointer('pointerup',dropX,dropY));
  await until(async()=>!!(await invoke('get_state')).config.quickbar.pins.length);
  if(!(await invoke('get_state')).config.fences.find(f=>f.id===source.id)?.items.includes(item.id))throw Error('Pin removed original frame icon');
  await invoke('frontend_checks_passed',{checks:['QUICKBAR_FRAME_SOURCE_DRAG_SUCCESS']});
  await invoke('quickbar_remove',{path:item.path});await wait(1600);await invoke('frontend_checks_passed',{checks:['QUICKBAR_POLISH_CAPTURE_READY']});await wait(2000);await invoke('quickbar_pin',{paths:[item.path]});


  s=await invoke('get_state');const mm=s.monitors.find(x=>x.name===m.name),expected=m.h/m.scale-96;
  if(Math.abs(mm.work[1]+mm.work[3]-expected)>3)throw Error('Work-area reservation wrong '+JSON.stringify(mm));
  const fid=await invoke('create_fence',{origin:[m.x+100*m.scale,m.y+(m.h/m.scale-80)*m.scale]});s=await invoke('get_state');let f=s.config.fences.find(f=>f.id===fid);
  if(f.y+f.h>expected+1)throw Error('Fence creation crossed dock');
  f.y=m.h/m.scale;await invoke('update_fence',{fence:f});s=await invoke('get_state');f=s.config.fences.find(f=>f.id===fid);if(f.y+f.h>expected+1)throw Error('Persistence crossed dock');
  const preview=window.DeskBoxQuickbar.constrain(100,m.h/m.scale,320,260);if(preview.y+260>expected+1)throw Error('Preview crossed dock');
  const wid=await invoke('create_widget',{kind:'notes',origin:[m.x+120*m.scale,m.y+m.h-10]});s=await invoke('get_state');let w=s.config.widgets.find(x=>x.id===wid);w.y=m.h/m.scale;w.style.widgetHeight=400;await invoke('update_widget',{widget:w});s=await invoke('get_state');w=s.config.widgets.find(x=>x.id===wid);if(w.y+w.style.widgetHeight>expected+1)throw Error('Widget resize crossed dock');
  const other=await invoke('create_fence',{origin:[m.x+500*m.scale,m.y+120*m.scale]});const gid=await invoke('group_fences',{source:fid,target:other});await invoke('move_fence_group',{group:gid,point:[300,m.h/m.scale]});s=await invoke('get_state');const group=s.config.fenceGroups.find(g=>g.id===gid);for(const id of group.members){const f=s.config.fences.find(f=>f.id===id);if(group.y+(f.collapsed?38:f.h)>expected+1)throw Error('Group crossed dock');}
  await invoke('frontend_checks_passed',{checks:['QUICKBAR_ENABLED','QUICKBAR_REFERENCE_AND_PROTECTION_SUCCESS']});
  // Orchestrator kills only this test main process in crash mode, leaving its guard alive.
  if(window.__DB_CRASH_TEST){await wait(60000);return;}
  await wait(5000);
  for(let i=0;i<3;i++){await invoke('update_settings',{settings:{...settings,hideTaskbar:false}});await invoke('update_settings',{settings});await until(()=>invoke('quickbar_test_status'));}
  await invoke('frontend_checks_passed',{checks:['QUICKBAR_FAST_TOGGLE_SUCCESS']});
  await invoke('update_settings',{settings:{...settings,hideTaskbar:false}});await invoke('frontend_checks_passed',{checks:['QUICKBAR_DISABLED']});
  await invoke('update_settings',{settings});await invoke('frontend_checks_passed',{checks:['QUICKBAR_REENABLED_FOR_EXIT']});
 }catch(e){await invoke('frontend_error',{message:String(e.stack||e)});}await invoke('quit_app');})();
});
