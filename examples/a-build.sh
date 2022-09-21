#!/usr/bin/env bash

set -euo pipefail

cd a

cargo build --release

mkdir -p ../dist
cp target/release/a ../dist/a
