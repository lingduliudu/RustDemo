use std::fs;
use std::io::{BufRead, BufReader};
const P: &str = "D:/test/测试.txt";
pub fn test01() {
    let x = fs::read_to_string(P).unwrap();
    println!("{}", x);
}

pub fn test02() {
    let x = fs::File::open(P).unwrap();
    let y = BufReader::new(x);
    for line in y.lines() {
        let line = line.unwrap();
        println!("{}", line);
    }
}
