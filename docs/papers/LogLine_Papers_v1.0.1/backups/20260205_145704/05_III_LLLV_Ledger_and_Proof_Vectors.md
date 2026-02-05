---
id: llf.paper.lllv.v1
title: "Paper III — LLLV: The Retrieval Atom"
version: 1.0.1
kind: Canon/Spec
status: adopted
date: 2026-02-03
author: Dan (Voulezvous)
institution: The LogLine Foundation
lineage:
  - llf.paper.logline-protocol.v1
  - llf.paper.json-atomic.v1
gates_required: ["G1", "G2", "G3", "G4"]
thesis: "Memory becomes infrastructure only when retrieval is proof-carrying, content-addressed, and time-aware."
hash: ""
signer: ""
---

# Paper III — LLLV: The Retrieval Atom

**Verifiable Memory for Accountable Agents**

*Normative keywords per RFC 2119/8174 (MUST/SHOULD/MAY) apply.*

---

## The Story

**January 2025. A legal AI assistant. A malpractice lawsuit.**

The AI had recommended a specific legal strategy based on "relevant case law." The case was lost. The client sued. In discovery, the question arose: **why did the AI retrieve those specific cases?**

The vendor's answer: "The vector search returned them as most relevant."

The lawyer's follow-up: "Prove it."

Silence. The retrieval system was a black box. No one could explain why those three cases were returned instead of the four that would have won the case. No one could prove the index hadn't been tampered with. No one could reconstruct the decision.

**$4.2 million settlement. The AI couldn't explain its own memory.**

Now imagine a different architecture.

Every retrieval returns an **evidence chain**:

```json
{
  "type": "LLLV_TOPK_EVIDENCE_V1",
  "query_cid": "b3:7f3a9b2c...",
  "index_pack_cid": "b3:4d5e6f7a...",
  "results": [
    {
      "id": "case:smith_v_jones_2019",
      "dist": 0.1831,
      "proof": {
        "block": "POSTINGS",
        "merkle_path": ["b3:a1b2c3...", "b3:d4e5f6..."]
      }
    }
  ],
  "stats": {
    "algo": "hnsw",
    "params_cid": "b3:8f9a0b1c...",
    "visited": 812
  },
  "signature": "ed25519:responder_key"
}
```

This evidence can be verified offline, years later, by anyone with the pack file. The proof is cryptographic. The explanation is deterministic.

**The lawsuit becomes a hash comparison.**

This is LLLV.

---

## I. The Problem

Modern retrieval systems are black boxes. They return results without explanation. They claim relevance without proof. They mutate state without receipts.

When an agent retrieves information to make a decision:
- How do we know the retrieval was honest?
- How do we know the index wasn't tampered with?
- How do we reconstruct "why these results" after the fact?
- How do we audit memory at scale?

**Vector search without verification is not infrastructure. It is faith.**

---

## II. The Thesis

> **Retrieval becomes infrastructure when answers are provable artifacts, not opaque guesses.**

LLLV defines:
1. **Vector Capsule** — the atomic, content-addressed unit binding embedding to provenance
2. **Index Pack** — a portable, merkleized ANN index verifiable offline
3. **Top-K Evidence** — a proof bundle explaining why these K items were returned
4. **Temporal Narrative** — an append-only timeline turning time into evidence

When memory can be audited, it can be trusted.

---

## III. Install It Now

```bash
# Add to your Rust project
cargo add lllv

# Or install the CLI
cargo install logline-cli
```

```rust
use lllv::{IndexPack, VectorCapsule, TopKEvidence, Query};

fn main() -> Result<(), lllv::Error> {
    // Load a verifiable index pack
    let pack = IndexPack::load("knowledge_base.lllv.idx")?;

    // Verify pack integrity before using
    pack.verify()?;  // Checks all Merkle proofs

    // Query with evidence
    let query = Query::new("What is the statute of limitations?");
    let (results, evidence) = pack.search_with_evidence(&query, 10)?;

    // Evidence is verifiable offline
    assert!(evidence.verify(&pack.manifest())?);

    // Print results with provenance
    for result in results {
        println!(
            "Doc: {} (distance: {:.4}, proof: {})",
            result.id,
            result.distance,
            result.proof.merkle_root
        );
    }

    Ok(())
}
```

