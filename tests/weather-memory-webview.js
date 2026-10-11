(() => {
 const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),wait=ms=>new Promise(r=>setTimeout(r,ms));
 const realFetch=window.fetch;let ipCalls=0;
 window.fetch=async(url,...args)=>{if(String(url)==='https://ipapi.co/json/'){ipCalls++;return {ok:true,json:async()=>({city:'Shanghai',latitude:31.23,longitude:121.47})};}if(String(url).startsWith('https://api.open-meteo.com/'))return {ok:true,json:async()=>({current:{temperature_2m:20,weather_code:95,relative_humidity_2m:50},daily:{time:['2026-10-10'],weather_code:[73],temperature_2m_min:[10],temperature_2m_max:[22]}})};return realFetch(url,...args);};
 const state=()=>invoke('get_state');const host=id=>document.querySelector(`[data-widget="${id}"]`);
 async function until(fn){for(let i=0;i<160;i++){if(await fn())return;await wait(100);}throw Error('Weather/memory timeout');}
 async function check(name,ok){if(!ok)throw Error(name);await invoke('frontend_checks_passed',{checks:[name]});}
 async function run(){await until(()=>window.DeskBoxWidgets&&window.DeskBoxFenceMerge?.inspect().monitors?.length);await wait(1200);
 const s=await state(),m=s.monitors[0],id=await invoke('create_widget',{kind:'clock',origin:[m.x+80,m.y+80]});
 await until(async()=>!!(await state()).config.widgets.find(w=>w.id===id).style.widgetData.weatherCache);
 const get=async()=>(await state()).config.widgets.find(w=>w.id===id);
 await check('weather_ip_created_city_saved',ipCalls===1&&(await get()).style.widgetData.weatherCity.source==='ip');
 await check('weather_medium_emoji',host(id).textContent.includes('⛈️'));
 let w=await get();w.style.widgetWidth=220;w.style.widgetHeight=150;await invoke('update_widget',{widget:w});await until(()=>host(id).dataset.shape==='small');await check('weather_small_omits_emoji',!host(id).textContent.includes('⛈️'));
 w=await get();w.style.widgetWidth=460;w.style.widgetHeight=320;await invoke('update_widget',{widget:w});await until(()=>host(id).textContent.includes('❄️'));await check('weather_large_forecast_emoji',true);
 await invoke('patch_widget_data',{id,patch:{weatherCity:{name:'Manual city',latitude:10,longitude:20}}});
 const applied=await invoke('apply_weather_location',{id,city:{name:'Wrong city',latitude:30,longitude:40}});await check('weather_auto_never_overrides_manual',!applied&&(await get()).style.widgetData.weatherCity.name==='Manual city');
 const before=JSON.stringify((await state()).config);const report=await invoke('optimize_memory');await check('memory_native_report:'+JSON.stringify(report),report.trimmedProcesses>=1&&Number.isFinite(report.beforeBytes)&&Number.isFinite(report.afterBytes));await wait(10100);await window.DeskBoxWidgets.optimizeMemory();
 await check('memory_ui_report', [...document.querySelectorAll('.db-widget-notice')].some(n=>n.textContent.includes('内存优化完成')));
 await check('memory_does_not_modify_configuration',before===JSON.stringify((await state()).config));
 let throttled=false;try{await invoke('optimize_memory');}catch(e){throttled=String(e).includes('10秒');}await check('memory_backend_cooldown',throttled);
 await invoke('frontend_checks_passed',{checks:['WEATHER_MEMORY_SUCCESS','fence_merge_release']});await invoke('quit_app');}
 window.addEventListener('error',e=>invoke('frontend_error',{message:String(e.message)}));window.addEventListener('unhandledrejection',e=>invoke('frontend_error',{message:String(e.reason)}));
 window.addEventListener('DOMContentLoaded',()=>{const t=setInterval(()=>{if(document.querySelector('#app')?.children.length){clearInterval(t);invoke('frontend_ready',{nodes:1,role:document.documentElement.dataset.role});if(window.__TAURI_INTERNALS__.metadata.currentWindow.label==='overlay-0')run().catch(async e=>{await invoke('frontend_error',{message:String(e.stack||e)});await invoke('quit_app');});}},100);});
})();
