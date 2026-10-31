#!/usr/bin/env bash
set -euo pipefail
cargo test --workspace --all-targets --locked
cargo build --release --locked --target wasm32-wasip1 -p termcade-asteroid -p termcade-brickough -p termcade-tetris
node scripts/test-wasm.mjs
python3 scripts/test-recover-catalog.py
