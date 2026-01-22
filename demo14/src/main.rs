#![windows_subsystem = "windows"]
use std::ptr::null_mut;
use windows::{
    core::w,
    Win32::{
        Foundation::*,
        System::LibraryLoader::*,
        UI::{
            Input::KeyboardAndMouse::*,
            Shell::*,
            WindowsAndMessaging::*,
        },
    },
};

static mut HOOK: HHOOK = HHOOK(null_mut());
const TRAY_MSG: u32 = WM_USER + 1;

unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam.0 == WM_MOUSEWHEEL as usize {
        let info = *(lparam.0 as *const MSLLHOOKSTRUCT);
        let pt = info.pt;

        let screen_height = GetSystemMetrics(SM_CYSCREEN);

        // 屏幕底部 40 像素认为是任务栏区域
        if pt.y > screen_height - 40 {
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

    nid.hIcon = LoadIconW(None, IDI_APPLICATION).unwrap_or_default();

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

        HOOK = SetWindowsHookExW(
            WH_MOUSE_LL,
            Some(mouse_hook_proc),
            hinstance,
            0,
        )
        .unwrap();

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(null_mut()), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = UnhookWindowsHookEx(HOOK);
        remove_tray_icon(hwnd);
    }
}
