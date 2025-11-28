use lazy_static::lazy_static;
use std::sync::Mutex;
lazy_static! {
    pub static ref X: Mutex<String> = Mutex::new(String::from(""));
}
pub static mut PORT: u16 = 10000;
