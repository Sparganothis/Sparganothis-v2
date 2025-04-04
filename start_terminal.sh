#!/bin/bash
set -ex
export RUST_LOG=info,iroh=error,iroh-gossip=error,iroh-relay=error,webrtc=error,webrtc_ice=error
# export RUST_LOG=debug
# export RUST_BACKTRACE=1
# export RUST_LOG=info
cargo build --bin client_terminal
time cargo-watch -x "run --bin client_terminal " \
    --watch client_terminal \
    --watch game --watch protocol \
    --watch start_terminal.sh