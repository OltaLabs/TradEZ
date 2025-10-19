Avantages : 
- Shared types between kernel and sequencer
- Shared types between run and tests

Build kernel : 
```
./build_kernel.sh
```

Launch tests : 
```
cargo test --package tradez-tests --lib
```