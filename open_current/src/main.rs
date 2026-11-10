use std::env;
use std::process::Command;

fn main() {
    // 获取当前工作目录
    let dir = match env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("无法获取当前路径: {e}");
            std::process::exit(1);
        }
    };

    // Windows 下用 explorer 打开该目录
    match Command::new("explorer").arg(&dir).spawn() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("打开失败: {e}");
            std::process::exit(1);
        }
    }
}
