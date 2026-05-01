#!/bin/bash

set -ex

sudo apt install -y clang build-essential

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

rustup target add wasm32-unknown-unknown

curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

cargo binstall dioxus-cli@0.6.3  wasm-bindgen-cli@0.2.100  --no-confirm --force


