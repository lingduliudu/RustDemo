use std::collections::*;

#[derive(Debug)]
struct Member {
    age: i32,
}

pub fn test01() {
    let mut contacts = HashMap::new();
    contacts.insert("Daniel", Member { age: 12 });
    contacts.insert("Daniel2", Member { age: 13 });
    for i in &contacts {
        println!("{:?}", i);
    }
    match &contacts.get("Daniel2") {
        Some(v) => {
            println!("获取数据{}", v.age);
            println!("获取数据{:?}", v);
        }
        _ => {
            println!("None");
        }
    }
    for i in &contacts {
        println!("{:?}", i);
    }
}
