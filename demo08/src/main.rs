mod sqlite_config;
use sqlite_config::Sqlite;

use crate::entitys::User;
mod entitys;
mod test_sql;
fn main() {
    let connection = Sqlite::open().conn;
    // 初始化数据
    //test_sql::init_test_data(&connection);
    let mut users = Vec::new();
    let mut statement = connection.prepare("SELECT * FROM users ").unwrap();
    while let Ok(sqlite::State::Row) = statement.next() {
        users.push(User {
            name: statement.read::<String, _>("name").unwrap(),
            age: statement.read::<i64, _>("age").unwrap() as i32,
        });
    }
    println!("{:?}", users);
}
