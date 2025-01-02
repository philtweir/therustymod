use tokio::task::JoinHandle;

use std::sync::atomic::AtomicU16;
use lazy_static::lazy_static;
use dsll::DoublySortedLinkedList;
use std::sync::{Arc, Mutex};

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

pub fn add_to_log(string: String) -> () {
    let total = LIST_LEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let line_str = Arc::new(Mutex::new(string));
    let log_line = LogLine { ix: total, name: line_str };
    LIST.insert(log_line)
}

pub fn log_to_vec() -> Vec<String> {
    let mut current_node = LIST.head.clone();
    let len = LIST_LEN.load(std::sync::atomic::Ordering::Relaxed);
    let mut vec: Vec<String> = Vec::with_capacity(len.into()); // approx
    loop {
        let guarded_current_node = current_node.lock().unwrap();

        if guarded_current_node.is_none() {
            break;
        }

        if !guarded_current_node.as_ref().unwrap().is_helper {
            let value = &guarded_current_node.as_ref().unwrap().value;
            vec.push(value.name.lock().unwrap().to_string());
        }

        let next_node = guarded_current_node.as_ref().unwrap().next.clone();

        drop(guarded_current_node);

        current_node = next_node;
    }
    vec
}