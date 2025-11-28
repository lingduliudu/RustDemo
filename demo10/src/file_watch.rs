use crate::server::*;
use actix::prelude::*;
use log::info;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::PathBuf, thread::sleep};
use tokio::time::Duration;
use tot_macro::to_async;
#[to_async]
pub fn run_file_watcher(path_to_watch: PathBuf, chat_server: Addr<ChatServer>) {
    info!(" 开始监听文件: {}", path_to_watch.display());
    loop {
        let final_chat_server = chat_server.clone();
        let watch_path_clone = path_to_watch.clone();
        // 1. 创建文件系统观察者 (Watcher)
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                match res {
                    Ok(event) => {
                        // 过滤掉不重要的事件，只关注文件内容变动
                        if let EventKind::Modify(_) = event.kind {
                            // 检查变动的文件路径是否与我们监听的文件匹配
                            if event.paths.iter().any(|p| p == watch_path_clone.as_path()) {
                                final_chat_server.do_send(ClientMessage {
                                    id: 10,
                                    msg: String::from("change"),
                                });
                            }
                        }
                    }
                    Err(e) => info!("观察者错误: {:?}", e),
                }
            },
            notify::Config::default().with_poll_interval(Duration::from_millis(500)),
        )
        .unwrap();

        // 2. 告诉观察者开始监听这个文件
        // RecursiveMode::NonRecursive 只监听文件本身
        watcher
            .watch(&path_to_watch, RecursiveMode::NonRecursive)
            .unwrap();
        sleep(Duration::from_millis(300));
    }
}
