#!/usr/bin/env bash

set -euo pipefail

cargo build --release

mkdir -p dist
cp target/release/boost dist
