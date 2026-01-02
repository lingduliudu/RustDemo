use crate::htmlparse;
use crate::markdownparse;
use actix_web::{HttpResponse, Responder, get};
use crate::global_cache::{X};
/**************************************************************
* Description: 获取form信息
* Author: yuanhao
* Versions: V1
**************************************************************/
#[get("/")]
pub async fn index() -> impl Responder {
    let data_guard = X.lock().unwrap();

    let file_content = std::fs::read_to_string((*data_guard).to_string()).unwrap();
    let mut response_html = String::new();
    if is_html((*data_guard).as_str()){
        response_html = htmlparse::get_after(file_content.clone());
    }
    if is_markdown((*data_guard).as_str()){
        response_html = markdownparse::get_after(file_content.clone());
    }
    HttpResponse::Ok()
        // 关键：设置 Content-Type 为 text/html
        .content_type("text/html; charset=utf-8")
        // 设置响应体
        .body(format!(
            "{}",
            response_html
        ))
}


fn is_html(path: &str) -> bool {
    match std::path::Path::new(path).extension().unwrap().to_str().unwrap(){
        "html" => return true,
        _      => return false,
    }
}

fn is_markdown(path: &str) -> bool {
    match std::path::Path::new(path).extension().unwrap().to_str().unwrap() {
        "md" => return true,
        _      => return false,
    }
}
