#!/usr/bin/env bash

set -euo pipefail

mkdir -p dist

cargo build && cp target/debug/boost dist/
