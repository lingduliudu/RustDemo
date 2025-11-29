//use std::process::Command;
mod global_cache;
mod test;
mod test_thread;
use std::{thread::sleep, time::Duration};

use test::add_into;
mod test2;
use test2::add_into as add_into2;

use crate::{global_cache::GLOBAL_MAP, test_trait::LockExt};
mod test_iter;
mod test_trait;
#[link(name = "tot_utils", kind = "static")]
unsafe extern "C" {
    pub fn add(left: u64, right: u64) -> u64;
}

fn test(f1: fn(i32) -> i32) {
    f1(45);
}
fn prtest(x: i32) -> i32 {
    println!("Hello, world! {}", x);
    32
}

/**************************************************************
* Description: 测试生命周期
* Author: yuanhao
* Versions: V1
**************************************************************/
fn comp_a_b<'a>(a: &'a str, b: &'a str) -> &'a str {
    //if a.len() > b.len() { a } else { b }
    if a.len() > b.len() {
        return a;
    }
    return b;
}
fn main() {
    test(prtest);
    test(|x| {
        println!("abc {}", x);
        45
    });
    // 测试生命周期
    let x = comp_a_b("baccdd", "abcd");
    println!("第一次{}", x);
    println!("第二次{}", x);
    let y = unsafe { add(12, 2332) };
    println!("结果是 {}", y);
    //Command::new("cmd").args(["/C", "pause"]).status().unwrap();

    // 测试全局缓存
    add_into();
    add_into2();
    let c = test_iter::Countor::new();
    c.test();
    for i in c {
        println!("{}", i);
    }

    // if let Some(value) = global_cache::GLOBAL_MAP.lock().unwrap().get("key1") {
    //     println!("Value: {}", value);
    // }

    println!("---------------------------");
    println!("{}", unsafe { global_cache::PI });
    global_cache::change_pi();
    println!("{}", unsafe { global_cache::PI });

    test_thread::sync_fu();
    sleep(Duration::from_secs(4));
    test_global_1();
    test_global_2();
}

fn test_global_1() {
    let mut x = GLOBAL_MAP.with_lock(|a| a.clone()).unwrap();
    x.insert(String::from("name"), 1);
    println!("---{:?}\n", x);
}
fn test_global_2() {
    let result_of_with_lock: Result<std::collections::HashMap<String, i32>, _> = GLOBAL_MAP
        .with_lock(|a| {
            // let mut y = a.clone();
            // y.insert(String::from("zhagn"), 22);
            return a.clone(); // 隐式返回 y
        });
    let mut x: std::collections::HashMap<String, i32> = result_of_with_lock.unwrap();
    x.insert(String::from("name"), 1);
    // 打印 x
    println!("---{:?}\n", x);
}
