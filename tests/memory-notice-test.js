const assert=require('node:assert/strict'),vm=require('node:vm'),fs=require('node:fs');
const notices=[];let regions=0;const context={window:{addEventListener(){}},document:{body:{append:n=>notices.push(n)},createElement:()=>({attributes:{},setAttribute(k,v){this.attributes[k]=v;},remove(){}})},setTimeout:()=>0,Date,Map,Set};
vm.runInNewContext(fs.readFileSync(require.resolve('../ui/widgets.js'),'utf8'),context);
context.window.DeskBoxWidgets.bind({region:()=>regions++,invoke:async()=>({beforeBytes:2097152,afterBytes:1048576,trimmedProcesses:1,skippedProcesses:0})});
(async()=>{await context.window.DeskBoxWidgets.optimizeMemory();assert(notices.some(n=>n.textContent.includes('减少 1.0 MB')));assert(notices.every(n=>'data-hit' in n.attributes));assert.equal(regions,2);console.log('MEMORY_NOTICE_REGION_OK');})().catch(e=>{console.error(e);process.exitCode=1;});
