#!/bin/bash
set -e
cd "`dirname $0`"
RUSTFLAGS='-C link-arg=-s' cargo build --all --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/book_tracker_contract.wasm ./out/main.wasm
