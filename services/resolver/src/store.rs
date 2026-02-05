use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::fs;
use crate::model::{DiamondCard, RunManifest};

#[derive(Clone)]
pub struct CardStore {
    base_path: PathBuf,
}

impl CardStore {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    fn card_dir(&self, did: &str) -> PathBuf {
        self.base_path.join(did)
    }

    pub async fn ensure_dir(&self, did: &str) -> Result<()> {
        let dir = self.card_dir(did);
        fs::create_dir_all(&dir).await.context("create card dir")?;
        Ok(())
    }

    pub async fn write_card(&self, did: &str, card: &DiamondCard) -> Result<()> {
        self.ensure_dir(did).await?;
        let path = self.card_dir(did).join("card.json");
        let json = serde_json::to_vec_pretty(card)?;
        fs::write(&path, json).await.context("write card.json")?;
        Ok(())
    }

    pub async fn read_card(&self, did: &str) -> Result<DiamondCard> {
        let path = self.card_dir(did).join("card.json");
        let bytes = fs::read(&path).await.context("read card.json")?;
        let card = serde_json::from_slice(&bytes)?;
        Ok(card)
    }

    pub async fn write_manifest(&self, did: &str, manifest: &RunManifest) -> Result<()> {
        self.ensure_dir(did).await?;
        let path = self.card_dir(did).join("run.manifest.json");
        let json = serde_json::to_vec_pretty(manifest)?;
        fs::write(&path, json).await.context("write manifest")?;
        Ok(())
    }

    pub async fn read_manifest(&self, did: &str) -> Result<RunManifest> {
        let path = self.card_dir(did).join("run.manifest.json");
        let bytes = fs::read(&path).await.context("read manifest")?;
        let manifest = serde_json::from_slice(&bytes)?;
        Ok(manifest)
    }

    pub async fn write_bundle(&self, did: &str, bundle_bytes: Vec<u8>) -> Result<()> {
        self.ensure_dir(did).await?;
        let path = self.card_dir(did).join("bundle.zip");
        fs::write(&path, bundle_bytes).await.context("write bundle.zip")?;
        Ok(())
    }

    pub async fn read_bundle(&self, did: &str) -> Result<Vec<u8>> {
        let path = self.card_dir(did).join("bundle.zip");
        let bytes = fs::read(&path).await.context("read bundle.zip")?;
        Ok(bytes)
    }
}