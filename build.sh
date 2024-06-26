set -e
rm -rf ./target/wasm32-unknown-unknown/
case $1 in
  dev)
    cargo build -p invariant-wasm --release --features "test" &&
    cargo build -p "gear-erc20-wasm" --release --features "test"
    ;;
  dev-sdk)
    cargo build -p invariant-wasm --release &&
    cargo build -p "gear-erc20-wasm" --release --features "test"
    ;; 
  *)
    cargo build -p invariant-wasm --release &&
    cargo build -p "gear-erc20-wasm" --release
  ;;
esac