#!/bin/sh
set -x

rm -rf ~/.tradez-rollup-node

octez-client originate smart rollup \
  "tradez_rollup" from "bootstrap1" \
  of kind wasm_2_0_0 of type bytes \
  with kernel file:tradez_kernel_installer.hex --burn-cap 3

mkdir -p ~/.tradez-node/wasm_2_0_0

cp preimages/* ~/.tradez-rollup-node/wasm_2_0_0/
