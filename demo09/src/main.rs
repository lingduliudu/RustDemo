use std::{thread::sleep, time::Duration};

use log::*;
use tot_macro::{to_async, totlog};
fn init_log() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
}

#[totlog]
fn test() -> i32 {
    32
}
#[totlog]
fn test2(x: i32) -> i32 {
    x
}

#[to_async]
fn test_async() {
    sleep(Duration::from_secs(2));
    info!("结束1");
}

#[to_async]
fn test_async2() -> i32 {
    sleep(Duration::from_secs(3));
    info!("结束2");
    23
}

fn test_sync() {
    info!("先结束");
}

fn main() {
    init_log();
    test_async();
    test_async2();
    test_sync();
    test();
    test2(23);
    info!("test");
    sleep(Duration::from_secs(6));
}
