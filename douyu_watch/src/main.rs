#![windows_subsystem = "windows"]

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs,
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, SystemTime},
};

use reqwest::Client;
use serde::Deserialize;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

// ============================================================
// 配置
// ============================================================

const CHECK_INTERVAL: Duration = Duration::from_secs(10);

const CONFIG_FILE: &str = "watch_room.txt";

const DOUYU_URL: &str = "https://www.douyu.com/betard/";

// ============================================================
// 斗鱼 API 数据结构
// ============================================================

#[derive(Debug, Deserialize)]
struct DouyuResponse {
    room: Option<DouyuRoom>,
}

#[derive(Debug, Deserialize)]
struct DouyuRoom {
    #[serde(default)]
    show_status: i32,

    #[serde(rename = "videoLoop", default)]
    video_loop: i32,

    #[serde(default)]
    nickname: String,

    #[serde(default)]
    room_name: String,
}

// ============================================================
// 房间状态
// ============================================================

#[derive(Debug, Clone)]
struct RoomStatus {
    room_id: String,
    nickname: String,
    room_name: String,
    is_live: bool,
}

// ============================================================
// 后台监控事件
// ============================================================

enum MonitorEvent {
    CheckNow,
    ReloadConfig,
    Quit,
}

// ============================================================
// 托盘事件
// ============================================================

enum UserEvent {
    Menu(MenuEvent),
}

// ============================================================
// 获取 EXE 所在目录
// ============================================================

fn get_exe_dir() -> Result<PathBuf, Box<dyn Error>> {
    let exe = std::env::current_exe()?;

    let dir = exe.parent().ok_or("无法获取 EXE 所在目录")?;

    Ok(dir.to_path_buf())
}

// ============================================================
// 获取配置文件路径
//
// 最终读取：
// xxx/target/debug/watch_room.txt
//
// 或：
// xxx/target/release/watch_room.txt
// ============================================================

fn get_config_path() -> Result<PathBuf, Box<dyn Error>> {
    Ok(get_exe_dir()?.join(CONFIG_FILE))
}

// ============================================================
// 加载房间号
//
// 支持：
//
// 793400
// 24000
//
// 或：
//
// 793400,24000
//
// 或：
//
// 793400
// 24000
// ============================================================

fn load_room_ids() -> Result<Vec<String>, Box<dyn Error>> {
    let path = get_config_path()?;

    if !path.exists() {
        return Err(format!("找不到配置文件：{}", path.display()).into());
    }

    let content = fs::read_to_string(&path)?;

    let mut rooms = Vec::new();
    let mut seen = HashSet::new();

    for id in content.split(|c: char| c == ',' || c == '\n' || c == '\r' || c == ' ' || c == '\t') {
        let id = id.trim();

        if id.is_empty() {
            continue;
        }

        // 只允许数字房间号
        if !id.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        if seen.insert(id.to_string()) {
            rooms.push(id.to_string());
        }
    }

    if rooms.is_empty() {
        return Err("watch_room.txt 中没有有效的斗鱼房间号".into());
    }

    Ok(rooms)
}

// ============================================================
// 获取配置文件修改时间
// ============================================================

fn config_modified_time() -> Option<SystemTime> {
    let path = get_config_path().ok()?;

    fs::metadata(path).ok()?.modified().ok()
}

// ============================================================
// 查询斗鱼房间
// ============================================================

async fn fetch_room_status(
    client: &Client,
    room_id: &str,
) -> Result<RoomStatus, Box<dyn Error + Send + Sync>> {
    let url = format!("{}{}", DOUYU_URL, room_id);

    let response = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
             AppleWebKit/537.36 \
             Chrome/138.0.0.0 Safari/537.36",
        )
        .header("Referer", format!("https://www.douyu.com/{}", room_id))
        .timeout(Duration::from_secs(8))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("HTTP 状态码：{}", response.status()).into());
    }

    let data: DouyuResponse = response.json().await?;

    let room = data.room.ok_or("斗鱼 API 没有返回 room")?;

    /*
     * show_status == 1
     *      表示正在直播
     *
     * videoLoop == 0
     *      排除某些特殊视频/循环状态
     */

    let is_live = room.show_status == 1 && room.video_loop == 0;

    Ok(RoomStatus {
        room_id: room_id.to_string(),
        nickname: room.nickname,
        room_name: room.room_name,
        is_live,
    })
}

// ============================================================
// Windows 通知
// ============================================================

