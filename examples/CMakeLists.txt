cmake_minimum_required(VERSION 3.16) # Match root or higher

project(file_writer_example LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 11)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(cmake/find_rustc.cmake)

include(FetchContent)
FetchContent_Declare(
    file_writer
    SOURCE_DIR ${CMAKE_CURRENT_LIST_DIR}/..
)
#FetchContent_Declare(
#    file_writer
#    GIT_REPOSITORY https://github.com/iamwrm/file_writer_rs.git
#    GIT_TAG main
#)
FetchContent_MakeAvailable(file_writer)


# --- Fetch Catch2 for testing ---
FetchContent_Declare(
    Catch2
    GIT_REPOSITORY https://github.com/catchorg/Catch2.git
    GIT_TAG        v3.5.2 # Use a specific version
)
FetchContent_MakeAvailable(Catch2)
# ---

# Define the example executable
add_executable(example_app main.cpp)

# Link the example app against the Rust library and Catch2
target_link_libraries(example_app PRIVATE
    Catch2::Catch2WithMain       # Link Catch2 main
    file_writer # Use the target name provided by find_package
)

# Add the test using Catch2's discovery
include(CTest)
include(Catch)
catch_discover_tests(example_app)
