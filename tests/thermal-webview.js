(async()=>{
 const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a);
 try{
 const s=await invoke('get_state');const m=s.monitors.find(m=>m.primary)||s.monitors[0];
 await invoke('create_widget',{kind:'sysmon',origin:[m.x+50,m.y+50]});
 // Sampling events are also logged by isolated diagnostic mode.
 await new Promise(r=>setTimeout(r,6500));
 const temp=document.querySelector('.mon .row.stats span:last-child b')?.textContent;
 if(!temp||!/[0-9]/.test(temp))throw Error('Temperature missing from widget: '+temp);
 await invoke('frontend_checks_passed',{checks:['THERMAL_WIDGET_RENDERED','THERMAL_VISIBLE='+temp]});
 }catch(e){await invoke('frontend_error',{message:String(e)});}
 await invoke('quit_app');
})();