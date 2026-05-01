#!/bin/bash
set -ex
export RUSTFLAGS=''
export RUST_LOG='info,iroh=error,'
cargo run --package server --bin server