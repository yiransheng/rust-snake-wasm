CRATE_NAME := snake_wasm
DIST := docs

BUILD := release
build_dir=${CURDIR}/target/wasm32-unknown-unknown/${BUILD}

WASM_FILES := $(CRATE_NAME).js $(CRATE_NAME)_bg.wasm

release: BUILD=release
release: clean cargo_release wasm dist opt

dev: BUILD=debug
dev: clean cargo_debug wasm dist
	yarn serve

opt: dist
	DIST_WASM=$(shell find ./docs  -name \*.wasm) && \
	wasm-opt -Os $$DIST_WASM -o optimized.wasm && \
	mv optimized.wasm $$DIST_WASM

dist: wasm
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
