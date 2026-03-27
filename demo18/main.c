#include <stdio.h>
#ifdef _WIN32
#include <windows.h>
#endif
int main(void) {
    SetConsoleOutputCP(CP_UTF8);
    #ifdef _MSC_VER
        printf("当前使用的是 MSVC 编译器。\n");
        printf("MSVC 版本号: %d\n", _MSC_VER);
    #elif defined(__clang__)
        printf("当前使用的是 Clang 编译器。\n");
    #elif defined(__GNUC__)
        printf("当前使用的是 GCC 编译器。\n");
    #else
        printf("未知编译器。\n");
    #endif
    return 0;
}
