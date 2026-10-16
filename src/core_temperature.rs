use serde::Deserialize;
#[derive(Default,Debug)]
pub struct Reading {pub min:Option<f64>,pub max:Option<f64>,pub package:Option<f64>,pub status:String}
#[derive(Deserialize)]struct Core {id:String,value:f64}
#[derive(Deserialize)]struct Packet {status:String,timestamp:u64,cores:Vec<Core>,package:Option<f64>}
fn valid(v:f64)->bool{v.is_finite()&&(-20.0..=150.0).contains(&v)}
pub fn decode(text:&str,now:u64)->Reading {
 let Ok(p)=serde_json::from_str::<Packet>(text) else{return Reading{status:"read_failed".into(),..Default::default()}};
 if p.timestamp>now.saturating_add(2000)||now.saturating_sub(p.timestamp)>5000{return Reading{status:"stale".into(),..Default::default()};}
 let mut out=Reading{status:p.status.clone(),package:p.package.filter(|v|valid(*v)),..Default::default()};
 if p.status=="ok"{let mut ids=std::collections::BTreeSet::new();for c in p.cores {if valid(c.value)&&!c.id.is_empty()&&ids.insert(c.id){out.min=Some(out.min.map_or(c.value,|v|v.min(c.value)));out.max=Some(out.max.map_or(c.value,|v|v.max(c.value)));}}if out.min.is_none(){out.status="sensor_unavailable".into();}}
 out
}
pub fn append_packet(buffer:&mut Vec<u8>,chunk:&[u8])->Result<Option<String>,String>{
 if buffer.len()+chunk.len()>128*1024{return Err("Temperature packet too large".into());}buffer.extend_from_slice(chunk);
 if let Some(end)=buffer.iter().position(|b|*b==b'\n'){return String::from_utf8(buffer[..end].to_vec()).map(Some).map_err(|_|"Invalid temperature packet".into());}Ok(None)
}
#[cfg(all(windows,feature="gui"))]
mod native {
 use super::*;
 use std::{ffi::c_void,os::windows::{fs::OpenOptionsExt,io::AsRawHandle,process::CommandExt},process::{Command,Stdio},time::{Instant,Duration,SystemTime,UNIX_EPOCH}};
 #[repr(C)]struct Overlapped {internal:usize,internal_high:usize,offset:u32,offset_high:u32,event:*mut c_void}
 #[link(name="kernel32")]
 unsafe extern "system" {
  fn CreateEventW(attrs:*const c_void,manual:i32,initial:i32,name:*const u16)->*mut c_void;
  fn CloseHandle(handle:*mut c_void)->i32;
  fn ResetEvent(handle:*mut c_void)->i32;
  fn ReadFile(handle:*mut c_void,buffer:*mut u8,size:u32,read:*mut u32,overlapped:*mut Overlapped)->i32;
  fn WaitForSingleObject(handle:*mut c_void,millis:u32)->u32;
  fn GetOverlappedResult(handle:*mut c_void,overlapped:*mut Overlapped,count:*mut u32,wait:i32)->i32;
  fn CancelIoEx(handle:*mut c_void,overlapped:*mut Overlapped)->i32;
  fn GetNamedPipeServerProcessId(pipe:*mut c_void,pid:*mut u32)->i32;
  fn OpenProcess(access:u32,inherit:i32,pid:u32)->*mut c_void;
  fn QueryFullProcessImageNameW(process:*mut c_void,flags:u32,path:*mut u16,size:*mut u32)->i32;
 }

