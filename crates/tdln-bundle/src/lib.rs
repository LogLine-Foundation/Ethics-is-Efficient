
use serde_json::Value;
use zip::write::FileOptions;
use std::io::Write;

pub fn build_bundle(card_json: &Value, manifest_json: &Value, signatures_json: &Value) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zipw = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let opts = FileOptions::default();
        let card = serde_json::to_vec_pretty(card_json).unwrap();
        let mani = serde_json::to_vec_pretty(manifest_json).unwrap();
        let sigs = serde_json::to_vec_pretty(signatures_json).unwrap();
        zipw.start_file("card.json", opts).unwrap();
        zipw.write_all(&card).unwrap();
        zipw.start_file("run.manifest.json", opts).unwrap();
        zipw.write_all(&mani).unwrap();
        zipw.start_file("signatures.sig", opts).unwrap();
        zipw.write_all(&sigs).unwrap();
        zipw.finish().unwrap();
    }
    buf
}
