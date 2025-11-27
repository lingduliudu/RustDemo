use crate::server::*;
use crate::ws as tws;
use actix::prelude::*;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use tot_macro::totlog;
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: u32,
    pub name: String,
}
#[derive(Deserialize, Debug)]
pub struct QueryParams {
    pub page: Option<u32>,
    pub size: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct UserPath {
    pub id: u32,
}

#[derive(Deserialize, Debug)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[get("/users")]
#[totlog("获取用户")]
pub async fn users() -> impl Responder {
    web::Json(vec![
        User {
            id: 1,
            name: "Alice".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
        },
    ])
}

#[get("/users/{id}")]
#[totlog]
pub async fn get_user(path: web::Path<UserPath>) -> impl Responder {
    HttpResponse::Ok().body(format!("User ID = {}", path.id))
}

#[get("/list")]
#[totlog]
pub async fn list(query: web::Query<QueryParams>) -> impl Responder {
    format!("page = {:?}, size = {:?}", query.page, query.size)
}

#[post("/create")]
#[totlog]
pub async fn create(body: web::Json<User>) -> impl Responder {
    format!("User = {} ({})", body.name, body.id)
}

#[post("/login")]
#[totlog]
pub async fn login(form: web::Form<LoginForm>) -> String {
    format!("username={}, password={}", form.username, form.password)
}

#[get("/header")]
#[totlog]
pub async fn header(req: HttpRequest) -> String {
    let key = req.headers().get("X-Token");
    format!("token = {:?}", key)
}

#[post("/raw")]
#[totlog]
pub async fn raw(body: web::Bytes) -> String {
    let s = String::from_utf8_lossy(&body);
    format!("raw body = {}", s)
}

/// 接收 HTTP 请求并主动广播到所有 WebSocket 会话
#[post("/broadcast")]
pub async fn broadcast_http(body: String, server: web::Data<Addr<ChatServer>>) -> HttpResponse {
    server.do_send(crate::server::BroadcastMessage {
        msg: format!("(API) {}", body),
    });
    HttpResponse::Ok().body("sent")
}

/// 接收 HTTP 请求并主动广播到所有 WebSocket 会话
#[post("/sendToClient/{id}")]
pub async fn send_to_client(
    path: web::Path<UserPath>,
    body: String,
    server: web::Data<Addr<ChatServer>>,
) -> HttpResponse {
    let idu: usize = path.id as usize;
    server.do_send(crate::server::ClientMessage {
        id: idu,
        msg: format!("(API) {}", body),
    });
    HttpResponse::Ok().body("sent")
}

pub async fn ws_route(
    path: web::Path<UserPath>,
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<ChatServer>>,
) -> Result<HttpResponse, actix_web::Error> {
    let x = path.id;
    ws::start(
        tws::WsSession::new(x as usize, srv.get_ref().clone()),
        &req,
        stream,
    )
}
