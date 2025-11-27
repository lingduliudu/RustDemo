pub struct Countor {
    now: i32,
}

impl Countor {
    pub fn new() -> Self {
        Self { now: 0 }
    }
    pub fn test(&self) {
        println!("默认:{}", self.now);
    }
}

impl Iterator for Countor {
    type Item = i32;
    fn next(&mut self) -> Option<Self::Item> {
        self.now += 1;
        if self.now < 6 {
            return Some(self.now);
        }
        return None;
    }
}
