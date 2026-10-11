(() => {
const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),wait=ms=>new Promise(r=>setTimeout(r,ms)),state=()=>invoke('get_state');
async function until(fn){for(let i=0;i<160;i++){if(await fn())return;await wait(100);}throw Error('Widget timeout')}
async function check(name,ok){if(!ok)throw Error(name);await invoke('frontend_checks_passed',{checks:[name]});}
const host=id=>document.querySelector(`[data-widget="${id}"]`);
async function run(){
 await until(()=>window.DeskBoxWidgets&&window.DeskBoxFenceMerge?.inspect().monitors?.length);await wait(1200);
 const s=await state(),m=s.monitors[0],origin=[m.x+80*m.scale,m.y+80*m.scale];
 const ids={};for(const kind of ['clock','sysmon','timer','notes','album']){ids[kind]=await invoke('create_widget',{kind,origin});if(kind==='clock')await invoke('patch_widget_data',{id:ids[kind],patch:{weatherAutoLocate:false}});await until(()=>host(ids[kind]));}
 await check('widgets_existing_and_new_kinds',Object.keys(ids).length===5&&host(ids.sysmon).querySelector('.mon'));
 const notes=host(ids.notes),textarea=notes.querySelector('textarea');textarea.value='重启后保留的便签';textarea.dispatchEvent(new Event('input',{bubbles:true}));
 await until(async()=>(await state()).config.widgets.find(w=>w.id===ids.notes).style.widgetData?.note===textarea.value);await check('widget_notes_autosaved',true);
 const input=notes.querySelector('.db-todo-add input');input.value='测试待办';input.dispatchEvent(new Event('input',{bubbles:true}));input.dispatchEvent(new KeyboardEvent('keydown',{bubbles:true,key:'Enter'}));
 await until(()=>host(ids.notes).querySelector('.db-todo input'));host(ids.notes).querySelector('.db-todo input').click();
 await until(async()=>(await state()).config.widgets.find(w=>w.id===ids.notes).style.widgetData.todos[0].done===true);await check('widget_todo_add_toggle_saved',true);
 for(const text of ['快速添加一','快速添加二']){input.value=text;input.dispatchEvent(new Event('input',{bubbles:true}));input.dispatchEvent(new KeyboardEvent('keydown',{bubbles:true,key:'Enter'}));}await until(async()=>(await state()).config.widgets.find(w=>w.id===ids.notes).style.widgetData.todos.length===3);await check('widget_rapid_todo_add_preserves_both',true);
 const node=host(ids.notes);node.setPointerCapture=()=>{};node.hasPointerCapture=()=>false;
 const handle=node.querySelector('.db-widget-resize.rb'),r=node.getBoundingClientRect();
 handle.dispatchEvent(new PointerEvent('pointerdown',{bubbles:true,pointerId:1,button:0,clientX:r.right-2,clientY:r.bottom-2}));
 window.dispatchEvent(new PointerEvent('pointermove',{bubbles:true,pointerId:1,clientX:r.right+148,clientY:r.bottom+98}));
 await invoke('patch_widget_data',{id:ids.notes,patch:{resizeProbe:true}});await wait(200);await check('widget_resize_survives_background_publish',Math.round(node.getBoundingClientRect().width)===470);
 window.dispatchEvent(new PointerEvent('pointerup',{bubbles:true,pointerId:1,button:0,clientX:r.right+148,clientY:r.bottom+98}));
 await until(()=>host(ids.notes).dataset.shape==='large');await until(async()=>(await state()).config.widgets.find(w=>w.id===ids.notes).style.widgetWidth===470);await check('widget_drag_resize_large_persisted',true);
 let w=(await state()).config.widgets.find(w=>w.id===ids.notes);w.style.widgetWidth=220;w.style.widgetHeight=150;await invoke('update_widget',{widget:w});
 await until(()=>host(ids.notes).dataset.shape==='small'&&!host(ids.notes).querySelector('.db-todo-add'));await check('widget_small_layout_changes_content',true);
 w=(await state()).config.widgets.find(w=>w.id===ids.notes);w.style.widgetWidth=320;w.style.widgetHeight=240;await invoke('update_widget',{widget:w});await until(()=>host(ids.notes).dataset.shape==='medium');
 await invoke('patch_widget_data',{id:ids.timer,patch:{timer:{mode:'pomodoro',phase:'focus',focusMinutes:25,breakMinutes:5,running:true,deadline:Date.now()+500}}});
 await until(async()=>(await state()).config.widgets.find(w=>w.id===ids.timer).style.widgetData.timer.phase==='break');await check('widget_timer_deadline_completes_once',true);
 let timerHost=host(ids.timer);[...timerHost.querySelectorAll('button')].find(b=>b.textContent==='开始').click();await until(async()=>(await state()).config.widgets.find(w=>w.id===ids.timer).style.widgetData.timer.running);
 [...host(ids.timer).querySelectorAll('button')].find(b=>b.textContent==='暂停').click();await until(async()=>(await state()).config.widgets.find(w=>w.id===ids.timer).style.widgetData.timer.running===false);await check('widget_timer_start_pause',true);
 const fixture=s.items.find(i=>i.name==='media-test.gif');if(!fixture)throw Error('fixture unavailable');const storage=fixture.path.replace(/[/\\]rename-fixtures[/\\].*$/,'')+'/album-storage';
 await invoke('widget_album_action',{id:ids.album,action:'directory',directory:storage,index:null,sources:null});
 const png=fixture.path.replace('media-test.gif','photo-one.png'),png2=fixture.path.replace('media-test.gif','photo-two.png');
 await invoke('widget_album_action',{id:ids.album,action:'add',sources:[png,png2],index:null,directory:null});
 await until(()=>host(ids.album).querySelector('img')?.naturalWidth>0);await check('widget_album_import_and_asset_loaded',true);
 const defaultAlbum=await invoke('create_widget',{kind:'album',origin});await invoke('widget_album_action',{id:defaultAlbum,action:'add',sources:[png],index:null,directory:null});const defaultData=(await state()).config.widgets.find(w=>w.id===defaultAlbum).style.widgetData;await check('widget_album_defaults_to_program_albums',/[/\\]Albums$/i.test(defaultData.albumDirectory)&&!defaultData.albumDirectory.toLowerCase().includes('desktop'));
 const first=host(ids.album).querySelector('img').src;await invoke('patch_widget_data',{id:ids.album,patch:{interval:2,fit:'contain'}});await until(()=>host(ids.album).querySelector('img').src!==first);await check('widget_album_rotates_images',true);
 await invoke('patch_widget_data',{id:ids.album,patch:{paused:true}});await wait(1000);const paused=host(ids.album).querySelector('img').src;await wait(2300);await check('widget_album_pause_stops_rotation',host(ids.album).querySelector('img').src===paused);
 let denied=false;try{await invoke('patch_widget_data',{id:ids.album,patch:{photos:[{path:'C:/fake.png'}]}})}catch(e){denied=true}await check('widget_album_arbitrary_path_rejected',denied);
 // Deterministic provider transport, while real production endpoint is separately checked.
 const originalFetch=window.fetch;window.fetch=async(url,...args)=>String(url).startsWith('https://api.open-meteo.com/')?{ok:true,json:async()=>({current:{temperature_2m:23,weather_code:0,relative_humidity_2m:60},daily:{time:['2026-10-10','2026-10-11','2026-10-12'],temperature_2m_min:[17,18,19],temperature_2m_max:[24,25,26],weather_code:[0,3,61]}})}:originalFetch(url,...args);
 await invoke('patch_widget_data',{id:ids.clock,patch:{weatherCity:{name:'测试城市',latitude:31.23,longitude:121.47}}});await until(()=>host(ids.clock).textContent.includes('23° 晴'));await check('widget_clock_weather_integrated',true);
 w=(await state()).config.widgets.find(w=>w.id===ids.clock);w.style.widgetWidth=460;w.style.widgetHeight=340;await invoke('update_widget',{widget:w});await until(()=>host(ids.clock).querySelectorAll('.db-weather-days>div').length===3);await check('widget_clock_large_forecast',true);
 host(ids.album).dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:130,clientY:130}));await until(()=>[...document.querySelectorAll('.pm-item')].some(e=>e.textContent.includes('添加图片')));await check('widget_album_menu_actions',true);
 await invoke('frontend_checks_passed',{checks:['fence_merge_release','WIDGETS_NATIVE_SUCCESS']});await invoke('quit_app');
}
window.addEventListener('error',e=>invoke('frontend_error',{message:String(e.message)}));window.addEventListener('unhandledrejection',e=>invoke('frontend_error',{message:String(e.reason)}));
window.addEventListener('DOMContentLoaded',()=>{const timer=setInterval(()=>{if(document.querySelector('#app')?.children.length){clearInterval(timer);invoke('frontend_ready',{nodes:1,role:document.documentElement.dataset.role});if(window.__TAURI_INTERNALS__.metadata.currentWindow.label==='overlay-0')run().catch(async e=>{await invoke('frontend_error',{message:String(e.stack||e)});await invoke('quit_app');});}},100);});
})();
