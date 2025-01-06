use askama::Template;
use serde::Serialize;

use crate::module::WebModule;

#[derive(Serialize)]
pub struct ModuleInfo {
    pub name: String,
    pub author: String,
    pub tags: String,
    pub link: String,
    pub description: String,
    pub has_status_data: bool,
    pub has_page: bool,
}

impl From<WebModule> for ModuleInfo {
    fn from(module: WebModule) -> Self {
        ModuleInfo {
            name: module.name,
            author: module.author,
            tags: module.tags,
            link: module.link,
            description: module.description,
            has_status_data: module.status_data.is_some(),
            has_page: module.page.is_some()
        }
    }
}

// Not very efficient, but TODO
impl From<&WebModule> for ModuleInfo {
    fn from(module: &WebModule) -> Self {
        module.clone().into()
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct HomePage {
    pub modules: Vec<ModuleInfo>
}

impl HomePage {
    fn modules_having_page(&self) -> impl Iterator<Item = &ModuleInfo> {
        self.modules.iter().filter(|mdl| mdl.has_page)
    }
}

#[derive(Template)]
#[template(path = "log.html")]
pub struct LogPage {
    pub log_lines: Vec<(usize, String, String)>
}

#[derive(Template)]
#[template(path = "module.html", escape = "none")]
pub struct ModulePage {
    pub module_name: String,
    pub module_description: String,
    pub module_author: String,
    pub module_tags: Vec<String>,
    pub module_link: String,
    pub content: String,
}
