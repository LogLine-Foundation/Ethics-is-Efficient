
use serde_json::{Value, Map};
use std::collections::BTreeMap;

pub fn canonize(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut sorted = BTreeMap::new();
            for (k, val) in m.iter() {
                sorted.insert(k.clone(), canonize(val));
            }
            let mut out = Map::new();
            for (k, val) in sorted.into_iter() {
                out.insert(k, val);
            }
            Value::Object(out)
        }
        Value::Array(a) => Value::Array(a.iter().map(canonize).collect()),
        _ => v.clone()
    }
}

pub fn cid_b3_hex(v: &Value) -> String {
    let canon = canonize(v);
    let s = serde_json::to_string(&canon).unwrap();
    let h = blake3::hash(s.as_bytes());
    format!("b3:{}", h.to_hex())
}

pub fn did_ulid() -> String {
    format!("did:tdln:{}", ulid::Ulid::new().to_string())
}


pub fn blake3_hex_bytes(data: &[u8]) -> String {
    let h = blake3::hash(data);
    format!("b3:{}", h.to_hex())
}

pub fn bundle_hash_card_manifest(card: &Value, manifest: &Value) -> String {
    let cc = serde_json::to_vec(&canonize(card)).unwrap();
    let mm = serde_json::to_vec(&canonize(manifest)).unwrap();
    let mut v = Vec::with_capacity(cc.len()+1+mm.len());
    v.extend_from_slice(&cc);
    v.push(0);
    v.extend_from_slice(&mm);
    blake3_hex_bytes(&v)
}
