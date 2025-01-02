#![allow(non_snake_case)]
#[macro_use] extern crate rocket;

mod http;
mod log;

use therustymod_gen::{therustymod_lib};

#[therustymod_lib(daemon=true)]
mod mod_forward_lantern {
    use crate::http::launch;

    async fn __run() {
        print!("Launching rocket...\n");
        launch().await
    }

    fn init_mod_forward_lantern() {
        log::add_to_log("init".to_string())
    }

    fn do_stuff_mod_forward_lantern() {
        log::add_to_log("do stuff".to_string())
    }
}
