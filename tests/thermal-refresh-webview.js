window.addEventListener('DOMContentLoaded',()=>{
 if(window.__TAURI_INTERNALS__.metadata.currentWindow.label!=='overlay-0')return;
 const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),wait=ms=>new Promise(r=>setTimeout(r,ms));
 const check=(name)=>invoke('frontend_checks_passed',{checks:[name]});
 (async()=>{try{
 let samples=[];const callback=window.__TAURI_INTERNALS__.transformCallback(e=>samples.push(e.payload));
 await invoke('plugin:event|listen',{event:'sysmon',target:{kind:'Any'},handler:callback});
 const s=await invoke('get_state'),m=s.monitors.find(m=>m.primary)||s.monitors[0];
 await invoke('create_widget',{kind:'sysmon',origin:[m.x+50,m.y+50]});
 for(let i=0;i<120&&samples.length<5;i++)await wait(100);
 if(samples.length<5)throw Error('System monitor stopped refreshing: '+samples.length);
 await check('THERMAL_REFRESH_REAL_SAMPLES='+JSON.stringify(samples.map(x=>({cpuTemp:x.cpuTemp,cpuUsage:x.cpuUsage}))));
 const text=()=>document.querySelector('.mon .row.stats span:last-child b')?.textContent;
 for(const temp of [41,67,null,39]){
 await invoke('plugin:event|emit',{event:'sysmon',payload:{...samples.at(-1),cpuTemp:temp}});await wait(80);
 const value=text();if(temp===null?/\d/.test(value):!value?.includes(String(temp)))throw Error('Stale UI for '+temp+': '+value);
 }
 await check('THERMAL_CHANGED_AND_MISSING_VALUES_RENDER');await check('THERMAL_REFRESH_NATIVE_SUCCESS');
 }catch(e){await invoke('frontend_error',{message:String(e.stack||e)});}await invoke('quit_app');})();
});
