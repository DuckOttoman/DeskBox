let forcedReduced=false;const originalMatchMedia=window.matchMedia.bind(window);window.matchMedia=query=>{const media=originalMatchMedia(query);if(query==='(prefers-reduced-motion: reduce)'){Object.defineProperty(media,'matches',{get:()=>forcedReduced});window.motionTestMedia=media;}return media;};
window.addEventListener('DOMContentLoaded',()=>{
 const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),wait=ms=>new Promise(r=>setTimeout(r,ms));
 const check=async(n,v)=>{if(!v)throw Error(n);await invoke('frontend_checks_passed',{checks:[n]});};
 async function until(test){for(let i=0;i<100;i++){if(await test())return;await wait(30);}throw Error('Motion timeout');}
if(window.__TAURI_INTERNALS__.metadata.currentWindow.label!=='overlay-0')return;(async()=>{try{
 await until(()=>window.DeskBoxWidgets&&window.DeskBoxFenceMerge.inspect().monitors?.length);await wait(900);
 const s=await invoke('get_state'),m=s.monitors.find(m=>m.primary)||s.monitors[0];const id=await invoke('create_widget',{kind:'timer',origin:[m.x+100*m.scale,m.y+100*m.scale]});
 await until(()=>document.querySelector(`[data-widget="${id}"] .db-timer`));
 const geometry=(await invoke('get_state')).config.widgets.find(w=>w.id===id);
 await invoke('patch_widget_data',{id,patch:{timer:{mode:'pomodoro',phase:'focus',focusMinutes:25,running:true,deadline:Date.now()-1000}}});
 await until(()=>window.DeskBoxMotion.inspect().timer===1);await check('completion_ring_visible',!!document.querySelector('.db-timer-completion-ring'));
 await check('completion_notice',Array.from(document.querySelectorAll('.db-widget-notice')).some(n=>n.textContent.includes('专注完成')));
 await wait(850);await check('timer_effect_cleanup_once',window.DeskBoxMotion.inspect().active===0&&window.DeskBoxMotion.inspect().timer===1&&!document.querySelector('.db-timer-completion-ring'));
 let w=(await invoke('get_state')).config.widgets.find(w=>w.id===id);await check('timer_state_and_position_preserved',w.x===geometry.x&&w.y===geometry.y&&!w.style.widgetData.timer.running&&w.style.widgetData.timer.phase==='break');
 forcedReduced=true;window.motionTestMedia.dispatchEvent(new Event('change'));
 await invoke('patch_widget_data',{id,patch:{timer:{mode:'countdown',running:true,deadline:Date.now()-1000}}});
 await until(async()=>!(await invoke('get_state')).config.widgets.find(w=>w.id===id).style.widgetData.timer.running);await wait(200);
 await check('reduced_timer_finishes_without_animation',window.DeskBoxMotion.inspect().timer===1&&window.DeskBoxMotion.inspect().active===0&&!document.querySelector('.db-timer-completion-ring'));
 await invoke('frontend_checks_passed',{checks:['TIMER_MOTION_NATIVE_SUCCESS']});
 }catch(e){await invoke('frontend_error',{message:String(e.stack||e)});}await invoke('quit_app');})();});