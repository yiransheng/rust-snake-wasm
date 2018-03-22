cargo  +nightly build --target wasm32-unknown-unknown --release
wasm-gc target/wasm32-unknown-unknown/release/snake_wasm.wasm target/wasm32-unknown-unknown/release/snake_wasm.small.wasm
