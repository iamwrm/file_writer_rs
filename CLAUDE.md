# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust library that provides a C FFI (Foreign Function Interface) for file writing operations. The project demonstrates how to create a static-linked Rust library that can be consumed by C/C++ applications.

## Core Architecture

- **Rust Library**: `src/lib.rs` contains the main file writing implementation with a C-compatible API
- **C Header**: `include/file_writer/file_writer.h` defines the C interface 
- **Build System**: Uses both Cargo (Rust) and CMake (C/C++ integration) with Corrosion for bridging
- **FFI Design**: Opaque handle pattern with `FileWriterHandle` struct and error enums for C compatibility

The library provides buffered file writing with functions for:
- Creating/opening files with append or write modes
- Writing raw bytes and C strings
- Buffer size management
- Explicit flushing and closing

## Build Commands

### Rust Development
```bash
cargo build                    # Build the Rust library
cargo test                     # Run Rust unit tests  
cargo bench                    # Run performance benchmarks
```

### C++ Example/Testing
```bash
bash examples/run.sh           # Build and run C++ example with tests
```

The C++ example uses Catch2 for testing and demonstrates integration patterns.

## Key Implementation Details

- The Rust library uses `#[repr(C)]` enums for C compatibility
- Buffer management is handled through `BufWriter<File>` with configurable sizes
- Error handling maps Rust `io::Error` to C-compatible error codes
- Memory safety is maintained through careful FFI boundary management
- Parent directories are automatically created when opening files

## Testing Strategy

- Rust unit tests in `src/lib.rs` (use `cargo test`)
- C++ integration tests in `examples/main.cpp` using Catch2
- Benchmarks available in `benches/file_writer_benchmark.rs`

## Development Tasks

- check cargo clippy and fix the warning