---

## IV. The Vector Capsule

The atomic unit of LLLV is the **Vector Capsule**: a signed, content-addressed envelope binding an embedding to its provenance.

### Wire Format

```
┌─────────────────────────────────────────────────────────┐
│  MAGIC    u16 [2]   0x4C56 (LV)                         │
│  VER      u8  [1]   Wire version (0x01)                 │
│  FLAGS    u8  [1]   Encrypted | Priority | ...          │
│  TS       u64 [8]   UTC nanoseconds                     │
│  CID      [32]      BLAKE3(payload)                     │
│  DIM      u16 [2]   Vector dimensionality               │
│  LEN      u32 [4]   Payload length                      │
│  SIG      [64]      Ed25519(header ‖ payload)           │
├─────────────────────────────────────────────────────────┤
│  PAYLOAD  var       Canonical manifest + vector bytes   │
└─────────────────────────────────────────────────────────┘
```

Total header: 114 bytes

### Implementation

```rust
// lllv/src/capsule.rs

use blake3::Hasher;
use ed25519_dalek::{Signature, SigningKey, VerifyingKey};

/// A Vector Capsule: signed, content-addressed embedding with provenance
#[derive(Debug, Clone)]
pub struct VectorCapsule {
    pub header: CapsuleHeader,
    pub manifest: CapsuleManifest,
    pub vector: Vec<f32>,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleHeader {
    pub magic: u16,      // 0x4C56
    pub version: u8,     // 0x01
    pub flags: u8,
    pub timestamp: u64,  // UTC nanoseconds
    pub cid: [u8; 32],   // BLAKE3(payload)
    pub dim: u16,
    pub payload_len: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleManifest {
    pub vector_id: String,
    pub source_uri: String,
    pub mime: String,
    pub content_hash: ContentAddress,
    pub dim: u16,
    pub quant: Quantization,
    pub encoder: EncoderInfo,
    pub policy_ref: String,
    pub ts_ingest: Timestamp,
}

impl VectorCapsule {
    /// Create a new capsule from content
    pub fn create(
        content: &[u8],
        source_uri: &str,
        encoder: &Encoder,
        signing_key: &SigningKey,
    ) -> Result<Self, CapsuleError> {
        // 1. Compute content hash
        let content_hash = ContentAddress::from_blake3(blake3::hash(content));

        // 2. Generate embedding
        let vector = encoder.encode(content)?;

        // 3. Create manifest
        let manifest = CapsuleManifest {
            vector_id: format!("vec:{}", &content_hash.to_string()[3..11]),
            source_uri: source_uri.to_string(),
            mime: mime_guess::from_path(source_uri)
                .first_or_octet_stream()
                .to_string(),
            content_hash,
            dim: vector.len() as u16,
            quant: Quantization::F32,
            encoder: encoder.info(),
            policy_ref: "tdln://policy/lllv.ingest@v1".to_string(),
            ts_ingest: Timestamp::now(),
        };

        // 4. Serialize payload (canonical)
        let payload = Self::serialize_payload(&manifest, &vector)?;

        // 5. Create header
        let header = CapsuleHeader {
            magic: 0x4C56,
            version: 0x01,
            flags: 0x00,
            timestamp: Timestamp::now().as_nanos(),
            cid: *blake3::hash(&payload).as_bytes(),
            dim: manifest.dim,
            payload_len: payload.len() as u32,
        };

        // 6. Sign
        let sig_material = Self::signature_material(&header, &payload);
        let signature = signing_key.sign(&sig_material);

        Ok(Self {
            header,
            manifest,
            vector,
            signature,
        })
    }

    /// Verify capsule integrity
    pub fn verify(&self, public_key: &VerifyingKey) -> Result<(), CapsuleError> {
        // 1. Verify CID
        let payload = Self::serialize_payload(&self.manifest, &self.vector)?;
        let computed_cid = blake3::hash(&payload);
        if computed_cid.as_bytes() != &self.header.cid {
            return Err(CapsuleError::CidMismatch);
        }

        // 2. Verify signature
        let sig_material = Self::signature_material(&self.header, &payload);
        public_key.verify(&sig_material, &self.signature)
            .map_err(|_| CapsuleError::InvalidSignature)?;

        Ok(())
    }

    fn serialize_payload(
        manifest: &CapsuleManifest,
        vector: &[f32],
    ) -> Result<Vec<u8>, CapsuleError> {
        let mut payload = json_atomic::canonize(manifest)?;
        for v in vector {
            payload.extend_from_slice(&v.to_le_bytes());
        }
        Ok(payload)
    }

    fn signature_material(header: &CapsuleHeader, payload: &[u8]) -> Vec<u8> {
        let mut material = Vec::new();
        material.extend_from_slice(b"lllv.capsule.v1");
        material.extend_from_slice(&header.magic.to_le_bytes());
        material.extend_from_slice(&[header.version, header.flags]);
        material.extend_from_slice(&header.timestamp.to_le_bytes());
        material.extend_from_slice(&header.cid);
        material.extend_from_slice(&header.dim.to_le_bytes());
        material.extend_from_slice(&header.payload_len.to_le_bytes());
        material.extend_from_slice(payload);
        material
    }
}
```

