use std::{cell::RefCell, rc::Rc};

pub fn test01() {
    let y = Rc::new(RefCell::new(1));
    let z = Rc::clone(&y);
    test02(z);
    println!("{}", y.borrow());
}

fn test02(x: Rc<RefCell<i32>>) {
    *x.borrow_mut() += 10;
    println!("进入内部方法 但是依旧可以使用 {}", x.borrow());
}

#[derive(Debug)]
struct Person {
    age: i32,
    stus: Option<Box<Person>>,
}

pub fn test03() {
    let x = Person {
        age: 1,
        stus: Some(Box::new(Person { age: 2, stus: None })),
    };
    println!("x = {:?}", x.age);
    println!("x = {:?}", x.stus.unwrap().age);
}
/**************************************************************
* Description: 测试共享引用计数器
* Author: yuanhao
* Versions: V1
**************************************************************/
use std::sync::Arc;
use std::thread;

pub fn test04() {
    for i in 1..10 {
        let ac = Arc::new(Box::new(String::from("test")));
        thread::spawn(move || {
            println!("数据是:{}", ac);
        });
    }
}
