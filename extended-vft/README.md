# VFT (Vara Fungible Token)

The VFT program constitutes a fungible token contract that incorporates a complicated role management mechanism. It enables the creation of fungible tokens with configurable parameters, such as the token name, symbol, and decimal precision. The contract implements essential token functionalities, including minting, burning, transferring tokens, and managing allowances, while enforcing stringent role-based access controls to safeguard the system and ensure the proper delegation of authority over these operations.

### 🏗️ Building

```sh
cargo b -r 
```

### ✅ Testing

Run all tests, except `gclient` ones:
```sh
cargo t -r 
```

Run all tests:
```sh
# Download the node binary.
cargo t -r -- --ignored
```

