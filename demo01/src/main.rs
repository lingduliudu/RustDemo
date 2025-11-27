use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use log::info;
mod api;
use api::*;
use log4rs;
mod auth;
mod server;
use actix::prelude::*;
use auth::TokenCheck;
use server::ChatServer;
mod ws;
/**************************************************************
* Description: 初始化日志
* Author: yuanhao
* Versions: V1
**************************************************************/
fn init_log() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
}

/**************************************************************
* Description: 主方法
* Author: yuanhao
* Versions: V1
**************************************************************/
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_log();
    info!("启动成功");
    let server = ChatServer::new().start();
    HttpServer::new(move || {
        // 允许跨域
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .wrap(TokenCheck)
            .app_data(web::Data::new(server.clone()))
            .route("/ws/{id}", web::get().to(api::ws_route))
            .service(users)
            .service(broadcast_http)
            .service(get_user)
            .service(send_to_client)
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
