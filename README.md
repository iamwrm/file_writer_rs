# file_writer_rs

This repository contains a Rust library for writing to files. It is designed to be used as a C library and provides a C API for writing to files.

Also a demo on how to use (static linked) Rust library from C/C++.

## Rust

```bash
cargo build
cargo bench
```

## C++ example

```bash
bash examples/run.sh
```


## To use `file_writer` in your C++ project

In your `CMakeLists.txt` add:

```cmake
include(FetchContent)
FetchContent_Declare(
    file_writer
    GIT_REPOSITORY https://github.com/iamwrm/file_writer_rs.git
    GIT_TAG main
)
FetchContent_MakeAvailable(file_writer)

add_executable(your_app main.cpp)
target_link_libraries(your_app PRIVATE 
    file_writer 
    file_writer_header
)
```

To test, just replace `examples/CMakeLists.txt` with 

```
<<<<<< SEARCH
FetchContent_Declare(
    file_writer
    SOURCE_DIR ${CMAKE_CURRENT_LIST_DIR}/..
)
=======
FetchContent_Declare(
    file_writer
    GIT_REPOSITORY https://github.com/iamwrm/file_writer_rs.git
    GIT_TAG main
)
>>>>>>> REPLACE
```

