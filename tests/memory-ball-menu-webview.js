(() => {
const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),wait=ms=>new Promise(r=>setTimeout(r,ms));
async function until(fn){for(let i=0;i<160;i++){if(await fn())return;await wait(100);}throw Error('Ball memory menu timeout');}
async function run(){await until(()=>window.DeskBoxWidgets&&document.querySelector('.ball'));await wait(1200);document.querySelector('.ball').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:300,clientY:300}));await until(()=>document.querySelector('.pm'));
const item=[...document.querySelectorAll('.pm-item')].find(n=>n.textContent.trim()==='内存优化');if(!item)throw Error('Missing menu');item.click();
await until(()=>[...document.querySelectorAll('.db-widget-notice')].some(n=>n.textContent.includes('内存优化完成')&&n.hasAttribute('data-hit')));
await invoke('frontend_checks_passed',{checks:['MEMORY_BALL_MENU_SUCCESS','fence_merge_release']});await invoke('quit_app');}
window.addEventListener('error',e=>invoke('frontend_error',{message:String(e.message)}));window.addEventListener('unhandledrejection',e=>invoke('frontend_error',{message:String(e.reason)}));
window.addEventListener('DOMContentLoaded',()=>{const t=setInterval(()=>{if(document.querySelector('#app')?.children.length){clearInterval(t);invoke('frontend_ready',{nodes:1,role:document.documentElement.dataset.role});if(window.__TAURI_INTERNALS__.metadata.currentWindow.label==='ball')run().catch(async e=>{await invoke('frontend_error',{message:String(e.stack||e)});await invoke('quit_app');});}},100);});})();
