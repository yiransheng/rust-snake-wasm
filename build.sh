#!/bin/sh


set -ex
cd "$(dirname $0)"

cargo +nightly build --target wasm32-unknown-unknown

wasm-bindgen ./target/wasm32-unknown-unknown/debug/snake_wasm.wasm --out-dir .

yarn build