### Capsule Invariants

| ID | Guarantee |
|----|-----------|
| **VC-I1** | Metadata canonicalized via Paper II before sealing |
| **VC-I2** | `CID = BLAKE3(payload_bytes)` — mismatch rejected |
| **VC-I3** | Signature covers header and payload |
| **VC-I4** | Replay defense by `(source_uri, content_hash)` |
| **VC-I5** | Policy reference bound to capsule identity |
| **VC-I6** | Capsules are evidence, not capability |

---

## V. The Index Pack

An **Index Pack** is a portable file (`.lllv.idx`) containing everything needed for verifiable retrieval.

### Structure

```
┌─────────────────────────────────────────────────────────┐
│  PACK HEADER                                            │
│    MAGIC | VER | FLAGS | TS | PACK_CID | MANIFEST_SIG   │
├─────────────────────────────────────────────────────────┤
│  TABLE OF CONTENTS                                      │
│    n_blocks | [BlockDesc { kind, offset, len, cid }]    │
├─────────────────────────────────────────────────────────┤
│  BLOCKS                                                 │
│    ANN_PARAMS      Algorithm configuration              │
│    VECTOR_STORAGE  Quantized embeddings                 │
│    POSTINGS        Neighbor lists, levels               │
│    DOC_TABLE       Canonical metadata rows              │
│    STATS           Centroids, norms, histograms         │
│    MERKLE_INDEX    Block proofs                         │
│    MANIFEST        Signed canonical description         │
└─────────────────────────────────────────────────────────┘
```

### Implementation

