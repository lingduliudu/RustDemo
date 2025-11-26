use std::process::Command;
#[link(name = "tot_utils", kind = "dylib")]
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
    Command::new("cmd").args(["/C", "pause"]).status().unwrap();
}
