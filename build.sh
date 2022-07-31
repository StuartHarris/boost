#!/usr/bin/env bash

set -euo pipefail

mkdir -p dist

cargo build --release && cp target/debug/boost dist/
