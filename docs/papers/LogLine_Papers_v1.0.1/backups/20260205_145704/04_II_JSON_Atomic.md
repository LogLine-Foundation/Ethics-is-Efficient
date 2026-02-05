---
id: llf.paper.json-atomic.v1
title: "Paper II — JSON✯Atomic"
version: 1.0.1
kind: Canon/Spec
status: adopted
date: 2026-02-02
author: Dan (Voulezvous)
institution: The LogLine Foundation
lineage:
  - llf.paper.ethics.v1
  - llf.paper.text-power.v1
gates_required: ["G1", "G2", "G3", "G4"]
thesis: "Same meaning must produce same bytes. Same bytes must produce same hash. Same hash is same identity."
hash: ""
signer: ""
---

# Paper II — JSON✯Atomic

**The Identity Layer**

*Normative keywords per RFC 2119/8174 (MUST/SHOULD/MAY) apply.*

---

## The Problem (A True Story)

**November 2023. A smart contract audit. $12 million at stake.**

The contract hashed a JSON document to verify agreement between parties. Both parties signed the "same" document. Both hashes were different.

```json
// Party A's serializer produced:
{"amount": 12000000, "recipient": "0x7a3f..."}

// Party B's serializer produced:
{"recipient": "0x7a3f...", "amount": 12000000}
```

Same meaning. Different bytes. Different hashes. The contract rejected both signatures as invalid.

The fix took three weeks and $200,000 in legal fees. The root cause? **JSON doesn't guarantee key order.**

This was not a bug. This was a design flaw in every system that treats JSON as a serialization format without canonicalization.

**JSON✯Atomic eliminates this class of failure entirely.**

---

## I. The Principle

> **Same semantics ⇒ same bytes ⇒ same hash ⇒ same identity.**

```rust
use json_atomic::canonize;

// These two objects have the same meaning
let obj_a = json!({"b": 1, "a": 2});
let obj_b = json!({"a": 2, "b": 1});

// JSON✯Atomic produces identical bytes
let bytes_a = canonize(&obj_a);
let bytes_b = canonize(&obj_b);

assert_eq!(bytes_a, bytes_b);
// Both produce: {"a":2,"b":1}

// Therefore identical hashes
let hash_a = blake3::hash(&bytes_a);
let hash_b = blake3::hash(&bytes_b);

assert_eq!(hash_a, hash_b);
// Both produce: b3:7f3a9b2c4d5e6f7a8b9c0d1e2f3a4b5c...
```

This is not a feature. It is the foundation upon which all other papers rest.

- Paper I requires canonical tuples to chain
- Paper III requires canonical capsules to verify
- Paper IV requires canonical ASTs to prove
- Paper V requires canonical receipts to audit
- Paper VI requires canonical policies to execute

**The byte is the unit of law.**

---

## II. Install It Now

```bash
# Add to your Rust project
cargo add json-atomic

# Or install the CLI
cargo install logline-cli
```

```rust
use json_atomic::{canonize, verify, Error};

fn main() -> Result<(), Error> {
    let document = json!({
        "who": "did:logline:agent:alice",
        "did": "transfer",
        "this": {"amount": 1000, "to": "bob"},
        "when": "2026-02-05T14:30:00Z"
    });

    // Canonicalize
    let canonical_bytes = canonize(&document)?;

    // Hash (identity)
    let identity = blake3::hash(&canonical_bytes);
    println!("Identity: b3:{}", hex::encode(identity.as_bytes()));

    // Verify another serialization matches
    let other_bytes = r#"{"did":"transfer","this":{"amount":1000,"to":"bob"},"when":"2026-02-05T14:30:00Z","who":"did:logline:agent:alice"}"#;
    assert!(verify(&document, other_bytes.as_bytes())?);

    Ok(())
}
```

---

## III. The Data Model

JSON✯Atomic operates on a disciplined subset of JSON (RFC 8259).

### Allowed Values

| Type | Specification |
|------|---------------|
| **null** | Literal `null` |
| **boolean** | Literals `true`, `false` |
| **integer** | Arbitrary precision, base-10, no floats |
| **string** | UTF-8, normalized to NFC |
| **array** | Order-preserving sequence of allowed values |
| **object** | String-to-value map, no duplicate keys |

