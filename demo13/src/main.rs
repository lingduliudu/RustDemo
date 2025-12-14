struct Person;
impl Person {
    fn new() -> Self {
        Person {}
    }
}

fn test(p: &Person) {}
fn main() {
    println!("testprint");
    let p1 = Person::new();
    test(&p1);
}
