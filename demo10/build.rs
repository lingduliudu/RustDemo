use std::env;
use std::fs;
use std::path::Path;
fn main() {
    let profile = env::var("PROFILE").unwrap();
    let pre_path = if profile == "debug" {
        "target/debug/"
    } else {
        "target/release/"
    };
    fs::copy("icon.ico", Path::new(&pre_path).join("icon.ico")).unwrap();
    set_icon();
}

/**************************************************************
* Description: 设置图标
* Author: yuanhao
* Versions: V1
**************************************************************/
fn set_icon() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        // 编译资源
        match res.compile() {
            Ok(_) => {}
            Err(_e) => {}
        }
    }
}
