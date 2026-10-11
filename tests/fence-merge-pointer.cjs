const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
let now=0,interval,simulateNativeFocusBlur=false;const listeners=new Map(),calls=[];
function element(id,r){return {dataset:{fence:id},style:{},classList:{add(){},remove(){}},offsetWidth:160,offsetHeight:35,
 getBoundingClientRect:()=>({...r,right:r.left+r.width,bottom:r.top+r.height}),querySelectorAll:()=>[],closest:()=>null,
 setPointerCapture(){this.captured=true},releasePointerCapture(){},remove(){},setAttribute(){},animate(){return {finished:Promise.resolve(),cancel(){}}}}}
const src=element('a',{left:50,top:80,width:320,height:260}),dst=element('b',{left:600,top:80,width:320,height:260});
const root={append(){}};const doc={body:root,querySelectorAll:()=>[src,dst],createElement:()=>element('',{left:0,top:0,width:160,height:35})};
const win={addEventListener:(t,f)=>{if(!listeners.has(t))listeners.set(t,new Set());listeners.get(t).add(f)},removeEventListener:(t,f)=>listeners.get(t)?.delete(f)};
const context={window:win,document:doc,performance:{now:()=>now},innerWidth:1920,innerHeight:1080,getComputedStyle:e=>({zIndex:e.style.zIndex||'0'}),setInterval:f=>{interval=f;return 1},clearInterval:()=>{},setTimeout:(f)=>{f();return 1},clearTimeout:()=>{},Promise,Map,Set,Date,console};
vm.runInNewContext(fs.readFileSync('ui/fence-merge.js','utf8'),context);
const s={busy:new Set(),config:{settings:{animSpeed:1},fenceGroups:[],mergeRestores:[],fences:[{id:'a',x:50,y:80,w:320,h:260,items:[]},{id:'b',x:600,y:80,w:320,h:260,items:[]}]}};
win.DeskBoxFenceMerge.bind({state:s,invoke:async(c,a)=>calls.push({c,a}),hold:()=>()=>{},region(){},focus:async on=>{if(on&&simulateNativeFocusBlur){await Promise.resolve();emit('blur',{});await Promise.resolve();}}});
const event=(x,y,button=0)=>({pointerId:7,clientX:x,clientY:y,button,target:src,currentTarget:src});
const emit=(t,e)=>{for(const f of [...(listeners.get(t)||[])])f(e)};
const flush=()=>new Promise(r=>setImmediate(r));
(async()=>{
 const start=()=>win.DeskBoxFenceMerge.pointer(event(70,95),s.config.fences[0],(_id,x,y)=>({x,y}));
 start();assert.equal(src.captured,undefined,'unmoved title click must retain its original target');emit('pointerup',event(70,95));await flush();assert.equal(calls.length,0);
 simulateNativeFocusBlur=true;
 start();emit('pointermove',event(170,210));await flush();await flush();
 assert.equal(win.DeskBoxFenceMerge.dragging,true,'native focus handoff must not cancel an active drag');
 assert.equal(s.config.fences[0].x,150,'focus handoff must not snap the fence back');
 emit('pointerup',event(170,210));await flush();await flush();
 assert.equal(calls.at(-1).c,'update_fence','plain drag must save on left release');assert.equal(calls.at(-1).a.fence.y,195);
 start();emit('pointermove',event(200,250));await flush();await flush();emit('blur',{});await flush();await flush();
 assert.equal(win.DeskBoxFenceMerge.dragging,false,'real focus loss after handoff must still cancel');assert.equal(s.config.fences[0].x,150);
 simulateNativeFocusBlur=false;calls.length=0;Object.assign(s.config.fences[0],{x:50,y:80});
 start();emit('pointermove',event(630,100));now=6000;interval();assert.equal(win.DeskBoxFenceMerge.preview,'contents');assert.equal(calls.length,0,'long held pointer never submits');
 emit('pointerup',event(630,100,2));assert.equal(calls.length,0,'right button release never submits');
 emit('pointercancel',event(630,100));await flush();await flush();assert.equal(calls.length,0,'cancel never submits');assert.equal(win.DeskBoxFenceMerge.dragging,false);
 start();emit('pointermove',event(630,100));now+=4999;emit('pointerup',event(630,100));await flush();await flush();assert.equal(calls.at(-1).c,'group_fences');
 start();emit('pointermove',event(630,100));now+=5000;interval();assert.equal(calls.length,1);emit('pointerup',event(630,100));await flush();await flush();assert.equal(calls.at(-1).c,'merge_fence_contents');
 assert.equal(calls.length,2);console.log('FENCE_MERGE_POINTER_OK: native focus handoff without rollback, genuine blur cancels, plain drag saves, unmoved click, held >5s no commit, right release ignored, cancel, group/content left release');
})().catch(e=>{console.error(e);process.exitCode=1});
