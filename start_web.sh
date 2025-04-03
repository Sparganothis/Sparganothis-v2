#!/bin/bash
set -ex
export RUSTFLAGS='--cfg getrandom_backend="wasm_js"'
dx serve --package client --bin client