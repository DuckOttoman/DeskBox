const assert=require('node:assert/strict');const {format}=require('../ui/cpu-temperature.js');
const s={cpuCoreMin:37.2,cpuCoreMax:62.6,cpuTempStatus:'ok'};
assert.equal(format(s,'min').value,'37°C');assert.equal(format(s,'max').value,'63°C');assert.equal(format(s,'both').value,'37°C / 63°C');
assert.equal(format({cpuThermalZone:27.85,cpuTempStatus:'enable_required'},'both').value,'不可用');assert.match(format({cpuThermalZone:27.85,cpuTempStatus:'enable_required'},'both').detail,/ACPI 热区/);
assert.equal(format({...s,cpuTempStatus:'stale'},'both').value,'不可用');console.log('CPU_TEMPERATURE_UI_POLICY_OK');

const fs=require('node:fs'),vm=require('node:vm');const w={id:'one',kind:'sysmon',style:{widgetData:{}}},other={id:'two',kind:'sysmon',style:{widgetData:{cpuTemperatureMode:'min'}}};
const context={window:{addEventListener(){}},document:{visibilityState:'visible'},Date,Map,Set,console};vm.runInNewContext(fs.readFileSync(require.resolve('../ui/widgets.js'),'utf8'),context);
const api=context.window.DeskBoxWidgets;api.bind({listen:()=>Promise.resolve(()=>{}),state:{config:{widgets:[w,other]}},invoke:async(c,a)=>{assert.equal(c,'patch_widget_data');Object.assign(w.style.widgetData,a.patch);}});
(async()=>{let menu=api.menu(w,[]).find(x=>x.label==='CPU 温度');assert.equal(menu.children[1].checked,true);await menu.children[2].action();assert.equal(w.style.widgetData.cpuTemperatureMode,'both');assert.equal(other.style.widgetData.cpuTemperatureMode,'min');assert.equal(api.menu(w,[])[0].children[2].checked,true);console.log('CPU_TEMPERATURE_MENU_MODES_OK');})().catch(e=>{console.error(e);process.exitCode=1});
