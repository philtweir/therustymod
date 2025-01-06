use std::time::Duration;
use tokio::runtime::Runtime;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};
use std::os::raw::c_char;
use std::ffi::CString;

pub use vtable_rs::{vtable, VPtr};

use therustymod_tdm::LibraryABI::LibraryABI;

const ABI_VERSION: u32 = 1;
type Initializer = Box<dyn Fn() -> std::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>> + Send + Sync>;
type Future = std::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>>;

pub struct TRMModuleData {
    pub module_name: CString,
    pub run: Option<Initializer>
}

pub struct TRMSystem {
    rt: Option<Arc<Runtime>>,
    module_data: Option<Arc<Mutex<TRMModuleData>>>,
    pub abi: Option<LibraryABI>
}

fn initialize(module_data: Arc<Mutex<TRMModuleData>>) -> Future {
    let module_data = module_data.as_ref();
    let trm_module_data = module_data.lock().unwrap();
    let run = (*trm_module_data).run.as_ref();
    print!("Initializing...\n");
    run.unwrap()()
    // http::launch().await;
}


impl TRMSystem {
    fn run(&mut self) {
        if self.is_daemon() {
            let module_data = self.module_data.as_ref().unwrap().clone();
            let _rt = Runtime::new().unwrap();
            _rt.spawn(async {
                print!("Spawning a TRM daemon thread...\n");
                initialize(module_data).await
            });
            self.rt = Some(Arc::new(_rt));
        }
    }

    fn is_daemon(&self) -> bool {
        let module_data = self.module_data.as_ref();
        if let Some(module_data) = module_data {
            let trm_module_data = module_data.lock().unwrap();
            (*trm_module_data).run.is_some()
        } else {
            false
        }
    }

    fn shutdown(&mut self) {
        if self.is_daemon() {
            if let Some(rt) = self.rt.take() {
               Arc::into_inner(rt).expect("Could not grab daemon runtime").shutdown_timeout(Duration::from_millis(100));
            }
        }
    }

    pub fn _set_module_data_once(&mut self, module_data: Arc<Mutex<TRMModuleData>>) {
        print!("Setting module data\n");
        self.module_data = Some(module_data)
    }
}

lazy_static! {
    pub static ref TRM_SYSTEM: Mutex<TRMSystem> = {
        Mutex::new(TRMSystem { rt: None, module_data: None, abi: None })
    };
}

#[vtable]
pub trait TRMSysIdClassVmt {
    fn trm__test(&self) -> u32;
}

#[derive(Default)]
#[repr(C)]
struct TRMSysIdClass {
    vftable: VPtr<dyn TRMSysIdClassVmt, Self>,
}

impl TRMSysIdClassVmt for TRMSysIdClass {
    #[no_mangle]
    extern "C" fn trm__test(&self) -> u32 {
        ABI_VERSION
    }
}

#[no_mangle]
extern "C" fn trm__initialize(abi: LibraryABI) -> bool {
    let mut trm_system = TRM_SYSTEM.lock().unwrap();
    trm_system.abi = Some(abi);
    unsafe { (abi.confirmLoad.unwrap())() };
    trm_system.run();
    true
}

#[no_mangle]
extern "C" fn trm__deinitialize() -> bool {
    TRM_SYSTEM.lock().unwrap().shutdown();
    true
}

#[no_mangle]
extern "C" fn trm__abi_version() -> u32 {
    ABI_VERSION
}

#[no_mangle]
extern "C" fn trm__module_name() -> *const c_char {
    let trm_system = TRM_SYSTEM.lock().unwrap();
    let trm_module_data = trm_system.module_data.as_ref().expect("Module data missing").lock().expect("Could not lock module data");
    trm_module_data.module_name.as_ptr()
}
