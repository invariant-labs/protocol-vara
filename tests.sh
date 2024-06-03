./build.sh dev
# Download node binary
cargo xtask node
# Run tests with backtrace
RUST_BACKTRACE=1 cargo test --workspace --features gear-erc20/test