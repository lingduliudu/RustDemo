use actix_web::{App, HttpServer};
use log::info;
mod api;
use api::*;
use log4rs;

fn init_log() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_log();
    info!("启动成功");
    HttpServer::new(|| {
        App::new()
            .service(users)
            .service(get_user)
            .service(list)
            .service(create)
            .service(login)
            .service(header)
            .service(raw)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
