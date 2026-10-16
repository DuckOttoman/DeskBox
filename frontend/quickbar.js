(function () {
  'use strict';
  function layout(entries, width, count) {
    const max = Math.max(12, Math.floor(width * .68 / 72));
    const requested = String(count).toUpperCase() === 'MAX' ? max : Math.max(10, Math.floor(Number(count) || 12));
    const slots = Math.max(1, Math.min(requested, max, Math.floor(width * .68 / 72)));
    const more = entries.length > slots;
    return { slots, max, visible: entries.slice(0, more ? Math.max(0, slots-1) : slots), overflow: more ? entries.slice(Math.max(0,slots-1)) : [] };
  }
  function createFreeze() {
    const reasons = new Set(); let pending = null;
    return {set(key,on){on ? reasons.add(key) : reasons.delete(key);}, accept(state,force=false){if(!force&&reasons.size){pending=state;return null;}pending=null;return state;},take(){if(reasons.size)return null;const state=pending;pending=null;return state;}};
  }
  function themeName(theme){return ['glass-dark','glass-light','minimal','neon','sakura','forest'].includes(theme)?theme:'glass-dark';}
  function emptyDockWidth(viewport){return Math.min(336,Math.max(0,viewport)*.68);}
  function renderKey(state,viewport){return JSON.stringify([state.config,state.monitor,state.entries,viewport]);}
  function previewGeometry(payload,width,height){
    if(!payload||!Number.isFinite(payload.x)||!Number.isFinite(payload.y))return null;
    const x=payload.x-24,y=payload.y-24,left=Math.max(0,x),top=Math.max(0,y),right=Math.min(width,x+48),bottom=Math.min(height,y+48);
    if(right<=left||bottom<=top)return null;
    return {x,y,width:48,height:48,clip:{x:left,y:top,width:right-left,height:bottom-top}};
  }
  function insertionGeometry(payload,rect,baseWidth,viewport){const active=!!payload&&Number.isFinite(payload.x)&&Number.isFinite(payload.y)&&payload.x>=rect.left-24&&payload.x<=rect.right+24&&payload.y>=rect.top-36&&payload.y<=rect.bottom+12;const width=Math.min(baseWidth+(active?72:0),viewport*.68);return {active,width,inside:active&&width-baseWidth<71};}
  function createRefreshRunner(run){
    let busy=false,queued=false,active;
    return function request(force=false){
      if(busy){queued=queued||force;return active;}busy=true;
      active=(async()=>{let forced=force,failure;try{do{queued=false;try{await run(forced);failure=null;}catch(err){failure=err;}forced=queued;}while(queued);if(failure)throw failure;}finally{busy=false;}})();return active;
    };
  }
  class ReorderSession {
    constructor(paths,path,x,y){this.paths=paths.slice();this.path=path;this.x=x;this.y=y;this.active=false;this.cancelled=false;this.target=null;}
    move(x,y,target){if(this.cancelled)return;if(Math.hypot(x-this.x,y-this.y)>=4)this.active=true;this.target=this.paths.includes(target)?target:null;}
    cancel(){this.cancelled=true;this.target=null;}
    finish(button){if(button!==0||this.cancelled||!this.active||!this.target||this.target===this.path)return null;const paths=this.paths.slice(),from=paths.indexOf(this.path),to=paths.indexOf(this.target);if(from<0)return null;paths.splice(to,0,paths.splice(from,1)[0]);return paths;}
  }
  if (typeof module === 'object' && module.exports) { module.exports={layout,createFreeze,renderKey,themeName,ReorderSession,emptyDockWidth,previewGeometry,createRefreshRunner,insertionGeometry}; return; }
  const $=id=>document.getElementById(id), dock=$('dock'), overflow=$('overflow'), menu=$('menu'), notice=$('notice'), tooltip=$('tooltip');
  const bridge=window.__TAURI_INTERNALS__, freeze=createFreeze();
  let state, pointerDrag=null, suppressClickUntil=0, timer, regionTimer, nativeDrag=false, noticeTimer, lastKey='', ready=false, lastError=null, preview=null, previewPayload=null, previewReported=false, previewGeneration=0;
  const previewNode=document.createElement('div');previewNode.id='drag-preview';previewNode.hidden=true;previewNode.inert=true;previewNode.setAttribute('aria-hidden','true');document.body.append(previewNode);
  const insertionSlot=document.createElement('div');insertionSlot.id='insertion-slot';insertionSlot.hidden=true;insertionSlot.inert=true;insertionSlot.setAttribute('aria-hidden','true');insertionSlot.textContent='+';
  let baseDockWidth=0, dockTargetWidth=0, widthAnimation=null, widthRegion=null, insertionState=null;
  const invoke=(cmd,args)=>bridge.invoke(cmd,args);
  function error(err){lastError=String(err?.message||err);console.error('DeskBox quickbar:',err);showNotice('快捷栏错误：'+lastError);}
  const reduced=()=>window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  function animate(node,frames,ms){if(!node.animate||reduced()||document.hidden)return;const speed=Number(state?.config?.settings?.animSpeed)||1;return node.animate(frames,{duration:ms/Math.max(.25,speed),easing:'cubic-bezier(.2,.7,.2,1)'});}
  function collectRegions(){
    const rects=[...document.querySelectorAll('[data-hit]')].filter(e=>!e.hidden).map(e=>{const r=e.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height};});
    if(preview&&!previewNode.hidden)rects.push({...preview.clip,radius:0});if(widthRegion)rects.push({...widthRegion,radius:22});return rects;
  }
  function regions(){
    clearTimeout(regionTimer); regionTimer=setTimeout(()=>{
      invoke('quickbar_regions',{regions:collectRegions()}).catch(error);
    },0);
  }
  async function syncRegions(){
    await invoke('quickbar_regions',{regions:collectRegions()});
  }
  function resizeDock(width,ms=220,from){
    if(width===dockTargetWidth&&from===undefined)return;
    const rect=dock.getBoundingClientRect(),oldWidth=from??rect.width;
    if(widthAnimation){widthAnimation.cancel();widthAnimation=null;}
    dockTargetWidth=width;dock.style.width=width+'px';
    const union=Math.max(oldWidth,width);widthRegion={x:(innerWidth-union)/2,y:rect.top,width:union,height:rect.height};
    const current=animate(dock,[{width:oldWidth+'px'},{width:width+'px'}],ms);widthAnimation=current;regions();
    if(current)current.finished.catch(()=>{}).finally(()=>{if(widthAnimation===current){widthAnimation=null;widthRegion=null;regions();}});else{widthRegion=null;regions();}
  }
  function applyInsertion(payload,from){
    if(!baseDockWidth)return;
    insertionState=insertionGeometry(payload,dock.getBoundingClientRect(),baseDockWidth,innerWidth);
    insertionSlot.hidden=!insertionState.active;dock.classList.toggle('insertion-open',insertionState.active);dock.classList.toggle('insertion-inside',insertionState.inside);
    if(!dock.contains(insertionSlot))dock.append(insertionSlot);
    resizeDock(insertionState.width,insertionState.active?200:160,from);
  }
  function showPreview(payload){
    if(!payload){previewGeneration++;previewPayload=null;preview=null;previewReported=false;previewNode.hidden=true;previewNode.replaceChildren();applyInsertion(null);lock('sourceDrag',false);regions();return;}
    const first=!previewPayload;previewPayload=payload;preview=previewGeometry(payload,innerWidth,innerHeight);lock('sourceDrag',true);
    if(first){previewGeneration++;previewReported=false;}
    applyInsertion(payload);
    if(!preview){previewNode.hidden=true;regions();return;}
    const generation=previewGeneration;
    previewNode.style.left=preview.x+'px';previewNode.style.top=preview.y+'px';previewNode.hidden=false;
    if(previewNode.dataset.icon!==(payload.icon||'')||!previewNode.firstChild){previewNode.dataset.icon=payload.icon||'';previewNode.replaceChildren();if(payload.icon){const img=document.createElement('img');img.alt='';img.draggable=false;img.src=asset(payload.icon);img.onerror=()=>{if(previewNode.contains(img)){img.remove();previewNode.textContent='▣';}};previewNode.append(img);}else previewNode.textContent='▣';}
    regions();
    if(state?.diagnostic&&!previewReported){previewReported=true;setTimeout(async()=>{if(generation!==previewGeneration||previewNode.hidden)return;try{await syncRegions();await invoke('quickbar_dom_report',{ok:true,message:'QUICKBAR_DRAG_PREVIEW_RENDERED'});}catch(err){error(err);}},0);}
  }
  function lock(key,on){freeze.set(key,on);if(!on){const next=freeze.take();if(next)render(next);}}
  function showNotice(text,undo){
    clearTimeout(noticeTimer);notice.replaceChildren(document.createTextNode(text));notice.hidden=false;notice.dataset.hit='';
    if(undo){const b=document.createElement('button');b.textContent='撤销';b.onclick=async()=>{try{await invoke('quickbar_undo',{path:undo});notice.hidden=true;await refresh(true);regions();}catch(e){error(e);}};notice.append(b);}
    regions();noticeTimer=setTimeout(()=>{notice.hidden=true;delete notice.dataset.hit;regions();},5000);
  }
  function closeMenu(){menu.hidden=true;delete menu.dataset.hit;lock('menu',false);regions();}
  function asset(path){return bridge.convertFileSrc ? bridge.convertFileSrc(path) : 'http://asset.localhost/'+encodeURIComponent(path);}
  function entryButton(entry){
    const b=document.createElement('button');b.className='item'+(entry.missing?' missing':'');b.type='button';b.setAttribute('aria-label',entry.name+(entry.missing?'（路径不可用）':''));b.dataset.path=entry.path;
    b.setAttribute('aria-disabled',String(!!entry.missing));b.draggable=false;b.style.touchAction='none';
    const face=document.createElement('span');face.className='face';
    if(entry.icon){const img=document.createElement('img');img.src=asset(entry.icon);img.alt='';img.draggable=false;img.onerror=()=>{img.remove();face.textContent='▣';};face.append(img);}else face.textContent='▣';
    b.append(face);if(entry.pinned){const dot=document.createElement('i');dot.className='pin';b.append(dot);}
    b.onclick=async()=>{if(entry.missing||performance.now()<suppressClickUntil)return;try{await invoke('quickbar_open',{path:entry.path});b.classList.add('launched');setTimeout(()=>b.classList.remove('launched'),360);}catch(e){error(e);}};
    b.onpointerenter=()=>{tooltip.textContent=entry.name+(entry.missing?' · 路径不可用':'');tooltip.hidden=false;const r=b.getBoundingClientRect();tooltip.style.left=Math.max(8,Math.min(innerWidth-220,r.x+r.width/2-100))+'px';};
    b.onpointerleave=()=>{tooltip.hidden=true;};
    b.oncontextmenu=e=>{e.preventDefault();lock('menu',true);menu.replaceChildren();const remove=document.createElement('button');remove.textContent='从快捷栏移除';remove.setAttribute('role','menuitem');remove.onclick=async()=>{try{await invoke('quickbar_remove',{path:entry.path});closeMenu();showNotice('已从快捷栏移除，源文件保留',entry.path);await refresh(true);}catch(err){error(err);}};menu.append(remove);menu.hidden=false;menu.dataset.hit='';menu.style.left=Math.max(8,Math.min(innerWidth-190,e.clientX-90))+'px';regions();};
    b.onpointerdown=e=>{if(e.button!==0||!entry.pinned||pointerDrag)return;pointerDrag={session:new ReorderSession(state.entries.filter(x=>x.pinned).map(x=>x.path),entry.path,e.clientX,e.clientY),id:e.pointerId,node:b};b.setPointerCapture(e.pointerId);lock('drag',true);};
    b.onpointermove=e=>{if(!pointerDrag||pointerDrag.id!==e.pointerId)return;const target=document.elementFromPoint(e.clientX,e.clientY)?.closest('[data-path]');const session=pointerDrag.session;session.move(e.clientX,e.clientY,target?.dataset.path);if(session.active){e.preventDefault();tooltip.hidden=true;b.style.opacity='.55';for(const node of document.querySelectorAll('[data-path]'))node.style.outline=node.dataset.path===session.target?'2px solid var(--accent)':'';}};
    b.onpointerup=e=>{if(pointerDrag?.id===e.pointerId)finishPointer(e.button,false);};
    b.onpointercancel=e=>{if(pointerDrag?.id===e.pointerId)finishPointer(e.button,true);};
    b.onlostpointercapture=()=>{if(pointerDrag?.node===b)finishPointer(-1,true);};
    b.ondragstart=e=>e.preventDefault();return b;
  }
  async function finishPointer(button,cancel){
    const drag=pointerDrag;if(!drag)return;pointerDrag=null;if(cancel)drag.session.cancel();
    if(drag.session.active)suppressClickUntil=performance.now()+450;
    const paths=drag.session.finish(button);drag.node.style.opacity='';for(const node of document.querySelectorAll('[data-path]'))node.style.outline='';
    if(drag.node.hasPointerCapture(drag.id))drag.node.releasePointerCapture(drag.id);
    try{if(paths)await invoke('quickbar_reorder',{paths});}catch(err){error(err);}finally{lock('drag',false);if(paths)await refresh(true);}
  }
  function render(next){
    state=next;const key=renderKey(next,innerWidth);if(key===lastKey)return;lastKey=key;
    const before=new Map([...dock.querySelectorAll('[data-path]')].map(e=>[e.dataset.path,{rect:e.getBoundingClientRect(),node:e}]));
    const oldWidth=dock.getBoundingClientRect().width;
    const settings=state.config.settings||state.config, width=(state.monitor.w||innerWidth)/(state.monitor.scale||1);
    const result=layout(state.entries||[],width,settings.quickbarCount ?? settings.quickbar_count ?? state.config.quickbarCount);
    document.documentElement.dataset.theme=themeName(state.config.theme);
    document.documentElement.style.setProperty('--accent',window.DeskBoxPlugins?.themeAccent()||settings.accent||'#8aabff');
    const alpha=Number(settings.opacity);document.documentElement.style.setProperty('--alpha',Number.isFinite(alpha)?Math.min(1,Math.max(.35,alpha>1?alpha/100:alpha)):.88);
    document.documentElement.style.setProperty('--frost',(settings.frost===false?0:18)+'px');
    const speed=Number(settings.animSpeed);document.documentElement.style.setProperty('--speed',Number.isFinite(speed)?1/Math.max(.25,speed):1);
    dock.replaceChildren(...result.visible.map(entryButton));
    const emptyState=!state.entries?.length;dock.classList.toggle('is-empty',emptyState);dock.style.minWidth=emptyState?emptyDockWidth(innerWidth)+'px':'';
    if(emptyState){const empty=document.createElement('div');empty.className='empty';empty.innerHTML='<span class="empty-icon" aria-hidden="true"><svg viewBox="0 0 24 24" fill="none"><path d="M12 5v14M5 12h14" stroke="currentColor" stroke-width="1.65" stroke-linecap="round"/></svg></span><span class="empty-copy"><span class="empty-title">从文件夹框拖入图标</span><span class="empty-hint">固定常用项目，原位置保持不变</span></span>';dock.append(empty);}
    if(result.overflow.length){const more=document.createElement('button');more.className='item more';more.textContent='•••';more.setAttribute('aria-label','更多快捷项目');more.setAttribute('aria-expanded',String(!overflow.hidden));more.onclick=()=>{overflow.hidden=!overflow.hidden;if(overflow.hidden)delete overflow.dataset.hit;else overflow.dataset.hit='';lock('overflow',!overflow.hidden);more.setAttribute('aria-expanded',String(!overflow.hidden));regions();};dock.append(more);}
    overflow.replaceChildren(...result.overflow.map(entryButton));
    if(widthAnimation){widthAnimation.cancel();widthAnimation=null;}widthRegion=null;
    dock.classList.remove('insertion-open','insertion-inside');dock.style.width='auto';const newWidth=dock.getBoundingClientRect().width;dock.style.width=newWidth+'px';baseDockWidth=newWidth;dockTargetWidth=newWidth;

    const after=new Set();
    for(const e of dock.querySelectorAll('[data-path]')){
      const path=e.dataset.path;after.add(path);const old=before.get(path), rect=e.getBoundingClientRect();
      if(old){const dx=old.rect.left-rect.left;if(Math.abs(dx)>.5)animate(e,[{translate:dx+'px 0'},{translate:'0 0'}],180);}
      else if(before.size){animate(e,[{opacity:0,scale:'.65',outline:'2px solid var(--accent)'},{opacity:1,scale:'1',outline:'2px solid transparent'}],240);}
    }
    for(const [path,old] of before){if(after.has(path))continue;const ghost=old.node.cloneNode(true);ghost.classList.add('departing');ghost.style.left=old.rect.left+'px';ghost.style.top=old.rect.top+'px';ghost.removeAttribute('data-path');document.body.append(ghost);const anim=animate(ghost,[{opacity:1,scale:'1'},{opacity:0,scale:'.65'}],140);if(anim)anim.finished.catch(()=>{}).finally(()=>ghost.remove());else ghost.remove();}
    if(previewPayload)applyInsertion(previewPayload,oldWidth);else if(oldWidth&&oldWidth!==newWidth)resizeDock(newWidth,220,oldWidth);
    regions();
  }
  const refresh=createRefreshRunner(async force=>{const received=await invoke('quickbar_state');const next=freeze.accept(received,force);if(next)render(next);});
  for(const surface of [dock,overflow,menu,notice]){surface.addEventListener('pointerenter',()=>lock(surface.id,true));surface.addEventListener('pointerleave',()=>lock(surface.id,false));}
  document.addEventListener('pointerdown',e=>{if(!menu.contains(e.target))closeMenu();if(!overflow.contains(e.target)&&!e.target.closest('.more')){overflow.hidden=true;delete overflow.dataset.hit;lock('overflow',false);regions();}});
  document.addEventListener('keydown',e=>{if(e.key==='Escape'){finishPointer(-1,true);closeMenu();overflow.hidden=true;delete overflow.dataset.hit;lock('overflow',false);regions();}});
  async function nativeEvent(event){
    const p=event.payload||event;const type=p.type||event.event||'';
    if(type==='enter'||type==='over'||type.includes('drag-enter')||type.includes('drag-over')){nativeDrag=true;lock('nativeDrag',true);dock.classList.add('dropping');}
    if(type==='drop'||type.includes('drag-drop')){try{if(p.paths?.length)await invoke('quickbar_pin',{paths:p.paths});}catch(e){error(e);}finally{nativeDrag=false;dock.classList.remove('dropping');lock('nativeDrag',false);await refresh(true);}}
    if(type==='leave'||type.includes('drag-leave')){nativeDrag=false;dock.classList.remove('dropping');lock('nativeDrag',false);}
  }
  async function start(){
    if(!bridge)throw new Error('原生桥接未就绪');await refresh(true);await syncRegions();
    const label=bridge.metadata?.currentWindow?.label;
    if(!label)throw new Error('无法识别快捷栏窗口');
    for(const event of ['tauri://drag-enter','tauri://drag-over','tauri://drag-drop','tauri://drag-leave'])await invoke('plugin:event|listen',{event,target:{kind:'Window',label},handler:bridge.transformCallback(nativeEvent)});
    await invoke('plugin:event|listen',{event:'quickbar-drag-preview',target:{kind:'Window',label},handler:bridge.transformCallback(event=>showPreview(event.payload))});
    await invoke('plugin:event|listen',{event:'quickbar-pins-changed',target:{kind:'Window',label},handler:bridge.transformCallback(()=>refresh(true).catch(error))});
    await invoke('plugin:event|listen',{event:'quickbar-hide',target:{kind:'Window',label},handler:bridge.transformCallback(()=>{showPreview(null);finishPointer(-1,true);tooltip.hidden=true;dock.classList.remove('ready');animate(dock,[{opacity:1,translate:'0 0',scale:'1'},{opacity:0,translate:'0 12px',scale:'.97'}],240);dock.style.opacity='0';})});
    window.DeskBoxPlugins.bindTheme({invoke,get state(){return state;},listen:(event,handler)=>invoke('plugin:event|listen',{event,target:{kind:'Window',label},handler:bridge.transformCallback(handler)})});await invoke('quickbar_ready');ready=true;dock.classList.add('ready');timer=setInterval(()=>{if(!document.hidden)refresh().catch(error);},1500);
  }
  window.addEventListener('resize',()=>{if(state)render(state);if(previewPayload)showPreview(previewPayload);});
  window.addEventListener('blur',()=>{finishPointer(-1,true);if(!nativeDrag){closeMenu();tooltip.hidden=true;}});
  window.__DeskBoxQuickbarTest={inspect:()=>({ready,lastError,state,items:dock.querySelectorAll('[data-path]').length,overflow:overflow.querySelectorAll('[data-path]').length,theme:document.documentElement.dataset.theme,insertion:{...insertionState,baseWidth:baseDockWidth,targetWidth:dockTargetWidth,slotVisible:!insertionSlot.hidden,widthRegion},preview:{visible:!previewNode.hidden,geometry:preview,icon:previewPayload?.icon||null,name:previewPayload?.name||null,reported:previewReported,hitMarker:previewNode.hasAttribute('data-hit')}}),refresh:()=>refresh(true)};
  window.addEventListener('unload',()=>clearInterval(timer));start().catch(error);
})();
