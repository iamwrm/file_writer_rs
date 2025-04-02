#!/bin/bash
set -ueo pipefail

SCRIPT_DIR=$(dirname "$0")
PROJECT_ROOT="$SCRIPT_DIR/.." 
EXAMPLE_DIR="$SCRIPT_DIR"
BUILD_DIR="$EXAMPLE_DIR/build"

# rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

cmake -S "$EXAMPLE_DIR" -B "$BUILD_DIR" -DCMAKE_BUILD_TYPE=Release -G Ninja
cmake --build "$BUILD_DIR" 

echo "--- Running Example (Catch2 Tests) ---"
cd "$BUILD_DIR"
ctest --output-on-failure
# ./example_app

echo "--- Example Run Finished ---"