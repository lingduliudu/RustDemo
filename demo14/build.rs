use std::env;
use std::path::PathBuf;
fn main() {
    println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let rc_path = PathBuf::from(&manifest_dir).join("app.rc");
    embed_resource::compile("app.rc", std::iter::empty::<&str>());
}
