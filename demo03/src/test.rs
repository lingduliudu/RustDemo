use crate::global_cache::GLOBAL_MAP;

pub fn add_into() {
    GLOBAL_MAP.lock().unwrap().insert("key1".to_string(), 42);
}
