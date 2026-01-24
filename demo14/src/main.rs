#![windows_subsystem = "windows"]
#![allow(unused)]
use std::ptr::null_mut;
use windows::{
    core::w,
    Win32::{
        Foundation::*,
        System::LibraryLoader::*,
        UI::{Shell::*, WindowsAndMessaging::*},
    },
};

use std::time::{Duration, Instant};
static mut LAST_TRIGGER: Option<Instant> = None;
const TRIGGER_INTERVAL: Duration = Duration::from_millis(1000); // 1000ms 内只触发一次

static mut HOOK: HHOOK = HHOOK(null_mut());
const TRAY_MSG: u32 = WM_USER + 1;

unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam.0 == WM_MOUSEWHEEL as usize {
        let info = *(lparam.0 as *const MSLLHOOKSTRUCT);
        let pt = info.pt;

        // let screen_height = GetSystemMetrics(SM_CYSCREEN);
        let taskbar = FindWindowW(w!("Shell_TrayWnd"), None);
        let mut rect = RECT::default();
        GetWindowRect(taskbar.unwrap(), &mut rect);
        let in_rect = pt.x >= rect.left && pt.x <= rect.right && pt.y >= rect.top && pt.y <= rect.bottom;
        // 2. 任务栏是否可见（没有被前台窗口覆盖）
        let foreground = GetForegroundWindow();
        let mut fg_rect = RECT::default();
        GetWindowRect(foreground, &mut fg_rect);
        let taskbar_visible = !(fg_rect.left <= rect.left
            && fg_rect.right >= rect.right
            && fg_rect.top <= rect.top
            && fg_rect.bottom >= rect.bottom);
        // 屏幕底部 40 像素认为是任务栏区域
        // if pt.y > screen_height - 40 {
        if in_rect && taskbar_visible {
            // ======== 新增：节流逻辑 ========
            let now = Instant::now();
            if let Some(last) = LAST_TRIGGER {
                if now.duration_since(last) < TRIGGER_INTERVAL {
                    return CallNextHookEx(HHOOK(null_mut()), code, wparam, lparam);
                }
            }
            LAST_TRIGGER = Some(now);
            // =================================
            let _ = ShellExecuteW(
                None,
                w!("open"),
                w!("explorer.exe"),
                w!("shell:MyComputerFolder"),
                None,
                SW_SHOWNORMAL,
            );
        }
    }

    CallNextHookEx(HHOOK(null_mut()), code, wparam, lparam)
}

unsafe fn create_tray_icon(hwnd: HWND) {
    let mut nid = NOTIFYICONDATAW::default();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.uCallbackMessage = TRAY_MSG;

    nid.hIcon = LoadIconW(GetModuleHandleW(None).unwrap(), w!("IDI_ICON1")).unwrap_or_default();

    let tip = "Taskbar Scroll Opener\0";
    let tip_utf16: Vec<u16> = tip.encode_utf16().collect();
    for (i, ch) in tip_utf16.iter().enumerate() {
        if i >= nid.szTip.len() {
            break;
        }
        nid.szTip[i] = *ch;
    }

    let _ = Shell_NotifyIconW(NIM_ADD, &nid);
}

unsafe fn remove_tray_icon(hwnd: HWND) {
    let mut nid = NOTIFYICONDATAW::default();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = 1;
    let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, _w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        TRAY_MSG => {
            // 托盘图标消息：右键弹起 → 退出
            if l.0 as u32 == WM_RBUTTONUP {
                PostQuitMessage(0);
            }
        }
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => {}
    }

    DefWindowProcW(hwnd, msg, _w, l)
}

fn main() {
    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap();

        let class_name = w!("TaskbarScrollTrayClass");

        // 用 WNDCLASSW + RegisterClassW（0.58 里这是最稳的组合）
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: HINSTANCE(hinstance.0),
            lpszClassName: class_name,
            ..Default::default()
        };

        let _atom = RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            w!(""),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            HWND(null_mut()),
            HMENU(null_mut()),
            hinstance,
            None,
        )
        .unwrap();

        create_tray_icon(hwnd);

        HOOK = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), hinstance, 0).unwrap();

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(null_mut()), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = UnhookWindowsHookEx(HOOK);
        remove_tray_icon(hwnd);
    }
}
