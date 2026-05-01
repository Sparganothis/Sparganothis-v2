#!/bin/bash
set -ex
# dx build --package client --verbose
dx build --package client --verbose --release
cargo build --package server --bin server --release
# cargo build --package server --bin echo_cli --release


rm -rf ./dist || true
mkdir -p dist
cp -av ./target/dx/client/release/web/public/* ./dist
# cp -av ./target/dx/client/debug/web/public/* ./dist
echo '/* /index.html 200' > dist/_redirects && cp dist/index.html dist/404.html

rm -rf dist_server || true
mkdir -p dist_server
cp -av ./target/release/server dist_server
# cp -av ./target/release/echo_cli dist_server

rm -rf dist2 || true
mkdir -p dist2
mv dist dist2
mv dist_server dist2