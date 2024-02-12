# Start node
./gear --dev

# Build 
cargo build --release

# optimalize wasm
wasm-proc target/wasm32-unknown-unknown/release/invariant.wasm

# Run tests
cargo test --release

