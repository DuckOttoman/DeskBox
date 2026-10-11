/* Persistent fence operations; previews never invoke a mutating backend command. */
(() => {
  class HoverSession {
    constructor(clock = () => performance.now()) { this.clock = clock; this.leave(); }
    enter(target) { if (this.target !== target) { this.target = target; this.started = this.clock(); } }
    mode() { return this.target ? (this.clock() - this.started >= 5000 ? 'contents' : 'group') : null; }
    leave() { this.target = null; this.started = 0; }
    cancel() { this.leave(); }
    release() { const result = this.target ? { target: this.target, mode: this.mode() } : null; this.leave(); return result; }
  }
  if (typeof module !== 'undefined') module.exports = { HoverSession };
  if (typeof window === 'undefined') return;
  let bridge, dragging = null, menu = null, menuRelease = null, suppressedClick = 0;
  const state = () => bridge.state;
  const config = () => state().config;
  const fence = id => config()?.fences.find(f => f.id === id);
  const group = id => config()?.fenceGroups?.find(g => g.members.includes(id));
  const node = id => [...document.querySelectorAll('[data-fence]')].find(n => n.dataset.fence === id);
  const duration = n => n / Math.max(.25, config()?.settings.animSpeed || 1);
  const wait = n => new Promise(r => setTimeout(r, n));
  const animate = (el, frames, options) => {
    const a = el.animate(frames, { duration: duration(240), easing: 'cubic-bezier(.2,.8,.2,1)', ...options });
    return a.finished.catch(() => {});
  };
  const invoke = (command, args) => bridge.invoke(command, args);
  function visible(f) { const g = group(f.id); return !g || g.active === f.id; }
  function geometry(f) { const g = group(f.id); return g ? { x: g.x, y: g.y } : f; }
  function closeMenu() { menu?.remove(); menu = null; menuRelease?.(); menuRelease = null; bridge?.region(); }
  function notice(message) {
    const el = document.createElement('div'); el.className = 'db-merge-notice'; el.textContent = message;
    el.setAttribute('role', 'status'); document.body.append(el);
    setTimeout(() => el.remove(), 2600);
  }
  function title(event, f) {
    if (performance.now() < suppressedClick || dragging) return;
    const g = group(f.id); if (!g || event.detail > 1) return;
    event.stopPropagation(); closeMenu();
    menu = document.createElement('div'); menu.className = 'db-fence-switch'; menu.dataset.hit = '';
    menu.setAttribute('role', 'menu'); menu.setAttribute('aria-label', '切换文件夹框');
    for (const id of g.members) {
      const member = fence(id); if (!member) continue;
      const button = document.createElement('button'); button.type = 'button'; button.setAttribute('role', 'menuitem');
      button.textContent = `${id === g.active ? '✓ ' : ''}${member.title} (${member.items.length})`;
      button.dataset.member = id;
      button.onclick = async () => { closeMenu(); try { await invoke('select_fence', { group: g.id, member: id }); } catch (e) { notice(String(e)); } };
      menu.append(button);
    }
    const bounds = event.currentTarget.getBoundingClientRect(); document.body.append(menu);
    menu.style.left = Math.max(8, Math.min(innerWidth - menu.offsetWidth - 8, bounds.left)) + 'px';
    menu.style.top = Math.max(8, Math.min(innerHeight - menu.offsetHeight - 8, bounds.bottom + 6)) + 'px';
    menuRelease = bridge.hold(); bridge.region();
  }
  function menuEntries(entries, f) {
    entries.splice(Math.max(0,entries.length-2),0,{
      label:f.style.hideIconNames?'显示图标名':'隐藏图标名',icon:'eye',
      action:async()=>{try{const current=fence(f.id);if(!current)return;const next=JSON.parse(JSON.stringify(current));next.style.hideIconNames=!current.style.hideIconNames;await invoke('update_fence',{fence:next});}catch(e){notice(String(e));}}
    });
    const records = (config()?.mergeRestores || []).filter(r => r.target === f.id).slice().reverse();
    if (records.length) entries.splice(entries.length - 2, 0, {
      label: '还原内容合并', icon: 'refresh', children: records.map((r, i) => ({
        label: `${r.source.title} · ${new Date(Number(r.createdAt)).toLocaleTimeString()}`,
        disabled: i > 0, action: () => restore(r.id, f.id)
      }))
    });
    return entries;
  }
  async function restore(record, target) {
    const before = [...document.querySelectorAll('[data-fence]')].map(el => ({ id: el.dataset.fence, rect: el.getBoundingClientRect() }));
    const targetRect = node(target)?.getBoundingClientRect(); const release = bridge.hold();
    try {
      const source = (config().mergeRestores || []).find(r => r.id === record)?.source.id;
      await invoke('restore_fence_merge', { record }); await wait(45);
      const tasks = [];
      for (const { id, rect } of before) {
        const el = node(id); if (!el) continue; const after = el.getBoundingClientRect();
        if (id === target) { el.classList.add('db-cell-split'); tasks.push(animate(el, [{borderRadius:'45%',transform:'scale(.86)'},{borderRadius:'var(--fb-radius,14px)',transform:'none'}],{duration:duration(650)}).then(()=>el.classList.remove('db-cell-split'))); }
        else if (rect.left !== after.left || rect.top !== after.top) tasks.push(animate(el,[{transform:`translate(${rect.left-after.left}px,${rect.top-after.top}px)`},{transform:'none'}],{duration:duration(650)}));
      }
      const child = node(source);
      if (child && targetRect) { const r = child.getBoundingClientRect(); child.classList.add('db-cell-split'); tasks.push(animate(child,[{transform:`translate(${targetRect.left-r.left}px,${targetRect.top-r.top}px) scale(.75)`,borderRadius:'45%',opacity:.5},{transform:'none',borderRadius:'var(--fb-radius,14px)',opacity:1}],{duration:duration(650)}).then(()=>child.classList.remove('db-cell-split'))); }
      await Promise.all(tasks); notice('已还原合并前的文件夹框');
    } catch(e) { notice(String(e)); } finally { release(); bridge.region(); }
  }
  function groupPointer(event,initial,snap){
    const id=initial.id,g=group(id),gid=g.id,original={x:g.x,y:g.y},start={x:event.clientX,y:event.clientY},element=event.currentTarget;
    let moved=false,done=false,hold=null;dragging={id,hover:new HoverSession()};
    const current=()=>config()?.fenceGroups?.find(g=>g.id===gid);
    const cleanup=()=>{clearTimeout(holdTimer);window.removeEventListener('pointermove',move);window.removeEventListener('pointerup',up);window.removeEventListener('pointercancel',cancel);window.removeEventListener('blur',cancel);window.removeEventListener('keydown',key);try{element.releasePointerCapture(event.pointerId);}catch{}state().busy.delete(id);dragging=null;hold?.();bridge.region();};
    const end=async(save)=>{if(done)return;done=true;clearTimeout(holdTimer);const active=current();try{if(moved&&active){if(save)await invoke('move_fence_group',{group:gid,point:[active.x,active.y]});else Object.assign(active,original);}}catch(e){if(current())Object.assign(current(),original);notice(String(e));}finally{if(moved)suppressedClick=performance.now()+350;cleanup();}};
    const move=e=>{if(e.pointerId!==event.pointerId||done)return;const dx=e.clientX-start.x,dy=e.clientY-start.y;if(!moved&&Math.hypot(dx,dy)<4)return;clearTimeout(holdTimer);if(!moved){moved=true;try{element.setPointerCapture(event.pointerId);}catch{}state().busy.add(id);hold=bridge.hold();}const active=current(),f=fence(id);if(!active||!f){end(false);return;}const p=snap(id,original.x+dx,original.y+dy,f.w,f.collapsed?38:f.h);active.x=p.x;active.y=p.y;bridge.region();};
    const up=e=>{if(e.pointerId===event.pointerId&&e.button===0)end(true);};
    const cancel=()=>end(false);const key=e=>{if(e.key==='Escape'){e.preventDefault();cancel();}};
    const holdTimer=setTimeout(async()=>{if(done||moved)return;const active=current(),f=fence(id);if(!active||active.active!==id||!f){end(false);return;}done=true;suppressedClick=performance.now()+500;
      const point=[Math.max(0,Math.min(innerWidth-f.w,active.x+32)),Math.max(0,Math.min(innerHeight-(f.collapsed?38:f.h),active.y+32))];
      try{await invoke('detach_fence',{member:id,point});notice('已分离当前文件夹框');}catch(e){notice(String(e));}finally{cleanup();}
    },2000);
    window.addEventListener('pointermove',move);window.addEventListener('pointerup',up);window.addEventListener('pointercancel',cancel);window.addEventListener('blur',cancel);window.addEventListener('keydown',key);
  }
  function pointer(event, initial, snap) {
    if (event.button !== 0 || dragging || event.target.closest('button,input')) return;
    closeMenu();
    if(group(initial.id)){groupPointer(event,initial,snap);return;}
    const element = event.currentTarget, id = initial.id, original = {x:initial.x,y:initial.y};
    const origin = { ...geometry(initial) }, start = {x:event.clientX,y:event.clientY};
    const offset = {x:start.x-origin.x,y:start.y-origin.y};
    const hover = new HoverSession(); let moved = false, releaseHold = null, preview = null, timer = null, done = false, focusPending = false;
    let last = {x:event.clientX,y:event.clientY}; let previousTarget = null;
    dragging = { id, hover };
    const get = () => fence(id);
    const findTarget = p => {
      const sourceGroup = group(id);
      const candidates = [...document.querySelectorAll('[data-fence]')];
      return candidates.map((el,index)=>({el,index,z:Number(getComputedStyle(el).zIndex)||0})).sort((a,b)=>b.z-a.z||b.index-a.index).find(({el}) => {
        const other = el.dataset.fence;
        if (other === id || sourceGroup?.members.includes(other)) return false;
        const r = el.getBoundingClientRect();
        return p.x >= r.left && p.x <= r.right && p.y >= r.top && p.y <= r.bottom;
      })?.el.dataset.fence;
    };
    const updatePreview = () => {
      if (!moved || done) return;
      const other = findTarget(last);
      if (other) hover.enter(other); else hover.leave();
      const mode = hover.mode();
      if (previousTarget !== other || preview?.mode !== mode) {
        preview?.cancel(); preview = other ? createPreview(id, other, mode) : null;
        previousTarget = other;
      }
      preview?.position();
    };
    const move = e => {
      if (e.pointerId !== event.pointerId || done) return;
      if (!moved && Math.hypot(e.clientX-start.x,e.clientY-start.y) < 4) return;
      if (!moved) {
        moved = true; element.setPointerCapture(event.pointerId); state().busy.add(id); releaseHold = bridge.hold();
        // SetFocus(host) -> WebView2.MoveFocus temporarily blurs this WebView.
        // That handoff is not a user cancellation; genuine later blur still is.
        focusPending = true;
        Promise.resolve(bridge.focus?.(true)).catch(()=>{}).finally(()=>{focusPending=false;});
        node(id)?.classList.add('db-fence-dragging'); timer = setInterval(updatePreview,40);
      }
      const f = get(); if (!f) { finish(null); return; }
      const p = snap(id,e.clientX-offset.x,e.clientY-offset.y,f.w,f.collapsed?38:f.h);
      f.x=p.x; f.y=p.y; last={x:e.clientX,y:e.clientY}; updatePreview();
    };
    const clearListeners = () => {
      window.removeEventListener('pointermove',move); window.removeEventListener('pointerup',up); window.removeEventListener('pointercancel',cancel);
      window.removeEventListener('keydown',key); window.removeEventListener('blur',blur);
      clearInterval(timer); try { element.releasePointerCapture(event.pointerId); } catch {}
    };
    async function finish(result) {
      if (done) return; done=true; clearListeners();
      if (!moved) { dragging=null; return; }
      suppressedClick=performance.now()+350;
      const f=get(); const point=f ? [f.x,f.y] : [origin.x,origin.y];
      try {
        if (result && f) {
          const target = result.target;
          // The only preview-to-persistence boundary: finish receives a result exclusively from pointerup.
          await invoke(result.mode==='group'?'group_fences':'merge_fence_contents',{source:id,target});
          await preview?.complete(); notice(result.mode==='group'?'已合并文件夹外框':'已合并文件夹内容');
        } else if (result === false && f) {
          await invoke('update_fence',{fence:JSON.parse(JSON.stringify(f))});
          await preview?.cancel();
        } else { if(f)Object.assign(f,original); await preview?.cancel(); }
      } catch(e) { if(f)Object.assign(f,original); await preview?.cancel(); notice(String(e)); }
      finally { state().busy.delete(id); node(id)?.classList.remove('db-fence-dragging'); dragging=null; await bridge.focus?.(false).catch(()=>{}); releaseHold?.(); bridge.region(); }
    }
    const up = e => { if(e.pointerId!==event.pointerId || e.button!==0)return; last={x:e.clientX,y:e.clientY}; updatePreview(); finish(hover.release() || false); };
    const cancel = () => { hover.cancel(); finish(null); };
    const blur = () => { if(!focusPending)cancel(); };
    const key = e => { if(e.key==='Escape'){e.preventDefault();cancel();} };
    window.addEventListener('pointermove',move);window.addEventListener('pointerup',up);window.addEventListener('pointercancel',cancel);
    window.addEventListener('keydown',key);window.addEventListener('blur',blur);
  }
  function createPreview(source, target, mode) {
    const src=node(source), dst=node(target); if(!src||!dst)return null;
    const bubble=document.createElement('div'); bubble.className='db-merge-bubble'; bubble.textContent=mode==='group'?'合并文件夹外框':'合并文件夹内容';
    bubble.dataset.mergeMode=mode; bubble.setAttribute('role','status');document.body.append(bubble);
    const animations=[],clones=[],dimmed=[],hidden=[],empties=[]; let ended=false,batchTimer=null;
    const originals={transform:dst.style.transform,transformOrigin:dst.style.transformOrigin,zIndex:dst.style.zIndex};
    const originalBounds=dst.getBoundingClientRect();
    function position(){const r=dst.getBoundingClientRect();bubble.style.left=Math.max(8,Math.min(innerWidth-bubble.offsetWidth-8,r.left+r.width/2-bubble.offsetWidth/2))+'px';bubble.style.top=Math.max(8,Math.min(innerHeight-bubble.offsetHeight-8,r.top-46))+'px';}
    if(mode==='group'){src.classList.add('db-merge-header');dst.classList.add('db-merge-header');}
    else {
      dst.classList.add('db-merge-content'); dst.style.zIndex='10000';
      const left=Math.max(0,Math.min(innerWidth-originalBounds.width*1.5,originalBounds.left-originalBounds.width*.25));
      const top=Math.max(0,Math.min(innerHeight-originalBounds.height*1.5,originalBounds.top-originalBounds.height*.25));
      dst.style.transformOrigin='0 0';dst.style.transform=`translate(${left-originalBounds.left}px,${top-originalBounds.top}px) scale(1.5)`;
      const expanded={left,top,right:left+originalBounds.width*1.5,bottom:top+originalBounds.height*1.5};
      for(const el of document.querySelectorAll('[data-fence]')){if(el===src||el===dst)continue;const r=el.getBoundingClientRect();if(r.right>expanded.left&&r.left<expanded.right&&r.bottom>expanded.top&&r.top<expanded.bottom){dimmed.push([el,el.style.opacity]);el.style.opacity='.1';}}
      const grid=dst.querySelector?.('.grid');
      const targetItems=new Set(fence(target)?.items||[]);
      const icons=[...src.querySelectorAll('.item')].filter(el=>!targetItems.has(el.dataset.item));
      if(grid){
        // Preview slots use the recovered grid and target CSS inheritance, so
        // wrapping, icon size and labels exactly match the destination items.
        for(const el of grid.querySelectorAll('.empty')){empties.push([el,el.style.display]);el.style.display='none';}
        for(const el of icons){
          const clone=el.cloneNode(true);clone.removeAttribute('data-item');clone.removeAttribute('data-hit');
          clone.className='item db-merge-slot';clone.setAttribute('aria-hidden','true');clone.inert=true;
          Object.assign(clone.style,{visibility:'hidden',transform:'none',opacity:'1'});
          grid.append(clone);clones.push(clone);
          hidden.push([el,el.style.visibility]);el.style.visibility='hidden';
        }
        function batch(offset){
          if(ended)return;
          for(const a of animations)a.cancel();animations.length=0;
          const scale=dst.getBoundingClientRect().width/dst.offsetWidth;
          clones.slice(offset,offset+40).forEach((clone,i)=>{
            const original=icons[offset+i],from=original.getBoundingClientRect(),to=clone.getBoundingClientRect();
            const dx=(from.left-to.left)/scale,dy=(from.top-to.top)/scale;
            const iconFrom=original.querySelector('.ico')?.getBoundingClientRect();
            const iconTo=clone.querySelector('.ico')?.getBoundingClientRect();
            const initialScale=iconFrom&&iconTo?.width?iconFrom.width/iconTo.width:1;
            clone.style.visibility='visible';clone.style.transformOrigin='0 0';
            const a=clone.animate([
              {transform:`translate(${dx}px,${dy}px) scale(${initialScale})`,opacity:1},
              {transform:`translate(${dx}px,${dy}px) scale(.08)`,opacity:.7,offset:.14},
              {transform:'translate(0,0) scale(.08)',opacity:.85,offset:.8},
              {transform:'none',opacity:1}
            ],{duration:duration(360),delay:duration(i*12),fill:'backwards',easing:'cubic-bezier(.3,0,.2,1)'});
            animations.push(a);
          });
          if(offset+40<clones.length)batchTimer=setTimeout(()=>batch(offset+40),duration(900));
        }
        // Measure after the 150% expansion settles, rather than using a
        // hard-coded viewport grid or mid-transition coordinates.
        batchTimer=setTimeout(()=>batch(0),duration(260));
      }
    }
    position();bridge.region();
    async function cleanup(reverse){
      if(ended)return;ended=true;clearTimeout(batchTimer);
      // Cancellation restores real source items in place immediately. No
      // reverse copies can flash outside the frame or duplicate real icons.
      for(const a of animations)a.cancel();
      for(const el of clones)el.remove();
      for(const [el,display] of empties)el.style.display=display;
      for(const [el,opacity] of dimmed)el.style.opacity=opacity;
      for(const [el,visibility] of hidden)el.style.visibility=visibility;
      src.classList.remove('db-merge-header');dst.classList.remove('db-merge-header','db-merge-content');Object.assign(dst.style,originals);
      if(reverse)await animate(bubble,[{transform:'scale(1)',opacity:1},{transform:'scale(.15)',opacity:0}],{duration:duration(140)});
      for(const a of animations)a.cancel();for(const el of clones)el.remove();
      bubble.remove();bridge.region();
    }
    return {mode,position,cancel:()=>cleanup(true),complete:()=>cleanup(false)};
  }
  window.addEventListener('pointerdown',e=>{if(menu&&!menu.contains(e.target)&&!e.target.closest('.title'))closeMenu();},true);
  window.addEventListener('keydown',e=>{if(e.key==='Escape')closeMenu();});
  window.DeskBoxFenceMerge={HoverSession,bind:b=>{bridge=b;},visible,geometry,pointer,title,menu:menuEntries,restore,
    inspect:()=>({fences:config()?.fences.map(f=>({id:f.id,monitor:f.monitor,visible:visible(f)})),monitors:state()?.monitors,fencesVisible:config()?.fencesVisible}),
    get preview(){return dragging?.hover.mode()||null;},get dragging(){return !!dragging;}};
})();
