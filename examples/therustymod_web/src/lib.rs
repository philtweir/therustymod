#![allow(non_snake_case)]
#[macro_use] extern crate rocket;

use std::os::raw::{c_char, c_int};
use std::ffi::{CStr, CString};

mod http;
mod log;
mod templates;
mod module;

use module::{WebModule, get_module, register_module};

use therustymod_gen::{therustymod_lib};

static MODULE_NAME: &'static str = "mod_web_browser";

#[therustymod_lib(daemon=true)]
mod mod_web_browser {
    use crate::http::launch;

    async fn __run() {
        print!("Launching rocket...\n");
        launch().await
    }

    fn init_mod_web_browser() -> bool {
        log::add_to_log("init".to_string(), MODULE_NAME.to_string()).is_ok()
    }

    fn register_module(name: *const c_char, author: *const c_char, tags: *const c_char, link: *const c_char, description: *const c_char) -> c_int {
        let name = unsafe { CStr::from_ptr(name) }.to_string_lossy().clone();
        let author = unsafe { CStr::from_ptr(author) }.to_string_lossy().clone();
        let tags = unsafe { CStr::from_ptr(tags) }.to_string_lossy().clone();
        let link = unsafe { CStr::from_ptr(link) }.to_string_lossy().clone();
        let description = unsafe { CStr::from_ptr(description) }.to_string_lossy().clone();
        let module_num = register_module(
            name.clone().into(),
            author.into(),
            tags.into(),
            link.into(),
            description.into()
        ).try_into().unwrap();
        print!("Registering {} in mod_web_browser Rust-side as {}\n", name, module_num);
        module_num
    }

    fn register_page(module_num: c_int, page: *const c_char) {
        print!("Setting page for webmodule {}\n", module_num);
        let module: WebModule = get_module(module_num as usize);
        let page = unsafe { CStr::from_ptr(page) }.to_string_lossy().clone().to_string();
        module.set_page(page);
    }

    fn retrieve_drop(module_num: c_int) -> *const c_char {
        let module: WebModule = get_module(module_num as usize);
        if let Some(drop_data) = module.drop_data {
            CString::new(drop_data.lock().unwrap().clone()).unwrap().into_raw()
        } else {
            std::ptr::null_mut()
        }
    }

    fn update_status(module_num: usize, status_data: *const c_char) {
        let module: WebModule = get_module(module_num);
        let status_data = unsafe { CStr::from_ptr(status_data) }.to_string_lossy().clone().to_string();
        module.set_status_data(status_data);
    }

    fn do_log_to_web_browser(module_num: usize, log_line: *const c_char) -> bool {
        let module: WebModule = get_module(module_num);
        let log_line = unsafe { CStr::from_ptr(log_line) }.to_string_lossy().clone().to_string();
        log::add_to_log(log_line, module.name).is_ok()
    }
}
