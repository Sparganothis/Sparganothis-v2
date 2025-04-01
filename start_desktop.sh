#!/bin/bash
set -ex
export RUST_BACKTRACE=full
cd client
dx build --package client --platform desktop --profile desktop-dev --features desktop --bin client
dx run --package client --platform desktop --profile desktop-dev --features desktop --bin client
# ./target/desktop-dev/client.exe