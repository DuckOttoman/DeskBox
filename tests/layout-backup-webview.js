window.addEventListener("DOMContentLoaded",()=>{if(window.__TAURI_INTERNALS__.metadata.currentWindow.label!=="overlay-0")return;
(async()=>{

const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),wait=ms=>new Promise(r=>setTimeout(r,ms));
const state=()=>invoke('get_state'),action=(action,id=null,name=null)=>invoke('layout_backup_action',{action,id,name});
const check=async(n,v)=>{if(!v)throw Error(n);await invoke('frontend_checks_passed',{checks:[n]});};
async function until(test){for(let i=0;i<100;i++){if(await test())return;await wait(60);}throw Error('Layout backup check timed out');}
try{
 await until(()=>window.DeskBoxWidgets&&window.DeskBoxFenceMerge.inspect().monitors?.length);await wait(1200);
 let s=await state();const m=s.monitors.find(m=>m.primary)||s.monitors[0];
 const a=await invoke('create_fence',{origin:[m.x+160*m.scale,m.y+90*m.scale]}),z=await invoke('create_fence',{origin:[m.x+600*m.scale,m.y+90*m.scale]});
 s=await state();let f=s.config.fences.find(f=>f.id===a);f.category='document';f.style.opacity=.62;f.w=360;await invoke('update_fence',{fence:f});
 const gid=await invoke('group_fences',{source:a,target:z});
 const notes=await invoke('create_widget',{kind:'notes',origin:[m.x+80*m.scale,m.y+400*m.scale]});
 await invoke('patch_widget_data',{id:notes,patch:{note:'before backup',todos:[{id:'one',text:'original',done:false}]}});await wait(250);
 const initial=await state(),g=initial.config.fenceGroups.find(g=>g.id===gid);
 await invoke('request_layout_backup',{action:'create'});await until(async()=> (await action('list')).backups.length===1);
 const backup=(await action('list')).backups[0];await check('backup_created_from_ball_action',backup.createdAt>0&&backup.name.startsWith('布局备份 '));
 await invoke('move_fence_group',{group:gid,point:[500,350]});s=await state();f=s.config.fences.find(f=>f.id===a);f.style.opacity=.95;f.w=440;await invoke('update_fence',{fence:f});
 await invoke('patch_widget_data',{id:notes,patch:{note:'latest survives restore',todos:[{id:'two',text:'latest',done:true}]}});
 await invoke('request_layout_backup',{action:'manage'});await until(()=>document.querySelector(`[data-backup="${backup.id}"] time`));
 let row=document.querySelector(`[data-backup="${backup.id}"]`);await check('backup_time_visible',row.querySelector('time').textContent.startsWith('备份时间：')&&row.querySelector('time').dateTime===new Date(backup.createdAt).toISOString());
 const click=(row,text)=>[...row.querySelectorAll('button')].find(b=>b.textContent===text).click();
 click(row,'重命名');row.querySelector('input').value='我的工作布局';click(row,'保存名称');await until(async()=> (await action('list')).backups[0].name==='我的工作布局');
 await check('rename_keeps_backup_time',(await action('list')).backups[0].createdAt===backup.createdAt);await wait(100);
 row=document.querySelector(`[data-backup="${backup.id}"]`);click(row,'还原');await until(async()=> (await state()).config.fenceGroups.find(x=>x.id===gid)?.x===g.x);
 s=await state();f=s.config.fences.find(f=>f.id===a);const w=s.config.widgets.find(w=>w.id===notes);
 await check('restore_group_position_and_member_properties',s.config.fenceGroups.find(x=>x.id===gid).y===g.y&&f.w===360&&f.style.opacity===.62);
 await check('restore_keeps_current_widget_contents',w.style.widgetData.note==='latest survives restore'&&w.style.widgetData.todos[0].done===true);
 const all=s.config.fences.flatMap(f=>f.items);await check('restore_current_desktop_classification',new Set(all).size===s.items.length&&all.length===s.items.length&&s.items.every(i=>s.config.fences.some(f=>f.items.includes(i.id)&&(f.category===i.category||f.category==='inbox'))));
 await until(async()=> (await action('list')).backups.length===2);await check('automatic_pre_restore_backup',(await action('list')).backups.some(b=>b.name.startsWith('还原前自动备份')));await wait(150);
 row=document.querySelector(`[data-backup="${backup.id}"]`);click(row,'删除');await check('delete_requires_explicit_second_click',(await action('list')).backups.some(b=>b.id===backup.id));click(row,'确认删除');
 await until(async()=> !(await action('list')).backups.some(b=>b.id===backup.id));await check('backup_deleted',true);

 const victim=await invoke('create_widget',{kind:'notes',origin:[m.x+900*m.scale,m.y+400*m.scale]});
 await until(()=>document.querySelector(`[data-widget="${victim}"] .db-note`));
 const input=document.querySelector(`[data-widget="${victim}"] .db-note`);input.value='typed private draft';input.dispatchEvent(new Event('input',{bubbles:true}));await wait(650);
 await window.DeskBoxWidgets.createLayoutBackup();const victimBackup=(await action('list')).backups.find(b=>b.name.startsWith('布局备份 '));
 await invoke('delete_widget',{id:victim});await wait(150);
 await window.DeskBoxWidgets.layoutAction('restore',victimBackup.id);await until(()=>document.querySelector(`[data-widget="${victim}"] .db-note`));await wait(200);
 const restored=(await state()).config.widgets.find(w=>w.id===victim);
 await check('deleted_note_restores_empty_backend_and_editor',!restored.style.widgetData.note&&document.querySelector(`[data-widget="${victim}"] .db-note`).value==='');
 await invoke('frontend_checks_passed',{checks:['LAYOUT_BACKUP_NATIVE_SUCCESS']});
}catch(e){await invoke('frontend_error',{message:String(e)});}
await invoke('quit_app');
})();
});
