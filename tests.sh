# Build 
cargo build --release

# Download node binary
cargo xtask node
# Run tests
cargo test

wait -n
test_status=$?

exit $test_status