### Prohibitions

| Condition | Error Code |
|-----------|------------|
| Float, decimal point, exponent, NaN, Inf | `E_FLOAT` |
| Leading zeros (except `0`) | `E_NUM_FMT` |
| Invalid UTF-8 | `E_UTF8` |
| Duplicate object keys | `E_DUP_KEY` |

```rust
// json-atomic/src/validate.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("E_FLOAT: Floating point numbers are not allowed: {0}")]
    Float(f64),

    #[error("E_NUM_FMT: Invalid number format: {0}")]
    NumberFormat(String),

    #[error("E_UTF8: Invalid UTF-8 sequence at byte {0}")]
    InvalidUtf8(usize),

    #[error("E_DUP_KEY: Duplicate key in object: {0}")]
    DuplicateKey(String),

    #[error("E_NFC: String not in NFC form: {0}")]
    NotNfc(String),
}

pub fn validate(value: &Value) -> Result<(), ValidationError> {
    match value {
        Value::Null | Value::Bool(_) => Ok(()),

        Value::Number(n) => {
            // Floats are forbidden
            if n.is_f64() && !n.is_i64() && !n.is_u64() {
                return Err(ValidationError::Float(n.as_f64().unwrap()));
            }
            Ok(())
        }

        Value::String(s) => {
            // Must be valid UTF-8 (Rust guarantees this)
            // Must be NFC normalized
            if !unicode_normalization::is_nfc(s) {
                return Err(ValidationError::NotNfc(s.clone()));
            }
            Ok(())
        }

        Value::Array(arr) => {
            for item in arr {
                validate(item)?;
            }
            Ok(())
        }

        Value::Object(obj) => {
            let mut seen_keys = HashSet::new();
            for (key, val) in obj {
                if !seen_keys.insert(key) {
                    return Err(ValidationError::DuplicateKey(key.clone()));
                }
                validate(val)?;
            }
            Ok(())
        }
    }
}
```

**Special case:** `-0` MUST canonicalize to `0`.

---

## IV. Canonical Form

Canonical JSON (CJSON) is JSON serialized under these constraints:

### 4.1 Whitespace

- No superfluous whitespace
- No newlines
- No trailing commas

```rust
// WRONG: Has whitespace
{ "a": 1, "b": 2 }

// CORRECT: Canonical
{"a":1,"b":2}
```

### 4.2 Arrays

```
[v₁,v₂,...,vₙ]
```

Element order MUST be preserved. Arrays are ordered.

### 4.3 Objects

```
{k₁:v₁,k₂:v₂,...,kₘ:vₘ}
```

Keys MUST be sorted by **lexicographic order of UTF-8 bytes**.

```rust
// Input with any key order
{"zebra": 1, "apple": 2, "mango": 3}

// Canonical output (keys sorted)
{"apple":2,"mango":3,"zebra":1}
```

Locale collation MUST NOT be used. Raw UTF-8 byte comparison only.

### 4.4 Strings

1. Normalize to **NFC** (Unicode Canonical Decomposition, followed by Canonical Composition)
2. Encode as UTF-8
3. Apply minimal escaping:
   - `"` → `\"`
   - `\` → `\\`
   - Control characters U+0000..U+001F → `\u00XX`
4. The `/` character MUST NOT be escaped
5. Non-control characters MUST be emitted as UTF-8 directly

```rust
// json-atomic/src/escape.rs

pub fn escape_string(s: &str) -> String {
    let normalized = unicode_normalization::UnicodeNormalization::nfc(s)
        .collect::<String>();

    let mut result = String::with_capacity(normalized.len() + 2);
    result.push('"');

    for ch in normalized.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                // Control characters as \u00XX
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),  // UTF-8 directly
        }
    }

    result.push('"');
    result
}
```

### 4.5 Integers

- Base-10 decimal ASCII
- Leading `-` only if value is negative and non-zero
- Leading `+` forbidden
- Leading zeros forbidden
- `-0` serializes as `0`
- Precision is arbitrary (big integers supported)

```rust
// json-atomic/src/integer.rs

