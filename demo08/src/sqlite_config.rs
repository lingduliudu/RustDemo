pub struct Sqlite {
    pub conn: sqlite::Connection,
}

impl Sqlite {
    pub fn open() -> Self {
        // 内存级别
        // let connection = sqlite::open(":memory:").unwrap();
        // 文件
        let connection = sqlite::open("my.db").unwrap();
        Self { conn: connection }
    }
}
