use std::fmt::format;

use crate::global_cache::{PORT, X};
use actix_web::{HttpResponse, Responder, get};
/**************************************************************
* Description: 获取form信息
* Author: yuanhao
* Versions: V1
**************************************************************/
#[get("/")]
pub async fn index() -> impl Responder {
    let data_guard = X.lock().unwrap();

    HttpResponse::Ok()
        // 关键：设置 Content-Type 为 text/html
        .content_type("text/html; charset=utf-8")
        // 设置响应体
        .body(format!(
            "{}{}",
            std::fs::read_to_string((*data_guard).to_string()).unwrap(),
            get_after()
        ))
}

fn get_after() -> String {
    let s1 = String::from(
        r#"
    <script>
        const WS_ADDRESS = "ws://127.0.0.1:"#,
    );
    let mut x: u16 = 0;
    unsafe {
        x = PORT;
    }
    let s2 = format!("{}", x);
    let s3 = String::from(
        r#"/ws/10";
            let socket;
            let reconnectAttempts = 0;
            function connectWebSocket(){
                socket = new WebSocket(WS_ADDRESS);
                socket.onopen = function(e) {
                };
                socket.onmessage = function(event) {
                    console.log(`[message] 收到数据: ${event.data}`);
                    window.location.reload();
                };
                socket.onclose = function(event) {
                };
                socket.onerror = function(error) {
                };
            }
            window.onload = connectWebSocket;
        </script>
"#,
    );
    format!("{}{}{}", s1, s2, s3)
}
