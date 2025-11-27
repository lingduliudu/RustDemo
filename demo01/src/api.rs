use crate::server::*;
use actix::prelude::*;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use askama::Template;
use serde::{Deserialize, Serialize};
use tot_macro::totlog;
/**************************************************************
* Description: 基本体
* Author: yuanhao
* Versions: V1
**************************************************************/
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

/**************************************************************
* Description: 获取json信息
* Author: yuanhao
* Versions: V1
**************************************************************/
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
/**************************************************************
* Description: 获取路径信息
* Author: yuanhao
* Versions: V1
**************************************************************/
#[get("/users/{id}")]
#[totlog]
pub async fn get_user(path: web::Path<UserPath>) -> impl Responder {
    HttpResponse::Ok().body(format!("User ID = {}", path.id))
}

/**************************************************************
* Description: 获取list数据
* Author: yuanhao
* Versions: V1
**************************************************************/
#[get("/list")]
#[totlog]
pub async fn list(query: web::Query<QueryParams>) -> impl Responder {
    format!("page = {:?}, size = {:?}", query.page, query.size)
}

/**************************************************************
* Description: 获取json体
* Author: yuanhao
* Versions: V1
**************************************************************/
#[post("/create")]
#[totlog]
pub async fn create(body: web::Json<User>) -> impl Responder {
    format!("User = {} ({})", body.name, body.id)
}

/**************************************************************
* Description: 获取form信息
* Author: yuanhao
* Versions: V1
**************************************************************/
#[post("/login")]
#[totlog]
pub async fn login(form: web::Form<LoginForm>) -> String {
    format!("username={}, password={}", form.username, form.password)
}

/**************************************************************
* Description: 获取http header
* Author: yuanhao
* Versions: V1
**************************************************************/
#[get("/header")]
#[totlog]
pub async fn header(req: HttpRequest) -> String {
    let key = req.headers().get("X-Token");
    format!("token = {:?}", key)
}

/**************************************************************
* Description: 测试http 原文
* Author: yuanhao
* Versions: V1
**************************************************************/
#[post("/raw")]
#[totlog]
pub async fn raw(body: web::Bytes) -> String {
    let s = String::from_utf8_lossy(&body);
    format!("raw body = {}", s)
}

/**************************************************************
* Description:  接收 HTTP 请求并主动广播到所有 WebSocket 会话
* Author: yuanhao
* Versions: V1
**************************************************************/
#[post("/broadcast")]
pub async fn broadcast_http(body: String, server: web::Data<Addr<ChatServer>>) -> HttpResponse {
    server.do_send(crate::server::BroadcastMessage {
        msg: format!("(API) {}", body),
    });
    HttpResponse::Ok().body("sent")
}

/**************************************************************
* Description: 模板渲染
* Author: yuanhao
* Versions: V1
**************************************************************/

#[derive(Template)]
#[template(path = "test.html")]
struct HelloTemplate {
    name: String,
}

#[get("/parseTemplate")]
pub async fn parse_template() -> impl Responder {
    let tpl = HelloTemplate {
        name: String::from("中国"),
    };
    match tpl.render() {
        Ok(body) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(body),
        Err(e) => HttpResponse::InternalServerError().body(format!("Render error: {}", e)),
    }
}

/**************************************************************
* Description: 直接发送消息给对应客户端
* Author: yuanhao
* Versions: V1
**************************************************************/
#[post("/sendToClient/{id}")]
pub async fn send_to_client(
    path: web::Path<UserPath>,
    body: String,
    server: web::Data<Addr<ChatServer>>,
) -> String {
    let idu: usize = path.id as usize;
    // 同步发送判断结果
    let result = server
        .send(crate::server::ClientMessage {
            id: idu,
            msg: format!("{}", body),
        })
        .await
        .unwrap();
    if result == 0 {
        return format!("对方已离线");
    }
    return format!("发送成功");
}
