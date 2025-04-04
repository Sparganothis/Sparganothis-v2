#!/bin/bash
set -ex
export RUST_LOG=info,iroh=error,iroh-gossip=error,iroh-relay=error,webrtc=error,webrtc_ice=error
# export RUST_LOG=debug
# export RUST_BACKTRACE=1
# export RUST_LOG=info
cargo build --bin terminal
time cargo run --bin terminal 