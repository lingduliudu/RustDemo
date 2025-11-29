use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
lazy_static! {
    // 定义一个静态的 Mutex，内部包裹着 HashMap
   pub static  ref  GLOBAL_MAP: Mutex<HashMap<String, i32>> = {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), 10);
        map.insert("key2".to_string(), 20);
        Mutex::new(map)
    };
}

pub static mut PI: f32 = 3.14;

pub fn change_pi() {
    unsafe {
        PI = 23.0;
    }
}
