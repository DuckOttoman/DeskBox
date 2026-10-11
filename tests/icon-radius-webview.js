(() => {
const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),wait=ms=>new Promise(r=>setTimeout(r,ms));
async function until(fn){for(let i=0;i<160;i++){if(await fn())return;await wait(100)}throw Error('Media timeout')}
const state=()=>invoke('get_state');
async function check(name,ok){if(!ok)throw Error(name);await invoke('frontend_checks_passed',{checks:[name]})}
async function run(){
 await until(()=>window.DeskBoxMedia&&window.DeskBoxFenceMerge?.inspect().fences.length);await wait(1500);
 const s=await state(),m=s.monitors[0],id=await invoke('create_fence',{origin:[m.x+100*m.scale,m.y+100*m.scale]});
 const fixture=s.items.find(i=>i.name==='media-test.mp4');if(!fixture)throw Error('fixture missing');
 let prepared=(await state()).config.fences.find(f=>f.id===id);prepared.style.radius=26;prepared.style.imageBlur=8;prepared.items=[fixture.id,s.items.find(i=>i.name==='rename-fixture.txt').id];await invoke('update_fence',{fence:prepared});await invoke('import_fence_background',{id,source:fixture.path});await invoke('keyboard_focus',{on:true});
 await wait(2500);await invoke('frontend_checks_passed',{checks:['media_diagnostic:'+JSON.stringify({visibility:document.visibilityState,policy:await invoke('media_playback_allowed'),entries:window.DeskBoxMedia.inspect(),videoCount:document.querySelectorAll('video').length})]});
 const video=()=>document.querySelector(`[data-fence="${id}"] video.db-media-background`);
 await until(()=>video());await check('media_rounded_clip_matches_fence',getComputedStyle(document.querySelector(`[data-fence="${id}"] > .bg`)).clipPath.includes('26px'));
 await until(()=>video()?.videoWidth===854&&video()?.videoHeight===480&&!video().paused);
 await check('media_eco_actual_480p',true);const t=video().currentTime;await wait(350);await check('media_clock_advances',video().currentTime!==t);
 document.querySelector(`[data-fence="${id}"] header`).dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:130,clientY:130}));
 await until(()=>[...document.querySelectorAll('.pm-item')].some(e=>e.textContent.includes('外观')));
 [...document.querySelectorAll('.pm-item')].find(e=>e.textContent.includes('外观')).click();
 await until(()=>document.querySelector('.db-video-mode'));
 const select=document.querySelector('.db-video-mode');await check('media_two_ui_modes',select.options.length===2&&select.value==='eco');select.value='original';select.dispatchEvent(new Event('change',{bubbles:true}));
 await until(()=>video()?.videoWidth===1280&&video()?.videoHeight===720&&!video().paused);await check('media_original_actual_resolution',true);
 await until(()=>document.querySelector(`[data-fence="${id}"] .image`).style.visibility==='hidden');await check('media_no_poster_ghosting',true);
 let rounded=(await state()).config.fences.find(f=>f.id===id);rounded.style.radius=28;await invoke('update_fence',{fence:rounded});await until(()=>getComputedStyle(document.querySelector(`[data-fence="${id}"] > .bg`)).clipPath.includes('28px'));await check('media_radius_change_updates_clip',true);
 document.querySelector('.sp .x')?.click();
 const host=()=>document.querySelector(`[data-fence="${id}"]`);
 const openMenu=()=>host().querySelector('header').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:130,clientY:130}));
 await until(()=>host().querySelectorAll('.grid .item .label').length===2);
 openMenu();await until(()=>[...document.querySelectorAll('.pm-item')].some(e=>e.textContent==='隐藏图标名'));[...document.querySelectorAll('.pm-item')].find(e=>e.textContent==='隐藏图标名').click();
 await until(()=>[...host().querySelectorAll('.grid .item .label')].every(e=>getComputedStyle(e).display==='none'));await check('icon_names_hidden_preserve_items_and_tooltips',host().querySelectorAll('.grid .item[title]').length===2&&(await state()).config.fences.find(f=>f.id===id).style.hideIconNames===true);
 openMenu();await until(()=>[...document.querySelectorAll('.pm-item')].some(e=>e.textContent==='显示图标名'));[...document.querySelectorAll('.pm-item')].find(e=>e.textContent==='显示图标名').click();await until(()=>host().querySelectorAll('.grid .item .label').length===2&&[...host().querySelectorAll('.grid .item .label')].every(e=>getComputedStyle(e).display!=='none'));await check('icon_names_show_restores_labels',true);
 let f=(await state()).config.fences.find(f=>f.id===id);f.collapsed=true;await invoke('update_fence',{fence:f});
 await until(()=>!video()?.getAttribute('src'));await check('media_collapsed_releases_source',true);
 f.collapsed=false;await invoke('update_fence',{fence:f});await until(()=>video()?.videoWidth===1280&&!video()?.paused);
 const gif=s.items.find(i=>i.name==='media-test.gif');await invoke('import_fence_background',{id,source:gif.path});
 await until(()=>document.querySelector(`[data-fence="${id}"] img.db-media-background`)?.naturalWidth===160);await check('media_gif_loaded',true);
 await until(()=>document.querySelector(`[data-fence="${id}"] .image`).style.visibility==='hidden');
 f=(await state()).config.fences.find(f=>f.id===id);delete f.style.media;await invoke('update_fence',{fence:f});
 await until(()=>!document.querySelector(`[data-fence="${id}"] .db-media-background`)&&document.querySelector(`[data-fence="${id}"] .image`)?.style.visibility!=='hidden');await check('media_static_replacement_visible',true);
 await invoke('clear_fence_image',{id});await until(()=>!document.querySelector(`[data-fence="${id}"] .db-media-background`));await check('media_clear_removes_decoder',true);
 await invoke('frontend_checks_passed',{checks:['fence_merge_release','ICON_RADIUS_NATIVE_SUCCESS']});
 await invoke('quit_app');
}
window.addEventListener('error',e=>invoke('frontend_error',{message:String(e.message)}));
window.addEventListener('unhandledrejection',e=>invoke('frontend_error',{message:String(e.reason)}));
window.addEventListener('DOMContentLoaded',()=>{const timer=setInterval(()=>{if(document.querySelector('#app')?.children.length){clearInterval(timer);invoke('frontend_ready',{nodes:1,role:document.documentElement.dataset.role});if(window.__TAURI_INTERNALS__.metadata.currentWindow.label==='overlay-0')run().catch(e=>invoke('frontend_error',{message:String(e.stack||e)}))}},100)});
})();
