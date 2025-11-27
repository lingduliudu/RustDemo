use crate::global_cache::GLOBAL_MAP;

pub fn add_into() {
    GLOBAL_MAP.lock().unwrap().insert("key2".to_string(), 42);
}
