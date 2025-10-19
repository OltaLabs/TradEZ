#!/bin/sh
set -x

cargo build --release --target wasm32-unknown-unknown --package tradez-kernel

wasm-strip target/wasm32-unknown-unknown/release/tradez_kernel.wasm

cp target/wasm32-unknown-unknown/release/tradez_kernel.wasm tradez_kernel.wasm
cp target/wasm32-unknown-unknown/release/tradez_kernel.wasm crates/tradez-tests/tradez_kernel.wasm