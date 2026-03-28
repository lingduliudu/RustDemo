# 指定最低 CMake 版本
cmake_minimum_required(VERSION 3.12)

# 设置项目名称和语言
project(MyCpp20 VERSION 1.0.0 LANGUAGES CXX)
# 设置unicode subwindow必须
add_definitions(-DUNICODE -D_UNICODE)
if(MSVC)
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} /SUBSYSTEM:WINDOWS")
    add_compile_options(/utf-8)  # 告诉 MSVC 源文件是 UTF-8
endif()
# 设置 C++ 标准为 C++20
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_GENERATOR "Visual Studio 17 2022" CACHE INTERNAL "" FORCE)

#添加头文件目录
include_directories(${CMAKE_SOURCE_DIR}/header)
#源码目录
file(GLOB SOURCES "${CMAKE_SOURCE_DIR}/src/*.cpp")
add_definitions(-D_CRT_SECURE_CPP_OVERLOAD_STANDARD_NAMES=1)
# 添加可执行文件
#add_executable(my_cpp20 main.cpp ${SOURCES})
add_executable(my_cpp20 src/ontop.cpp ${SOURCES})
target_link_libraries(my_cpp20 user32 gdi32 comctl32)
