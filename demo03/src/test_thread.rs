use std::{thread::sleep, time::Duration};
use tot_macro::to_async;

#[to_async]
fn async_fu(a: i32) -> i32 {
    sleep(Duration::from_secs(2));
    println!("async_fu {}", a);
    32
}

pub fn sync_fu() {
    async_fu(32);
    println!("sync_fu");
}
