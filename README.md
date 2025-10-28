Avantages : 
- Shared types between kernel and sequencer
- Shared types between run and tests

Prerequisites : 
```
rustup target add wasm32-unknown-unknown
```

Build kernel : 
```
./build_kernel.sh
```

Launch tests : 
```
cargo build --release && cargo test --package tradez-tests --lib
```

TODO:
- Fix buy/sell with faucet
- Unwraps in kernel
- Check maybe remove nonce
- Becareful with 1 XTZ is 10000000 XTZ
- U256 ? Perf ?

Send one big batch of op to the kernel run in smartrollupnode to mutualise a lot of cost to counter the time it takes to launch a vm