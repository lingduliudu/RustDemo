use std::fs;
fn main() {
    fs::copy("tot_utils.dll", "target/debug/tot_utils.dll").unwrap();
    // 1. 告诉 Cargo 在哪里可以找到 mylib.lib
    // "search" 告诉链接器要搜索的路径
    println!("cargo:rustc-link-search=native=libs");

    // 2. 告诉 Cargo 要链接哪个库
    // "dylib" 表示动态链接库，使用 mylib.dll 对应的 mylib.lib 文件
    // "mylib" 是库的名称（不带 lib 前缀和 .lib 后缀）
    println!("cargo:rustc-link-lib=dylib=tot_utils");
}
