## Build Tools:
## ===================
## 1. rustup + cargo:
## https://rustwasm.github.io/wasm-bindgen/whirlwind-tour/basic-usage.html
#
# rustup target add wasm32-unknown-unknown --toolchain nightly
#
## 2. wasm-bindgen-cli
# cargo +nightly install wasm-bindgen-cli
#
## 3. nodejs + yarn
# yarn
#
## 4 (optional) wasm-opt: https://github.com/WebAssembly/binaryen

CRATE_NAME := snake_wasm
DIST := docs

BUILD := release
build_dir=${CURDIR}/target/wasm32-unknown-unknown/${BUILD}

WASM_FILES := $(CRATE_NAME).js $(CRATE_NAME)_bg.wasm

dist: BUILD=release
dist: clean cargo_release wasm webpack opt

dev: BUILD=debug
dev: clean cargo_debug wasm webpack
	yarn serve

opt: webpack
	DIST_WASM=$(shell find ./docs  -name \*.wasm) && \
	wasm-opt -Oz $$DIST_WASM -o optimized.wasm && \
	mv optimized.wasm $$DIST_WASM

webpack: wasm
	yarn build

wasm:
	wasm-bindgen ${build_dir}/${CRATE_NAME}.wasm --out-dir .

cargo_release:
	cargo +nightly build --release --target wasm32-unknown-unknown

cargo_debug:
	cargo +nightly build --features "std" --target wasm32-unknown-unknown


.PHONY: clean

clean:
	rm -f ${WASM_FILES}
	rm -f ${DIST}/*
