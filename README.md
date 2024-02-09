## Prerequisite

```bash
sudo apt install -y build-essential clang cmake
```

```bash
curl https://get.gear.rs/gear-v1.0.2-x86_64-unknown-linux-gnu.tar.xz | tar xJ
```

```bash
rustup toolchain add nightly-2023-09-18
```

```bash
rustup target add wasm32-unknown-unknown --toolchain nightly-2023-09-18
```

```bash
cargo install --locked --git https://github.com/gear-tech/gear.git wasm-proc
```

## Build

```bash
cargo build --release
```

### Optimize Wasm binary

```bash
wasm-proc target/wasm32-unknown-unknown/release/[module-name].wasm
```

```bash
wasm-proc target/wasm32-unknown-unknown/release/invariant.wasm
```

### Output files

- `target/wasm32-unknown-unknown/release/invariant.wasm`
- `target/wasm32-unknown-unknown/release/invariant.opt.wasm`
