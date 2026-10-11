window.DeskBoxLearningCheck=async function(invoke){
 const state=()=>invoke('get_state');
 let s=await state();const checks=[];
 const check=(name,value)=>{if(!value)throw Error(name);checks.push(name)};
 for(const f of s.config.fences.filter(f=>!s.monitors.some(m=>m.name===f.monitor))){
   const old=f.monitor;f.x+=17;await invoke('update_fence',{fence:f});
   const saved=(await state()).config.fences.find(x=>x.id===f.id);
   check('orphan_fence_ipc_save',saved.x===f.x&&saved.monitor===old);
 }
 for(const w of s.config.widgets.filter(w=>!s.monitors.some(m=>m.name===w.monitor))){
   const old=w.monitor;w.y+=13;await invoke('update_widget',{widget:w});
   const saved=(await state()).config.widgets.find(x=>x.id===w.id);
   check('orphan_widget_ipc_save',saved.y===w.y&&saved.monitor===old);
 }
 const link=s.items.find(x=>x.name==='learn-alias');
 const url=s.items.find(x=>x.name==='learn-url');
 if(link&&url){
   const id=await invoke('create_fence',{});
   const f=(await state()).config.fences.find(x=>x.id===id);f.category='media';
   await invoke('update_fence',{fence:f});
   await invoke('move_item',{item:link.id,to:id});await invoke('move_item',{item:url.id,to:id});
   s=await state();
   check('native_lnk_target_learning',s.config.learned['t:notepad.exe']==='media');
   check('native_url_learning',s.config.learned['u:https://example.com/path?q=a']==='media');
   check('native_move_keeps_scan_category',s.items.find(x=>x.id===link.id).category===link.category);
   await invoke('organize_now',{full:true});s=await state();
   const alias=s.items.find(x=>x.name==='learn-other-alias');
   check('native_alias_shared_learning',s.config.fences.some(x=>x.category==='media'&&x.items.includes(alias.id)));
   await invoke('reset_learned');s=await state();
   check('native_reset_learning',Object.keys(s.config.learned).length===0);
 }
 if(checks.length)await invoke('frontend_checks_passed',{checks});
};

