#pragma once

#ifdef _WIN32
  #ifdef LIBRARY_EXPORTS
    #define LIB_API __declspec(dllexport)
  #else
    #define LIB_API __declspec(dllimport)
  #endif
#else
  #define LIB_API
#endif

extern "C" LIB_API BOOL APIENTRY DllMain(HMODULE hModule, DWORD ul_reason_for_call, LPVOID lpReserved);
