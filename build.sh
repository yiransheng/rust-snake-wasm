#!/bin/sh

set -ex
cd "$(dirname $0)"

FLAG=""

if [ "$1" = "--release" ]; then
  FLAG=$1
fi

cargo +nightly build $FLAG --target wasm32-unknown-unknown

wasm-bindgen ./target/wasm32-unknown-unknown/debug/snake_wasm.wasm --out-dir .

# wasm-opt -Os snake_wasm_bg.wasm -o snake_wasm.wasm

yarn build
