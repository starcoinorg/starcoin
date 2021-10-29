#!/bin/bash

SCRIPT_DIR="$( cd "$( dirname "$0" )" >/dev/null 2>&1 && pwd )";
docker run -d \
    -p 9090:9090 \
    -v "${SCRIPT_DIR}"/prometheus.yml:/etc/prometheus/prometheus.yml \
    prom/prometheus