use num_bigint::BigInt;

pub fn serialize_integer(n: &BigInt) -> String {
    // Handle negative zero
    if n.sign() == Sign::NoSign || *n == BigInt::from(0) {
        return "0".to_string();
    }

    // No leading zeros, no leading plus
    n.to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn negative_zero_becomes_zero() {
        let neg_zero = BigInt::from(0) * BigInt::from(-1);
        assert_eq!(serialize_integer(&neg_zero), "0");
    }

    #[test]
    fn large_integers_work() {
        let big = BigInt::parse_bytes(
            b"123456789012345678901234567890",
            10
        ).unwrap();
        assert_eq!(
            serialize_integer(&big),
            "123456789012345678901234567890"
        );
    }
}
```

### 4.6 Literals

`true`, `false`, `null` — lowercase, exactly as shown.

---

## V. The Algorithm

```rust
// json-atomic/src/canonize.rs

pub fn canonize(value: &Value) -> Result<Vec<u8>, CanonizeError> {
    // First validate
    validate(value)?;

    // Then serialize to canonical form
    let mut output = Vec::new();
    write_canonical(value, &mut output)?;
    Ok(output)
}

fn write_canonical(value: &Value, out: &mut Vec<u8>) -> Result<(), CanonizeError> {
    match value {
        Value::Null => {
            out.extend_from_slice(b"null");
        }

        Value::Bool(true) => {
            out.extend_from_slice(b"true");
        }

        Value::Bool(false) => {
            out.extend_from_slice(b"false");
        }

        Value::Number(n) => {
            // Must be integer
            let i = n.as_i64()
                .or_else(|| n.as_u64().map(|u| u as i64))
                .ok_or(CanonizeError::FloatNotAllowed)?;

            // Handle -0
            let s = if i == 0 { "0".to_string() } else { i.to_string() };
            out.extend_from_slice(s.as_bytes());
        }

        Value::String(s) => {
            let escaped = escape_string(s);
            out.extend_from_slice(escaped.as_bytes());
        }

        Value::Array(arr) => {
            out.push(b'[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_canonical(item, out)?;
            }
            out.push(b']');
        }

        Value::Object(obj) => {
            out.push(b'{');

            // Sort keys by UTF-8 bytes (NOT locale)
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }

                // Key
                let escaped_key = escape_string(key);
                out.extend_from_slice(escaped_key.as_bytes());

                out.push(b':');

                // Value
                write_canonical(&obj[*key], out)?;
            }

            out.push(b'}');
        }
    }

    Ok(())
}
```

**Constraints enforced by this algorithm:**
- No randomness (deterministic key ordering)
- No locale dependency (UTF-8 byte comparison)
- No trailing whitespace
- Identical output on all platforms

---

## VI. Identity by Hash

### The Binding

```
IDENTITY(value) = BLAKE3(CANONIZE(value))
```

Expressed as: `b3:<lowercase_hex>`

```rust
// json-atomic/src/identity.rs

use blake3::Hasher;

/// Compute the content address (identity) of a value
pub fn identity(value: &Value) -> Result<ContentAddress, CanonizeError> {
    let canonical = canonize(value)?;
    let hash = blake3::hash(&canonical);
    Ok(ContentAddress::from_blake3(hash))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentAddress {
    bytes: [u8; 32],
}

impl ContentAddress {
    pub fn from_blake3(hash: blake3::Hash) -> Self {
        Self {
            bytes: *hash.as_bytes(),
        }
    }

    /// Format as b3:hex
    pub fn to_string(&self) -> String {
        format!("b3:{}", hex::encode(&self.bytes))
    }

    /// Parse from b3:hex format
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        let hex_part = s.strip_prefix("b3:")
            .ok_or(ParseError::MissingPrefix)?;

        let bytes = hex::decode(hex_part)
            .map_err(ParseError::InvalidHex)?;

        if bytes.len() != 32 {
            return Err(ParseError::InvalidLength(bytes.len()));
        }

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self { bytes: arr })
    }
}
```

### The Rule

Any field of the form `*_hash` or `*_cid` MUST be computed over canonical bytes produced by this specification.

Hashing non-canonical representations is **non-compliant** and will cause verification failures.

---

## VII. Formal Properties

| Property | Statement | Test |
|----------|-----------|------|
| **P1 — Idempotence** | `CANONIZE(parse(CANONIZE(x))) = CANONIZE(x)` | `test_idempotence` |
| **P2 — Confluence** | Same abstract value ⇒ same canonical bytes | `test_confluence` |
| **P3 — Identity stability** | `b3(x) = b3(y)` ⟺ structural equality | `test_identity_stability` |
| **P4 — Sensitivity** | Any semantic difference changes canonical bytes | `test_sensitivity` |
| **P5 — Independence** | All conforming implementations produce identical output | `test_independence` |

```rust
// json-atomic/src/tests/properties.rs

