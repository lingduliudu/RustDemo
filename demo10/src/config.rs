use crate::server::*;
use crate::ws as tws;
use actix::prelude::*;
use actix_web::{HttpRequest, HttpResponse, web};
use actix_web_actors::ws;
use serde::Deserialize;
#[derive(Deserialize, Debug)]
pub struct UserIdPath {
    pub id: u32,
}

/**************************************************************
* Description: webscoket路由
* Author: yuanhao
* Versions: V1
**************************************************************/
pub async fn ws_route(
   // path: web::Path<UserIdPath>,
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<ChatServer>>,
) -> Result<HttpResponse, actix_web::Error> {
    let x = 10;
    ws::start(
        tws::WsSession::new(x as usize, srv.get_ref().clone()),
        &req,
        stream,
    )
}
