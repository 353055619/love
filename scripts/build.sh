#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

cargo build --release --target wasm32-unknown-unknown
rm -rf web/pkg dist
wasm-bindgen \
  target/wasm32-unknown-unknown/release/love.wasm \
  --target web \
  --out-dir web/pkg \
  --no-typescript

mkdir -p dist
cp -R web/. dist/
