#!/usr/bin/env bash
set -euo pipefail

pushd frontend
trunk build
popd

pushd server
cargo run --release -- --port 8080
popd
