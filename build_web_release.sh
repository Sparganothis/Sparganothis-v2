#!/bin/bash
set -ex
export RUSTFLAGS='--cfg getrandom_backend="wasm_js"'
# dx build --package client --release

rm -rf ./dist || true
mkdir -p dist
cp -av ./target/dx/client/release/web/public/* ./dist
echo '/* /index.html 200' > dist/_redirects && cp dist/index.html dist/404.html