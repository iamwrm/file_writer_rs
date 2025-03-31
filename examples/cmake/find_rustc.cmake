# --- Setup variables for build-local Rust installation ---
set(CORROSION_LOCAL_RUST_DIR "${CMAKE_CURRENT_BINARY_DIR}/_rust_local")
set(CORROSION_LOCAL_CARGO_HOME "${CORROSION_LOCAL_RUST_DIR}/cargo")
set(CORROSION_LOCAL_RUSTUP_HOME "${CORROSION_LOCAL_RUST_DIR}/rustup")
set(CORROSION_LOCAL_CARGO_BIN "${CORROSION_LOCAL_CARGO_HOME}/bin")

# --- Clear any previous Rust variables to avoid conflicts ---
unset(Rust_COMPILER CACHE)
unset(Rust_CARGO CACHE)
unset(Rust_CARGO_TARGET CACHE)
unset(Rust_TOOLCHAIN CACHE)
unset(Rust_RESOLVE_RUSTUP_TOOLCHAINS CACHE)

# --- Check if Rust is already installed and usable ---
find_program(SYSTEM_RUSTC rustc)
find_program(SYSTEM_CARGO cargo)

if(SYSTEM_RUSTC AND SYSTEM_CARGO)
    # Test if the system rustc works properly
    execute_process(
        COMMAND ${SYSTEM_RUSTC} --version
        RESULT_VARIABLE RUSTC_VERSION_RESULT
        OUTPUT_VARIABLE RUSTC_VERSION_OUTPUT
        ERROR_QUIET
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )
    
    if(RUSTC_VERSION_RESULT EQUAL 0)
        message(STATUS "Found working system rustc: ${SYSTEM_RUSTC}")
        message(STATUS "Using system Rust installation")
        
        # Let Corrosion find Rust on its own
        # Don't set any variables - Corrosion will find the tools in PATH
        return()
    else()
        message(STATUS "System rustc found but not working properly. Will install local version.")
    endif()
endif()

# --- We need to install Rust locally ---
message(STATUS "No working Rust installation found. Installing to: ${CORROSION_LOCAL_RUST_DIR}")

