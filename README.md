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