(() => {
  function duration(base,speed=1,reduced=false){
    if(reduced||speed<=0)return 0;
    if(!Number.isFinite(speed))speed=1;
    return Math.round(base/Math.max(.2,Math.min(5,speed)));
  }
  if(typeof module!=='undefined')module.exports={duration};
  if(typeof window==='undefined')return;
  let bridge;const jobs=new Set(),motion=matchMedia('(prefers-reduced-motion: reduce)');
  const counts={timer:0,open:0,close:0};
  const speed=()=>Number(bridge?.state.config?.settings.animSpeed??1);
  const time=base=>duration(base,speed(),motion.matches||document.hidden);
  function run(element,frames,base,cleanup=()=>{},kind='effect'){
    const ms=time(base);if(!ms){cleanup();return;}
    let animation;
    try{animation=element.animate(frames,{duration:ms,easing:'cubic-bezier(.2,.8,.2,1)',fill:'none'});}catch{cleanup();return;}
    const job={animation,kind};jobs.add(job);
    animation.finished.catch(()=>{}).finally(()=>{jobs.delete(job);cleanup();});
  }
  function cancelCloses(){for(const job of jobs)if(job.kind==='close')job.animation.cancel();}
  function sync(){
    document.documentElement.classList.toggle('db-motion-paused',document.hidden||motion.matches||speed()<=0);
    document.documentElement.style.setProperty('--db-breath-time',`${duration(3800,speed(),false)||3800}ms`);
    if(document.hidden||motion.matches||speed()<=0)for(const job of jobs)job.animation.cancel();
  }
  function menuGeometry(menu){const b=document.querySelector('.ball')?.getBoundingClientRect();if(!b)return null;
    return {x:b.left+b.width/2-(menu.offsetLeft+menu.offsetWidth/2),y:b.top+b.height/2-(menu.offsetTop+menu.offsetHeight/2)};
  }
  function openBallMenu(){
    cancelCloses();const menu=document.querySelector('.pm:not(.db-ball-menu-ghost)'),p=menu&&menuGeometry(menu);if(!p||!time(240))return;
    counts.open++;menu.style.transformOrigin='center center';
    run(menu,[{transform:`translate(${p.x}px,${p.y}px) scale(.18)`,opacity:0},{transform:'none',opacity:1}],240,()=>{},'open');
  }
  function closeBallMenu(){
    cancelCloses();const menu=document.querySelector('.pm:not(.db-ball-menu-ghost)'),p=menu&&menuGeometry(menu);
    if(!p||!time(160)||document.querySelector('.ball.pressed'))return;
    const clone=menu.cloneNode(true);clone.classList.add('db-ball-menu-ghost');clone.setAttribute('aria-hidden','true');clone.inert=true;
    clone.removeAttribute('data-hit');for(const n of clone.querySelectorAll('[data-hit]'))n.removeAttribute('data-hit');
    Object.assign(clone.style,{pointerEvents:'none',animation:'none',transformOrigin:'center center'});
    const release=bridge.hold();document.body.append(clone);counts.close++;
    run(clone,[{transform:getComputedStyle(menu).transform,opacity:1},{transform:`translate(${p.x}px,${p.y}px) scale(.12)`,opacity:0}],160,()=>{clone.remove();release();bridge.region();},'close');
  }
  function timerCompleted(id){
    const widget=[...document.querySelectorAll('[data-widget]')].find(el=>el.dataset.widget===id),content=widget?.querySelector('.db-timer');
    if(!content||!time(650))return;counts.timer++;
    const ring=document.createElement('span');ring.className='db-timer-completion-ring';ring.setAttribute('aria-hidden','true');
    const size=Math.min(content.clientWidth,content.clientHeight)*.72;ring.style.width=ring.style.height=`${size}px`;content.append(ring);
    run(ring,[{opacity:.85,transform:'translate(-50%,-50%) scale(.55)'},{opacity:0,transform:'translate(-50%,-50%) scale(1.3)'}],650,()=>ring.remove());
    if(!bridge.state.busy?.has(id))run(content,[{transform:'scale(1)'},{transform:'scale(1.035)',offset:.35},{transform:'scale(1)'}],500);
  }
  function bind(b){bridge=b;sync();b.listen('state',()=>setTimeout(sync,0)).catch(()=>{});}
  document.addEventListener('visibilitychange',sync);motion.addEventListener('change',sync);
  window.addEventListener('pagehide',()=>{for(const job of jobs)job.animation.cancel();});
  window.DeskBoxMotion={bind,timerCompleted,openBallMenu,closeBallMenu,duration,
    inspect:()=>({...counts,active:jobs.size,clones:document.querySelectorAll('.db-ball-menu-ghost').length,reduced:motion.matches,paused:document.documentElement.classList.contains('db-motion-paused')})};
})();
