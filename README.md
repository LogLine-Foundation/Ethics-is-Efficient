
# G1 Resolver Skeleton (compile-ready)

This is a minimal, compile-ready skeleton implementing:
- POST /v2/run (returns 202 with did/cid/url/status + preview)
- GET  /v2/cards/:did (returns a minimal DiamondCard per schema)
- GET  /v2/cards/:did/bundle.zip (offline bundle with card + manifest + signatures placeholder)

## Build
```
cargo build
```

## Run (dev)
```
RUST_LOG=info cargo run -p resolver
```

## Smoke test
```
bash tests/e2e_resolver_smoke.sh
```

> NOTE: Signatures are placeholders for now. Replace with real ed25519 when wiring tdln-signatures.
