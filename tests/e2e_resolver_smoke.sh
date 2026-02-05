
#!/usr/bin/env bash
set -euo pipefail
cargo build -q
RUST_LOG=warn cargo run -p resolver > /tmp/resolver.log 2>&1 &
PID=$!
sleep 1
curl -s localhost:8080/v2/run -H 'content-type: application/json' -d @examples/tuples/logline-astronaut/request.json | tee /tmp/run.json
DID=$(jq -r .did /tmp/run.json)
curl -s "localhost:8080/v2/cards/$DID" | jq .schema
curl -s -o /tmp/bundle.zip "localhost:8080/v2/cards/$DID/bundle.zip"
cargo run -q -p tdln-verify -- /tmp/bundle.zip || true
kill $PID || true
