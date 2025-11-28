use mysql::prelude::Queryable;

mod entitys;
mod mysql_config;

fn main() {
    let my_sql = mysql_config::Mysql::new();
    let mut conn = my_sql.get_conn().unwrap();
    let x = conn
        .query_map(
            "SELECT id,data_id from config_info limit 3 ",
            |(id, data_id)| entitys::ConfigInfo { id, data_id },
        )
        .unwrap();
    for v in x {
        println!("{:?}", v);
    }
}
