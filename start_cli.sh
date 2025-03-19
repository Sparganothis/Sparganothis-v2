#!/bin/bash
set -ex
export RUST_LOG=info,iroh=error,iroh-gossip=error,iroh-relay=error
# export RUST_LOG=info
cargo run --bin echo_cli 