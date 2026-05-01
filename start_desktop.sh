#!/bin/bash
set -ex
export RUST_BACKTRACE=full
cd client
dx build --package client --platform desktop --profile desktop-dev --features desktop --bin client
cargo run --package client --profile desktop-dev --features desktop --bin client
# ./target/desktop-dev/client.exe