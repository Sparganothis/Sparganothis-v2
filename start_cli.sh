#!/bin/bash
set -ex
export RUST_LOG=info,iroh=error
cargo run --bin echo_cli 