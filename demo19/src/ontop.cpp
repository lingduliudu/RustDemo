#include <windows.h>
#include <wingdi.h>
#include <dwmapi.h>
#include <shellapi.h>
#include <string>
#include <vector>
#pragma comment(lib, "User32.lib")
#pragma comment(lib, "Shell32.lib")
#pragma comment(lib, "Dwmapi.lib")

#define ID_COMBO 101
#define ID_CHECK_TOPMOST 102
#define ID_TRAY_ICON 2001
#define WM_TRAY (WM_USER + 1)

HWND hCombo, hCheck;
HFONT hFont;
NOTIFYICONDATA nid = {0};

struct WindowInfo {
    HWND hwnd;
    std::wstring title;
};
std::vector<WindowInfo> windows;

// 判断窗口是否可见且有标题
BOOL CALLBACK EnumWindowsProc(HWND hwnd, LPARAM lParam) {
    if (!IsWindowVisible(hwnd)) return TRUE;

    wchar_t title[256];
    GetWindowTextW(hwnd, title, 256);
    if (wcslen(title) == 0) return TRUE;

    windows.push_back({hwnd, title});
    return TRUE;
}

// 刷新下拉框
void RefreshProcessList() {
    SendMessage(hCombo, CB_RESETCONTENT, 0, 0);
    windows.clear();
    EnumWindows(EnumWindowsProc, 0);

    for (auto& w : windows) {
        SendMessage(hCombo, CB_ADDSTRING, 0, (LPARAM)w.title.c_str());
    }
    SendMessage(hCombo, CB_SETCURSEL, 0, 0);
}

// 设置置顶
void ApplyTopMost() {
    int sel = (int)SendMessage(hCombo, CB_GETCURSEL, 0, 0);
    if (sel < 0 || sel >= windows.size()) return;

    HWND target = windows[sel].hwnd;
    BOOL checked = (BOOL)SendMessage(hCheck, BM_GETCHECK, 0, 0);

    SetWindowPos(target, checked ? HWND_TOPMOST : HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
}

// 添加托盘图标
void AddTrayIcon(HWND hwnd) {
    nid.cbSize = sizeof(nid);
    nid.hWnd = hwnd;
    nid.uID = ID_TRAY_ICON;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_TRAY;
    nid.hIcon = LoadIcon(NULL, IDI_INFORMATION);
    wchar_t buf[100];
    //wcscpy_s(nid.szTip, L"窗口置顶工具");
    wcscpy_s(buf, L"窗口置顶工具");
    Shell_NotifyIcon(NIM_ADD, &nid);
}

// 移除托盘图标
void RemoveTrayIcon() { Shell_NotifyIcon(NIM_DELETE, &nid); }

// 托盘菜单
void ShowTrayMenu(HWND hwnd) {
    POINT pt;
    GetCursorPos(&pt);

    HMENU menu = CreatePopupMenu();
    AppendMenuW(menu, MF_STRING, 1, L"显示主界面");
    AppendMenuW(menu, MF_STRING, 2, L"退出");

    SetForegroundWindow(hwnd);
    int cmd = TrackPopupMenu(menu, TPM_RETURNCMD, pt.x, pt.y, 0, hwnd, NULL);

    if (cmd == 1) {
        ShowWindow(hwnd, SW_SHOW);
    } else if (cmd == 2) {
        RemoveTrayIcon();
        PostQuitMessage(0);
    }
    DestroyMenu(menu);
}

// 美化背景颜色
HBRUSH hBackground = CreateSolidBrush(RGB(245, 245, 245));

LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    switch (msg) {
        case WM_CREATE:
            // 设置背景色
            SetClassLongPtr(hwnd, GCLP_HBRBACKGROUND, (LONG_PTR)hBackground);

            // 创建字体（Win10 风格）
            hFont = CreateFontW(-16, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, DEFAULT_PITCH, L"Segoe UI");

            // 下拉框
            hCombo = CreateWindowW(L"COMBOBOX", NULL, WS_CHILD | WS_VISIBLE | CBS_DROPDOWNLIST, 20, 20, 300, 200, hwnd, (HMENU)ID_COMBO, NULL, NULL);

            // 复选框
            hCheck = CreateWindowW(L"BUTTON", L"置顶", WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX, 20, 60, 200, 25, hwnd, (HMENU)ID_CHECK_TOPMOST, NULL, NULL);

            // 设置字体
            SendMessage(hCombo, WM_SETFONT, (WPARAM)hFont, TRUE);
            SendMessage(hCheck, WM_SETFONT, (WPARAM)hFont, TRUE);

            RefreshProcessList();
            AddTrayIcon(hwnd);
            break;

        case WM_COMMAND:
            if (LOWORD(wParam) == ID_CHECK_TOPMOST) {
                ApplyTopMost();
            }
            break;

        case WM_CTLCOLORSTATIC:
        case WM_CTLCOLORBTN:
        case WM_CTLCOLORLISTBOX: {
            HDC hdc = (HDC)wParam;
            SetBkColor(hdc, RGB(245, 245, 245));
            return (LRESULT)hBackground;
        }

        case WM_CLOSE:
            ShowWindow(hwnd, SW_HIDE);
            return 0;

        case WM_TRAY:
            if (lParam == WM_RBUTTONUP) {
                ShowTrayMenu(hwnd);
            } else if (lParam == WM_LBUTTONDBLCLK) {
                ShowWindow(hwnd, SW_SHOW);
            }
            break;

        case WM_DESTROY:
            RemoveTrayIcon();
            DeleteObject(hBackground);
            DeleteObject(hFont);
            PostQuitMessage(0);
            break;
    }
    return DefWindowProc(hwnd, msg, wParam, lParam);
}

/*************************************************************************************************
 * Description: 程序入口
 * Author:       yuanhao
 * Versions:     V1.0
 *************************************************************************************************/
int APIENTRY wWinMain(HINSTANCE hInstance, HINSTANCE, LPWSTR, int nCmdShow) {
    WNDCLASSW wc = {0};
    wc.lpfnWndProc = WndProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = L"TopmostTool";
    wc.hCursor = LoadCursor(NULL, IDC_ARROW);
    wc.hbrBackground = hBackground;

    RegisterClassW(&wc);

    HWND hwnd = CreateWindowW(L"TopmostTool", L"窗口置顶工具", WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX, CW_USEDEFAULT, CW_USEDEFAULT, 360, 160, NULL, NULL, hInstance, NULL);

    ShowWindow(hwnd, nCmdShow);
    UpdateWindow(hwnd);

    MSG msg;
    while (GetMessage(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
    return 0;
}
