#!/bin/bash
set -e

echo "🚀 G1 Smoke Test"

# 0) Build
cargo build --workspace

# 1) Start resolver (background)
RUST_LOG=info cargo run -p resolver &
SERVER_PID=$!
sleep 2

# 2) Run astronaut tuple
mkdir -p .out
curl -s localhost:8080/v2/run \
  -H 'content-type: application/json' \
  -d @examples/tuples/logline-astronaut/request.json | tee .out/run.json

# 3) Extract DID
DID=$(jq -r '.did' .out/run.json)
echo "📍 DID: $DID"

# 4) Get card
curl -s "localhost:8080/v2/cards/$DID" | tee .out/card.json

# 5) Download bundle
curl -s -o .out/bundle.zip "localhost:8080/v2/cards/$DID/bundle.zip"

# 6) Verify
cargo run -p tdln-verify -- .out/bundle.zip
echo "✅ Verify exit=$?"

# 7) Check decision
DECISION=$(jq -r '.decision.type' .out/card.json)
if [ "$DECISION" = "ACK" ]; then
  echo "✅ PASS: Decision is ACK"
else
  echo "❌ FAIL: Expected ACK, got $DECISION"
  exit 1
fi

# Cleanup
kill $SERVER_PID
echo "🎉 G1 smoke test PASSED"