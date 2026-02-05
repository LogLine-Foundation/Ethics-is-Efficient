
use clap::Parser;
use zip::ZipArchive;
use std::fs::File;
use std::io::Read;

#[derive(Parser)]
struct Args {
    bundle: String,
    #[arg(long)]
    pubkey_hex: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let f = File::open(&args.bundle)?;
    let mut zip = ZipArchive::new(f)?;

    // Read entries
    let mut card = String::new();
    zip.by_name("card.json")?.read_to_string(&mut card)?;
    let card_v: serde_json::Value = serde_json::from_str(&card)?;

    let mut manifest = String::new();
    zip.by_name("run.manifest.json")?.read_to_string(&mut manifest)?;
    let manifest_v: serde_json::Value = serde_json::from_str(&manifest)?;

    let mut sigs = String::new();
    zip.by_name("signatures.sig")?.read_to_string(&mut sigs)?;
    let sigs_v: serde_json::Value = serde_json::from_str(&sigs)?;

    // recompute bundle_hash
    let cc = serde_json::to_vec(&card_v)?;
    let mm = serde_json::to_vec(&manifest_v)?;
    let mut v = Vec::with_capacity(cc.len()+1+mm.len());
    v.extend_from_slice(&cc);
    v.push(0);
    v.extend_from_slice(&mm);
    let bh = blake3::hash(&v);
    let bh_hex = format!("b3:{}", bh.to_hex());

    let claimed = sigs_v.get("bundle_hash").and_then(|s| s.as_str()).unwrap_or("?");
    if claimed != bh_hex {
        eprintln!("BUNDLE HASH MISMATCH: recomputed={} claimed={}", bh_hex, claimed);
        std::process::exit(1);
    }

    // verify signature
    let sig_hex = sigs_v.get("sig_hex").and_then(|s| s.as_str()).unwrap_or("");
    let pub_hex = args.pubkey_hex.or_else(|| sigs_v.get("pubkey_hex").and_then(|s| s.as_str()).map(|s| s.to_string())).unwrap_or_default();

    let sig_bytes = hex::decode(sig_hex)?;
    let pk_bytes = hex::decode(pub_hex)?;

    let vk = ed25519_dalek::VerifyingKey::from_bytes(&pk_bytes.try_into().unwrap())?;
    let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes.try_into().unwrap());

    vk.verify(bh_hex.as_bytes(), &sig)
        .map_err(|e| anyhow::anyhow!("signature verify failed: {e}"))?;

    println!("OK schema={} decision={} fuel={} bundle_hash={}",
        card_v.get("schema").and_then(|s| s.as_str()).unwrap_or("?"),
        card_v.get("decision").and_then(|s| s.as_str()).unwrap_or("?"),
        card_v.pointer("/runtime/fuel").and_then(|n| n.as_u64()).unwrap_or(0),
        bh_hex
    );
    Ok(())
}
