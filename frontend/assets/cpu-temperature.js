(()=>{
 const reasons={enable_required:'需要启用温度采集，请在右键菜单中启用或重试',driver_missing:'缺少 PawnIO 采集组件，请重新启用',sensor_unavailable:'硬件未提供可读的逐核心温度，或采集权限不足',package_only:'硬件仅提供封装温度，没有逐核心温度',stale:'温度采集已过期，等待重新连接',starting:'正在连接温度采集器',read_failed:'温度采集失败'};
 const number=v=>typeof v==='number'&&Number.isFinite(v)&&v>=-20&&v<=150;
 const degrees=v=>`${Math.round(v)}°C`;
 function format(s={},mode='max'){
  const label=mode==='min'?'最低核心':mode==='both'?'核心最低 / 最高':'最高核心';
  if(s.cpuTempStatus==='ok'&&number(s.cpuCoreMin)&&number(s.cpuCoreMax))return {label,value:mode==='min'?degrees(s.cpuCoreMin):mode==='both'?`${degrees(s.cpuCoreMin)} / ${degrees(s.cpuCoreMax)}`:degrees(s.cpuCoreMax),detail:'当前这一轮采样的核心温度'};
  let detail=reasons[s.cpuTempStatus]||'正在等待温度采样';
  if(number(s.cpuPackageTemp))detail+=`；CPU 封装 ${degrees(s.cpuPackageTemp)}`;
  if(number(s.cpuThermalZone))detail+=`；ACPI 热区 ${degrees(s.cpuThermalZone)}（非核心温度）`;
  return {label,value:'不可用',detail};
 }
 function view(h,w,s){const f=format(s,w?.style?.widgetData?.cpuTemperatureMode||'max');return h('span',{class:'db-cpu-temperature',title:f.detail},[h('b',null,f.value,1),h('small',null,f.label,1)],8,['title']);}
 if(typeof module!=='undefined')module.exports={format};if(typeof window!=='undefined')window.DeskBoxCpuTemperature={format,view};
})();
