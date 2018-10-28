#!/bin/sh

set -ex
cd "$(dirname $0)"

FLAG=""

if [ "$1" = "--release" ]; then
  FLAG=$1
fi

rm -rf ./dist/*.wasm
rm -rf ./snake_wasm.js
rm -rf ./snake_wasm_bg.wasm

cargo +nightly build $FLAG --target wasm32-unknown-unknown

if [ "$1" = "--release" ]; then
  wasm-bindgen ./target/wasm32-unknown-unknown/release/snake_wasm.wasm --out-dir .
else 
  wasm-bindgen ./target/wasm32-unknown-unknown/debug/snake_wasm.wasm --out-dir .
fi

# wasm-opt -Os snake_wasm_bg.wasm -o snake_wasm.wasm

yarn build
