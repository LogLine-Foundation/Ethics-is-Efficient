#!/usr/bin/env bash
# tdln-smoke.sh — end-to-end smoke test for LogLine stack
# Checks health of all services, creates a Registry entity, issues a token, evaluates a decision, and prints telemetry JSON lines.
# Usage: chmod +x tdln-smoke.sh && ./tdln-smoke.sh
set -euo pipefail

ISSUER="https://issuer.logline.world"
REGISTRY="https://registry.logline.world"
TDLN="https://tdln.logline.world"
SHIELD="https://ops.logline.world"

jq --version >/dev/null 2>&1 || { echo "jq is required"; exit 1; }
curl --version >/dev/null 2>&1 || { echo "curl is required"; exit 1; }

pass() { echo -e "\033[32m✓\033[0m $1"; }
fail() { echo -e "\033[31m✗\033[0m $1"; exit 1; }

section() { echo -e "\n\033[34m==> $1\033[0m"; }

section "Health checks"
H_ISSUER=$(curl -fsS "$ISSUER/v1/health" | jq -c .) && pass "Issuer healthy" || fail "Issuer down"
H_REGISTRY=$(curl -fsS "$REGISTRY/v1/health" | jq -c .) && pass "Registry healthy" || fail "Registry down"
H_TDLN=$(curl -fsS "$TDLN/v1/health" | jq -c .) && pass "TDLN Engine healthy" || fail "TDLN down"
H_SHIELD=$(curl -fsS "$SHIELD/health" | jq -c .) && pass "Shield healthy" || fail "Shield down"

section "Create persona in Registry (append-only)"
PERSONA=$(curl -fsS -X POST "$REGISTRY/v1/entity" \
  -H 'Content-Type: application/json' \
  -d '{"kind":"persona","payload":{"name":"Smoke User","email":"smoke@example.com","tier":"basic","country":"PT"}}')
PID=$(echo "$PERSONA" | jq -r .id)
[[ -n "$PID" && "$PID" != "null" ]] && pass "Persona created: $PID" || fail "Persona creation failed"

section "Get auth token from Issuer"
TOKEN_RESP=$(curl -fsS -X POST "$ISSUER/v1/session/start" \
  -H 'Content-Type: application/json' \
  -d '{"email":"smoke@example.com"}')
TOKEN=$(echo "$TOKEN_RESP" | jq -r .token)
SUBJECT=$(echo "$TOKEN_RESP" | jq -r .subject // "smoke@example.com")
[[ -n "$TOKEN" && "$TOKEN" != "null" ]] && pass "Token issued for $SUBJECT" || fail "Token issuance failed"

section "TDLN evaluate (inputs wrapper)"
START=$(date +%s%3N)
DECISION=$(curl -fsS -X POST "$TDLN/v1/evaluate" \
  -H 'Content-Type: application/json' \
  -d "{\"inputs\":{\"actor\":{\"id\":\"$PID\",\"email\":\"smoke@example.com\",\"tier\":\"basic\",\"country\":\"PT\"},\"action\":\"api_call\",\"resource\":{\"endpoint\":\"/v1/data\"}}}")
END=$(date +%s%3N)
LAT_MS=$((END-START))
pass "TDLN decision received in ${LAT_MS}ms"

SPAN=$(echo "$DECISION" | jq -r '.receipt_span.span_id // empty')
OUTCOME=$(echo "$DECISION" | jq -r '.decision // "allow"')

section "JWKS"
JWKS=$(curl -fsS "$ISSUER/v1/keys.json" | jq -c .) && pass "JWKS fetched" || fail "JWKS fetch failed"

section "Telemetry lines (copy/paste to logs)"
TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Decision metric line
echo "{\"ts\":\"$TS\",\"service\":\"tdlne\",\"kind\":\"decision\",\"node_id\":\"tdlne\",\"payload\":{\"chip_id\":\"chip.slice-gate.health.v1\",\"slice_slug\":\"health.v1\",\"decision\":\"$OUTCOME\",\"obligations\":[],\"span_id\":\"$SPAN\",\"inputs_hash\":\"$(echo \"$DECISION\" | jq -r '.receipt_span.inputs_hash // empty')\",\"actor_tier\":\"basic\",\"actor_country\":\"PT\",\"latency_ms\":$LAT_MS,\"cold_start_estimate_s\":null,\"sla_cold_start_max_s\":null},\"units\":{\"timebase_ns\":1,\"energy_unit\":\"J\",\"cost_unit\":\"USD\"}}"

# Registry write metric line
echo "{\"ts\":\"$TS\",\"service\":\"registry\",\"kind\":\"entity_write\",\"node_id\":\"registry\",\"payload\":{\"entity_kind\":\"persona\",\"status\":\"created\",\"entity_id\":\"$PID\"},\"units\":{\"timebase_ns\":1,\"energy_unit\":\"J\",\"cost_unit\":\"USD\"}}"

# Issuer token line
ROLE=$(echo "$TOKEN_RESP" | jq -r '.roles[0] // "user"')
echo "{\"ts\":\"$TS\",\"service\":\"issuer\",\"kind\":\"auth_issue\",\"node_id\":\"issuer\",\"payload\":{\"role\":\"$ROLE\"},\"units\":{\"timebase_ns\":1,\"energy_unit\":\"J\",\"cost_unit\":\"USD\"}}"

echo ""
pass "Smoke test complete"