fn notify_live(status: &RoomStatus) -> Result<(), Box<dyn Error>> {
    use winrt_notification::{Duration as ToastDuration, Toast};

    let title = if status.nickname.is_empty() {
        format!("斗鱼 {} 开播了", status.room_id)
    } else {
        format!("{} 开播了", status.nickname)
    };

    let body = if status.room_name.is_empty() {
        format!("房间号：{}", status.room_id)
    } else {
        format!("{}\n房间号：{}", status.room_name, status.room_id)
    };

    Toast::new(Toast::POWERSHELL_APP_ID)
        .title(&title)
        .text1(&body)
        .duration(ToastDuration::Short)
        .show()?;

    Ok(())
}

// ============================================================
// 创建 HTTP Client
// ============================================================

fn create_http_client() -> Result<Client, Box<dyn Error>> {
    Ok(Client::builder().timeout(Duration::from_secs(10)).build()?)
}

// ============================================================
// 检查所有房间
// ============================================================

async fn check_all_rooms(
    client: &Client,
    room_ids: &[String],
    previous: &mut HashMap<String, bool>,
) {
    for room_id in room_ids {
        match fetch_room_status(client, room_id).await {
            Ok(status) => {
                let old_live = previous.get(room_id).copied().unwrap_or(false);

                /*
                 * 只有：
                 *
                 * 未直播 -> 直播
                 *
                 * 才发送通知。
                 *
                 * 因此同一场直播不会重复通知。
                 */

                if !old_live && status.is_live {
                    let _ = notify_live(&status);
                }

                previous.insert(room_id.clone(), status.is_live);
            }

            Err(_) => {
                /*
                 * 网络错误时不要把状态强制设置成 false。
                 *
                 * 否则：
                 *
                 * 直播中
                 *     ↓
                 * 网络错误
                 *     ↓
                 * 被认为未直播
                 *     ↓
                 * 网络恢复
                 *     ↓
                 * 又通知一次
                 *
                 * 所以网络错误时保持旧状态。
                 */
            }
        }
    }
}

// ============================================================
// 重新加载配置
// ============================================================

fn reload_config(
    current_rooms: &mut Vec<String>,
    previous: &mut HashMap<String, bool>,
) -> Result<(), Box<dyn Error>> {
    let new_rooms = load_room_ids()?;

    /*
     * 删除已经不再监控的房间状态
     */
    previous.retain(|room_id, _| new_rooms.contains(room_id));

    *current_rooms = new_rooms;

    Ok(())
}

// ============================================================
// 后台监控线程
// ============================================================

fn start_monitor(rx: mpsc::Receiver<MonitorEvent>) {
    thread::spawn(move || {
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => return,
        };

        runtime.block_on(async move {
            let client = match create_http_client() {
                Ok(client) => client,
                Err(_) => return,
            };

            let mut room_ids = match load_room_ids() {
                Ok(rooms) => rooms,
                Err(_) => return,
            };

            let mut previous_status: HashMap<String, bool> = HashMap::new();

            let mut config_time = config_modified_time();

            /*
             * 程序启动立即检查一次
             */
            check_all_rooms(&client, &room_ids, &mut previous_status).await;

            loop {
                match rx.recv_timeout(CHECK_INTERVAL) {
                    /*
                     * 托盘：
                     * 立即检查
                     */
                    Ok(MonitorEvent::CheckNow) => {
                        check_all_rooms(&client, &room_ids, &mut previous_status).await;
                    }

                    /*
                     * 托盘：
                     * 重新加载配置
                     */
                    Ok(MonitorEvent::ReloadConfig) => {
                        if reload_config(&mut room_ids, &mut previous_status).is_ok() {
                            config_time = config_modified_time();

                            check_all_rooms(&client, &room_ids, &mut previous_status).await;
                        }
                    }

                    /*
                     * 托盘：
                     * 退出
                     */
                    Ok(MonitorEvent::Quit) => {
                        break;
                    }

                    /*
                     * 10 秒定时检查
                     */
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        /*
                         * 检查配置文件是否被修改
                         */
                        let new_time = config_modified_time();

                        if new_time != config_time {
                            if reload_config(&mut room_ids, &mut previous_status).is_ok() {
                                config_time = new_time;
                            }
                        }

                        check_all_rooms(&client, &room_ids, &mut previous_status).await;
                    }

                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }
        });
    });
}

// ============================================================
// 创建托盘图标
// ============================================================

fn create_tray_icon(menu: Menu) -> Result<TrayIcon, Box<dyn Error>> {
    let width: u32 = 16;
    let height: u32 = 16;

    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let dx = x as i32 - 8;
            let dy = y as i32 - 8;

            if dx * dx + dy * dy <= 49 {
                // 蓝色圆形
                rgba.push(30);
                rgba.push(144);
                rgba.push(255);
                rgba.push(255);
            } else {
                // 透明
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
            }
        }
    }

    let icon = Icon::from_rgba(rgba, width, height)?;

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("斗鱼直播监控")
        .with_icon(icon)
        .build()?;

    Ok(tray)
}

