#!/bin/bash
set -ex
export RUSTFLAGS=''
cargo run --package server --bin server