#!/bin/bash
set -ex
export MSYS_NO_PATHCONV=1
cd _docker
docker-compose rm -f  || true
docker-compose up -d --remove-orphans
docker-compose logs -f
