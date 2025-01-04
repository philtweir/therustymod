use lazy_static::lazy_static;
use std::vec::Vec;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct WebModule {
    pub ix: usize,
    pub name: String,
    pub author: String,
    pub tags: String,
    pub link: String,
    pub description: String,
    pub page: Option<Arc<Mutex<String>>>,
    pub status_data: Option<Arc<Mutex<String>>>,
    pub drop_data: Option<Arc<Mutex<String>>>,
}

lazy_static! {
    pub static ref MODULE_REGISTER: Arc<Mutex<Vec<WebModule>>> = Arc::new(Mutex::new(Vec::new()));
}

impl WebModule {
    pub fn set_status_data(&self, status_data: String) {
        let mut register = MODULE_REGISTER.lock().unwrap();
        let module = &mut register[self.ix];
        module.status_data = Some(Arc::new(Mutex::new(status_data)))
    }

    pub fn set_drop_data(&self, drop_data: String) {
        let mut register = MODULE_REGISTER.lock().unwrap();
        let module = &mut register[self.ix];
        module.drop_data = Some(Arc::new(Mutex::new(drop_data)))
    }

    pub fn set_page(&self, page: String) {
        let mut register = MODULE_REGISTER.lock().unwrap();
        let module = &mut register[self.ix];
        module.page = Some(Arc::new(Mutex::new(page)))
    }
}

pub fn register_module(name: String, author: String, tags: String, link: String, description: String) -> usize {
    let mut register = MODULE_REGISTER.lock().unwrap();
    let ix = register.len().try_into().unwrap();
    let module = WebModule { ix, name, author, tags, link, description, page: None, status_data: None, drop_data: None };
    register.push(module);
    ix
}

pub fn get_module(ix: usize) -> WebModule {
    let register = MODULE_REGISTER.lock().unwrap();
    let module = register[ix].clone();
    module
}

pub fn get_module_by_name(name: String) -> Option<WebModule> {
    let register = MODULE_REGISTER.lock().unwrap();
    if let Some(module) = register.iter().filter(|mdl| mdl.name == name).next() {
        Some(module.clone())
    } else {
        None
    }
}

pub fn get_all_modules() -> Vec<WebModule> {
    let register = MODULE_REGISTER.lock().unwrap();
    register.iter().map(|mdl| mdl.clone()).collect()
}
