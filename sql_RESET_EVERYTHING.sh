#!/bin/bash
set -ex
export RUST_LOG=info,iroh=error,iroh-gossip=error,iroh-relay=error,webrtc=error,webrtc_ice=error

export DATABASE_URL="mariadb://root:sparganothis@127.0.0.1/sparganothis"

cd server

sqlx database drop -y
sleep 1
sqlx database create
sleep 1
sqlx migrate run

cd ..
./sql_prepare.sh