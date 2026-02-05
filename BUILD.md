
# Build Guide — Native + WASM

## Requirements
- Rust toolchain (1.75+)
- `musl-gcc` for static binaries (on macOS via `brew install FiloSottile/musl-cross/musl-cross`)
- `jq` (optional for Makefile checks)

## One-liners
```bash
make            # builds native (musl) + wasm into ./dist
make build-native
make build-wasm
```

### Outputs
- `dist/resolver` — native static binary (musl)
- `dist/tdln-verify` — verifier (if the crate exists)
- `dist/guest-ack.wasm` — WASM guest that returns ACK (0)
- `dist/guest-ask.wasm` — WASM guest that returns ASK (1)

## Manual
```bash
# Native
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl -p resolver

# WASM guests
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown -p guest-ack
cargo build --release --target wasm32-unknown-unknown -p guest-ask
```