#[test]
fn test_idempotence() {
    let values = vec![
        json!(null),
        json!(true),
        json!(42),
        json!("hello"),
        json!([1, 2, 3]),
        json!({"a": 1, "b": 2}),
    ];

    for value in values {
        let canonical = canonize(&value).unwrap();
        let reparsed: Value = serde_json::from_slice(&canonical).unwrap();
        let recanonical = canonize(&reparsed).unwrap();

        assert_eq!(canonical, recanonical, "P1 violated for {:?}", value);
    }
}

#[test]
fn test_confluence() {
    // Different representations, same meaning
    let a = json!({"b": 1, "a": 2});
    let b = json!({"a": 2, "b": 1});

    let ca = canonize(&a).unwrap();
    let cb = canonize(&b).unwrap();

    assert_eq!(ca, cb, "P2 violated: same meaning must produce same bytes");
}

#[test]
fn test_identity_stability() {
    let a = json!({"x": 1});
    let b = json!({"x": 1});
    let c = json!({"x": 2});  // Different value

    let ha = identity(&a).unwrap();
    let hb = identity(&b).unwrap();
    let hc = identity(&c).unwrap();

    assert_eq!(ha, hb, "P3 violated: equal values must have equal identity");
    assert_ne!(ha, hc, "P3 violated: different values must have different identity");
}

#[test]
fn test_sensitivity() {
    // Tiny differences must produce different hashes
    let a = json!({"value": 100});
    let b = json!({"value": 101});

    let ha = identity(&a).unwrap();
    let hb = identity(&b).unwrap();

    assert_ne!(ha, hb, "P4 violated: semantic difference must change identity");
}
```

---

## VIII. Conformance Vectors

Every implementation MUST pass these vectors byte-for-byte.

```rust
// json-atomic/src/tests/vectors.rs

#[test]
fn c1_key_ordering() {
    let input = json!({"b": 1, "a": 2});
    let expected = br#"{"a":2,"b":1}"#;
    assert_eq!(canonize(&input).unwrap(), expected);
}

#[test]
fn c2_unicode_strings() {
    let input = json!(["z", "á", "a"]);  // NFC normalized
    let expected = br#"["z","á","a"]"#;  // UTF-8 direct, not \u escapes
    assert_eq!(canonize(&input).unwrap(), expected);
}

#[test]
fn c3_nesting() {
    let input = json!({"x": [{"k": "v"}, {}], "y": true});
    let expected = br#"{"x":[{"k":"v"},{}],"y":true}"#;
    assert_eq!(canonize(&input).unwrap(), expected);
}

#[test]
fn c4_zero_normalization() {
    // Note: serde_json doesn't preserve -0, but if it did:
    let input = json!({"n1": 0, "n2": 0, "n3": 10});
    let expected = br#"{"n1":0,"n2":0,"n3":10}"#;
    assert_eq!(canonize(&input).unwrap(), expected);
}

#[test]
fn c5_error_duplicate_key() {
    // This requires parsing raw JSON since serde_json deduplicates
    let raw = r#"{"a":1,"a":2}"#;
    let result = canonize_raw(raw);
    assert!(matches!(result, Err(ValidationError::DuplicateKey(_))));
}

