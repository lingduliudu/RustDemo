use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("用法: {} <输入文件> <输出文件>", args[0]);
        return;
    }
    let input_path = &args[1];
    let output_path = &args[2];
    // 读取文件
    let content = match fs::read_to_string(input_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("读取文件失败: {}", e);
            return;
        }
    };
    let converted = zhconv::zhconv(&content, "zh-Hans".parse().unwrap());
    // 写入输出文件
    if let Err(e) = fs::write(output_path, converted) {
        eprintln!("写入文件失败: {}", e);
        return;
    }
    println!("转换完成，输出文件: {}", output_path);
}
