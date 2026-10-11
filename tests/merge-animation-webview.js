(() => {
 const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a);
 const wait=ms=>new Promise(r=>setTimeout(r,ms));
 const state=()=>invoke('get_state'); const checks=[];
 const check=(n,v)=>{if(!v)throw Error(n);checks.push(n);invoke('frontend_checks_passed',{checks:[n]});};
 async function until(test){for(let i=0;i<80;i++){if(await test())return;await wait(50)}throw Error('Merge check timeout')}
 const el=id=>[...document.querySelectorAll('[data-fence]')].find(e=>e.dataset.fence===id);
 function start(id){const header=el(id).querySelector('header');header.setPointerCapture=()=>{};header.releasePointerCapture=()=>{};const r=header.getBoundingClientRect();const fire=(kind,x,y)=>header.dispatchEvent(new PointerEvent(kind,{bubbles:true,button:0,buttons:kind==='pointerup'?0:1,pointerId:771,clientX:x,clientY:y}));fire('pointerdown',r.left+20,r.top+15);return fire;}
 async function run(){
   await until(()=>window.DeskBoxFenceMerge?.inspect().fences?.length>0);await wait(1500);
   const initial=await state();const monitor=initial.monitors.find(m=>m.primary)||initial.monitors[0];
   const source=await invoke('create_fence',{origin:[monitor.x+80*monitor.scale,monitor.y+100*monitor.scale]});
   const target=await invoke('create_fence',{origin:[monitor.x+600*monitor.scale,monitor.y+100*monitor.scale]});
   let s=await state();let a=s.config.fences.find(f=>f.id===source),b=s.config.fences.find(f=>f.id===target);
   a.title='合并源测试';a.category='document';a.items=[s.items.find(i=>i.name==='rename-fixture.txt').id,...s.items.filter(i=>i.name!=='rename-fixture.txt').slice(0,3).map(i=>i.id)];a.style.opacity=.63;a.style.iconSize=24;
   b.title='合并目标测试';b.category='other';b.style.opacity=.82;b.style.iconSize=64;b.items=s.items.filter(i=>!a.items.includes(i.id)).slice(0,3).map(i=>i.id);
   await invoke('update_fence',{fence:a});await invoke('update_fence',{fence:b});
   await wait(800);await invoke('frontend_checks_passed',{checks:['merge_dom:'+JSON.stringify([...document.querySelectorAll('[data-fence]')].map(e=>e.dataset.fence)),'merge_store:'+JSON.stringify(window.DeskBoxFenceMerge.inspect())]});
   await until(()=>!!el(source)&&!!el(target));await wait(180);
   const originalItems=a.items.slice();let fire=start(source);fire('pointermove',630,120);await wait(150);
   check('group_preview_bubble',document.querySelector('[data-merge-mode="group"]'));
   s=await state();check('group_preview_no_commit',s.config.fenceGroups.length===0&&s.config.fences.find(f=>f.id===source).items.length===4);
   fire('pointerup',630,120);await until(async()=>(await state()).config.fenceGroups.length===1);
   s=await state();const g=s.config.fenceGroups[0];check('group_release_preserves_properties',s.config.fences.find(f=>f.id===source).style.opacity===.63&&s.config.fences.find(f=>f.id===target).style.opacity===.82);
   await until(()=>!window.DeskBoxFenceMerge.dragging);await wait(420);
   el(target).querySelector('.title').dispatchEvent(new MouseEvent('click',{bubbles:true,detail:1}));
   check('group_title_switch_menu',document.querySelectorAll('.db-fence-switch button').length===2);
   document.querySelector(`[data-member="${source}"]`).click();await until(()=>!!el(source)&&!el(target));
   check('group_active_member_switch',true);fire=start(source);fire('pointermove',1050,600);fire('pointerup',1050,600);
   await until(async()=>(await state()).config.fenceGroups.length===0&&!window.DeskBoxFenceMerge.dragging);await wait(100);
   check('group_drag_detaches_current',!!el(source)&&!!el(target));
   // Long hold changes visuals only. A cancellation must leave persistent state byte-identical.
   const before=JSON.stringify((await state()).config);fire=start(source);fire('pointermove',630,120);await wait(5250);
   check('contents_preview_after_5s',window.DeskBoxFenceMerge.preview==='contents'&&document.querySelector('[data-merge-mode="contents"]'));
   check('held_contents_never_commits',JSON.stringify((await state()).config)===before);
   await wait(750);
   const grid=el(target).querySelector('.grid'), slots=[...grid.querySelectorAll('.db-merge-slot')], real=[...grid.querySelectorAll('.item:not(.db-merge-slot)')];
   check('preview_slots_append_after_existing_items',slots.length===4&&real.length===3&&[...grid.querySelectorAll('.item')].slice(3).every(e=>e.classList.contains('db-merge-slot')));
   const last=real.at(-1).getBoundingClientRect(), first=slots[0].getBoundingClientRect();
   check('preview_first_slot_follows_last_item',first.top>last.top+1||(Math.abs(first.top-last.top)<1&&first.left>=last.right-1));
   const width=real[0].querySelector('.ico').getBoundingClientRect().width;
   check('preview_icons_match_target_size',slots.every(e=>Math.abs(e.querySelector('.ico').getBoundingClientRect().width-width)<1));
   check('preview_has_no_body_particles',document.querySelectorAll('.db-merge-particle').length===0);

   fire('pointercancel',630,120);check('cancel_immediately_restores_in_source',!document.querySelector('.db-merge-slot,.db-merge-particle')&&[...el(source).querySelectorAll('.item')].every(e=>e.style.visibility!=='hidden'));await until(()=>!window.DeskBoxFenceMerge.dragging);await wait(100);
   check('cancel_restores_all',JSON.stringify((await state()).config)===before&&!document.querySelector('.db-merge-bubble,.db-merge-particle,.db-merge-content,.db-merge-header'));
   fire=start(source);fire('pointermove',630,120);await wait(5250);
   check('second_hold_no_restore_point',(await state()).config.mergeRestores.length===0);
   fire('pointerup',630,120);await until(async()=>(await state()).config.mergeRestores.length===1&&!window.DeskBoxFenceMerge.dragging);
   s=await state();check('contents_release_commits',!s.config.fences.some(f=>f.id===source)&&s.config.fences.find(f=>f.id===target).items.includes(originalItems[0]));
   check('contents_target_properties',s.config.fences.find(f=>f.id===target).style.opacity===.82);
   const entries=window.DeskBoxFenceMerge.menu([{separator:true},{label:'删除'}],s.config.fences.find(f=>f.id===target));
   check('restore_submenu',entries.some(e=>e.label==='还原内容合并'&&e.children.length===1));
   await window.DeskBoxFenceMerge.restore(s.config.mergeRestores[0].id,target);s=await state();
   check('restore_consumed_and_split',s.config.mergeRestores.length===0&&s.config.fences.some(f=>f.id===source));
   check('restore_inherits_live_target_properties',s.config.fences.find(f=>f.id===source).style.opacity===.82&&s.config.fences.find(f=>f.id===source).items.includes(originalItems[0]));
   await invoke('frontend_checks_passed',{checks:[...checks,'fence_merge_release']});
 }
 window.addEventListener('error',e=>invoke('frontend_error',{message:String(e.message)}));
 window.addEventListener('unhandledrejection',e=>invoke('frontend_error',{message:String(e.reason)}));
 window.addEventListener('DOMContentLoaded',()=>{let count=0;const timer=setInterval(()=>{if(document.querySelector('#app')?.children.length){clearInterval(timer);const role=document.documentElement.dataset.role;invoke('frontend_ready',{nodes:1,role});if(window.__TAURI_INTERNALS__.metadata.currentWindow.label==='overlay-0')run().catch(e=>invoke('frontend_error',{message:String(e.stack||e)}));}else if(++count>150){clearInterval(timer);invoke('frontend_error',{message:'Merge frontend did not mount'});}},100);});
})();
