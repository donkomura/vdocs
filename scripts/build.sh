#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "Building vim_core Wasm module..."
wasm-pack build --target web --out-dir ../../extension/pkg crates/vim_core

echo "Build complete. Extension files are ready in extension/"
