set -e

./build.sh dev
# Download node binary
cargo xtask node
# Run tests with backtrace
# Wasm packages are excluded since they've already been built and linker fails when they are built together
RUST_BACKTRACE=1 cargo test --workspace --exclude "gear-erc20-wasm" --exclude "invariant-wasm" $1