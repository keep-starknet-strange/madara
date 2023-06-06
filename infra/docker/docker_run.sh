#!/usr/bin/env bash
# This script is meant to be run on Unix/Linux based systems
set -e

echo "*** Start Madara node (Docker) ***"
cd $(dirname $0)/../..

docker build -t madara/docker -f infra/docker/Dockerfile .
docker run -it madara/docker $@
