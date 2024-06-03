case $1 in
  dev)
    cargo build --release --features "debug"
    cargo build -p "gear-erc20-wasm" --release --features "test"
    ;; 
  *)
    cargo build --release
    cargo build --release -p "gear-erc20-wasm"
  ;;
esac