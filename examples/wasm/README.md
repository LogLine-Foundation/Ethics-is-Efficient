
# Minimal Guest WASM (deterministic)

Exports only:
```wat
(module
  (func (export "run") (result i32)
    i32.const 0 ;; 0=ACK
  )
)
```

Use with /v2/run:
```bash
BASE64=$(base64 -w0 examples/wasm/run_ack.wasm)
curl -s localhost:8080/v2/run \
  -H 'content-type: application/json' \
  -d '{"realm":"trust","inputs":{"policy":{"kind":"wasm","payload_b64":"'$'BASE64"}}}' | jq .
```


## ASK example
```bash
BASE64=$(base64 -w0 examples/wasm/run_ask.wasm)
curl -s localhost:8080/v2/run \
  -H 'content-type: application/json' \
  -d '{"realm":"trust","inputs":{"policy":{"kind":"wasm","payload_b64":"'"$BASE64"'"}}}' | jq .
```
