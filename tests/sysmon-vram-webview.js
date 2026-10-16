window.addEventListener('DOMContentLoaded',()=>{
 if(window.__TAURI_INTERNALS__.metadata.currentWindow.label!=='overlay-0')return;
 const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),wait=ms=>new Promise(r=>setTimeout(r,ms));
 async function until(test){for(let i=0;i<150;i++){if(await test())return;await wait(50);}throw Error('VRAM timeout');}
 (async()=>{try{
 await until(()=>window.DeskBoxWidgets&&window.DeskBoxFenceMerge.inspect().monitors?.length);await wait(900);
 const s=await invoke('get_state'),m=s.monitors.find(m=>m.primary)||s.monitors[0];
 const id=await invoke('create_widget',{kind:'sysmon',origin:[m.x+50*m.scale,m.y+50*m.scale]});
 const root=()=>document.querySelector(`[data-widget="${id}"]`);
 await until(()=>root()?.querySelector('.db-vram'));
 const memory=root().querySelector('.mon > section:nth-child(3) > .head .name'),vram=()=>root().querySelector('.db-vram .head .name');
 if(getComputedStyle(memory).fontSize!==getComputedStyle(vram()).fontSize||getComputedStyle(memory).fontWeight!==getComputedStyle(vram()).fontWeight)throw Error('Memory and VRAM typography differs');
 if(Math.abs(memory.getBoundingClientRect().left-vram().getBoundingClientRect().left)>1)throw Error('Numeric row alignment differs');
 const emit=async(used,total)=>{await invoke('plugin:event|emit',{event:'sysmon',payload:{cpuName:'CPU',cpuUsage:10,cpuMhz:3000,cpuTempStatus:'enable_required',gpuName:'GPU',gpuUsage:10,gpuMhz:500,gpuTemp:40,memUsed:8*1024**3,memTotal:32*1024**3,vramUsed:used,vramTotal:total}});};
 await emit(2*1024**3,8*1024**3);await until(()=>vram()?.textContent==='2.0 / 8.0 GB');
 if(root().querySelector('.db-vram .bar i').style.transform!=='scaleX(0.25)')throw Error('VRAM bar ratio wrong');
 if(getComputedStyle(root().querySelector('.db-vram .bar')).height!==getComputedStyle(root().querySelector('.mon > section:nth-child(3) > .bar')).height)throw Error('Bar sizing differs');
 await emit(0,8*1024**3);await until(()=>vram()?.textContent==='0.0 / 8.0 GB');
 await emit(null,8*1024**3);await until(()=>vram()?.textContent==='— / 8.0 GB');
 await emit(null,null);await until(()=>!root().querySelector('.db-vram'));
 await emit(4*1024**3,8*1024**3);await until(()=>vram()?.textContent==='4.0 / 8.0 GB');
 await invoke('frontend_checks_passed',{checks:['VRAM_MEMORY_LAYOUT_MATCH','VRAM_VALUES_REFRESH_AND_UNKNOWN_HANDLED','VRAM_DISPLAY_NATIVE_SUCCESS']});
 }catch(e){await invoke('frontend_error',{message:String(e.stack||e)});}await invoke('quit_app');})();});
