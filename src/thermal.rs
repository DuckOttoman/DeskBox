// Same Windows thermal-zone performance counter found in the original executable.
pub fn celsius(kelvin: f64) -> Option<f64> {
    let value = kelvin - 273.15;
    (value.is_finite() && (-20.0..=150.0).contains(&value)).then_some(value)
}
#[cfg(windows)]
mod native {
    use std::ffi::c_void;
    #[repr(C)] struct Value { status: u32, number: f64 }
    #[repr(C)] struct Item { name: *const u16, value: Value }
    #[link(name="pdh",kind="raw-dylib")]
    unsafe extern "system" {
        fn PdhOpenQueryW(source:*const u16,user:usize,query:*mut *mut c_void)->u32;
        fn PdhAddEnglishCounterW(query:*mut c_void,path:*const u16,user:usize,counter:*mut *mut c_void)->u32;
        fn PdhCollectQueryData(query:*mut c_void)->u32;
        fn PdhGetFormattedCounterArrayW(counter:*mut c_void,format:u32,size:*mut u32,count:*mut u32,items:*mut Item)->u32;
        fn PdhCloseQuery(query:*mut c_void)->u32;
    }
    pub struct Thermal { query:*mut c_void,counter:*mut c_void }
    // Handles are exclusively owned by the sampling thread after creation.
    unsafe impl Send for Thermal {}
    impl Thermal {
        pub fn open()->Option<Self>{unsafe{
            let mut query=std::ptr::null_mut();
            if PdhOpenQueryW(std::ptr::null(),0,&mut query)!=0{return None;}
            let path:Vec<u16>=r"\Thermal Zone Information(*)\Temperature".encode_utf16().chain(Some(0)).collect();
            let mut counter=std::ptr::null_mut();
            if PdhAddEnglishCounterW(query,path.as_ptr(),0,&mut counter)!=0{PdhCloseQuery(query);return None;}
            Some(Self{query,counter})
        }}
        pub fn sample(&self)->Option<f64>{unsafe{
            if PdhCollectQueryData(self.query)!=0{return None;}
            let(mut size,mut count)=(0u32,0u32);
            let status=PdhGetFormattedCounterArrayW(self.counter,0x200,&mut size,&mut count,std::ptr::null_mut());
            if status!=0x800007D2||size==0||size>1024*1024{return None;}
            // u64 backing storage provides the alignment required by Item and double.
            let mut storage=vec![0u64;(size as usize+7)/8];
            if PdhGetFormattedCounterArrayW(self.counter,0x200,&mut size,&mut count,storage.as_mut_ptr().cast())!=0{return None;}
            if count as usize>storage.len()*8/std::mem::size_of::<Item>(){return None;}
            let items=std::slice::from_raw_parts(storage.as_ptr().cast::<Item>(),count as usize);
            items.iter().filter(|i|matches!(i.value.status,0|1)).filter_map(|i|super::celsius(i.value.number)).reduce(f64::max)
        }}
    }
    impl Drop for Thermal {fn drop(&mut self){unsafe{PdhCloseQuery(self.query);}}}
}
#[cfg(windows)] pub use native::Thermal;
