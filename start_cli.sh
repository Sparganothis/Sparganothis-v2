#!/bin/bash
set -ex
export RUST_LOG=info,iroh=error,iroh-gossip=error,iroh-relay=error,webrtc=error,webrtc_ice=error
# export RUST_LOG=info
cargo run --bin echo_cli 