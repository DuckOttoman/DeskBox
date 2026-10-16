/* Shared protected geometry and reference-only internal pinning. */
(() => {
 function clamp(point,size,work){const [l,t,w,h]=work;return {x:Math.max(l,Math.min(l+w-size[0],point[0])),y:Math.max(t,Math.min(t+h-size[1],point[1]))};}
 if(typeof module!=='undefined')module.exports={clamp};if(typeof window==='undefined')return;
 let bridge,hint,busy=false,boundaryAt=0,previewPending=null,previewSending=false;
 async function sendPreview(payload){previewPending=payload;if(previewSending)return;previewSending=true;try{while(previewPending){const next=previewPending;previewPending=null;try{await bridge.invoke('quickbar_drag_preview',next);}catch(e){if(next.id&&!previewPending)previewPending={id:null,source:null,point:null};}}}finally{previewSending=false;}}

 const current=()=>bridge?.state.monitors?.find(m=>m.label===bridge.label);
 function bounds(){const m=current(),w=innerWidth,h=innerHeight;return bridge?.state.config?.settings.hideTaskbar&&m?{left:m.work[0],top:m.work[1],right:m.work[0]+m.work[2],bottom:m.work[1]+m.work[3]}:{left:0,top:0,right:w,bottom:h};}
 function constrain(x,y,w,h){const b=bounds(),p=clamp([x,y],[w,h],[b.left,b.top,b.right-b.left,b.bottom-b.top]);if(bridge?.state.config?.settings.hideTaskbar&&(p.x!==x||p.y!==y)&&performance.now()-boundaryAt>350){boundaryAt=performance.now();document.documentElement.classList.remove('db-quickbar-boundary');void document.documentElement.offsetWidth;document.documentElement.classList.add('db-quickbar-boundary');setTimeout(()=>document.documentElement.classList.remove('db-quickbar-boundary'),310);}return p;}
 function drag(x,y){const m=current(),enabled=bridge?.state.config?.settings.hideTaskbar;const active=!!(enabled&&m&&m.work[1]+m.work[3]<m.h/m.scale-70&&y>=m.h/m.scale-96&&x>=innerWidth*.16&&x<=innerWidth*.84);
  if(active&&!hint){hint=document.createElement('div');hint.className='db-quickbar-pin-hint';hint.textContent='松开以固定到快捷栏';document.body.append(hint);}if(!active){hint?.remove();hint=null;}return active;
 }
 function drop(id,event,source){const belongs=bridge?.state.config?.fences.some(f=>f.id===source&&f.items.includes(id));const active=belongs&&event?.type==='pointerup'&&event.button===0&&drag(event.clientX,event.clientY),path=bridge?.state.items?.[id]?.path;hint?.remove();hint=null;if(!active||!path)return false;bridge.invoke('quickbar_pin',{paths:[path]}).catch(notice);return true;}
 function itemPointer(event,id,source,session){
  if(event.button!==0)return;const node=event.currentTarget,start={x:event.clientX,y:event.clientY};let active=false,done=false,release,focusPending=false;
  const cleanup=()=>{for(const [type,fn]of [['pointermove',move],['pointerup',up],['pointercancel',cancel],['blur',blur],['keydown',key],['lostpointercapture',cancel]])window.removeEventListener(type,fn,true);try{node.releasePointerCapture(event.pointerId);}catch{}hint?.remove();hint=null;session.drag.id=null;session.drag.target=null;session.drag.before=null;if(active)sendPreview({id:null,source:null,point:null});release?.();bridge.region?.();if(active)Promise.resolve(bridge.focus?.(false)).catch(()=>{});};
  const finish=(e,save)=>{if(done)return;done=true;try{if(active&&save&&!drop(id,e,source)&&session.drag.target!==null)session.commit();}finally{cleanup();}};
  const move=e=>{if(done||e.pointerId!==event.pointerId)return;if(!active){if(Math.hypot(e.clientX-start.x,e.clientY-start.y)<6)return;active=true;release=bridge.hold?.();session.drag.id=id;session.drag.icon=bridge.state.items[id]?.icon?bridge.asset?.(bridge.state.items[id].icon)||'':'';session.drag.name=bridge.state.items[id]?.name||'';focusPending=true;Promise.resolve(bridge.focus?.(true)).catch(()=>{}).finally(()=>{focusPending=false;});}
   session.drag.x=e.clientX;session.drag.y=e.clientY;session.target(e.clientX,e.clientY);drag(e.clientX,e.clientY);sendPreview({id,source,point:[e.clientX,e.clientY]});};
  const up=e=>{if(e.pointerId===event.pointerId&&e.button===0)finish(e,true);};const cancel=e=>{if(e.pointerId===undefined||e.pointerId===event.pointerId)finish(e,false);};const blur=e=>{if(!focusPending)finish(e,false);};const key=e=>{if(e.key==='Escape')finish(e,false);};
  for(const [type,fn]of [['pointermove',move],['pointerup',up],['pointercancel',cancel],['blur',blur],['keydown',key],['lostpointercapture',cancel]])window.addEventListener(type,fn,true);try{node.setPointerCapture(event.pointerId);}catch{}
 }
 function notice(error){const p=document.createElement('div');p.className='db-merge-notice';p.textContent=String(error);p.setAttribute('role','status');document.body.append(p);setTimeout(()=>p.remove(),4000);}
 async function save(config,key,value){if(busy)return;busy=true;try{await bridge.invoke('update_settings',{settings:{...config.settings,[key]:value}}); }catch(e){notice(e);}finally{busy=false;}}
 function settings(h,c){const row=(title,help,control)=>h('div',{class:'row'},[h('div',null,[h('b',null,title),h('small',null,help)]),control]);
  const max=Math.max(12,...(bridge?.state.monitors||[]).map(m=>Math.floor(m.w/m.scale*.68/72)));
  return [row('隐藏任务栏','显示 DeskBox 快捷访问栏；关闭或退出后恢复 Windows 任务栏。全屏时自动避让。',h('input',{type:'checkbox',checked:!!c.settings.hideTaskbar,'aria-label':'隐藏任务栏',onChange:e=>save(c,'hideTaskbar',e.target.checked)})),
   row('快捷栏应用数量',`10–MAX；此电脑最大屏幕 MAX = ${max}，每个屏幕分别计算。拖入文件、文件夹或应用即可固定。`,h('select',{'aria-label':'快捷栏应用数量',value:String(c.settings.quickbarCount??12),disabled:!c.settings.hideTaskbar,onChange:e=>save(c,'quickbarCount',e.target.value==='MAX'?'MAX':Number(e.target.value))},[...Array.from({length:Math.max(3,max-9)},(_,i)=>h('option',{value:String(i+10)},String(i+10))),h('option',{value:'MAX'},'MAX（随屏幕变化）')]))];
 }
 async function toggle(){const s=await bridge.invoke('get_state');return save(s.config,'hideTaskbar',!s.config.settings.hideTaskbar);}
 function menuEntry(c){return {label:c?.settings.hideTaskbar?'恢复任务栏':'隐藏任务栏',icon:'settings',disabled:busy,action:()=>toggle().catch(notice)};}
 window.DeskBoxQuickbar={bounds,constrain,drag,drop,itemPointer,menuEntry,settings,bind:b=>{bridge=b;b.listen?.('quickbar-error',e=>notice(e.payload));},restore:()=>save(bridge.state.config,'hideTaskbar',false)};
})();
