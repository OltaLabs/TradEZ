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
- Don't duplicate orders in book and accounts
- Make RPC nodes
- Stream trades that succeeded 
- Fix buy/sell with faucet
- Unwraps in kernel
- Check maybe remove nonce
- Becareful with 1 XTZ is 10000000 XTZ
- U256 ? Perf ?

Send one big batch of op to the kernel run in smartrollupnode to mutualise a lot of cost to counter the time it takes to launch a vm