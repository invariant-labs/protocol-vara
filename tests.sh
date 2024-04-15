# Build 
cargo build --release --features debug
# Build token
cargo build -p fungible-token --release

# Download node binary
cargo xtask node
# Run tests with backtrace
RUST_BACKTRACE=1 cargo test --workspace