```rust
// lllv/src/index_pack.rs

use std::collections::HashMap;

/// A verifiable index pack
pub struct IndexPack {
    pub manifest: PackManifest,
    blocks: HashMap<BlockKind, Block>,
    merkle_tree: MerkleTree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    #[serde(rename = "type")]
    pub kind: String,  // "LLLV_INDEX_PACK"
    pub ver: u32,
    pub created_ts: Timestamp,
    pub encoder: EncoderInfo,
    pub ann: AnnConfig,
    pub dim: u16,
    pub quant: Quantization,
    pub vector_count: u64,
    pub root: ContentAddress,  // Merkle root
    pub policy_ref: String,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnConfig {
    pub algo: String,  // "hnsw"
    pub space: String, // "cosine"
    pub params: HnswParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswParams {
    #[serde(rename = "M")]
    pub m: u32,
    pub ef_construction: u32,
    pub ef_search: u32,
}

impl IndexPack {
    /// Load and verify a pack from disk
    pub fn load(path: &str) -> Result<Self, PackError> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);

        // Read header
        let header = PackHeader::read(&mut reader)?;
        if header.magic != 0x4C4C5650 {  // "LLVP"
            return Err(PackError::InvalidMagic);
        }

        // Read table of contents
        let toc = TableOfContents::read(&mut reader)?;

        // Read blocks
        let mut blocks = HashMap::new();
        for desc in &toc.blocks {
            let block = Block::read(&mut reader, desc)?;
            blocks.insert(desc.kind, block);
        }

        // Reconstruct Merkle tree
        let merkle_tree = MerkleTree::from_blocks(&blocks)?;

        // Parse manifest
        let manifest_block = blocks.get(&BlockKind::Manifest)
            .ok_or(PackError::MissingManifest)?;
        let manifest: PackManifest = serde_json::from_slice(&manifest_block.data)?;

        // Verify Merkle root matches manifest
        if merkle_tree.root() != manifest.root {
            return Err(PackError::MerkleRootMismatch);
        }

        Ok(Self {
            manifest,
            blocks,
            merkle_tree,
        })
    }

    /// Verify pack integrity (all blocks)
    pub fn verify(&self) -> Result<(), PackError> {
        // Verify each block's CID
        for (kind, block) in &self.blocks {
            let computed_cid = ContentAddress::from_blake3(blake3::hash(&block.data));
            if computed_cid != block.desc.cid {
                return Err(PackError::BlockCidMismatch { kind: *kind });
            }
        }

        // Verify Merkle proofs
        self.merkle_tree.verify_all()?;

        // Verify manifest signature
        self.manifest.verify_signature()?;

        Ok(())
    }

    /// Search with evidence generation
    pub fn search_with_evidence(
        &self,
        query: &Query,
        k: usize,
    ) -> Result<(Vec<SearchResult>, TopKEvidence), PackError> {
        // Get HNSW index
        let hnsw = self.get_hnsw_index()?;

        // Encode query
        let query_vector = self.encode_query(query)?;
        let query_cid = ContentAddress::from_blake3(
            blake3::hash(&query_vector_bytes(&query_vector))
        );

        // Search
        let (results, visited) = hnsw.search_with_stats(&query_vector, k)?;

        // Generate evidence
        let evidence_results: Vec<_> = results.iter()
            .map(|r| {
                let merkle_path = self.merkle_tree.proof_for_doc(r.id)?;
                Ok(EvidenceResult {
                    id: r.id.clone(),
                    dist: r.distance,
                    proof: ResultProof {
                        block: BlockKind::Postings,
                        merkle_path,
                    },
                })
            })
            .collect::<Result<_, PackError>>()?;

        let evidence = TopKEvidence {
            kind: "LLLV_TOPK_EVIDENCE_V1".to_string(),
            query_cid,
            index_pack_cid: self.manifest.root.clone(),
            results: evidence_results,
            stats: SearchStats {
                algo: self.manifest.ann.algo.clone(),
                params_cid: self.get_params_cid()?,
                visited,
            },
            signature: Signature::pending(),  // Signed by responder
        };

        Ok((results, evidence))
    }
}
```

### Merkleization

- Each block individually hashed (BLAKE3)
- Merkle root covers ordered block list
- Inclusion proofs enable partial verification

