
use axum::{routing::{post, get}, extract::{Path, State}, Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;
use time::OffsetDateTime;

use crate::model::{RunRequest, RunAccepted, ReceiptPreview};
use crate::store::CardStore;
use tdln_core::{cid_b3_hex, did_ulid, canonize, bundle_hash_card_manifest};
use tdln_bundle::build_bundle;
use tdln_wasm::run as wasm_run;
use ed25519_dalek::{SigningKey, VerifyingKey, Signer};
use jsonschema::{JSONSchema};
use base64::Engine;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<CardStore>,
    pub base_url: String,
    pub version: String,
    pub signing_key: Arc<SigningKey>,
    pub verifying_key: VerifyingKey,
    pub trust_schema: Value,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/v2/run", post(run))
        .route("/v2/cards/:did", get(get_card))
        .route("/v2/cards/:did/bundle.zip", get(get_bundle))
        .route("/trust/:did", get(get_trust_html))
        .with_state(state)
}

async fn run(State(state): State<AppState>, Json(req): Json<RunRequest>) -> Json<RunAccepted> {
    // 1) DID / CID / URL
    let did = did_ulid();
    let v = serde_json::to_value(&req).unwrap_or(json!({}));
    let cid = cid_b3_hex(&v);
    let realm = req.realm.clone();
    let url = format!("{}/{}/{}#{}", state.base_url, realm, did, cid);

    // 2) Try deterministic WASM if provided
    let mut decision = "ACK".to_string();
    let mut fuel = 0u64;
    if let Some(policy) = req.inputs.get("policy") {
        let kind = policy.get("kind").and_then(|s| s.as_str()).unwrap_or("");
        if kind == "wasm" {
            if let Some(b64) = policy.get("payload_b64").and_then(|s| s.as_str()) {
                if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64) {
                    match wasm_run(&bytes, &v, 5_000_000) {
                        Ok(out) => { decision = out.decision; fuel = out.fuel_consumed; },
                        Err(_e) => { decision = "ASK".into(); }
                    }
                } else {
                    decision = "ASK".into();
                }
            }
        }
    }

    let now = OffsetDateTime::now_utc();
    let now_str = now.to_string();

    // 3) Manifest
    let manifest = crate::model::RunManifest {
        did: did.clone(),
        input_cid: cid.clone(),
        request_cid: cid.clone(),
        request: Some(canonize(&v)),
        started_at: now_str.clone(),
        completed_at: Some(now_str.clone()),
        unit_cid: None,
    };

    // 4) Card (DiamondCard minimal shape)
    let card = crate::model::DiamondCard {
        schema: "tdln/trust@1".to_string(),
        realm: realm.clone(),
        object: "diamondcard".to_string(),
        did: did.clone(),
        card_id: "card:auto".to_string(),
        decision: decision.clone(),
        refs: crate::model::Refs {
            inputs: vec![],
            policy: crate::model::PolicyRef {
                cid: cid.clone(),
                rid: "auto".to_string(),
            },
        },
        runtime: crate::model::Runtime {
            engine: "wasmtime".to_string(),
            exec: "wasm".to_string(),
            version: Some(state.version.clone()),
            hash: None,
            fuel: Some(fuel),
        },
        poi: None,
        signatures: crate::model::Signatures {
            bundle_hash: "todo".to_string(),
            alg: "ed25519-blake3".to_string(),
            sig_hex: None,
        },
    };

    // Convert to JSON for schema validation and bundle building
    let card_json = serde_json::to_value(&card).unwrap();
    let manifest_json = serde_json::to_value(&manifest).unwrap();

    // 5) Schema validation
    let compiled = JSONSchema::compile(&state.trust_schema).unwrap();
    let res = compiled.validate(&card_json);
    if let Err(errors) = res {
        let msg = errors.map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
        return Json(RunAccepted{
            did: did.clone(),
            cid: cid.clone(),
            url: url.clone(),
            status: "ERROR".into(),
            receipt_preview: ReceiptPreview {
                realm: realm.clone(),
                decision: Some(format!("ERROR: {}", msg)),
            }
        });
    }

    // 6) Signature over bundle_hash(card, manifest)
    let bundle_hash = bundle_hash_card_manifest(&card_json, &manifest_json);
    let signing_key = &*state.signing_key;
    let sig = signing_key.sign(bundle_hash.as_bytes());
    let pubhex = hex::encode(state.verifying_key.to_bytes());
    let sighex = hex::encode(sig.to_bytes());
    let signatures = json!({
        "alg":"ed25519-blake3",
        "pubkey_hex": pubhex,
        "bundle_hash": bundle_hash.clone(),
        "sig_hex": sighex.clone()
    });

    // Update card with actual bundle hash and signature
    let mut final_card = card;
    final_card.signatures.bundle_hash = bundle_hash;
    final_card.signatures.sig_hex = Some(sighex);

    // Re-convert to JSON for bundle building (must match what was used for hash)
    let final_card_json = serde_json::to_value(&final_card).unwrap();

    // 7) Bundle - Write card/manifest/bundle
    let bundle = build_bundle(&final_card_json, &manifest_json, &signatures);
    
    // Write synchronously to ensure it's available immediately
    let _ = state.store.write_card(&did, &final_card).await;
    let _ = state.store.write_manifest(&did, &manifest).await;
    let _ = state.store.write_bundle(&did, bundle).await;

    // 8) Preview
    Json(RunAccepted {
        did,
        cid,
        url,
        status: "RUNNING".into(),
        receipt_preview: ReceiptPreview {
            realm,
            decision: Some(decision),
        }
    })
}