 #[repr(C)]#[derive(Default)]struct ServiceStatus {kind:u32,state:u32,controls:u32,win32:u32,specific:u32,checkpoint:u32,hint:u32,pid:u32,flags:u32}
 #[repr(C)]struct ServiceConfig {kind:u32,start:u32,error:u32,binary:*const u16,group:*const u16,tag:u32,dependencies:*const u16,account:*const u16,display:*const u16}
 #[link(name="advapi32")]
 unsafe extern "system" {
  fn OpenSCManagerW(machine:*const u16,database:*const u16,access:u32)->*mut c_void;
  fn OpenServiceW(manager:*mut c_void,name:*const u16,access:u32)->*mut c_void;
  fn CloseServiceHandle(handle:*mut c_void)->i32;
  fn QueryServiceStatusEx(service:*mut c_void,level:u32,buffer:*mut u8,size:u32,needed:*mut u32)->i32;
  fn QueryServiceConfigW(service:*mut c_void,config:*mut ServiceConfig,size:u32,needed:*mut u32)->i32;
 }
 fn verified_service(pid:u32,expected:&str)->bool{unsafe{
  let manager=OpenSCManagerW(std::ptr::null(),std::ptr::null(),1);if manager.is_null(){return false;}
  let name:Vec<u16>="DeskBoxCpuSensor".encode_utf16().chain(Some(0)).collect();let service=OpenServiceW(manager,name.as_ptr(),5);CloseServiceHandle(manager);if service.is_null(){return false;}
  let mut status=ServiceStatus::default();let mut needed=0;
  let ok=QueryServiceStatusEx(service,0,(&mut status as *mut ServiceStatus).cast(),std::mem::size_of::<ServiceStatus>() as u32,&mut needed)!=0&&status.pid==pid&&matches!(status.state,2|4);
  let mut verified=false;
  if ok{
   QueryServiceConfigW(service,std::ptr::null_mut(),0,&mut needed);
   if needed>0&&needed<=65536{
    let mut buffer=vec![0u64;(needed as usize+7)/8];let ptr=buffer.as_mut_ptr().cast::<ServiceConfig>();
    if QueryServiceConfigW(service,ptr,needed,&mut needed)!=0&&!(*ptr).binary.is_null(){
     let binary=(*ptr).binary;let mut n=0;while n<32768&&*binary.add(n)!=0{n+=1;}
     if n<32768{let path=String::from_utf16_lossy(std::slice::from_raw_parts(binary,n)).to_lowercase();verified=path==format!("\"{}\" --service",expected);}
    }
   }
  }
  CloseServiceHandle(service);verified
 }}
 fn pipe()->Option<String>{
  let file=std::fs::OpenOptions::new().read(true).custom_flags(0x40000000).open(r"\\.\pipe\DeskBoxCpuSensor.v1").ok()?;
  unsafe{
   let mut pid=0;if GetNamedPipeServerProcessId(file.as_raw_handle(),&mut pid)==0{return None;}
   let installed=std::path::PathBuf::from(std::env::var_os("ProgramW6432").or_else(||std::env::var_os("ProgramFiles")).unwrap_or_default()).join("DeskBox Sensor/DeskBox.Sensor.exe");
   let expected=installed.to_string_lossy().replace('/',"\\").to_lowercase();
   if !verified_service(pid,&expected){
    let process=OpenProcess(0x1000,0,pid);if process.is_null(){return None;}let mut path=[0u16;32768];let mut size=path.len() as u32;
    let found=QueryFullProcessImageNameW(process,0,path.as_mut_ptr(),&mut size);CloseHandle(process);if found==0{return None;}
    let server=String::from_utf16_lossy(&path[..size as usize]).to_lowercase();
    if server!=expected{return None;}
   }
   let event=CreateEventW(std::ptr::null(),1,0,std::ptr::null());if event.is_null(){return None;}
   let deadline=Instant::now()+Duration::from_millis(800);let mut packet=Vec::new();let mut answer=None;
   loop{
    let mut o=Overlapped{internal:0,internal_high:0,offset:0,offset_high:0,event};let mut bytes=[0u8;8192];let mut count=0;ResetEvent(event);
    let result=ReadFile(file.as_raw_handle(),bytes.as_mut_ptr(),bytes.len() as u32,&mut count,&mut o);
    let ok=if result!=0{true}else if std::io::Error::last_os_error().raw_os_error()==Some(997){
     let remaining=deadline.saturating_duration_since(Instant::now()).as_millis().min(800) as u32;
     if WaitForSingleObject(event,remaining)!=0{CancelIoEx(file.as_raw_handle(),&mut o);GetOverlappedResult(file.as_raw_handle(),&mut o,&mut count,1);false}else{GetOverlappedResult(file.as_raw_handle(),&mut o,&mut count,0)!=0}
    }else{false};
    if !ok||count==0{break;}
    match append_packet(&mut packet,&bytes[..count as usize]){Ok(Some(text))=>{answer=Some(text);break;},Err(_)=>break,_=>{}}
    if Instant::now()>=deadline{break;}
   }
   CloseHandle(event);answer
  }
 }
 pub struct Collector {retry:Instant}
 impl Collector {
  pub fn new()->Self{Self{retry:Instant::now()-Duration::from_secs(31)}}
  pub fn sample(&mut self)->Reading{
   if let Some(raw)=pipe(){return decode(&raw,SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64);}
   if self.retry.elapsed()>Duration::from_secs(30){self.retry=Instant::now();let _=Command::new("sc.exe").args(["start","DeskBoxCpuSensor"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).creation_flags(0x08000000).spawn();}
   Reading{status:"enable_required".into(),..Default::default()}
  }
 }
 pub fn request_enable(installed:bool)->Result<(),String>{
  let helper=std::env::current_exe().map_err(|e|e.to_string())?.parent().ok_or("Missing program directory")?.join("sensor-tools/DeskBox.Sensor.exe");
  if !helper.is_file(){return Err("缺少 CPU 采集组件，请使用完整更新包或重新安装".into());}
  let path=helper.to_string_lossy().replace('\'',"''");
  let mode=if installed{"--install"}else{"--portable"};
  let code=format!("$ErrorActionPreference='Stop'; if ((Get-FileHash -Algorithm SHA256 -LiteralPath '{path}').Hash -ne '{}') {{ throw 'CPU 采集组件校验失败' }}; $p=Start-Process -FilePath '{path}' -ArgumentList '{mode}', '{}' -Verb RunAs -PassThru{}",include_str!("../sensor-helper/helper-sha256.txt").trim(),include_str!("../sensor-helper/helper-sha256.txt").trim(),if installed{" -Wait; if($p.ExitCode -ne 0){throw '温度服务启用失败，请检查组件或管理员授权'}"}else{"; Start-Sleep -Seconds 2; if($p.HasExited -and $p.ExitCode -ne 0){throw '温度采集启用失败'}"});
  let out=Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-Command",&code]).creation_flags(0x08000000).output().map_err(|e|e.to_string())?;
  if out.status.success(){Ok(())}else{Err("CPU 温度采集未启用：授权被取消或组件启动失败，请重试".into())}
 }
}
#[cfg(all(windows,feature="gui"))]pub use native::{Collector,request_enable};
