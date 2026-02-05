use crate::model::{Decision, DiamondCard, InputRef, PolicyRef, Refs, Runtime, Signatures, RunManifest};
use anyhow::Result;
use ulid::Ulid;

pub fn generate_diamond_card(
    manifest: &RunManifest,
    request_cid: &str,
    realm: &str,
) -> Result<DiamondCard> {
    let card_id = format!("card:{}", Ulid::new());

    // Minimal decision logic (ACK for now)
    let decision = Decision {
        decision_type: "ACK".to_string(),
        no_hitl: true,
    };

    let refs = Refs {
        inputs: vec![InputRef {
            cid: manifest.input_cid.clone(),
            name: "request.json".to_string(),
            size: None,
        }],
        policy: PolicyRef {
            cid: "b3:00000000000000000000000000000000".to_string(), // Placeholder
            rid: format!("rid:policy/{}/default-v1", realm),
        },
    };

    let runtime = Runtime {
        engine: "engine-core@preview".to_string(),
        exec: "wasm@preview".to_string(),
        version: Some("0.1.0".to_string()),
        hash: None,
    };

    let signatures = Signatures {
        bundle_hash: "pending".to_string(),
        alg: "Ed25519-BLAKE3".to_string(),
        sig_hex: None,
    };

    Ok(DiamondCard {
        schema: format!("tdln/{}@1-preview", realm),
        realm: realm.to_string(),
        object: "diamondcard".to_string(),
        did: manifest.did.clone(),
        card_id,
        decision,
        refs,
        runtime,
        poi: None,
        signatures,
    })
}