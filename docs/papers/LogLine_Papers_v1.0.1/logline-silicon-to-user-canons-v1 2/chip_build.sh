#!/usr/bin/env bash
# chip_build.sh — G1: canonize + BLAKE3 + (optional) Ed25519 signature
# Requirements:
#   - json_atomic_canonize (preferred) OR jq -S as a fallback for local checks
#   - b3sum or blake3 CLI
#   - OpenSSL 3.x for optional Ed25519 signing (openssl pkeyutl -sign -rawin)
# Usage:
#   chmod +x chip_build.sh
#   ./chip_build.sh /path/to/nv-as-hybrid-secure-gate_policy.register_v0.51.patched.json [ed25519_private.pem]
set -euo pipefail

JSON_IN="${1:-nv-as-hybrid-secure-gate_policy.register_v0.51.patched.json}"
PRIVKEY="${2:-}"

STEM="${JSON_IN%.*}"
CANON="${STEM}.canon.json"
OUT="${STEM}.signed.json"
HASH_FILE="${STEM}.blake3"
SIG="${STEM}.sig"
PUB="${STEM}.pub.pem"

echo "==> 1) Canonize payload"
if command -v json_atomic_canonize >/dev/null 2>&1; then
  json_atomic_canonize --input "$JSON_IN" --output "$CANON"
else
  echo "WARN: json_atomic_canonize not found; using jq -S fallback for local checks"
  jq -S '.' "$JSON_IN" > "$CANON"
fi

echo "==> 2) Compute BLAKE3"
if command -v b3sum >/dev/null 2>&1; then
  HASH=$(b3sum "$CANON" | awk '{print $1}')
elif command -v blake3 >/dev/null 2>&1; then
  HASH=$(blake3 "$CANON" | awk '{print $1}')
else
  echo "ERROR: Install b3sum or blake3 CLI"; exit 1
fi
echo "$HASH" > "$HASH_FILE"
echo "BLAKE3=$HASH"

echo "==> 3) Inject hash into JSON (.hash.value)"
jq --arg h "blake3:${HASH}" '.hash.algo="blake3" | .hash.value=$h' "$JSON_IN" > "$OUT"

echo "==> 4) PASS/FAIL: re-canonize + re-hash"
if command -v json_atomic_canonize >/dev/null 2>&1; then
  json_atomic_canonize --input "$OUT" --output "$CANON"
else
  jq -S '.' "$OUT" > "$CANON"
fi
if command -v b3sum >/dev/null 2>&1; then
  NEW=$(b3sum "$CANON" | awk '{print $1}')
else
  NEW=$(blake3 "$CANON" | awk '{print $1}')
fi
if [[ "blake3:${NEW}" == "$(jq -r '.hash.value' "$OUT")" ]]; then
  echo "PASS: canonical hash matches injected .hash.value"
else
  echo "FAIL: canonical hash mismatch"; exit 1
fi

if [[ -n "${PRIVKEY}" ]]; then
  echo "==> 5) Ed25519 signing (detached)"
  openssl pkey -in "$PRIVKEY" -pubout -out "$PUB" >/dev/null 2>&1
  # raw Ed25519 over canonical bytes
  openssl pkeyutl -sign -inkey "$PRIVKEY" -rawin -in "$CANON" -out "$SIG"
  echo "Verify signature:"
  openssl pkeyutl -verify -pubin -inkey "$PUB" -sigfile "$SIG" -rawin -in "$CANON"         && echo "PASS: signature valid" || { echo "FAIL: signature invalid"; exit 1; }
else
  echo "INFO: skipping Ed25519 signing (no private key provided)"
fi

echo "✅ Done. Artifacts:"
echo "  - $OUT          (hash filled)"
echo "  - $CANON        (canonical bytes)"
echo "  - $HASH_FILE    (blake3 digest)"
echo "  - $SIG          (detached signature; if key provided)"
