cmake_minimum_required(VERSION 3.16) # Corrosion needs >= 3.15, FetchContent needs >= 3.11

project(file_writer)

# Include the Corrosion CMake file
include(FetchContent)
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.5.1 # Use a specific version/tag for stability
)
FetchContent_MakeAvailable(Corrosion)

# Import the Rust library
corrosion_import_crate(
    MANIFEST_PATH ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml
    CRATE_TYPES staticlib
)

target_include_directories(file_writer INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/include)