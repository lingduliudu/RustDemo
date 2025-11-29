use std::os::windows::process::CommandExt;
use std::process::Command;
fn main() {
    let args_to_pass: Vec<String> = std::env::args().skip(1).collect();
    let mut command = Command::new("handler.exe");
    command.creation_flags(0x08000000);
    command.args(args_to_pass).spawn().unwrap();
}
