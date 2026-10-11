(() => {
const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),wait=ms=>new Promise(r=>setTimeout(r,ms));
async function until(fn){for(let i=0;i<160;i++){if(await fn())return;await wait(100);}throw Error('Widget restart timeout');}
async function run(){await until(()=>window.DeskBoxWidgets&&document.querySelector('[data-widget]'));await wait(1000);const s=await invoke('get_state');const notes=s.config.widgets.find(w=>w.kind==='notes'),album=s.config.widgets.find(w=>w.kind==='album'),clock=s.config.widgets.find(w=>w.kind==='clock');
 if(!notes?.style.widgetData?.note||!notes.style.widgetData.todos[0]?.done||notes.style.widgetWidth!==320)throw Error('Notes/todos/geometry not restored');
 if(document.querySelector(`[data-widget="${notes.id}"] textarea`).value!==notes.style.widgetData.note)throw Error('Saved note not displayed');
 await until(()=>document.querySelector(`[data-widget="${album.id}"] img`)?.naturalWidth>0);
 await invoke('frontend_checks_passed',{checks:['widget_restart_data_geometry_asset_scope']});
 await invoke('patch_widget_data',{id:clock.id,patch:{weatherCity:{name:'上海',latitude:31.23,longitude:121.47},weatherCache:null}});
 const w=()=>window.DeskBoxWidgets.inspect(clock.id);await until(()=>w().data.weatherCity?.latitude===31.23);
 const current=(await invoke('get_state')).config.widgets.find(w=>w.id===clock.id);await window.DeskBoxWidgets.menu(current,[]).find(e=>e.label==='刷新天气').action();
 await until(()=>Number.isFinite(w().data.weatherCache?.temperature));await invoke('frontend_checks_passed',{checks:['widget_real_weather_webview_fetch','WIDGETS_RESTART_SUCCESS','fence_merge_release']});await invoke('quit_app');}
window.addEventListener('error',e=>invoke('frontend_error',{message:String(e.message)}));window.addEventListener('unhandledrejection',e=>invoke('frontend_error',{message:String(e.reason)}));
window.addEventListener('DOMContentLoaded',()=>{const t=setInterval(()=>{if(document.querySelector('#app')?.children.length){clearInterval(t);invoke('frontend_ready',{nodes:1,role:document.documentElement.dataset.role});if(window.__TAURI_INTERNALS__.metadata.currentWindow.label==='overlay-0')run().catch(async e=>{await invoke('frontend_error',{message:String(e.stack||e)});await invoke('quit_app');});}},100);});
})();
