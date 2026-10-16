const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const notices=[];const w={id:'cpu',kind:'sysmon',style:{}};
const c={window:{addEventListener(){}},document:{createElement:()=>({setAttribute(){},remove(){}}),body:{append:n=>notices.push(n.textContent)}},setTimeout(){},Date,Map,Set,console};
vm.runInNewContext(fs.readFileSync(require.resolve('../ui/widgets.js'),'utf8'),c);c.window.DeskBoxWidgets.bind({listen:()=>Promise.resolve(()=>{}),state:{config:{widgets:[w]}},region(){},invoke:async()=>{throw '管理员授权已取消'}});
(async()=>{await c.window.DeskBoxWidgets.menu(w,[])[0].children.at(-1).action();assert(notices.some(s=>s.includes('授权已取消')));assert(!notices.some(s=>s.includes('采集器已启动')));console.log('CPU_PERMISSION_CANCEL_FEEDBACK_OK');})().catch(e=>{console.error(e);process.exitCode=1;});
