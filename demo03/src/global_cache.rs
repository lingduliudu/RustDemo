use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub static GLOBAL_MAP: LazyLock<Mutex<HashMap<String, i32>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
