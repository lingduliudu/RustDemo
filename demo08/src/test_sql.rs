pub fn init_test_data(conn: &sqlite::Connection) {
    let query = "
        CREATE TABLE users (name TEXT, age INTEGER);
        INSERT INTO users VALUES ('Alice', 42);
        INSERT INTO users VALUES ('Bob', 69);
    ";
    conn.execute(query).unwrap();
}
