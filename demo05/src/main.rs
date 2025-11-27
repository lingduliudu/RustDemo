use std::os::windows::process::CommandExt;
use std::process::Command;
fn main() {
    Command::new("demo01.exe")
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()
        .unwrap();
}
