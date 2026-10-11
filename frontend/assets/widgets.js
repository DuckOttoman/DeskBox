(() => {
  const form=(w,h)=>w>=420&&h>=280?'large':w>=280&&h>=190?'medium':'small';
  const remaining=(t,now=Date.now())=>Math.max(0,t.running?(t.deadline||now)-now:(t.remaining??(t.focusMinutes||25)*60000));
  const nextPhoto=(i,n)=>n?(i+1)%n:0;
  const weatherText=c=>c===0?'晴':c<=3?'多云':c<=48?'雾':c<=67?'雨':c<=77?'雪':c<=82?'阵雨':c<=86?'阵雪':'雷雨';
  const weatherEmoji=c=>!Number.isInteger(c)?'🌡️':c===0?'☀️':c===1?'🌤️':c===2?'⛅':c===3?'☁️':c>=45&&c<=48?'🌫️':c>=51&&c<=67?'🌧️':c>=71&&c<=77?'❄️':c>=80&&c<=82?'🌦️':c>=85&&c<=86?'🌨️':c>=95&&c<=99?'⛈️':'🌡️';
  const ipCity=(r,provider='ipapi.co')=>{if(provider==='ipinfo.io'){if(typeof r?.loc!=='string'||!/^[-+]?\d+(?:\.\d+)?,[-+]?\d+(?:\.\d+)?$/.test(r.loc))return null;const [latitude,longitude]=r.loc.split(',').map(Number);r={...r,latitude,longitude};}return r&&!r.error&&typeof r.city==='string'&&r.city.trim()&&Number.isFinite(r.latitude)&&Math.abs(r.latitude)<=90&&Number.isFinite(r.longitude)&&Math.abs(r.longitude)<=180?{name:r.city.trim().slice(0,100),latitude:r.latitude,longitude:r.longitude,source:'ip',provider}:null;};
  if(typeof module!=='undefined')module.exports={form,remaining,nextPhoto,weatherText,weatherEmoji,ipCity};
  if(typeof window==='undefined')return;
  let bridge,modal,releaseModal,resizeSession,audio;
  let locationRequest;const locating=new Set();
  const drafts=new Map(),queues=new Map(),saveTimers=new Map(),slides=new Map(),weatherStatus=new Map(),finishing=new Set();
  const labels={clock:'时间与天气',sysmon:'系统状态',timer:'倒计时与番茄钟',notes:'便签与待办',album:'相册'};
  const get=id=>bridge?.state.config?.widgets.find(w=>w.id===id);
  const data=w=>w.style.widgetData||{};
  const dimensions=w=>({width:w.style.widgetWidth||(w.kind==='sysmon'?300:320),height:w.style.widgetHeight||(w.kind==='clock'?210:w.kind==='sysmon'?260:240)});
  const notice=text=>{const n=document.createElement('div');n.className='db-widget-notice';n.textContent=text;n.setAttribute('role','status');n.setAttribute('data-hit','');document.body.append(n);bridge?.region();setTimeout(()=>{n.remove();bridge?.region();},5000);};
  function patch(w,changes){
    const id=w.id,old=queues.get(id)||Promise.resolve();
    const next=old.catch(()=>{}).then(()=>bridge.invoke('patch_widget_data',{id,patch:changes}));queues.set(id,next);
    next.catch(e=>notice(String(e)));return next;
  }
  function interact(w,action,value=null){const id=w.id,old=queues.get(id)||Promise.resolve();const next=old.catch(()=>{}).then(()=>bridge.invoke('widget_interaction',{id,action,value}));queues.set(id,next);next.catch(e=>notice(String(e)));return next;}
  function flush(id){const timer=saveTimers.get(id);if(timer){clearTimeout(timer);saveTimers.delete(id);const w=get(id);if(w)patch(w,{note:drafts.get(id+':note')||''});}}
  function edit(ev){ev.stopPropagation();bridge.focus(true).then(()=>ev.target.focus()).catch(e=>notice(String(e)));}
  function closeDialog(){modal?.remove();modal=null;releaseModal?.();releaseModal=null;bridge?.focus(false).catch(()=>{});bridge?.region();}
  function dialog(title){closeDialog();releaseModal=bridge.hold();modal=document.createElement('section');modal.className='db-widget-dialog';modal.dataset.hit='';modal.setAttribute('role','dialog');modal.setAttribute('aria-label',title);const heading=document.createElement('header');heading.textContent=title;const close=document.createElement('button');close.textContent='×';close.setAttribute('aria-label','关闭');close.onclick=closeDialog;heading.append(close);modal.append(heading);document.body.append(modal);modal.addEventListener('pointerdown',e=>e.stopPropagation());modal.addEventListener('contextmenu',e=>e.preventDefault());bridge.region();bridge.focus(true).catch(e=>notice(String(e)));return modal;}
  function field(parent,label,value,type='text'){const row=document.createElement('label');row.textContent=label;const input=document.createElement('input');input.type=type;input.value=value;row.append(input);parent.append(row);input.addEventListener('pointerdown',edit);return input;}
  function button(parent,text,action){const b=document.createElement('button');b.textContent=text;b.onclick=()=>Promise.resolve(action()).catch(e=>notice(String(e)));parent.append(b);return b;}
  function timerSettings(w){const p=dialog('计时设置'),t=data(w).timer||{};const focus=field(p,'专注分钟',t.focusMinutes||25,'number'),rest=field(p,'休息分钟',t.breakMinutes||5,'number'),count=field(p,'倒计时分钟',t.countMinutes||10,'number');button(p,'保存',()=>{const nums=[focus,rest,count].map(x=>Number(x.value));if(nums.some(n=>!Number.isFinite(n)||n<1||n>1440))throw Error('请输入1至1440分钟');interact(get(w.id),'timer_settings',{focusMinutes:nums[0],breakMinutes:nums[1],countMinutes:nums[2]});closeDialog();});}
  async function json(url){const c=new AbortController(),timer=setTimeout(()=>c.abort(),12000);try{const r=await fetch(url,{signal:c.signal});if(!r.ok)throw Error('天气服务暂不可用');return await r.json();}finally{clearTimeout(timer);}}
  function cityDialog(w){const p=dialog('选择天气城市'),name=field(p,'城市名称',''),results=document.createElement('div');results.className='db-city-results';p.append(results);let sequence=0;button(p,'搜索',async()=>{const query=name.value.trim();if(query.length<2)throw Error('请输入至少两个字符');const seq=++sequence;results.textContent='搜索中…';try{const r=await json(`https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(query)}&count=8&language=zh&format=json`);if(seq!==sequence||!p.isConnected)return;results.textContent='';if(!r.results?.length)results.textContent='没有找到城市，请尝试英文名称';for(const city of r.results||[])button(results,[city.name,city.admin1,city.country].filter(Boolean).join(' · '),async()=>{await patch(get(w.id),{weatherCity:{name:city.name,latitude:city.latitude,longitude:city.longitude},weatherCache:null,weatherAutoLocate:false,weatherLocationError:false});weatherStatus.delete(w.id);closeDialog();refreshWeather(get(w.id),true);});}catch(e){results.textContent='搜索失败，请检查网络';}});}
  async function refreshWeather(w,force=false){
    if(!w)return;const d=data(w),city=d.weatherCity;if(!city)return;const status=weatherStatus.get(w.id)||{};
    if(status.pending||(!force&&Date.now()-(status.at||d.weatherCache?.at||0)<15*60000))return;
    const key=JSON.stringify(city);weatherStatus.set(w.id,{pending:true,at:Date.now(),error:false});
    try{const r=await json(`https://api.open-meteo.com/v1/forecast?latitude=${city.latitude}&longitude=${city.longitude}&current=temperature_2m,weather_code,relative_humidity_2m&daily=temperature_2m_max,temperature_2m_min,weather_code&timezone=auto&forecast_days=3`);
      if(JSON.stringify(data(get(w.id)||w).weatherCity)!==key)return;
      if(!r.current||!Number.isFinite(r.current.temperature_2m))throw Error('天气数据无效');
      await patch(get(w.id),{weatherCache:{at:Date.now(),temperature:r.current.temperature_2m,code:r.current.weather_code,humidity:r.current.relative_humidity_2m,daily:r.daily}});weatherStatus.set(w.id,{at:Date.now(),pending:false,error:false});
    }catch(e){weatherStatus.set(w.id,{at:Date.now(),pending:false,error:true});}
  }
  async function locateWeather(w){
    if(!w||data(w).weatherCity||!data(w).weatherAutoLocate||locating.has(w.id))return;locating.add(w.id);
    try{locationRequest ||= (async()=>{for(const [url,provider] of [['https://ipapi.co/json/','ipapi.co'],['https://ipinfo.io/json','ipinfo.io']]){try{const city=ipCity(await json(url),provider);if(city)return city;}catch(e){}}throw Error('IP定位不可用');})().catch(e=>{locationRequest=null;throw e;});
      const city=await locationRequest;await bridge.invoke('apply_weather_location',{id:w.id,city});const current=get(w.id);if(current)refreshWeather(current,true);
    }catch(e){const current=get(w.id);if(current&&!data(current).weatherCity)await patch(current,{weatherAutoLocate:false,weatherLocationError:true}).catch(()=>{});}
    finally{locating.delete(w.id);}
  }
  let memoryBusy=false;
  async function optimizeMemory(){if(memoryBusy)return;memoryBusy=true;notice('正在优化 DeskBox 内存…');try{const r=await bridge.invoke('optimize_memory');const delta=r.beforeBytes-r.afterBytes;notice(`内存优化完成：工作集${delta>=0?'减少':'增加'} ${(Math.abs(delta)/1048576).toFixed(1)} MB，处理 ${r.trimmedProcesses} 个应用进程${r.skippedProcesses?`，${r.skippedProcesses} 个未处理`:''}。系统会按需重新载入。`);}catch(e){notice(String(e));}finally{memoryBusy=false;}}
  function chime(){try{audio ||= new (window.AudioContext||window.webkitAudioContext)();audio.resume();const o=audio.createOscillator(),g=audio.createGain();o.frequency.value=660;g.gain.value=.08;o.connect(g);g.connect(audio.destination);o.start();g.gain.exponentialRampToValueAtTime(.001,audio.currentTime+.5);o.stop(audio.currentTime+.5);}catch(e){}}
  function timerAction(w,action){const t=data(w).timer||{};if(action==='toggle'){if(!t.running){try{audio ||= new (window.AudioContext||window.webkitAudioContext)();audio.resume();}catch(e){}}return interact(w,t.running?'timer_pause':'timer_start');}return interact(w,'timer_'+action,action==='mode'?(t.mode==='countdown'?'pomodoro':'countdown'):null);}
  function finishTimer(w){const t=data(w).timer;if(!t?.running||remaining(t)>0||finishing.has(w.id))return;finishing.add(w.id);const title=t.mode==='countdown'?'倒计时结束':t.phase==='break'?'休息结束，开始下一轮专注吧':'专注完成，休息一下吧';interact(w,'timer_finish').then(changed=>{if(changed){chime();notice(title);}}).finally(()=>finishing.delete(w.id)).catch(()=>{});}
  function content(h,w){const d=data(w),size=dimensions(w),shape=form(size.width,size.height),children=[];
    const btn=(text,action,extra={})=>h('button',{...extra,onClick:action,onPointerdown:e=>e.stopPropagation()},text);
    if(w.kind==='clock'){
      const now=new Date();children.push(h('div',{class:'db-clock-time'},now.toLocaleTimeString('zh-CN',{hour:'2-digit',minute:'2-digit',...(shape==='large'?{second:'2-digit'}:{})})),h('div',{class:'db-clock-date'},now.toLocaleDateString('zh-CN',{month:'long',day:'numeric',weekday:'long'})));
      const cache=d.weatherCache,status=weatherStatus.get(w.id),city=d.weatherCity;
      children.push(h('div',{class:'db-weather'},city?[h('span',{},`${city.name} · ${cache?`${shape!=='small'?weatherEmoji(cache.code)+' ':''}${Math.round(cache.temperature)}° ${weatherText(cache.code)}`:status?.error?'天气获取失败':'天气加载中…'}`),status?.error||cache&&Date.now()-cache.at>30*60000?h('small',{},' 离线缓存'):null]:[h('small',{},locating.has(w.id)?'IP城市定位中…':d.weatherLocationError?'自动定位失败，请选择城市':''),btn('选择城市查看天气',()=>cityDialog(w))]));
      if(shape==='large'&&cache?.daily)children.push(h('div',{class:'db-weather-days'},cache.daily.time.map((day,i)=>h('div',{},[h('small',{},i===0?'今天':day.slice(5)),h('span',{},`${weatherEmoji(cache.daily.weather_code[i])} ${weatherText(cache.daily.weather_code[i])}`),h('b',{},`${Math.round(cache.daily.temperature_2m_min[i])}° / ${Math.round(cache.daily.temperature_2m_max[i])}°`)]))));
      if(city?.source==='ip'&&shape!=='small')children.push(h('small',{class:'db-provider',title:'IP定位为大致城市，可通过右键菜单手动修改'},`IP 大致定位 · ${city.provider||'ipapi.co'}`));
      if(city&&shape!=='small')children.push(h('small',{class:'db-provider'},[btn('天气：Open-Meteo',()=>bridge.invoke('open_widget_weather_source'))]));
    }
    if(w.kind==='timer'){
      const t=data(w).timer||{},ms=remaining(t),seconds=Math.ceil(ms/1000),clock=`${String(Math.floor(seconds/60)).padStart(2,'0')}:${String(seconds%60).padStart(2,'0')}`;
      children.push(h('div',{class:'db-timer-phase'},t.mode==='countdown'?'倒计时':t.phase==='break'?'休息时间':'专注时间'),h('div',{class:'db-timer-digits'},clock),h('div',{class:'db-widget-actions'},[btn(t.running?'暂停':'开始',()=>timerAction(w,'toggle')),btn('重置',()=>timerAction(w,'reset'))]));
      if(shape!=='small')children.push(h('div',{class:'db-widget-actions'},[btn(t.mode==='countdown'?'切换番茄钟':'切换倒计时',()=>timerAction(w,'mode')),btn('时长设置',()=>timerSettings(w))]));
      if(shape==='large')children.push(h('p',{class:'db-widget-hint'},`专注 ${t.focusMinutes||25} 分钟 · 休息 ${t.breakMinutes||5} 分钟。结束时提示，手动开始下一阶段。`));
    }
    if(w.kind==='notes'){
      const todos=d.todos||[],key=w.id+':note',note=drafts.has(key)?drafts.get(key):d.note||'';
      children.push(h('textarea',{class:'db-note',maxLength:10000,value:note,placeholder:'写下便签…',onPointerdown:edit,onInput:e=>{drafts.set(key,e.target.value);clearTimeout(saveTimers.get(w.id));saveTimers.set(w.id,setTimeout(()=>{saveTimers.delete(w.id);const current=get(w.id);if(current)patch(current,{note:drafts.get(key)});},500));},onBlur:()=>flush(w.id)}));
      if(shape==='small')children.push(h('small',{},`${todos.filter(t=>!t.done).length} 项待办 · 放大查看`));
      else{
        const inputKey=w.id+':todo';children.push(h('div',{class:'db-todo-add'},[h('input',{placeholder:'新增待办，回车保存',value:drafts.get(inputKey)||'',onPointerdown:edit,onInput:e=>drafts.set(inputKey,e.target.value),onKeydown:e=>{if(e.key==='Enter'&&!e.isComposing){const text=(drafts.get(inputKey)||'').trim();if(text){interact(w,'todo_add',{text});drafts.set(inputKey,'');e.target.value='';}}}})]));
        children.push(h('div',{class:'db-todos'},todos.map(t=>h('div',{class:t.done?'db-todo done':'db-todo'},[h('input',{type:'checkbox',checked:t.done,onPointerdown:e=>e.stopPropagation(),onChange:e=>interact(w,'todo_set',{id:t.id,done:e.target.checked})}),h('span',{},t.text),btn('×',()=>interact(w,'todo_remove',{id:t.id}),{'aria-label':'删除待办'})]))));
      }
    }
    if(w.kind==='album'){
      const photos=d.photos||[];let session=slides.get(w.id);if(!session){session={index:0,at:Date.now(),failed:new Set()};slides.set(w.id,session);}session.index=Math.min(session.index,Math.max(0,photos.length-1));
      const photo=photos[session.index];
      children.push(photo?h('div',{class:'db-album-picture'},[h('img',{key:photo.path,src:bridge.asset(photo.path),alt:photo.name,draggable:false,style:{objectFit:d.fit||'cover'},onError:()=>{session.failed.add(photo.path);notice('图片不可用，请检查相册目录或重新添加图片');}})]):h('div',{class:'db-album-empty'},[h('span',{},'通过右键菜单添加图片'),btn('添加图片',()=>albumAction(w,'add'))]));
      if(shape!=='small')children.push(h('div',{class:'db-album-caption'},[h('span',{},photo?.name||'相册'),h('div',{class:'db-widget-actions'},[btn('‹',()=>{session.index=photos.length?(session.index+photos.length-1)%photos.length:0;session.at=Date.now();}),btn(d.paused?'播放':'暂停',()=>patch(w,{paused:!d.paused})),btn('›',()=>{session.index=nextPhoto(session.index,photos.length);session.at=Date.now();})])]));
      if(shape==='large')children.push(h('small',{class:'db-widget-hint'},`${photos.length?session.index+1:0} / ${photos.length} · ${d.interval||8} 秒切换`));
    }
    return h('div',{class:`db-widget-content db-${w.kind} db-size-${shape}`},children);
  }
  function albumAction(w,action,index){return bridge.invoke('widget_album_action',{id:w.id,action,index:index??null,sources:null,directory:null}).catch(e=>notice(String(e)));}
  function albumManage(w){const p=dialog('管理相册图片');const list=document.createElement('div');list.className='db-album-list';p.append(list);const render=()=>{list.replaceChildren();for(const [index,photo] of (data(get(w.id)).photos||[]).entries()){const row=document.createElement('div');row.textContent=photo.name;button(row,'上移',async()=>{await albumAction(w,'up',index);setTimeout(render,150);});button(row,'移除',async()=>{await albumAction(w,'remove',index);setTimeout(render,150);});list.append(row);}};render();button(p,'添加图片',async()=>{await albumAction(w,'add');setTimeout(render,150);});const interval=field(p,'轮播间隔（秒）',data(w).interval||8,'number');button(p,'保存间隔',()=>{const n=Number(interval.value);if(!Number.isFinite(n)||n<2||n>3600)throw Error('请输入2至3600秒');return patch(get(w.id),{interval:n});});const path=document.createElement('small');path.textContent=data(w).albumDirectory||'默认：程序目录 / Albums';p.append(path);}
  function menu(w,original){const options=[];if(w.kind==='clock')options.push({label:'选择天气城市',icon:'edit',action:()=>cityDialog(w)},{label:'刷新天气',icon:'refresh',disabled:!data(w).weatherCity,action:()=>refreshWeather(w,true)},{label:'天气数据：Open-Meteo',icon:'info',action:()=>bridge.invoke('open_widget_weather_source')});
    if(w.kind==='timer')options.push({label:'计时设置',icon:'clock',action:()=>timerSettings(w)});
    if(w.kind==='notes')options.push({label:'清除已完成待办',icon:'trash',action:()=>interact(w,'todo_clear_done')});
    if(w.kind==='album')options.push({label:'添加图片…',icon:'plus',action:()=>albumAction(w,'add')},{label:'管理图片与轮播…',icon:'edit',action:()=>albumManage(w)},{label:'更换相册存储目录…',icon:'folder',action:()=>albumAction(w,'directory')},{label:data(w).paused?'开始轮播':'暂停轮播',icon:'clock',action:()=>patch(w,{paused:!data(w).paused})},{label:'图片适应方式',children:['cover','contain'].map(v=>({label:v==='cover'?'填满裁切':'完整显示',checked:(data(w).fit||'cover')===v,action:()=>patch(w,{fit:v})}))});
    return [...options,...options.length?[{separator:true}]:[],...original];}
  function pointer(ev,w,snap,resize){
    if(ev.button!==0||resizeSession||(!resize&&ev.target.closest('button,input,textarea,.db-widget-content')))return;
    ev.stopPropagation();const node=ev.currentTarget.closest('[data-widget]')||ev.currentTarget,old={x:w.x,y:w.y,...dimensions(w)},start={x:ev.clientX,y:ev.clientY},release=bridge.hold();let active=false;
    bridge.state.busy.add(w.id);node.setPointerCapture(ev.pointerId);node.classList.add(resize?'resizing':'moving');resizeSession={id:w.id};
    const move=e=>{if(e.pointerId!==ev.pointerId)return;w=get(w.id)||w;const dx=e.clientX-start.x,dy=e.clientY-start.y;if(!active&&Math.hypot(dx,dy)<3)return;active=true;
      if(resize){const width=Math.min(1200,Math.max(180,Math.min(window.innerWidth-old.x,old.width+(resize.includes('r')?dx:0)))),height=Math.min(1000,Math.max(100,Math.min(window.innerHeight-old.y,old.height+(resize.includes('b')?dy:0))));w.style.widgetWidth=width;w.style.widgetHeight=height;}
      else{const pos=snap(w.id,old.x+dx,old.y+dy,old.width,old.height);w.x=pos.x;w.y=pos.y;}bridge.region();};
    const end=e=>{if(e.pointerId!==ev.pointerId)return;w=get(w.id)||w;window.removeEventListener('pointermove',move,true);window.removeEventListener('pointerup',end,true);window.removeEventListener('pointercancel',end,true);resizeSession=null;node.classList.remove('moving','resizing');if(node.hasPointerCapture(ev.pointerId))node.releasePointerCapture(ev.pointerId);release();bridge.state.busy.delete(w.id);
      if(e.type==='pointercancel'){w.x=old.x;w.y=old.y;w.style.widgetWidth=old.width;w.style.widgetHeight=old.height;}else if(active)bridge.invoke('update_widget',{widget:JSON.parse(JSON.stringify(w))}).catch(e=>notice(String(e)));bridge.region();};
    window.addEventListener('pointermove',move,true);window.addEventListener('pointerup',end,true);window.addEventListener('pointercancel',end,true);
  }
  function handles(h,w,snap){return ['r','b','rb'].map(edge=>h('div',{class:`db-widget-resize ${edge}`,'aria-label':'调整小组件大小',onPointerdown:e=>pointer(e,get(w.id),snap,edge)}));}
  function tick(w){if(w.kind==='timer')finishTimer(w);if(w.kind==='clock'&&document.visibilityState!=='hidden'&&bridge.state.config?.fencesVisible){if(!data(w).weatherCity)locateWeather(w);else refreshWeather(w);}
    if(w.kind==='album'){const d=data(w),s=slides.get(w.id),photos=d.photos||[];if(!s)return;if(document.visibilityState==='hidden'||!bridge.state.config?.fencesVisible||d.paused){s.at=Date.now();return;}if(Date.now()-s.at>=(d.interval||8)*1000){s.index=nextPhoto(s.index,photos.length);s.at=Date.now();}}
  }
  const backupTime=t=>new Date(t).toLocaleString('zh-CN',{hour12:false});
  function pruneWidgetCaches(){
    const ids=new Set((bridge.state.config?.widgets||[]).map(w=>w.id));
    for(const key of drafts.keys())if(!ids.has(key.slice(0,key.lastIndexOf(':'))))drafts.delete(key);
    for(const [id,timer] of saveTimers)if(!ids.has(id)){clearTimeout(timer);saveTimers.delete(id);}
    for(const map of [slides,weatherStatus])for(const id of map.keys())if(!ids.has(id))map.delete(id);
  }
  async function layoutAction(action,id=null,name=null){
    pruneWidgetCaches();
    if(bridge.state.busy?.size)throw Error('请先结束拖动或缩放，再操作布局备份');
    for(const w of bridge.state.config?.widgets||[])flush(w.id);
    await Promise.all([...queues.values()].map(p=>p.catch(()=>{})));
    return bridge.invoke('layout_backup_action',{action,id,name});
  }
  async function createLayoutBackup(){
    const title=`布局备份 ${backupTime(Date.now())}`;
    await layoutAction('create',null,title);notice('布局备份已保存');
  }
  async function manageLayoutBackups(){
    const p=dialog('还原备份'),list=document.createElement('div');list.className='db-layout-backups';p.append(list);
    const hint=document.createElement('small');hint.textContent='还原位置和属性；保留当前便签、待办、相册内容，桌面文件重新分类。还原前自动备份当前布局。';p.append(hint);
    let pending=false;
    async function perform(action,id,name){if(pending)return;pending=true;p.setAttribute('aria-busy','true');try{
      const r=await layoutAction(action,id,name);if(modal!==p)return;
      render(r.backups);if(action==='restore')notice(`布局已还原${r.missingBackgrounds?`；${r.missingBackgrounds} 个背景文件不可用，已使用普通背景`:''}`);
    }finally{pending=false;p.removeAttribute('aria-busy');}}
    function render(backups){list.replaceChildren();if(!backups.length){const empty=document.createElement('p');empty.textContent='暂无布局备份';list.append(empty);return;}
      for(const b of backups){const row=document.createElement('article');row.dataset.backup=b.id;
        const title=document.createElement('strong');title.textContent=b.name;row.append(title);
        const time=document.createElement('time');time.dateTime=new Date(b.createdAt).toISOString();time.textContent=`备份时间：${backupTime(b.createdAt)}`;row.append(time);
        const count=document.createElement('small');count.textContent=`${b.fenceCount} 个文件夹框 · ${b.widgetCount} 个小组件`;row.append(count);
        button(row,'还原',()=>perform('restore',b.id));
        button(row,'重命名',()=>{if(pending||row.querySelector('input'))return;const input=field(row,'备份名称',b.name);input.maxLength=100;button(row,'保存名称',()=>perform('rename',b.id,input.value));input.focus();});
        let confirm=false;const remove=button(row,'删除',()=>{if(pending)return;if(!confirm){confirm=true;remove.textContent='确认删除';return;}return perform('delete',b.id);});
        list.append(row);
      }
    }
    try{render((await layoutAction('list')).backups);bridge.region();}catch(e){notice(String(e));}
  }
  function bind(b){bridge=b;
    b.listen('state',()=>setTimeout(pruneWidgetCaches,0)).catch(e=>notice(String(e)));
    b.listen('layout-backup-panel',async event=>{if(event.payload?.label!==b.label)return;
      try{if(event.payload.action==='create')await createLayoutBackup();else await manageLayoutBackups();}catch(e){notice(String(e));}
    }).catch(e=>notice(String(e)));
  }
  window.addEventListener('keydown',e=>{if(e.key==='Escape'&&modal)closeDialog();});
  window.DeskBoxWidgets={createLayoutBackup,manageLayoutBackups,layoutAction,optimizeMemory,bind,dimensions,form,content,handles,menu,pointer,tick,label:w=>labels[w.kind],flush,inspect:id=>({data:data(get(id)),dimensions:dimensions(get(id)),shape:form(...Object.values(dimensions(get(id))))})};
})();
