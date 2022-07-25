#!/usr/bin/env bash
cargo build
mkdir -p dist
cp target/debug/boost dist/