```rust
// lllv/src/merkle.rs

pub struct MerkleTree {
    root: ContentAddress,
    nodes: Vec<MerkleNode>,
    block_indices: HashMap<BlockKind, usize>,
}

impl MerkleTree {
    /// Generate inclusion proof for a specific block
    pub fn proof_for_block(&self, kind: BlockKind) -> Result<Vec<ContentAddress>, MerkleError> {
        let idx = self.block_indices.get(&kind)
            .ok_or(MerkleError::BlockNotFound)?;

        let mut proof = Vec::new();
        let mut current = *idx;

        while current > 0 {
            let sibling = if current % 2 == 0 { current - 1 } else { current + 1 };
            if sibling < self.nodes.len() {
                proof.push(self.nodes[sibling].hash.clone());
            }
            current = (current - 1) / 2;
        }

        Ok(proof)
    }

    /// Verify an inclusion proof
    pub fn verify_proof(
        root: &ContentAddress,
        block_hash: &ContentAddress,
        proof: &[ContentAddress],
        index: usize,
    ) -> bool {
        let mut current = block_hash.clone();
        let mut idx = index;

        for sibling in proof {
            let combined = if idx % 2 == 0 {
                combine_hashes(&current, sibling)
            } else {
                combine_hashes(sibling, &current)
            };
            current = combined;
            idx /= 2;
        }

        current == *root
    }
}
```

---

## VI. Top-K Evidence

A retrieval result without evidence is an assertion. A retrieval result with evidence is a fact.

```rust
// lllv/src/evidence.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopKEvidence {
    #[serde(rename = "type")]
    pub kind: String,
    pub query_cid: ContentAddress,
    pub index_pack_cid: ContentAddress,
    pub results: Vec<EvidenceResult>,
    pub stats: SearchStats,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceResult {
    pub id: String,
    pub dist: f32,
    pub proof: ResultProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultProof {
    pub block: BlockKind,
    pub merkle_path: Vec<ContentAddress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    pub algo: String,
    pub params_cid: ContentAddress,
    pub visited: u64,
}

impl TopKEvidence {
    /// Verify evidence against a pack manifest
    pub fn verify(&self, manifest: &PackManifest) -> Result<(), EvidenceError> {
        // 1. Verify pack CID matches
        if self.index_pack_cid != manifest.root {
            return Err(EvidenceError::PackMismatch);
        }

        // 2. Verify each result's Merkle proof
        for result in &self.results {
            let valid = MerkleTree::verify_proof(
                &manifest.root,
                &self.compute_result_hash(result)?,
                &result.proof.merkle_path,
                self.result_index(result)?,
            );

            if !valid {
                return Err(EvidenceError::InvalidMerkleProof {
                    result_id: result.id.clone(),
                });
            }
        }

        // 3. Verify signature
        self.verify_signature()?;

        Ok(())
    }

    /// Verify offline - no network needed
    pub fn verify_offline(
        &self,
        pack_file: &str,
        public_key: &VerifyingKey,
    ) -> Result<(), EvidenceError> {
        let pack = IndexPack::load(pack_file)?;
        pack.verify()?;

        self.verify(&pack.manifest)?;

        // Verify responder signature
        let canonical = json_atomic::canonize(self)?;
        public_key.verify(&canonical, &self.signature)
            .map_err(|_| EvidenceError::InvalidSignature)?;

        Ok(())
    }
}
```

LLLV does not prove optimality. It proves **what was done** under declared parameters.

---

## VII. The Temporal Narrative

Time is not UI metadata. Time is evidence.