async fn get_card(State(state): State<AppState>, Path(did): Path<String>) -> Json<Value> {
    match state.store.read_card(&did).await {
        Ok(card) => Json(serde_json::to_value(&card).unwrap_or(json!({}))),
        Err(_) => Json(json!({"error":"not_found"})),
    }
}

async fn get_bundle(State(state): State<AppState>, Path(did): Path<String>) -> ([(axum::http::header::HeaderName, String);1], Vec<u8>) {
    use axum::http::header::CONTENT_TYPE;
    let data = state.store.read_bundle(&did).await.unwrap_or_default();
    ([ (CONTENT_TYPE, "application/zip".to_string()) ], data)
}



async fn get_trust_html(State(state): State<AppState>, Path(did): Path<String>) -> ([(axum::http::header::HeaderName, String);1], String) {
    use axum::http::header::CONTENT_TYPE;
    // Build URL first (used by both HTML and QR)
    let card = state.store.read_card(&did).await;
    let mani = state.store.read_manifest(&did).await;
    
    if card.is_err() || mani.is_err() {
        let html = "<html><body><h1>Not Found</h1><p>Card not found.</p></body></html>".to_string();
        return ([(CONTENT_TYPE, "text/html; charset=utf-8".to_string())], html);
    }
    
    let card = card.unwrap();
    let mani = mani.unwrap();
    
    // Now decision is a simple string
    let decision = &card.decision;
    let schema = &card.schema;
    let realm = &card.realm;
    let did_s = &card.did;
    let cid = &mani.request_cid;
    let bundle_hash = &card.signatures.bundle_hash;
    let url = format!("{}/{}/{}#{}", state.base_url, realm, did_s, cid);

    // Generate SVG QR inline (server-side) for the URL
    let svg_qr = match qrcode::QrCode::new(url.as_bytes()) {
        Ok(code) => {
            use qrcode::render::svg;
            code.render::<svg::Color>()
                .min_dimensions(160, 160)
                .build()
        },
        Err(_) => "<!-- qr generation failed -->".to_string()
    };

    let (badge, color) = match decision.as_str() {
        "ACK" => ("ACK", "#16a34a"),
        "ASK" => ("ASK", "#ca8a04"),
        "NACK" => ("NACK", "#dc2626"),
        _ => ("UNKNOWN", "#6b7280"),
    };

    let html = format!(r#"
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Resolver · {did}</title>
  <style>
    :root {{
      --bg: #0b0f14; --panel: #0f172a; --text: #e5e7eb; --muted:#94a3b8;
      --green:#16a34a; --amber:#ca8a04; --red:#dc2626; --blue:#38bdf8;
    }}
    * {{ box-sizing: border-box; }}
    body {{ margin:0; font-family: ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto; background:var(--bg); color:var(--text); }}
    .wrap {{ max-width: 980px; margin: 0 auto; padding: 28px; }}
    .grid {{ display:grid; grid-template-columns: 1fr 200px; gap: 18px; align-items: start; }}
    .card {{ background: linear-gradient(180deg, rgba(255,255,255,0.04), rgba(255,255,255,0.02)); border:1px solid rgba(255,255,255,0.08); border-radius: 16px; padding: 24px; }}
    .row {{ display:flex; gap:16px; flex-wrap: wrap; align-items: center; }}
    .badge {{ border-radius: 999px; padding: 6px 12px; font-weight: 700; letter-spacing: .04em; border:1px solid rgba(255,255,255,0.15); }}
    .muted {{ color: var(--muted); }}
    .mono {{ font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; }}
    .kvs {{ margin-top: 12px; display:grid; grid-template-columns: 180px 1fr; gap:6px 10px; align-items: center; }}
    a.btn {{ text-decoration:none; display:inline-block; padding:10px 14px; border-radius:10px; border:1px solid rgba(255,255,255,.12);}}
    a.btn:hover {{ border-color: rgba(255,255,255,.3);}}
    .actions {{ display:flex; gap:12px; flex-wrap: wrap; }}
    .pill {{ border-radius:12px; padding:4px 8px; border:1px dashed rgba(255,255,255,.15); }}
    .qrbox {{ background: rgba(255,255,255,.03); border:1px dashed rgba(255,255,255,.1); border-radius:12px; padding:10px; text-align:center; }}
  </style>
</head>
<body>
  <div class="wrap grid">
    <div class="card">
      <div class="row">
        <div style="font-size:22px;font-weight:800;">{schema}</div>
        <div class="badge" style="background:{color}22;color:{color};">{badge}</div>
        <span class="pill mono muted">{realm}</span>
      </div>

      <div class="kvs mono" style="margin-top:14px;">
        <div class="muted">DID</div><div>{did}</div>
        <div class="muted">CID</div><div>{cid}</div>
        <div class="muted">Bundle Hash</div><div>{bundle_hash}</div>
        <div class="muted">URL</div><div><a class="btn" href="{url}" target="_blank">{url}</a></div>
      </div>

      <div class="actions" style="margin-top:18px;">
        <a class="btn" href="/v2/cards/{did}/bundle.zip">Download Bundle (.zip)</a>
        <a class="btn" href="/v2/cards/{did}">View JSON</a>
      </div>
    </div>

    <div class="card qrbox">
      <div class="muted" style="margin-bottom:8px;">Scan to verify</div>
      {svg_qr}
      <div class="mono muted" style="font-size:12px; margin-top:8px; word-break:break-all;">{did}#{cid}</div>
    </div>
  </div>
</body>
</html>
"#, did=did_s, cid=cid, bundle_hash=bundle_hash, url=url, schema=schema, badge=badge, color=color, realm=realm, svg_qr=svg_qr);

    ([(CONTENT_TYPE, "text/html; charset=utf-8".to_string())], html)
}

