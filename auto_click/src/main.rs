// Release 下用 Windows 子系统: 双击 exe 只弹 GUI, 不弹黑框控制台
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();
use std::collections::HashMap;
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
const INTERVAL_MS: u64 = 100;
fn vk_of(c: char) -> u16 {
    match c {
        'a' | 'A' => 0x41,
        's' | 'S' => 0x53,
        'd' | 'D' => 0x44,
        'f' | 'F' => 0x46,
        'x' | 'X' => 0x58,
        'z' | 'Z' => 0x5A,
        _ => 0,
    }
}
struct HookState {
    keys: Arc<Mutex<Vec<Option<char>>>>,
    held: Arc<Mutex<HashMap<char, bool>>>,
}
static HOOK_STATE: OnceLock<Arc<HookState>> = OnceLock::new();
// 只观察不拦截的钩子: 永远走 CallNextHookEx, 从不 return 1, 不吞按键
unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let injected = kb.flags.0 & LLKHF_INJECTED.0 != 0;
        if !injected {
            if let Some(c) = match kb.vkCode {
                0x41 => Some('a'),
                0x53 => Some('s'),
                0x44 => Some('d'),
                0x46 => Some('f'),
                0x5A => Some('z'),
                0x58 => Some('x'),
                _ => None,
            } {
                if let Some(state) = HOOK_STATE.get() {
                    if state.keys.lock().unwrap().contains(&Some(c)) {
                        let mut held = state.held.lock().unwrap();
                        match wparam.0 as u32 {
                            WM_KEYDOWN | WM_SYSKEYDOWN => {
                                held.insert(c, true);
                            }
                            WM_KEYUP | WM_SYSKEYUP => {
                                held.insert(c, false);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}
// 合成一次点击: 只发扫描码 + 按下/松开分开发 + 按住 15ms, DNF 才认
fn send_click(c: char) {
    unsafe {
        let vk = vk_of(c);
        let scan = MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) as u16;
        let mk = |up: bool| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: scan,
                    dwFlags: if up {
                        KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP
                    } else {
                        KEYEVENTF_SCANCODE
                    },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let mut down = [mk(false)];
        let _ = SendInput(&mut down, std::mem::size_of::<INPUT>() as i32);
        thread::sleep(Duration::from_millis(15));
        let mut up = [mk(true)];
        let _ = SendInput(&mut up, std::mem::size_of::<INPUT>() as i32);
    }
}
fn main() -> Result<(), slint::PlatformError> {
    let ui = App::new()?;
    let keys = Arc::new(Mutex::new(vec![
        Some('a'),
        Some('s'),
        Some('d'),
    ]));
    let held: Arc<Mutex<HashMap<char, bool>>> = Arc::new(Mutex::new(HashMap::new()));
    let running = Arc::new(AtomicBool::new(false));
    HOOK_STATE
        .set(Arc::new(HookState {
            keys: keys.clone(),
            held: held.clone(),
        }))
        .ok();
    thread::spawn(|| unsafe {
        if let Ok(hook) = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0) {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            let _ = UnhookWindowsHookEx(hook);
        }
    });
    let keys2 = keys.clone();
    ui.on_update_key(move |idx, text| {
        let mut g = keys2.lock().unwrap();
        if let Some(k) = g.get_mut(idx as usize) {
            let normalized = text.trim().to_ascii_lowercase();
            *k = match normalized.as_str() {
                "a" | "s" | "d" | "z" | "x" => normalized.chars().next(),
                _ => None,
            };
        }
    });
    let running2 = running.clone();
    let keys3 = keys.clone();
    let held2 = held.clone();
    let ui_weak = ui.as_weak();
    ui.on_toggle_running(move || {
        let Some(ui) = ui_weak.upgrade() else { return };
        if running2.load(Ordering::SeqCst) {
            running2.store(false, Ordering::SeqCst);
            ui.set_running(false);
            ui.set_status("已停止".into());
            return;
        }
        running2.store(true, Ordering::SeqCst);
        ui.set_running(true);
        ui.set_status("监听中: 按住输入框中的按键即连点".into());
        let r = running2.clone();
        let k = keys3.clone();
        let h = held2.clone();
        thread::spawn(move || {
            let mut last_click: HashMap<char, Instant> = HashMap::new();
            let mut prev_held: HashMap<char, bool> = HashMap::new();
            while r.load(Ordering::SeqCst) {
                let snapshot = k.lock().unwrap().clone();
                for key in snapshot.into_iter().flatten() {
                    // 抬起判定: 直接读钩子记的真实状态, 没有 600ms 桥接, 抬起后不再多点
                    let is_held = h.lock().unwrap().get(&key).copied().unwrap_or(false);
                    if !is_held {
                        prev_held.insert(key, false);
                        last_click.remove(&key);
                        continue;
                    }
                    let now = Instant::now();
                    let first = !prev_held.get(&key).copied().unwrap_or(false);
                    let due = last_click
                        .get(&key)
                        .map(|t| now.duration_since(*t).as_millis() >= INTERVAL_MS as u128)
                        .unwrap_or(true);
                    if first || due {
                        // 发之前再验一次: 万一你正好在这 2ms 里抬起了, 就不发这一下
                        if !h.lock().unwrap().get(&key).copied().unwrap_or(false) {
                            prev_held.insert(key, false);
                            continue;
                        }
                        send_click(key);
                        last_click.insert(key, now);
                    }
                    prev_held.insert(key, true);
                }
                thread::sleep(Duration::from_millis(2));
            }
        });
    });
    ui.run()
}
