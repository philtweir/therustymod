#![allow(non_snake_case)]
#[macro_use] extern crate rocket;

mod http;
mod log;

use therustymod_gen::{therustymod_lib};

#[therustymod_lib(daemon=true)]
mod mod_web_browser {
    use crate::http::launch;

    async fn __run() {
        print!("Launching rocket...\n");
        launch().await
    }

    fn init_mod_web_browser() {
        log::add_to_log("init".to_string())
    }

    fn do_log_to_web_browser() {
        log::add_to_log("doing stuff".to_string())
    }
}
