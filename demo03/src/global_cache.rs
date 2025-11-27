use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub static GLOBAL_MAP: LazyLock<Mutex<HashMap<String, i32>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static mut PI: f32 = 3.14;

pub fn change_pi() {
    unsafe {
        PI = 23.0;
    }
}
