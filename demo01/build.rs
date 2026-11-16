use std::fs;
fn main() {
    fs::copy("log4rs.yaml", "target/debug/log4rs.yaml").unwrap();
}


