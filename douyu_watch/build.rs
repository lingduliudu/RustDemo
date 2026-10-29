use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=watch_room.txt");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let profile = env::var("PROFILE").unwrap();

    let target_dir = manifest_dir.join("target").join(&profile);

    let source = manifest_dir.join("watch_room.txt");
    let destination = target_dir.join("watch_room.txt");

    if !source.exists() {
        panic!("找不到配置文件: {}", source.display());
    }

    if let Err(err) = fs::create_dir_all(&target_dir) {
        panic!("创建目标目录失败: {}", err);
    }

    if let Err(err) = fs::copy(&source, &destination) {
        panic!("复制 watch_room.txt 失败: {}", err);
    }

    println!("已复制 watch_room.txt 到: {}", destination.display());
}