```rust
// lllv/src/narrative.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEvent {
    #[serde(rename = "type")]
    pub kind: String,  // "LLLV_NARRATIVE_V1"
    pub vector_id: String,
    pub source_uri: String,
    pub delta: DeltaType,
    pub prev_cid: Option<ContentAddress>,
    pub new_cid: ContentAddress,
    pub author_did: Did,
    pub policy_ref: String,
    pub ts: Timestamp,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeltaType {
    Initial,
    MinorEdit,
    MajorEdit,
    Retired,
}

/// A temporal narrative chain
pub struct Narrative {
    events: Vec<NarrativeEvent>,
}

impl Narrative {
    /// Load narrative for a vector
    pub fn for_vector(
        ledger: &Ledger,
        vector_id: &str,
    ) -> Result<Self, NarrativeError> {
        let events = ledger.query()
            .kind("LLLV_NARRATIVE_V1")
            .field("vector_id", vector_id)
            .order_by("ts", Ascending)
            .execute()?;

        // Verify chain integrity
        let mut prev_cid = None;
        for event in &events {
            if event.prev_cid != prev_cid {
                return Err(NarrativeError::BrokenChain {
                    event_ts: event.ts,
                });
            }
            prev_cid = Some(event.new_cid.clone());
        }

        Ok(Self { events })
    }

    /// Get state at a specific time
    pub fn state_at(&self, ts: Timestamp) -> Option<&NarrativeEvent> {
        self.events.iter()
            .filter(|e| e.ts <= ts)
            .last()
    }

    /// Is the vector retired?
    pub fn is_retired(&self) -> bool {
        self.events.last()
            .map(|e| matches!(e.delta, DeltaType::Retired))
            .unwrap_or(false)
    }
}

/// Temporal weighting for search results
pub fn temporal_weight(
    event_ts: Timestamp,
    now: Timestamp,
    tau: Duration,
) -> f64 {
    let age = now.duration_since(event_ts);
    (-age.as_secs_f64() / tau.as_secs_f64()).exp()
}

/// Combined scoring
pub fn combined_score(
    similarity: f64,
    event_ts: Timestamp,
    now: Timestamp,
    tau: Duration,
    policy_factor: f64,
) -> f64 {
    similarity * temporal_weight(event_ts, now, tau) * policy_factor
}
```

---

## VIII. CLI Usage

```bash
# Ingest documents into a pack
logline lllv ingest \
  --source ./documents/ \
  --encoder glassbox-v1 \
  --output knowledge.lllv.idx

# Output:
# Ingested 1,048,576 vectors
# Pack CID: b3:4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a...
# Manifest signed with: ed25519:ingest_key

# Verify a pack
logline lllv verify knowledge.lllv.idx

# Output:
# Pack integrity: VALID
# Merkle root: b3:4d5e6f7a...
# Block verification: 7/7 PASS
# Manifest signature: VALID

# Query with evidence
logline lllv query knowledge.lllv.idx \
  --query "What is the statute of limitations?" \
  --k 10 \
  --evidence-file results.evidence.json

# Output:
# Results:
#   1. doc:statute_guide_2024 (dist: 0.1831)
#   2. doc:legal_handbook_ch7 (dist: 0.1879)
#   ...
# Evidence written to results.evidence.json

# Verify evidence offline
logline lllv verify-evidence \
  --pack knowledge.lllv.idx \
  --evidence results.evidence.json

# Output:
# Query CID: b3:7f3a9b2c...
# Pack CID: b3:4d5e6f7a... (MATCH)
# Result proofs: 10/10 VALID
# Signature: VALID
# Evidence verification: PASS

# View temporal narrative
logline lllv narrative \
  --pack knowledge.lllv.idx \
  --vector-id "vec:9f3a8b2c"

# Output:
# Narrative for vec:9f3a8b2c
# 2026-01-15 10:30:00 INITIAL    b3:null -> b3:a1b2...
# 2026-01-20 14:15:00 MINOR_EDIT b3:a1b2... -> b3:c3d4...
# 2026-02-01 09:00:00 MAJOR_EDIT b3:c3d4... -> b3:e5f6...
# Chain integrity: VALID
```

---

## IX. Verification Procedure

Given a pack and evidence response:

