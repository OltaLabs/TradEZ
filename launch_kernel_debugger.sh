#!/bin/sh
set -x

cargo build --release --target wasm32-unknown-unknown --package tradez-kernel

wasm-strip target/wasm32-unknown-unknown/release/tradez_kernel.wasm

smart-rollup-installer get-reveal-installer \
  --upgrade-to target/wasm32-unknown-unknown/release/tradez_kernel.wasm \
  --output tradez_kernel_installer.wasm --preimages-dir preimages/

smart-rollup-installer get-reveal-installer \
  --upgrade-to target/wasm32-unknown-unknown/release/tradez_kernel.wasm \
  --output tradez_kernel_installer.hex --preimages-dir preimages/

octez-smart-rollup-wasm-debugger --kernel tradez_kernel_installer.wasm \
  --preimage-dir preimages/ --inputs debug_inputs/empty.json

