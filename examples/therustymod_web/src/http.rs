use tokio::task::JoinHandle;

use std::sync::atomic::AtomicU16;
use lazy_static::lazy_static;
use dsll::DoublySortedLinkedList;
use std::sync::{Arc, Mutex};

use crate::log;

#[derive(Clone, Debug, Default)]
pub struct LogLine {
    pub ix: u16,
    pub name: Arc<Mutex<String>>,
}

impl Ord for LogLine {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.ix.cmp(&(other.ix)) {
            std::cmp::Ordering::Equal => (*self.name.lock().unwrap()).cmp(&(*other.name.lock().unwrap())),
            o => o
        }
    }
}

impl PartialOrd for LogLine {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for LogLine {
    fn eq(&self, other: &Self) -> bool {
        self.ix == other.ix && *self.name.lock().unwrap() == *other.name.lock().unwrap()
    }
}

impl Eq for LogLine { }

lazy_static! {
    pub static ref LIST: DoublySortedLinkedList<LogLine> = DoublySortedLinkedList::new();
    pub static ref LIST_LEN: AtomicU16 = AtomicU16::new(0);
}

#[get("/")]
fn hello() -> String {
    let vec = log::log_to_vec();
    vec.join(",")
}

pub async fn launch() {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 9797)))
        .mount("/", routes![hello])
        .launch().await
        .expect("Could not launch rocket");
}
