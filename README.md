<div align="center">
    <h1>⚡Invariant protocol⚡</h1>
    <p>
        <a href="https://invariant.app/math-spec-vara.pdf">MATH SPEC 📄</a> |
        <a href="https://discord.gg/VzS3C9wR">DISCORD 🌐</a> |
    </p>
</div>

Invariant protocol is an AMM built on [Vara Network](https://vara.network), leveraging high capital efficiency and the ability to list markets in a permissionless manner. At the core of the DEX is the Concentrated Liquidity mechanism, designed to handle tokens compatible with the [gFT(ERC-20) standard](https://wiki.gear-tech.io/docs/examples/Standards/gft-20).

## 🔨 Getting Started

### Prerequisites

- Rust & Cargo ([rustup](https://www.rust-lang.org/tools/install))
- Install required packages
- Configure Rust toolchain
- Install Gear node


#### Rust & Cargo

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Install required packages
```bash
sudo apt install -y build-essential clang cmake curl
```

#### Configure Rust toolchain
```bash
rustup install nightly-2024-01-25
```
```bash
rustup target add wasm32-unknown-unknown --toolchain nightly-2024-01-25
```

#### Instal Gear node
```
curl https://get.gear.rs/gear-v1.1.0-x86_64-unknown-linux-gnu.tar.xz | tar Jx
```

## Build protocol
```bash
cargo build --release
```

## Run tests
```bash
chmod +x tests.sh
```
```bash
./tests.sh
```