if(UNIX)
    # Create the installation directory
    file(MAKE_DIRECTORY ${CORROSION_LOCAL_CARGO_HOME})
    file(MAKE_DIRECTORY ${CORROSION_LOCAL_RUSTUP_HOME})
    
    # Define the rustup installation command with custom directory
    set(RUSTUP_COMMAND "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | RUSTUP_HOME=${CORROSION_LOCAL_RUSTUP_HOME} CARGO_HOME=${CORROSION_LOCAL_CARGO_HOME} sh -s -- --no-modify-path --default-toolchain stable --profile minimal -y")
    
    # Execute rustup installation
    message(STATUS "Installing Rust to ${CORROSION_LOCAL_RUST_DIR}...")
    execute_process(
        COMMAND sh -c "${RUSTUP_COMMAND}"
        RESULT_VARIABLE RUSTUP_RESULT
        OUTPUT_VARIABLE RUSTUP_OUTPUT
        ERROR_VARIABLE RUSTUP_ERROR
        OUTPUT_STRIP_TRAILING_WHITESPACE
        ERROR_STRIP_TRAILING_WHITESPACE
    )
    
    if(NOT RUSTUP_RESULT EQUAL 0)
        message(WARNING "Rustup installation script failed with exit code ${RUSTUP_RESULT}.")
        if(RUSTUP_ERROR)
            message(WARNING "Installer stderr:\n${RUSTUP_ERROR}")
        endif()
        if(RUSTUP_OUTPUT)
            message(WARNING "Installer stdout:\n${RUSTUP_OUTPUT}")
        endif()
        
        message(FATAL_ERROR "Failed to install Rust locally. Please install Rust manually (https://www.rust-lang.org/tools/install) and ensure 'rustc' and 'cargo' are in your PATH, then re-run CMake.")
    endif()
    
    # Check if installation succeeded
    if(NOT EXISTS "${CORROSION_LOCAL_CARGO_BIN}/rustc" OR NOT EXISTS "${CORROSION_LOCAL_CARGO_BIN}/cargo")
        message(FATAL_ERROR "Rust installation completed, but binaries were not found at expected location: ${CORROSION_LOCAL_CARGO_BIN}")
    endif()
    
    # Find the actual stable rustc executable (not the rustup proxy)
    file(GLOB_RECURSE STABLE_TOOLCHAINS
         LIST_DIRECTORIES true
         "${CORROSION_LOCAL_RUSTUP_HOME}/toolchains/stable-*"
    )
    
    if(STABLE_TOOLCHAINS)
        list(GET STABLE_TOOLCHAINS 0 STABLE_TOOLCHAIN_PATH)
        message(STATUS "Found stable toolchain at: ${STABLE_TOOLCHAIN_PATH}")
        
        # Set the actual rustc binary path (not the rustup proxy)
        set(RUSTC_PATH "${STABLE_TOOLCHAIN_PATH}/bin/rustc")
        set(CARGO_PATH "${STABLE_TOOLCHAIN_PATH}/bin/cargo")
        
        if(EXISTS "${RUSTC_PATH}" AND EXISTS "${CARGO_PATH}")
            set(Rust_COMPILER "${RUSTC_PATH}" CACHE FILEPATH "Path to rustc compiler" FORCE)
            set(Rust_CARGO "${CARGO_PATH}" CACHE FILEPATH "Path to cargo" FORCE)
            message(STATUS "Using Rust toolchain: ${RUSTC_PATH}")
            message(STATUS "Using Cargo: ${CARGO_PATH}")
        else()
            # Fallback to rustup proxy if we couldn't find the direct toolchain executables
            set(Rust_COMPILER "${CORROSION_LOCAL_CARGO_BIN}/rustc" CACHE FILEPATH "Path to rustc compiler" FORCE)
            set(Rust_CARGO "${CORROSION_LOCAL_CARGO_BIN}/cargo" CACHE FILEPATH "Path to cargo" FORCE)
            set(Rust_TOOLCHAIN "stable" CACHE STRING "Rust toolchain to use" FORCE)
            message(STATUS "Using rustup proxy: ${CORROSION_LOCAL_CARGO_BIN}/rustc")
        endif()
        
        # Add to path for this CMake process
        set(ENV{PATH} "$ENV{PATH}:${CORROSION_LOCAL_CARGO_BIN}")
        
    else()
        message(STATUS "Could not find stable toolchain directory. Using rustup proxies.")
        set(Rust_COMPILER "${CORROSION_LOCAL_CARGO_BIN}/rustc" CACHE FILEPATH "Path to rustc compiler" FORCE)
        set(Rust_CARGO "${CORROSION_LOCAL_CARGO_BIN}/cargo" CACHE FILEPATH "Path to cargo" FORCE)
        set(Rust_TOOLCHAIN "stable" CACHE STRING "Rust toolchain to use" FORCE)
        
        # Add to path for this CMake process
        set(ENV{PATH} "$ENV{PATH}:${CORROSION_LOCAL_CARGO_BIN}")
    endif()
    
else() # Windows
    message(FATAL_ERROR "rustc not found, and automatic installation to build directory is not supported on Windows via this script. Please install Rust manually (https://www.rust-lang.org/tools/install) and ensure 'rustc' and 'cargo' are in your PATH, then re-run CMake.")
endif()

# Test that the selected rustc works
if(Rust_COMPILER)
    execute_process(
        COMMAND ${Rust_COMPILER} --version
        RESULT_VARIABLE RUSTC_TEST_RESULT
        OUTPUT_VARIABLE RUSTC_TEST_OUTPUT
        ERROR_VARIABLE RUSTC_TEST_ERROR
    )
    
    if(NOT RUSTC_TEST_RESULT EQUAL 0)
        message(WARNING "Selected rustc failed with:\n${RUSTC_TEST_ERROR}\n${RUSTC_TEST_OUTPUT}")
        message(FATAL_ERROR "The selected Rust compiler doesn't work. Please install Rust manually.")
    else()
        message(STATUS "Verified rustc works: ${RUSTC_TEST_OUTPUT}")
    endif()
endif()