(() => {
 const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),wait=ms=>new Promise(r=>setTimeout(r,ms));
 const state=()=>invoke('get_state');const host=id=>document.querySelector(`[data-widget="${id}"]`);
 async function until(fn){for(let i=0;i<400;i++){if(await fn())return;await wait(100);}throw Error('Weather/memory timeout');}
 async function check(name,ok){if(!ok)throw Error(name);await invoke('frontend_checks_passed',{checks:[name]});}
 async function run(){await until(()=>window.DeskBoxWidgets&&window.DeskBoxFenceMerge?.inspect().monitors?.length);await wait(1200);
 const s=await state(),m=s.monitors[0],id=await invoke('create_widget',{kind:'clock',origin:[m.x+80,m.y+80]});
 await until(async()=>!(await state()).config.widgets.find(w=>w.id===id).style.widgetData.weatherAutoLocate);
 const d=(await state()).config.widgets.find(w=>w.id===id).style.widgetData;
 if(d.weatherCity){await until(async()=>!!(await state()).config.widgets.find(w=>w.id===id).style.widgetData.weatherCache);await check('weather_live_ip_city_and_forecast',true);}else{await check('weather_live_ip_failure_manual_fallback',d.weatherLocationError&&host(id).textContent.includes('选择城市查看天气'));}
 await invoke('frontend_checks_passed',{checks:['WEATHER_LOCATION_LIVE_COMPLETED','fence_merge_release']});await invoke('quit_app');}
 window.addEventListener('error',e=>invoke('frontend_error',{message:String(e.message)}));window.addEventListener('unhandledrejection',e=>invoke('frontend_error',{message:String(e.reason)}));
 window.addEventListener('DOMContentLoaded',()=>{const t=setInterval(()=>{if(document.querySelector('#app')?.children.length){clearInterval(t);invoke('frontend_ready',{nodes:1,role:document.documentElement.dataset.role});if(window.__TAURI_INTERNALS__.metadata.currentWindow.label==='overlay-0')run().catch(async e=>{await invoke('frontend_error',{message:String(e.stack||e)});await invoke('quit_app');});}},100);});
})();
