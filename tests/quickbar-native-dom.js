setTimeout(async()=>{const invoke=(c,a)=>window.__TAURI_INTERNALS__.invoke(c,a);try{
 const x=window.__DeskBoxQuickbarTest.inspect();if(!x.ready||x.lastError||x.items!==1)throw Error('Dock not ready '+JSON.stringify(x));
 const icon=document.querySelector('#dock .face');if(getComputedStyle(icon).width!=='48px')throw Error('Incorrect icon size');
 if(document.querySelector('#dock').getBoundingClientRect().width>innerWidth*.68+1)throw Error('Dock width outside limit');
 const entry=x.state.entries[0];document.querySelector('#dock .item').dispatchEvent(new MouseEvent('contextmenu',{bubbles:true,clientX:innerWidth/2}));
 if(document.querySelector('#menu').hidden)throw Error('Own context menu missing');
 document.querySelector('#menu button').click();for(let i=0;i<60;i++){await new Promise(r=>setTimeout(r,50));if(!window.__DeskBoxQuickbarTest.inspect().state.entries.some(e=>e.path===entry.path))break;}
 if(window.__DeskBoxQuickbarTest.inspect().state.entries.some(e=>e.path===entry.path))throw Error('Remove UI stale');
 document.querySelector('#notice button').click();for(let i=0;i<60;i++){await new Promise(r=>setTimeout(r,50));if(window.__DeskBoxQuickbarTest.inspect().state.entries.some(e=>e.path===entry.path))break;}
 if(!window.__DeskBoxQuickbarTest.inspect().state.entries.some(e=>e.path===entry.path))throw Error('Undo UI failed');
 await invoke('quickbar_dom_report',{ok:true,message:'QUICKBAR_DOM_ICON_MENU_UNDO_SUCCESS'});
 }catch(e){await invoke('quickbar_dom_report',{ok:false,message:String(e.stack||e)});}},350);