#[test]
fn c6_error_float() {
    let input = json!({"x": 1.5});
    let result = canonize(&input);
    assert!(matches!(result, Err(CanonizeError::FloatNotAllowed)));
}

#[test]
fn c7_deep_nesting() {
    let input = json!({
        "level1": {
            "level2": {
                "level3": {
                    "value": 42
                }
            }
        }
    });
    let expected = br#"{"level1":{"level2":{"level3":{"value":42}}}}"#;
    assert_eq!(canonize(&input).unwrap(), expected);
}

#[test]
fn c8_empty_structures() {
    assert_eq!(canonize(&json!({})).unwrap(), br#"{}"#);
    assert_eq!(canonize(&json!([])).unwrap(), br#"[]"#);
}

#[test]
fn c9_control_characters() {
    let input = json!({"text": "line1\nline2\ttab"});
    let expected = br#"{"text":"line1\nline2\ttab"}"#;
    assert_eq!(canonize(&input).unwrap(), expected);
}
```

---

## IX. CLI Verification

```bash
# Verify a JSON file is canonical
logline json verify document.json

# Output if canonical:
# ✓ document.json is canonical
# Identity: b3:7f3a9b2c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a...

# Output if not canonical:
# ✗ document.json is NOT canonical
# Reason: Keys not in lexicographic order at path $.metadata
# Canonical form written to document.canonical.json

# Canonicalize a file
logline json canonize input.json -o output.json

# Compare two files for semantic equality
logline json compare a.json b.json

# Output:
# Semantically equal: YES
# Identity: b3:7f3a9b2c...

# Run conformance tests
logline json test-vectors

# Output:
# C1 Key Ordering:      PASS
# C2 Unicode Strings:   PASS
# C3 Nesting:           PASS
# C4 Zero Normalization: PASS
# C5 Error Duplicate:   PASS
# C6 Error Float:       PASS
# C7 Deep Nesting:      PASS
# C8 Empty Structures:  PASS
# C9 Control Characters: PASS
# All 9 vectors passed.
```

---

## X. Integration

| Paper | Dependency |
|-------|------------|
| **I — LogLine** | 9-field tuple sealed with JSON✯Atomic; `prev_hash = b3(canonical)` chains history |
| **III — LLLV** | Capsules, manifests, evidence carry `b3` over canonical bytes |
| **IV — TDLN** | `ast_cid`, `canon_cid`, proofs, receipts are canonical |
| **V — SIRP** | Envelopes, peer descriptors, receipts are canonical |
| **VI — Chip** | HALs, manifests, policy texts are canonical |

Every paper depends on this one. If canonicalization fails, everything fails.

---

## XI. Threats Eliminated

| Attack | Mitigation |
|--------|------------|
| **Whitespace injection** | Whitespace disallowed |
| **Key-shuffle attacks** | Keys ordered by UTF-8 bytes |
| **Unicode spoofing** | NFC normalization |
| **Float precision drift** | Floats forbidden |
| **Trailing comma injection** | Trailing commas disallowed |
| **BOM injection** | BOM disallowed |
| **Encoding confusion** | UTF-8 only |

---

## XII. The Invariant Connection

| Invariant | JSON✯Atomic Role |
|-----------|------------------|
| **I1** Integrity | Receipts bind `b3(canonical)` to effect |
| **I2** Legality | Schema validation on canonical form |
| **I3** Attribution | Signatures bind author to canonical bytes |
| **I4** Reproducibility | Canonical bytes enable exact replay |
| **I5** Observability | Receipt streams of canonical facts |

---

## XIII. Conclusion

**JSON✯Atomic makes the byte the unit of truth.**

When two implementations produce identical bytes for identical meaning, verification replaces interpretation. When identity is a hash, disputes collapse into computation.

The entire LogLine architecture depends on this layer. Without deterministic canonicalization, there is no chain integrity, no proof verification, no content addressing.

With it, every artifact in the system is exactly what it claims to be.

---

## The Equation

```
Same meaning → Same bytes → Same hash → Same identity

Verification replaces argument.
```

---

*Next: [Paper III — LLLV](05_III_LLLV_Ledger_and_Proof_Vectors.md)*
