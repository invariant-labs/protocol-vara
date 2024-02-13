# Build 
cargo build --release

# Optimize WASM
wasm-proc target/wasm32-unknown-unknown/release/invariant.wasm

# Run tests
cargo test &
./gear --dev & 
node_pid=$!

wait -n
kill $node_pid