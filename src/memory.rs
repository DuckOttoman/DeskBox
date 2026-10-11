//! Manual trim of this application's working set; never system-wide memory purge.
use std::collections::BTreeMap;
#[derive(Clone)]pub struct Process{pub id:u32,pub parent:u32,pub name:String,pub born:u64}
pub fn owned_descendants(list:&[Process],root:u32)->Vec<u32>{
 let Some(p)=list.iter().find(|p|p.id==root)else{return vec![]};
 let mut owned=BTreeMap::from([(root,p.born)]);
 loop {let old=owned.len();for p in list {if p.name.eq_ignore_ascii_case("msedgewebview2.exe")&&owned.get(&p.parent).is_some_and(|born|p.born>=*born){owned.insert(p.id,p.born);}}if old==owned.len(){break;}}
 owned.into_keys().collect()
}
#[cfg(all(windows,feature="gui"))]mod native {
 use super::*;use std::{ffi::c_void,mem::size_of,sync::Mutex,time::{Duration,Instant}};
 type Handle=*mut c_void;
 #[repr(C)]struct Entry {size:u32,usage:u32,id:u32,heap:usize,module:u32,threads:u32,parent:u32,priority:i32,flags:u32,name:[u16;260]}
 #[repr(C)]#[derive(Default)]struct FileTime{low:u32,high:u32}
 #[repr(C)]#[derive(Default)]struct Counters{size:u32,faults:u32,peak:usize,working:usize,paged_peak:usize,paged:usize,nonpaged_peak:usize,nonpaged:usize,pagefile:usize,pagefile_peak:usize}
 #[link(name="kernel32")]unsafe extern "system" {
 fn CreateToolhelp32Snapshot(flags:u32,id:u32)->Handle;fn Process32FirstW(h:Handle,e:*mut Entry)->i32;fn Process32NextW(h:Handle,e:*mut Entry)->i32;
 fn OpenProcess(access:u32,inherit:i32,id:u32)->Handle;fn CloseHandle(h:Handle)->i32;fn GetProcessTimes(h:Handle,c:*mut FileTime,e:*mut FileTime,k:*mut FileTime,u:*mut FileTime)->i32;
 fn K32GetProcessMemoryInfo(h:Handle,c:*mut Counters,size:u32)->i32;fn K32EmptyWorkingSet(h:Handle)->i32;
 }
 struct Owned(Handle);impl Drop for Owned{fn drop(&mut self){unsafe{CloseHandle(self.0);}}}
 fn born(h:Handle)->Option<u64>{let(mut c,mut e,mut k,mut u)=(FileTime::default(),FileTime::default(),FileTime::default(),FileTime::default());if unsafe{GetProcessTimes(h,&mut c,&mut e,&mut k,&mut u)}==0{None}else{Some((c.high as u64)<<32|c.low as u64)}}
 fn working(h:Handle)->Option<u64>{let mut c=Counters::default();c.size=size_of::<Counters>() as u32;if unsafe{K32GetProcessMemoryInfo(h,&mut c,c.size)}==0{None}else{Some(c.working as u64)}}
 #[derive(serde::Serialize)]#[serde(rename_all="camelCase")]pub struct Report{pub before_bytes:u64,pub after_bytes:u64,pub trimmed_processes:u32,pub skipped_processes:u32}
 static LAST:Mutex<Option<Instant>>=Mutex::new(None);
 pub fn optimize()->Result<Report,String>{
 let mut lock=LAST.try_lock().map_err(|_|"正在优化，请稍候")?;if lock.is_some_and(|t|t.elapsed()<Duration::from_secs(10)){return Err("请等待10秒后再优化".into());}*lock=Some(Instant::now());
 let root=std::process::id();let snapshot=unsafe{CreateToolhelp32Snapshot(2,0)};if snapshot as isize == -1{return Err(std::io::Error::last_os_error().to_string());}let snapshot=Owned(snapshot);
 let mut entry:Entry=unsafe{std::mem::zeroed()};entry.size=size_of::<Entry>() as u32;let mut list=vec![];let mut live=vec![];
 let mut ok=unsafe{Process32FirstW(snapshot.0,&mut entry)};
 while ok!=0 {let n=entry.name.iter().position(|c|*c==0).unwrap_or(260);let name=String::from_utf16_lossy(&entry.name[..n]);if entry.id==root||name.eq_ignore_ascii_case("msedgewebview2.exe") {let h=unsafe{OpenProcess(0x1000,0,entry.id)};if !h.is_null(){let h=Owned(h);if let Some(time)=born(h.0){list.push(Process{id:entry.id,parent:entry.parent,name,born:time});live.push(h);}}}ok=unsafe{Process32NextW(snapshot.0,&mut entry)};}
 let ids=owned_descendants(&list,root);let mut report=Report{before_bytes:0,after_bytes:0,trimmed_processes:0,skipped_processes:0};
 for id in ids {let h=unsafe{OpenProcess(0x1100,0,id)};if h.is_null(){report.skipped_processes+=1;continue;}let h=Owned(h);if born(h.0)!=list.iter().find(|p|p.id==id).map(|p|p.born){report.skipped_processes+=1;continue;}
 let Some(before)=working(h.0)else{report.skipped_processes+=1;continue;};if unsafe{K32EmptyWorkingSet(h.0)}==0{report.skipped_processes+=1;continue;}
 report.before_bytes+=before;report.after_bytes+=working(h.0).unwrap_or(before);report.trimmed_processes+=1;
 }if report.trimmed_processes==0{return Err("暂未能回收应用工作集，请稍后再试".into());}Ok(report)
 }
}
#[cfg(all(windows,feature="gui"))]pub use native::optimize;
