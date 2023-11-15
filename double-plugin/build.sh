#!/usr/bin/env sh

set -x
set -e

cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/double_plugin.wasm ../double_plugin.wasm
