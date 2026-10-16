let forcedReduced=false;const originalMatchMedia=window.matchMedia.bind(window);window.matchMedia=query=>{const media=originalMatchMedia(query);if(query==='(prefers-reduced-motion: reduce)'){Object.defineProperty(media,'matches',{get:()=>forcedReduced});window.motionTestMedia=media;}return media;};
window.addEventListener('DOMContentLoaded',()=>{
 const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),wait=ms=>new Promise(r=>setTimeout(r,ms));
 const check=async(n,v)=>{if(!v)throw Error(n);await invoke('frontend_checks_passed',{checks:[n]});};
 async function until(test){for(let i=0;i<100;i++){if(await test())return;await wait(30);}throw Error('Motion timeout');}
if(window.__TAURI_INTERNALS__.metadata.currentWindow.label!=='ball')return;(async()=>{try{
 await until(()=>document.querySelector('.ball')&&window.DeskBoxMotion);await wait(600);
 const ball=document.querySelector('.ball'),core=ball.querySelector('.core'),before=ball.getBoundingClientRect();
 await check('ball_breath_active',getComputedStyle(core).animationName==='db-ball-breathe');
 const toggle=()=>ball.dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,cancelable:true,button:2}));
 const close=()=>window.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape'}));
 toggle();await until(()=>document.querySelector('.pm')&&window.DeskBoxMotion.inspect().open===1);
 await check('menu_has_single_animation_layer',getComputedStyle(document.querySelector('.pm')).animationName==='none'&&getComputedStyle(document.querySelector('.pm-item')).animationName==='none');
 await check('ball_breath_stops_for_menu',getComputedStyle(core).animationName==='none');await wait(280);
 close();await until(()=>window.DeskBoxMotion.inspect().clones===1);
 const clone=document.querySelector('.db-ball-menu-ghost');await check('menu_exit_ghost_noninteractive',clone.inert&&clone.getAttribute('aria-hidden')==='true'&&!clone.hasAttribute('data-hit')&&getComputedStyle(clone).pointerEvents==='none');
 await check('ghost_rows_do_not_reanimate',[...clone.querySelectorAll('.pm-item')].every(row=>getComputedStyle(row).animationName==='none'));
 await wait(350);await check('menu_exit_cleanup',window.DeskBoxMotion.inspect().active===0&&window.DeskBoxMotion.inspect().clones===0&&!document.querySelector('.pm'));
 for(let i=0;i<3;i++){toggle();await until(()=>document.querySelector('.pm'));await wait(70);close();await wait(260);}
 await check('repeated_menu_cleanup',window.DeskBoxMotion.inspect().active===0&&window.DeskBoxMotion.inspect().clones===0);
 toggle();await until(()=>document.querySelector('.pm:not(.db-ball-menu-ghost)'));close();await wait(25);toggle();await until(()=>document.querySelector('.pm:not(.db-ball-menu-ghost)'));await wait(280);close();await wait(300);await check('rapid_reopen_no_ghost_leak',window.DeskBoxMotion.inspect().clones===0&&window.DeskBoxMotion.inspect().active===0);
 const after=ball.getBoundingClientRect();await check('ball_hit_geometry_unchanged',Math.abs(before.width-after.width)<.1&&Math.abs(before.left-after.left)<.1);
 forcedReduced=true;window.motionTestMedia.dispatchEvent(new Event('change'));
 await check('reduced_motion_stops_ball_breath',getComputedStyle(core).animationName==='none');const counts=window.DeskBoxMotion.inspect();
 toggle();await until(()=>document.querySelector('.pm'));await wait(50);await check('reduced_menu_css_disabled',getComputedStyle(document.querySelector('.pm')).animationName==='none'&&getComputedStyle(document.querySelector('.pm-item')).animationName==='none'&&getComputedStyle(document.querySelector('.pm-sub')).transitionDuration==='0s');close();await wait(50);
 const reduced=window.DeskBoxMotion.inspect();await check('reduced_menu_has_no_effects',reduced.open===counts.open&&reduced.close===counts.close&&reduced.clones===0&&reduced.active===0);
 await invoke('frontend_checks_passed',{checks:['BALL_MOTION_NATIVE_SUCCESS']});
 }catch(e){await invoke('frontend_error',{message:String(e.stack||e)});}await invoke('quit_app');})();});