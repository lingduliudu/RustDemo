cmake_minimum_required(VERSION 3.10)
project(Library)

add_library(mylib SHARED
    mylib.cpp
    mylib.h
)
target_compile_definitions(mylib PRIVATE LIBRARY_EXPORTS)
