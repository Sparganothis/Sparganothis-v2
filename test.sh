#!/bin/bash
set -ex


RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack test --node game  --features getrandom/wasm_js
# RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo test --package game --target wasm32-unknown-unknown 
cargo test