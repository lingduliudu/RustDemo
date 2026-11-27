// Release 下用 Windows 子系统: 双击 exe 只弹 GUI, 不弹黑框控制台
// Debug 下保持 console, 方便看 println / panic
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};
use windows::Win32::UI::Input::KeyboardAndMouse::*;

const INTERVAL_MS: u64 = 200;

fn vk_of(c: char) -> u16 {
    match c {
        'a' | 'A' => 0x41,
        's' | 'S' => 0x53,
        'd' | 'D' => 0x44,
        'f' | 'F' => 0x46,
        _ => 0,
    }
}

fn is_physically_down(c: char) -> bool {
    unsafe { GetAsyncKeyState(vk_of(c) as i32) as u16 & 0x8000 != 0 }
}

/// 合成一次点击: DNF 版的三个关键, 记事本版缺了这三个所以在 DNF 里卡顿:
/// 1) 只发扫描码: wVk=0 + KEYEVENTF_SCANCODE, DNF 走 DirectInput, 只看扫描码,
///    之前同时填 wVk + wScan 但没置 SCANCODE 标志, wScan 根本没生效, 等于只发了 VK。
/// 2) 按下和松开分两次 SendInput, 中间按住 15ms: 之前是按下+松开塞进同一个
///    SendInput 数组、同一时间戳、0ms 间隔, DNF 按帧采样, 0ms 的脉冲很容易被漏采,
///    表现为时有时无、卡顿。别的能用的连点器基本都是分开发 + 留 10~30ms 按住时间。
/// 3) 注入标记: SendInput 发的本来就会带 INJECTED 标记, DNF 能收到但会按注入处理,
///    这是改不了的 (驱动级才能去掉标记, 那就是另一回事了)。能做到的是让它像真按键一样:
///    扫描码 + 有按住时长 + 200ms 间隔, 而不是 0ms 的假脉冲。
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
        // 按下: 单独一次 SendInput
        let mut down = [mk(false)];
        let _ = SendInput(&mut down, std::mem::size_of::<INPUT>() as i32);
        // 按住 15ms: 给 DNF 一帧以上的时间采到“按下”状态, 0ms 会被漏采
        thread::sleep(Duration::from_millis(15));
        // 松开: 再单独一次 SendInput
        let mut up = [mk(true)];
        let _ = SendInput(&mut up, std::mem::size_of::<INPUT>() as i32);
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = App::new()?;
    let keys = Arc::new(Mutex::new(vec!['a', 's', 'd']));
    let running = Arc::new(AtomicBool::new(false));

    let keys2 = keys.clone();
    ui.on_update_key(move |idx, text| {
        let mut g = keys2.lock().unwrap();
        if let Some(k) = g.get_mut(idx as usize) {
            if let Some(c) = text.chars().next() {
                let c = c.to_ascii_lowercase();
                if matches!(c, 'a' | 's' | 'd' | 'f') {
                    *k = c;
                }
            }
        }
    });

    let running2 = running.clone();
    let keys3 = keys.clone();
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
        ui.set_status("监听中: 按住 a/s/d/f 即连点".into());

        let r = running2.clone();
        let k = keys3.clone();
        thread::spawn(move || {
            let mut last_click: HashMap<char, Instant> = HashMap::new();
            let mut last_physical_seen: HashMap<char, Instant> = HashMap::new();
            let mut prev_held: HashMap<char, bool> = HashMap::new();
            while r.load(Ordering::SeqCst) {
                let snapshot = k.lock().unwrap().clone();
                for key in snapshot {
                    let now = Instant::now();
                    let phys = is_physically_down(key);
                    if phys {
                        last_physical_seen.insert(key, now);
                    }

                    // 按住判定 + 桥接:
                    // 合成点击的松开会短暂把 GetAsyncKeyState 冲成 0, 要等键盘自动重连点 (~500ms) 才会重新变 1。
                    // 所以不能一看到 0 就当松手, 要看“上次真实按住”是不是超过 600ms 没再出现。
                    // 真松手: 600ms 内再无真实按住 -> 停, 最多多点 2~3 下, 可接受。
                    // 还按住: 自动重连点会在 500ms 内把状态重新顶成 1, 不会断。
                    let held = if phys {
                        true
                    } else if prev_held.get(&key).copied().unwrap_or(false) {
                        match last_physical_seen.get(&key) {
                            Some(t) => now.duration_since(*t).as_millis() < 600,
                            None => false,
                        }
                    } else {
                        false
                    };

                    if !held {
                        prev_held.insert(key, false);
                        last_click.remove(&key);
                        continue;
                    }

                    let first = !prev_held.get(&key).copied().unwrap_or(false);
                    let due = last_click
                        .get(&key)
                        .map(|t| now.duration_since(*t).as_millis() >= INTERVAL_MS as u128)
                        .unwrap_or(true);
                    if first || due {
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
