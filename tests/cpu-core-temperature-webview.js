window.addEventListener('DOMContentLoaded',()=>{
 if(window.__TAURI_INTERNALS__.metadata.currentWindow.label!=='overlay-0')return;
 const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),wait=ms=>new Promise(r=>setTimeout(r,ms));
 const check=(n)=>invoke('frontend_checks_passed',{checks:[n]});
 async function until(test){for(let i=0;i<150;i++){if(await test())return;await wait(100);}throw Error('CPU temperature timeout');}
 (async()=>{try{
 await until(()=>window.DeskBoxWidgets&&window.DeskBoxFenceMerge.inspect().monitors?.length);await wait(900);
 const samples=[];await invoke('plugin:event|listen',{event:'sysmon',target:{kind:'Any'},handler:window.__TAURI_INTERNALS__.transformCallback(e=>samples.push(e.payload))});
 const s=await invoke('get_state'),m=s.monitors.find(m=>m.primary)||s.monitors[0];
 const id=await invoke('create_widget',{kind:'sysmon',origin:[m.x+50,m.y+50]});
 await until(()=>document.querySelector(`[data-widget="${id}"] .db-cpu-temperature`));
 await until(()=>samples.filter(s=>s.cpuTempStatus==='ok').length>=3);
 if(!samples.some(s=>s.cpuCoreMin!==null&&s.cpuCoreMax>=s.cpuCoreMin))throw Error('No actual core range');
 await check('CPU_CORE_REAL_SAMPLES='+JSON.stringify(samples.map(s=>[s.cpuCoreMin,s.cpuCoreMax,s.cpuTempStatus])));
 await check('CPU_WIDGET_TARGET='+id+' DOM='+JSON.stringify([...document.querySelectorAll('[data-widget]')].map(x=>x.dataset.widget))+' monitors='+JSON.stringify(s.monitors));
 const widget=()=>document.querySelector(`[data-widget="${id}"]`),temperature=()=>widget()?.querySelector('.db-cpu-temperature');
 await until(()=>temperature()?.querySelector('b')?.textContent===Math.round(samples.at(-1).cpuCoreMax)+'°C');
 for(const [mode,label] of [['min','最低核心'],['max','最高核心'],['both','核心最低 / 最高']]){
 widget().dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,cancelable:true,clientX:100,clientY:100}));
 await until(()=>document.querySelector('.pm'));
 const cpu=[...document.querySelectorAll('.pm-item')].find(n=>n.textContent.trim()==='CPU 温度');if(!cpu)throw Error('CPU submenu missing');cpu.click();
 const wanted=mode==='min'?'最低核心温度':mode==='max'?'最高核心温度':'同时显示最低与最高';
 await until(()=>[...document.querySelectorAll('.pm-item')].some(n=>n.textContent.trim()===wanted));
 [...document.querySelectorAll('.pm-item')].find(n=>n.textContent.trim()===wanted).click();
 await until(()=>temperature()?.querySelector('small')?.textContent===label);
 const saved=(await invoke('get_state')).config.widgets.find(w=>w.id===id);if(saved.style.widgetData.cpuTemperatureMode!==mode)throw Error('Mode not persisted');
 }
 const second=await invoke('create_widget',{kind:'sysmon',origin:[m.x+400,m.y+50]});
 await until(()=>document.querySelector(`[data-widget="${second}"] .db-cpu-temperature small`));
 if(!document.querySelector(`[data-widget="${second}"] .db-cpu-temperature small`).textContent.includes('最高核心'))throw Error('Widget default mode wrong');
 if(!temperature().querySelector('b').textContent.includes('/'))throw Error('Both mode not retained independently');
 await check('CPU_CORE_MENU_MODES_PERSISTED_INDEPENDENTLY');await check('CPU_CORE_TEMPERATURE_NATIVE_SUCCESS');
 }catch(e){await invoke('frontend_error',{message:String(e.stack||e)});}await invoke('quit_app');})();
});
