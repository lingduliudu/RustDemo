# CMakeLists.txt

# 1. 指定所需的 CMake 版本
# 这是一个推荐的最佳实践，可以避免与旧版 CMake 的兼容性问题。
cmake_minimum_required(VERSION 3.10 FATAL_ERROR)

# 2. 设置项目名称和语言
# PROJECT 命令必须在任何 TARGET 命令之前。
# LANGUAGES 指定项目使用的编程语言，这里是 C++。
project(MyCMakeProject LANGUAGES CXX)

# 3. 设置 C++ 标准
# 推荐明确指定 C++ 标准，以确保代码在不同编译器上的一致性。
# cxx_std_17 表示 C++17 标准。
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON) # 强制要求编译器支持 C++17
set(CMAKE_CXX_EXTENSIONS OFF)      # 禁用编译器特有的 C++ 扩展

# 4. 配置编译输出目录 (可选但推荐)
# 通常将编译生成的文件放在一个名为 'bin' 的目录中，方便管理。
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

file(GLOB_RECURSE SOURCES "src/*.cpp")
file(GLOB_RECURSE HEADERS "src/include/*.h")
add_executable(test ${SOURCES} ${HEADERS})

# 6. 添加包含目录 (如果你的项目有额外的头文件)
# target_include_directories(target_name PUBLIC|PRIVATE|INTERFACE path/to/headers)
# 假设你的头文件在 'include/' 目录，或者你的源文件在 'src/' 目录下需要包含同目录的头文件。
# 这里我们假设 main.cpp 内部没有外部的额外头文件路径需求，所以这行暂时可以省略或根据实际情况添加。
# 例如：target_include_directories(MyApplication PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)

# 7. 链接库 (如果你的项目依赖其他库)
# target_link_libraries(target_name PUBLIC|PRIVATE|INTERFACE library_name...)
# 假设你的项目需要链接一个名为 'MyLibrary' 的外部库。
# 如果是系统库，直接写库名即可，如 'pthread' (Linux) 或 'ws2_32' (Windows)。
# target_link_libraries(MyApplication PUBLIC MyLibrary)

# 8. 启用生成 compile_commands.json (用于 LSP，如 clangd，强烈推荐)
# 这个文件对于代码编辑器的智能提示和诊断功能至关重要。
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# 9. 定义安装规则 (可选)
# 如果你想将编译后的文件安装到特定位置，可以在这里定义规则。
# install(TARGETS MyApplication DESTINATION bin)
# install(FILES ${CMAKE_CURRENT_SOURCE_DIR}/data/config.json DESTINATION etc)

# 10. 添加测试 (可选，如果使用 CTest)
# enable_testing()
# add_test(NAME MyTest COMMAND MyApplication --test-mode)
