cmake_minimum_required(VERSION 3.10)
if(MSVC)
    add_compile_options("/utf-8")
endif()
# 项目名称和语言
project(demo C)
# 指定 C 标准
set(CMAKE_C_STANDARD 17)
set(CMAKE_C_STANDARD_REQUIRED ON)
# 生成可执行文件
add_executable(demo main.c)
