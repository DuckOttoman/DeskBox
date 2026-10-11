(() => {
  function playbackSpec(media,allowed,{visible=true,collapsed=false}={}) {
    const source=media&&allowed&&visible&&!collapsed?(media.kind==='gif'?media.original:media.kind==='video'?(media.mode==='original'?media.original:media.eco):null):null;
    return {kind:media?.kind,source:source||null,poster:media?.poster||null};
  }
  class MediaSession {
    constructor(element,asset,onError=()=>{}) {
      this.element=element;this.asset=asset;this.path=null;this.failed=false;this.pending=false;this.generation=0;
      element.addEventListener('error',()=>{if(this.path){this.failed=true;element.dataset.state='error';element.style.display='none';onError('背景无法播放，请尝试节能模式或兼容的 MP4/WebM 文件。');}});
    }
    update(spec) {
      const el=this.element;
      if(this.path!==spec.source){
        this.generation++;this.pending=false;
        if(this.path){el.pause?.();el.removeAttribute('src');el.load?.();}
        this.path=spec.source;this.failed=false;delete el.dataset.error;
        if(this.path){el.src=this.asset(this.path);el.style.display='';el.dataset.state='playing';}
        else {el.style.display='none';el.dataset.state='paused';}
      }
      if(this.path&&!this.failed&&el.tagName==='VIDEO'&&el.paused&&!this.pending){
        this.pending=true;
        const generation=this.generation;
        Promise.resolve(el.play()).catch(e=>{if(this.path&&generation===this.generation){this.failed=true;el.dataset.state='error';el.style.display='none';}}).finally(()=>{if(generation===this.generation)this.pending=false;});
      }
    }
    destroy(){this.update({source:null});}
  }
  if(typeof module!=='undefined')module.exports={playbackSpec,MediaSession};
  if(typeof window==='undefined')return;
  let bridge,allowed=false,policyAt=0,policyPending=false,scheduled=false;
  const entries=new Map();
  function notice(message){
    const el=document.createElement('div');el.className='db-media-notice';el.setAttribute('role','status');el.textContent=message;document.body.append(el);setTimeout(()=>el.remove(),5500);
  }
  const setStyle=(el,name,value)=>{if(el.style[name]!==value)el.style[name]=value;};
  function sync(){
    scheduled=false;if(!bridge)return;
    const cfg=bridge.state.config;if(!cfg)return;
    const wanted=new Set();
    for(const host of document.querySelectorAll('[data-fence]')){
      const id=host.dataset.fence,f=cfg.fences.find(f=>f.id===id),media=f?.style.media,layers=host.querySelector(':scope > .bg');
      if(!media||!layers)continue;
      if(!layers.classList.contains('db-media-host'))layers.classList.add('db-media-host');
      wanted.add(id);let entry=entries.get(id);
      if(entry&&(entry.host!==host||entry.kind!==media.kind)){entry.session.destroy();entry.layer.remove();entries.delete(id);entry=null;}
      if(!entry){
        const layer=document.createElement('div');layer.className='db-media-layer';layer.setAttribute('aria-hidden','true');
        const el=document.createElement(media.kind==='video'?'video':'img');el.className='db-media-background';el.draggable=false;
        el.addEventListener('loadeddata',schedule);el.addEventListener('load',schedule);el.addEventListener('error',schedule);
        if(media.kind==='video'){el.muted=true;el.defaultMuted=true;el.volume=0;el.loop=true;el.playsInline=true;el.preload='metadata';el.setAttribute('muted','');el.setAttribute('playsinline','');}
        layer.append(el);layers.insertBefore(layer,layers.querySelector('.tint'));
        entry={host,layer,kind:media.kind,session:new MediaSession(el,bridge.asset,notice)};entries.set(id,entry);
      }
      const blur=f.style.imageBlur||0;
      setStyle(entry.layer,'opacity',String(f.style.imageOpacity??1));
      setStyle(entry.layer,'filter',blur?`blur(${blur}px)`:'none');
      setStyle(entry.layer,'inset',`${-blur*2}px`);
      const visible=cfg.fencesVisible&&!f.hidden&&window.DeskBoxFenceMerge.visible(f)&&document.visibilityState!=='hidden';
      entry.session.update(playbackSpec(media,allowed,{visible,collapsed:f.collapsed}));
      const poster=layers.querySelector(':scope > .image'),el=entry.session.element;
      if(poster)setStyle(poster,'visibility',entry.session.path&&!entry.session.failed&&(media.kind==='video'?el.readyState>=2:el.complete&&el.naturalWidth>0)?'hidden':'');
    }
    for(const [id,entry] of entries)if(!wanted.has(id)){entry.session.destroy();const bg=entry.host.querySelector(':scope > .bg');bg?.classList.remove('db-media-host');const poster=bg?.querySelector(':scope > .image');if(poster)setStyle(poster,'visibility','');entry.layer.remove();entries.delete(id);}
    if(wanted.size&&!policyPending&&performance.now()-policyAt>2000){
      policyPending=true;policyAt=performance.now();
      bridge.invoke('media_playback_allowed').then(value=>{allowed=value===true;schedule();}).catch(()=>{allowed=false;schedule();}).finally(()=>{policyPending=false;});
    }
  }
  function schedule(){if(!scheduled){scheduled=true;requestAnimationFrame(sync);}}
  window.DeskBoxMedia={
    bind(api){bridge=api;if(!api.label?.startsWith('overlay-'))return;policyAt=-3000;new MutationObserver(schedule).observe(document.body,{childList:true,subtree:true,attributes:true,attributeFilter:['class','style']});document.addEventListener('visibilitychange',schedule);setInterval(sync,750);schedule();},
    notice,
    async pick(fence){try{await bridge.invoke('pick_fence_image',{id:fence.id});schedule();}catch(e){notice(String(e));}},
    async clear(fence){try{await bridge.invoke('clear_fence_image',{id:fence.id});schedule();}catch(e){notice(String(e));}},
    controls(h,fence,busy){
      const media=fence.style.media;if(!media)return [];
      if(media.kind==='gif')return [h('p',{class:'db-media-hint'},'GIF 动画背景')];
      return [h('label',{},'视频播放模式'),h('select',{
        class:'db-video-mode',value:media.mode||'eco',disabled:busy,'aria-label':'视频播放模式',
        onChange:async event=>{const select=event.currentTarget;select.disabled=true;try{await bridge.invoke('set_fence_video_mode',{id:fence.id,mode:select.value});schedule();}catch(e){select.value=media.mode||'eco';notice(String(e));}finally{select.disabled=false;}}
      },[h('option',{value:'eco'},'节能模式（480p，24–30fps）'),h('option',{value:'original'},'原画模式（原分辨率、帧率）')]),h('p',{class:'db-media-hint'},media.mode==='original'?'直接播放原文件。':'首次生成节能副本，之后复用缓存。')];
    },
    panelStyle(x,y){const top=Math.max(8,Math.min(y,innerHeight-560));return {left:`${Math.max(8,Math.min(x,innerWidth-290))}px`,top:`${top}px`,maxHeight:`${Math.max(80,innerHeight-top-8)}px`,overflowY:'auto'};},
    resetStyle(style){return {image:style.image,...(style.media?{media:{...style.media,mode:style.media.kind==='video'?'eco':'original'}}:{})};},
    inspect:()=>[...entries].map(([id,e])=>({id,source:e.session.path,state:e.session.element.dataset.state,width:e.session.element.videoWidth,height:e.session.element.videoHeight}))
  };
})();