```rust
pub fn verify_retrieval(
    pack_path: &str,
    evidence: &TopKEvidence,
    responder_key: &VerifyingKey,
) -> Result<VerificationResult, VerifyError> {
    // 1. Load and verify pack
    let pack = IndexPack::load(pack_path)?;
    pack.verify()?;

    // 2. Verify manifest signature
    pack.manifest.verify_signature()?;

    // 3. Verify evidence against pack
    evidence.verify(&pack.manifest)?;

    // 4. Verify responder signature
    let canonical = json_atomic::canonize(evidence)?;
    responder_key.verify(&canonical, &evidence.signature)
        .map_err(|_| VerifyError::InvalidResponderSignature)?;

    Ok(VerificationResult {
        pack_valid: true,
        evidence_valid: true,
        all_proofs_valid: true,
        signature_valid: true,
    })
}
```

This procedure requires only:
- Pack bytes
- Evidence bytes
- Public keys

**No network. No trust. Pure verification.**

---

## X. Conformance

| Test | Requirement |
|------|-------------|
| **CT-III-01** | Canonical sealing produces stable CIDs |
| **CT-III-02** | Pack manifest + block CIDs verify offline |
| **CT-III-03** | Evidence binds to query + pack identities |
| **CT-III-04** | Narrative events are signed, chained facts |
| **CT-III-05** | Harness equivalence across implementations |

```rust
#[cfg(test)]
mod conformance {
    #[test]
    fn ct_iii_01_stable_cids() {
        let content = b"test document content";
        let capsule_a = VectorCapsule::create(content, "test.txt", &encoder, &key)?;
        let capsule_b = VectorCapsule::create(content, "test.txt", &encoder, &key)?;

        // Same content = same CID (deterministic)
        assert_eq!(capsule_a.header.cid, capsule_b.header.cid);
    }

    #[test]
    fn ct_iii_02_offline_verification() {
        let pack = IndexPack::load("test.lllv.idx")?;

        // No network calls - pure local verification
        pack.verify()?;
    }

    #[test]
    fn ct_iii_03_evidence_binding() {
        let (_, evidence) = pack.search_with_evidence(&query, 10)?;

        // Evidence must bind to pack identity
        assert_eq!(evidence.index_pack_cid, pack.manifest.root);

        // And to query identity
        let query_cid = ContentAddress::from_blake3(blake3::hash(&query_bytes));
        assert_eq!(evidence.query_cid, query_cid);
    }
}
```

---

## XI. Integration

| Paper | Relationship |
|-------|--------------|
| **I — LogLine** | Retrieval operations emit LogLine records |
| **II — JSON✯Atomic** | All manifests, evidence, receipts are canonical |
| **IV — TDLN** | Policies govern ingest/query/verify |
| **V — SIRP** | Artifacts transport as identity-bound capsules |
| **VI — Chip** | Executors refuse effects without evidence |

---

## XII. The Invariant Connection

| Invariant | LLLV Implementation |
|-----------|---------------------|
| **I1** Integrity | Capsules, packs, evidence canonical and signed |
| **I2** Legality | Retrieval alone grants no capability |
| **I3** Attribution | DIDs and signatures on all artifacts |
| **I4** Reproducibility | Same pack + query → same results + evidence |
| **I5** Observability | Block proofs and query stats metrified |

---

## XIII. Conclusion

> **Memory becomes infrastructure when it can be audited.**

LLLV transforms retrieval from black-box magic into provable fact:

- **Capsules** bind embeddings to provenance
- **Index Packs** are verifiable offline
- **Evidence Chains** explain every result
- **Narratives** make time auditable

When the system can prove why it remembered what it remembered, memory becomes trustworthy infrastructure for accountable agents.

The lawsuit from the opening story? Under LLLV, it ends in one day. Load the pack file. Load the evidence file. Run `logline lllv verify-evidence`. Done.

---

## The Equation

```
Query + Pack + Evidence = Verifiable Retrieval

Proof replaces faith.
```

---

*Next: [Paper IV — TDLN](06_IV_TDLN_Deterministic_Translation_of_Natural_Language.md)*
