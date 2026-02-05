use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RunRequest {
    pub realm: String,
    pub intent: String,
    pub inputs: serde_json::Value,
    #[serde(default)]
    pub options: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunAccepted {
    pub did: String,
    pub cid: String,
    pub url: String,
    pub status: String,
    pub receipt_preview: ReceiptPreview,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReceiptPreview {
    pub realm: String,
    pub decision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiamondCard {
    pub schema: String,
    pub realm: String,
    pub object: String,
    pub did: String,
    pub card_id: String,
    pub decision: Decision,
    pub refs: Refs,
    pub runtime: Runtime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poi: Option<serde_json::Value>,
    pub signatures: Signatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    #[serde(rename = "type")]
    pub decision_type: String, // ACK | ASK | NACK
    pub no_hitl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refs {
    pub inputs: Vec<InputRef>,
    pub policy: PolicyRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputRef {
    pub cid: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRef {
    pub cid: String,
    pub rid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runtime {
    pub engine: String,
    pub exec: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signatures {
    pub bundle_hash: String,
    pub alg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunManifest {
    pub did: String,
    pub input_cid: String,
    pub request_cid: String,
    pub started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_cid: Option<String>,
}