// ============================================================
// Winit Application
// ============================================================

struct App {
    tray: Option<TrayIcon>,

    tx: mpsc::Sender<MonitorEvent>,

    check_id: tray_icon::menu::MenuId,

    reload_id: tray_icon::menu::MenuId,

    quit_id: tray_icon::menu::MenuId,

    menu: Option<Menu>,
}

// ============================================================
// ApplicationHandler
// ============================================================

impl ApplicationHandler<UserEvent> for App {
    // --------------------------------------------------------
    // Winit 启动
    // --------------------------------------------------------

    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        /*
         * Windows 下必须在 event loop 已经运行的线程
         * 创建 TrayIcon。
         *
         * tray-icon 官方文档对此有明确要求。
         */

        if self.tray.is_some() {
            return;
        }

        if let Some(menu) = self.menu.take() {
            match create_tray_icon(menu) {
                Ok(tray) => {
                    self.tray = Some(tray);
                }

                Err(_) => {
                    /*
                     * 托盘创建失败，
                     * 直接退出程序。
                     */
                    std::process::exit(1);
                }
            }
        }
    }

    // --------------------------------------------------------
    // 菜单事件
    // --------------------------------------------------------

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Menu(event) => {
                let id = event.id();

                if id == &self.check_id {
                    let _ = self.tx.send(MonitorEvent::CheckNow);
                } else if id == &self.reload_id {
                    let _ = self.tx.send(MonitorEvent::ReloadConfig);
                } else if id == &self.quit_id {
                    let _ = self.tx.send(MonitorEvent::Quit);

                    /*
                     * 关闭 Winit event loop。
                     *
                     * TrayIcon 会随 App 销毁而退出。
                     */
                    event_loop.exit();
                }
            }
        }
    }

    // --------------------------------------------------------
    // 没有窗口，我们不处理 WindowEvent
    // --------------------------------------------------------

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
    }

    // --------------------------------------------------------
    // Event Loop 退出
    // --------------------------------------------------------

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        let _ = self.tx.send(MonitorEvent::Quit);

        /*
         * 显式释放 TrayIcon。
         */
        self.tray.take();
    }
}

// ============================================================
// main
// ============================================================

fn main() -> Result<(), Box<dyn Error>> {
    /*
     * ========================================================
     * 创建菜单
     * ========================================================
     */

    let menu = Menu::new();

    let status_item = MenuItem::new("斗鱼监控运行中", false, None);

    let check_item = MenuItem::new("立即检查", true, None);

    let reload_item = MenuItem::new("重新加载配置", true, None);

    let quit_item = MenuItem::new("退出", true, None);

    menu.append(&status_item)?;

    menu.append(&PredefinedMenuItem::separator())?;

    menu.append(&check_item)?;
    menu.append(&reload_item)?;

    menu.append(&PredefinedMenuItem::separator())?;

    menu.append(&quit_item)?;

    /*
     * ========================================================
     * 保存 MenuId
     *
     * 注意：
     * 不把 MenuItem 本身捕获进闭包。
     *
     * 这样可以避免之前的 E0277。
     * ========================================================
     */

    let check_id = check_item.id().clone();
    let reload_id = reload_item.id().clone();
    let quit_id = quit_item.id().clone();

    /*
     * ========================================================
     * 后台监控线程通信
     * ========================================================
     */

    let (tx, rx) = mpsc::channel::<MonitorEvent>();

    start_monitor(rx);

    /*
     * ========================================================
     * 创建 Winit EventLoop
     *
     * 使用 user event，让 tray-icon 可以唤醒
     * Winit EventLoop。
     * ========================================================
     */

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;

    /*
     * ========================================================
     * 创建 EventLoopProxy
     * ========================================================
     */

    let proxy = event_loop.create_proxy();

    /*
     * ========================================================
     * MenuEvent -> Winit UserEvent
     *
     * tray-icon 官方推荐的方式。
     *
     * 注意：
     * 这里闭包只捕获 EventLoopProxy。
     *
     * 不捕获 MenuItem。
     *
     * 因此不会出现之前的 E0277。
     * ========================================================
     */

    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let _ = proxy.send_event(UserEvent::Menu(event));
    }));

    /*
     * ========================================================
     * 创建 App
     * ========================================================
     */

    let mut app = App {
        tray: None,

        tx,

        check_id,

        reload_id,

        quit_id,

        menu: Some(menu),
    };

    /*
     * ========================================================
     * 启动 Winit
     *
     * 注意这里使用 run_app：
     *
     * 不再使用已经 deprecated 的：
     *
     * event_loop.run(...)
     * ========================================================
     */

    event_loop.run_app(&mut app)?;

    Ok(())
}
