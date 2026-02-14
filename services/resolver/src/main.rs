
use resolver::http::{router, AppState};
use resolver::store::CardStore;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

fn load_or_create_keys() -> (SigningKey, VerifyingKey) {
    let keydir = std::path::Path::new("./keys");
    std::fs::create_dir_all(&keydir).ok();
    let sk_path = keydir.join("privkey.hex");
    let pk_path = keydir.join("pubkey.hex");
    if sk_path.exists() && pk_path.exists() {
        let sk_hex = std::fs::read_to_string(sk_path).unwrap();
        let pk_hex = std::fs::read_to_string(pk_path).unwrap();
        let sk_bytes = hex::decode(sk_hex.trim()).unwrap();
        let pk_bytes = hex::decode(pk_hex.trim()).unwrap();
        let sk = SigningKey::from_bytes(&sk_bytes.try_into().unwrap());
        let pk = VerifyingKey::from_bytes(&pk_bytes.try_into().unwrap()).unwrap();
        (sk, pk)
    } else {
        let mut csprng = OsRng;
        let sk = SigningKey::generate(&mut csprng);
        let pk = sk.verifying_key();
        std::fs::write(sk_path, hex::encode(sk.to_bytes())).ok();
        std::fs::write(pk_path, hex::encode(pk.to_bytes())).ok();
        (sk, pk)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let store = Arc::new(CardStore::new("./.cards"));
    let (sk, pk) = load_or_create_keys();
    let trust_schema: serde_json::Value = serde_json::from_str(
        include_str!("../../../schemas/tdln/trust@1.json")
    ).expect("schema");

    let state = AppState {
        store,
        base_url: "https://cert.tdln.foundry".to_string(),
        version: "resolver-v3".to_string(),
        signing_key: Arc::new(sk),
        verifying_key: pk,
        trust_schema,
    };
    let app = router(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    tracing::info!("resolver v3 listening on :8080");
    axum::serve(listener, app).await.unwrap();
}
