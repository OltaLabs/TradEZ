THIS IS A POC FOR NOW. CODE ISN'T FINAL AND WAS MADE TO ITERATE A FIRST VERSION FAST.

Prerequisites : 
- Rust wasm target: 
```
rustup target add wasm32-unknown-unknown
```
- Octez binary in path

Build kernel : 
```
./build_kernel.sh
```

Launch tests : 
```
cargo build --release && cargo test --package tradez-tests --lib
```

Launch local environment :
```
cargo build --release && cd tradez-tests && cargo run --release
```

TODO:
- Send one message for multiple input using batch on rollup node (a bit like blueprint on etherlink)
- Make RPC nodes
- Unwraps in kernel
- Manage reboots
- Manage permissions for the sequencer
- bridge and so Reorg on Etherlink ?
- Check maybe remove nonce
- Send only the order that match a maker/taker to not send directly cancelled orders
- U256 ? Perf ?