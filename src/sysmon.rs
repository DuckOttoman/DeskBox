use libloading::Library;
use serde::Serialize;
use std::{ffi::{c_char, c_void, CStr}, path::PathBuf};

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    pub cpu_name: String,
    pub cpu_usage: Option<f64>,
    pub cpu_mhz: u32,
    pub cpu_temp: Option<f64>,
    pub cpu_core_min: Option<f64>,
    pub cpu_core_max: Option<f64>,
    pub cpu_package_temp: Option<f64>,
    pub cpu_thermal_zone: Option<f64>,
    pub cpu_temp_status: String,
    pub mem_used: u64,
    pub mem_total: u64,
    pub gpu_name: String,
    pub gpu_usage: Option<u32>,
    pub gpu_mhz: Option<u32>,
    pub gpu_temp: Option<u32>,
    pub vram_used: Option<u64>,
    pub vram_total: Option<u64>,
}
#[repr(C)] #[derive(Default)] struct FileTime { low: u32, high: u32 }
impl FileTime { fn value(&self) -> u64 { (self.high as u64) << 32 | self.low as u64 } }
#[repr(C)] #[derive(Default)] struct MemoryStatus {
    length: u32, load: u32, total_phys: u64, avail_phys: u64, total_page: u64,
    avail_page: u64, total_virtual: u64, avail_virtual: u64, avail_extended: u64,
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetSystemTimes(idle: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime) -> i32;
    fn GlobalMemoryStatusEx(status: *mut MemoryStatus) -> i32;
}
#[repr(C)] #[derive(Default)] struct Utilization { gpu: u32, memory: u32 }
#[repr(C)] #[derive(Default)] struct Memory { total: u64, free: u64, used: u64 }

struct Nvml { library: Library, device: *mut c_void }
impl Nvml {
    fn open() -> Option<Self> {
        // Load only installed NVIDIA locations, not the current directory.
        let paths = [PathBuf::from(std::env::var_os("SystemRoot")?).join("System32/nvml.dll"),
            PathBuf::from(std::env::var_os("ProgramFiles")?).join("NVIDIA Corporation/NVSMI/nvml.dll")];
        for path in paths {
            unsafe {
                let Ok(library) = Library::new(path) else { continue; };
                let Ok(init) = library.get::<unsafe extern "C" fn() -> i32>(b"nvmlInit_v2") else { continue; };
                if init() != 0 { continue; }
                let mut device = std::ptr::null_mut();
                let Ok(handle) = library.get::<unsafe extern "C" fn(u32, *mut *mut c_void) -> i32>(b"nvmlDeviceGetHandleByIndex_v2") else { continue; };
                if handle(0, &mut device) == 0 { return Some(Self { library, device }); }
            }
        }
        None
    }
    fn sample(&self, sample: &mut Sample) {
        unsafe {
            let mut name = [0 as c_char; 128];
            if let Ok(f) = self.library.get::<unsafe extern "C" fn(*mut c_void, *mut c_char, u32) -> i32>(b"nvmlDeviceGetName") {
                if f(self.device, name.as_mut_ptr(), name.len() as u32) == 0 {
                    sample.gpu_name = CStr::from_ptr(name.as_ptr()).to_string_lossy().into_owned();
                }
            }
            let mut util = Utilization::default();
            if let Ok(f) = self.library.get::<unsafe extern "C" fn(*mut c_void, *mut Utilization) -> i32>(b"nvmlDeviceGetUtilizationRates") {
                if f(self.device, &mut util) == 0 { sample.gpu_usage = Some(util.gpu); }
            }
            let mut mem = Memory::default();
            if let Ok(f) = self.library.get::<unsafe extern "C" fn(*mut c_void, *mut Memory) -> i32>(b"nvmlDeviceGetMemoryInfo") {
                if f(self.device, &mut mem) == 0 { sample.vram_used = Some(mem.used); sample.vram_total = Some(mem.total); }
            }
            let mut number = 0;
            if let Ok(f) = self.library.get::<unsafe extern "C" fn(*mut c_void, u32, *mut u32) -> i32>(b"nvmlDeviceGetTemperature") {
                if f(self.device, 0, &mut number) == 0 { sample.gpu_temp = Some(number); }
            }
            if let Ok(f) = self.library.get::<unsafe extern "C" fn(*mut c_void, u32, *mut u32) -> i32>(b"nvmlDeviceGetClockInfo") {
                if f(self.device, 0, &mut number) == 0 { sample.gpu_mhz = Some(number); }
            }
        }
    }
}
impl Drop for Nvml {
    fn drop(&mut self) {
        unsafe { if let Ok(f) = self.library.get::<unsafe extern "C" fn() -> i32>(b"nvmlShutdown") { f(); } }
    }
}

pub struct Sampler { core: crate::core_temperature::Collector, previous: Option<(u64, u64)>, nvml: Option<Nvml>, name: String, mhz: u32, thermal: Option<crate::thermal::Thermal> }
impl Sampler {
    pub fn new() -> Self {
        let cpu = windows_registry::LOCAL_MACHINE.open(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0").ok();
        Self { core: crate::core_temperature::Collector::new(), previous: None, nvml: Nvml::open(), thermal: crate::thermal::Thermal::open(),
            name: cpu.as_ref().and_then(|k| k.get_string("ProcessorNameString").ok()).unwrap_or_else(|| "CPU".into()),
            mhz: cpu.and_then(|k| k.get_u32("~MHz").ok()).unwrap_or(0) }
    }
    pub fn sample(&mut self) -> Sample {
        let mut sample = Sample { cpu_name: self.name.clone(), cpu_mhz: self.mhz, ..Default::default() };
        let mut idle = FileTime::default(); let mut kernel = FileTime::default(); let mut user = FileTime::default();
        unsafe {
            if GetSystemTimes(&mut idle, &mut kernel, &mut user) != 0 {
                let total = kernel.value() + user.value();
                if let Some((old_idle, old_total)) = self.previous {
                    let delta = total.saturating_sub(old_total);
                    if delta > 0 { sample.cpu_usage = Some((100. * (1. - idle.value().saturating_sub(old_idle) as f64 / delta as f64)).clamp(0., 100.)); }
                }
                self.previous = Some((idle.value(), total));
            }
            let mut mem = MemoryStatus { length: std::mem::size_of::<MemoryStatus>() as u32, ..Default::default() };
            if GlobalMemoryStatusEx(&mut mem) != 0 { sample.mem_total = mem.total_phys; sample.mem_used = mem.total_phys - mem.avail_phys; }
        }
        sample.cpu_thermal_zone = self.thermal.as_ref().and_then(|t|t.sample());
        let core=self.core.sample();sample.cpu_core_min=core.min;sample.cpu_core_max=core.max;
        sample.cpu_package_temp=core.package;sample.cpu_temp_status=core.status;sample.cpu_temp=core.max;
        if let Some(nvml) = &self.nvml { nvml.sample(&mut sample); }
        sample
    }
}
