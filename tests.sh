# Build 
cargo build --release
# Build token
cargo build -p fungible-token --release

# Download node binary
cargo xtask node
# Run tests
cargo test --workspace