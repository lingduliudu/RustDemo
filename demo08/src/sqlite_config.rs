pub struct Sqlite {
    pub conn: sqlite::Connection,
}

impl Sqlite {
    pub fn open() -> Self {
        let connection = sqlite::open(":memory:").unwrap();
        Self { conn: connection }
    }
}
