use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use log::{error, info};
mod api;
use api::*;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
mod server;
use actix::prelude::*;
use server::ChatServer;
mod config;
mod ws;
use config::ws_route;
use tot_macro::to_async;
mod file_watch;
mod global_cache;
/**************************************************************
* Description: 初始化日志
* Author: yuanhao
* Versions: V1
**************************************************************/
fn init_log() {
    //let encoder = PatternEncoder::new("{d(%H:%M:%S)} - {l} - {t} - {m}\n");
    let encoder = PatternEncoder::new("{d(%H:%M:%S)} - {l} - {m}\n");
    let stdout_appender = ConsoleAppender::builder()
        // 将编码器 Box 起来
        .encoder(Box::new(encoder))
        .target(log4rs::append::console::Target::Stdout)
        .build();

    // 3. 构建配置对象 Config
    let config = Config::builder()
        // 添加 Appender，取名为 "stdout"
        .appender(Appender::builder().build("stdout", Box::new(stdout_appender)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();
    log4rs::init_config(config).unwrap();
}

/**************************************************************
* Description: 主方法
* Author: yuanhao
* Versions: V1
**************************************************************/
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_log();
    let args: Vec<String> = std::env::args().collect();
    let mut port: u16 = 10000;
    if args.len() < 2 {
        // 直接退出
        info!("参数错误: 至少需要文件名");
        std::process::exit(0);
    }
    if args.len() == 3 {
        port = args[2].parse::<u16>().unwrap();
    }

    let current_dir = std::env::current_dir().unwrap();

    let filename_to_watch = &args[1];
    // 3. 构建完整的监听路径
    let full_path_to_watch = current_dir.join(filename_to_watch);
    if !full_path_to_watch.exists() {
        eprintln!(
            "错误: 文件不存在或路径无效 -> {}",
            full_path_to_watch.display()
        );
        std::process::exit(1);
    }
    let server = ChatServer::new().start();
    let final_path = full_path_to_watch.clone();
    file_watch::run_file_watcher(full_path_to_watch, server.clone());
    {
        let mut data = global_cache::X.lock().unwrap();
        data.push_str(final_path.to_str().unwrap());
    }
    unsafe {
        global_cache::PORT = port;
    }
    open_browner(port);
    HttpServer::new(move || {
        // 允许跨域
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(cors)
            //.wrap(TokenCheck)
            .app_data(web::Data::new(server.clone()))
            .route("/ws/{id}", web::get().to(ws_route))
            .service(index)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

#[to_async]
fn open_browner(port: u16) {
    // 您想要浏览器打开的 URL
    let url = format!("http://localhost:{}", port);
    match opener::open(url) {
        Ok(()) => {}
        Err(_e) => {}
    }
}
