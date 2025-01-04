use crate::log;

use rocket::fs::Options;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_include_dir::{Dir, StaticFiles};
use include_dir::include_dir;
use std::io::{Error, ErrorKind};
use serde::{Serialize, Deserialize};

use crate::templates;
use crate::module::{get_all_modules, get_module_by_name};

#[derive(Serialize)]
struct StatusData {
    pub module_name: String,
    pub data: String
}

#[derive(Deserialize)]
struct DropData {
    pub data: String
}

#[get("/")]
fn home_page() -> templates::HomePage {
    let modules = get_all_modules();
    templates::HomePage { modules: modules.iter().map(|mdl| mdl.into()).collect() }
}

#[get("/log")]
fn log_page() -> templates::LogPage {
    templates::LogPage { log_lines: log::log_to_vec() }
}

#[get("/module/<module_name>")]
fn module_page(module_name: String) -> Result<templates::ModulePage, (Status, Error)> {
    if let Some(module) = get_module_by_name(module_name) {
        let content = if let Some(page) = module.page {
            let content = page.lock().unwrap();
            content.clone()
        } else {
            "No page".to_string()
        };
        Ok(templates::ModulePage {
            module_name: module.name,
            module_description: module.description,
            module_author: module.author,
            module_tags: module.tags.split(",").map(|s| s.to_string()).collect(),
            module_link: module.link,
            content: content
        })
    } else {
        Err((
            Status::NotFound,
            Error::new(ErrorKind::Other, "no such module")
        ))
    }
}

#[get("/log")]
fn api_log() -> Json<Vec<(usize, String, String)>> {
    let vec = log::log_to_vec();
    Json(vec)
}

#[get("/modules")]
fn api_modules() -> Json<Vec<templates::ModuleInfo>> {
    let modules = get_all_modules();
    Json(modules.iter().map(|mdl| mdl.into()).collect())
}

#[get("/modules/<module_name>")]
fn api_module(module_name: String) -> Result<Json<templates::ModuleInfo>, (Status, Error)> {
    if let Some(module) = get_module_by_name(module_name) {
        Ok(Json(module.into()))
    } else {
        Err((
            Status::NotFound,
            Error::new(ErrorKind::Other, "no such module")
        ))
    }
}

#[get("/modules/<module_name>/data")]
fn api_module_status_data(module_name: String) -> Result<Json<StatusData>, (Status, Error)> {
    if let Some(module) = get_module_by_name(module_name) {
        if let Some(status_data) = module.status_data {
            Ok(Json(StatusData {
                module_name: module.name,
                data: status_data.lock().unwrap().clone()
            }))
        } else {
            Err((
                Status::NotFound,
                Error::new(ErrorKind::Other, "no status data")
            ))
        }
    } else {
        Err((
            Status::NotFound,
            Error::new(ErrorKind::Other, "no such module")
        ))
    }
}

#[post("/modules/<module_name>/data", data = "<data>")]
fn api_module_drop_data(module_name: String, data: Json<DropData>) -> Result<String, (Status, Error)> {
    if let Some(module) = get_module_by_name(module_name) {
        module.set_drop_data(data.into_inner().data);
        Ok("{}".to_string())
    } else {
        Err((
            Status::NotFound,
            Error::new(ErrorKind::Other, "no such module")
        ))
    }
}

pub async fn launch() {
    static STATIC_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 9797)))
        .mount("/", routes![home_page, log_page, module_page])
        .mount("/api/v1", routes![api_log, api_modules, api_module, api_module_status_data])
        .mount("/static", StaticFiles::new(&STATIC_DIR, Options::default()))
        .launch().await
        .expect("Could not launch rocket");
}
