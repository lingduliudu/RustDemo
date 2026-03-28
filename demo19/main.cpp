#include <windows.h>
#define ID_CHECKBOX 101
#define ID_TIMER 102
bool g_enabled = false;

/*************************************************************************************************
 * Description:  发送键位
 * Author:       yuanhao
 * Versions:     V1.0
 *************************************************************************************************/
void SendKey(WORD vk) {
    INPUT inputs[2] = {};
    // 按下
    inputs[0].type = INPUT_KEYBOARD;
    inputs[0].ki.wVk = vk;
    // 抬起
    inputs[1].type = INPUT_KEYBOARD;
    inputs[1].ki.wVk = vk;
    inputs[1].ki.dwFlags = KEYEVENTF_KEYUP;
    SendInput(2, inputs, sizeof(INPUT));
}

/*************************************************************************************************
 * Description: 定时检查
 * Author:       yuanhao
 * Versions:     V1.0
 *************************************************************************************************/
void OnTimer() {
    if (!g_enabled) return;
    if (GetAsyncKeyState('X') & 0x8000) SendKey('X');
    if (GetAsyncKeyState('S') & 0x8000) SendKey('S');
}

/*************************************************************************************************
 * Description: 主程序逻辑
 * Author:       yuanhao
 * Versions:     V1.0
 *************************************************************************************************/
LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    static HWND hCheck;
    switch (msg) {
        case WM_CREATE:
            hCheck = CreateWindowW(L"BUTTON", L"启用自动连发 (X / S)", WS_VISIBLE | WS_CHILD | BS_AUTOCHECKBOX, 20, 20, 220, 30, hwnd, (HMENU)ID_CHECKBOX, NULL, NULL);
            // 200ms 定时器
            SetTimer(hwnd, ID_TIMER, 100, NULL);
            break;
        case WM_COMMAND:
            if (LOWORD(wParam) == ID_CHECKBOX) {
                g_enabled = (SendMessage(hCheck, BM_GETCHECK, 0, 0) == BST_CHECKED);
            }
            break;

        case WM_TIMER:
            if (wParam == ID_TIMER) OnTimer();
            break;
        case WM_DESTROY:
            KillTimer(hwnd, ID_TIMER);
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
int WINAPI wWinMain(HINSTANCE hInstance, HINSTANCE, PWSTR, int nCmdShow) {
    const wchar_t CLASS_NAME[] = L"KeySpamWindow";
    WNDCLASSW wc = {};
    wc.lpfnWndProc = WndProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = CLASS_NAME;
    RegisterClassW(&wc);
    HWND hwnd = CreateWindowExW(0, CLASS_NAME, L"按键连发工具", WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, CW_USEDEFAULT, 300, 150, NULL, NULL, hInstance, NULL);
    ShowWindow(hwnd, nCmdShow);
    MSG msg = {};
    while (GetMessage(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
    return 0;
}
