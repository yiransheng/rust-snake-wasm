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

if [ "$1" = "--release" ]; then
  cargo +nightly build $FLAG --target wasm32-unknown-unknown

  wasm-bindgen ./target/wasm32-unknown-unknown/release/snake_wasm.wasm --out-dir .
else 
  cargo +nightly build $FLAG --features "std" --target wasm32-unknown-unknown

  wasm-bindgen ./target/wasm32-unknown-unknown/debug/snake_wasm.wasm --out-dir .
fi

# wasm-opt -Os ./dist/*.wasm

yarn build
