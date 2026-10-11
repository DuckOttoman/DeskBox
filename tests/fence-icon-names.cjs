const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const win={addEventListener(){}};const f={id:'a',title:'A',items:['file'],style:{radius:26,media:{mode:'eco'}}},state={config:{fences:[f],mergeRestores:[]}},calls=[];
vm.runInNewContext(fs.readFileSync('ui/fence-merge.js','utf8'),{window:win});
win.DeskBoxFenceMerge.bind({state,invoke:async(c,a)=>{calls.push({c,a});state.config.fences=[a.fence]}});
(async()=>{
let entry=win.DeskBoxFenceMerge.menu([{label:'删除'}],f).find(e=>e.label==='隐藏图标名');assert.ok(entry,'fence menu needs hide icon names');await entry.action();
assert.equal(calls[0].c,'update_fence');assert.equal(state.config.fences[0].style.hideIconNames,true);assert.equal(state.config.fences[0].style.radius,26);assert.equal(f.style.hideIconNames,undefined,'toggle must not mutate state before persistence');
entry=win.DeskBoxFenceMerge.menu([{label:'删除'}],state.config.fences[0]).find(e=>e.label==='显示图标名');assert.ok(entry);await entry.action();assert.equal(state.config.fences[0].style.hideIconNames,false);assert.equal(JSON.stringify(state.config.fences[0].items),JSON.stringify(['file']));
console.log('FENCE_ICON_NAMES_OK: per-fence persisted reversible menu toggle preserving content/style');
})().catch(e=>{console.error(e);process.exitCode=1});
