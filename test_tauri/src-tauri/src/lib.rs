// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            use tauri::{Listener, Manager};
            // 获取主窗口实例
            let main_window = app.get_webview_window("main").unwrap();
            // 监听前端发出的 'vue-ready' 事件
            let cloned_window = main_window.clone();
            main_window.once("vue-ready", move |_| {
                // 当收到事件后，在主线程上执行显示操作
                tauri::async_runtime::spawn(async move {
                    cloned_window.show().unwrap();
                });
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
