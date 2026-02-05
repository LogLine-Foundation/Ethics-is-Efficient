---
id: llf.paper.sirp.v1
title: "Paper V — SIRP: The Network Atom"
version: 1.0.1
kind: Canon/Spec
status: adopted
date: 2026-02-05
author: Dan Voulez
institution: The LogLine Foundation
lineage:
  - llf.paper.logline-protocol.v1
  - llf.paper.json-atomic.v1
  - llf.paper.lllv.v1
  - llf.paper.tdln.v1
gates_required: ["G1", "G2", "G3", "G4"]
thesis: "Identity must be routed, not locations. A packet is an accountable artifact when its meaning is content-addressed, signed, and receipted."
hash: ""
signer: ""
---

# Paper V — SIRP: The Network Atom

**Secure Intent Routing Protocol**

*Normative keywords per RFC 2119/8174 (MUST/SHOULD/MAY) apply.*

---

## The Story

**December 2024. A distributed AI system. A catastrophic failure.**

Three agents were supposed to coordinate a complex financial operation. Agent A sent instructions to Agent B. Agent B claimed it never received them. Agent C executed based on what it *thought* Agent B had decided. The result: $18 million in erroneous trades.

The post-mortem was brutal:
- TCP delivered the packets (probably)
- No proof of delivery existed
- No proof of receipt existed
- Each agent's version of events contradicted the others
- The network was a black box

**"We can't prove who said what to whom."**

Now imagine a different architecture.

Every message between agents is a **Capsule**—signed, content-addressed, receipted:

```json
{
  "magic": "0x5199",
  "ver": 1,
  "cid": "b3:7f3a9b2c4d5e6f7a8b9c0d1e2f3a4b5c...",
  "sender_did": "did:logline:agent:A",
  "payload": {
    "kind": "instruction",
    "action": "execute_trade",
    "params": {"symbol": "AAPL", "quantity": 1000}
  },
  "signature": "ed25519:..."
}
```

When Agent B receives this Capsule, it signs a **Delivery Receipt**:

```json
{
  "kind": "sirp.receipt.delivery.v1",
  "capsule_cid": "b3:7f3a9b2c...",
  "sender_did": "did:logline:agent:A",
  "receiver_did": "did:logline:agent:B",
  "ts_received": "2024-12-15T14:23:07.847Z",
  "outcome": "DELIVERED",
  "signature": "ed25519:agent_B_key"
}
```

Agent B cannot later claim it didn't receive the instruction. The receipt exists. The signature is verifiable. The dispute collapses into a hash comparison.

**The network becomes an audit trail.**

This is SIRP.

---

## I. The Problem

Networks route packets by location. But in a world of accountable agents, location is irrelevant. What matters is:
- Who is speaking?
- What do they intend?
- Can we prove delivery?

Traditional networks provide none of this:
- IP addresses change
- Packets can be forged
- Delivery is best-effort
- Routing leaves no audit trail

**When meaning must travel, it must travel as an accountable artifact.**

---

## II. The Thesis

> **Route by identity, not topology. Receipt every hop. Prove delivery.**

SIRP defines:
1. **Capsule** — the atomic, signed, content-addressed message
2. **Discovery** — identity-bound DHT mapping DIDs to endpoints
3. **TAL** — transport abstraction (UDP, QUIC, WebSocket, TCP)
4. **Receipts** — cryptographic proof of relay and delivery

**SIRP is to the network what the Gate is to execution: nothing meaningful happens without artifacts.**

---

## III. Install It Now

```bash
# Add to your Rust project
cargo add sirp

# Or install the CLI
cargo install logline-cli
```

```rust
use sirp::{Capsule, Node, Discovery, Receipt};

#[tokio::main]
async fn main() -> Result<(), sirp::Error> {
    // Create a SIRP node
    let node = Node::new(
        "did:logline:agent:alice",
        signing_key,
    ).await?;

    // Send a capsule
    let capsule = Capsule::new(
        "did:logline:agent:bob",
        json!({"action": "transfer", "amount": 1000}),
    )?;

    let receipt = node.send(capsule).await?;

    // Verify delivery
    assert!(matches!(receipt.outcome, Outcome::Delivered));
    println!("Delivered! Receipt CID: {}", receipt.cid());

    Ok(())
}
```

---

## IV. The Capsule

The atomic unit of SIRP transport is the **Capsule**: a self-contained, signed, content-addressed message.

### Wire Format

```
┌─────────────────────────────────────────────────────────┐
│  MAGIC    u16 [2]     0x5199 (Protocol ID)              │
│  VER      u8  [1]     0x01 (Wire version)               │
│  FLAGS    u8  [1]     Encrypted | ReceiptRequired | ... │
│  TTL      u8  [1]     Hop limit                         │
│  CID      [32]        BLAKE3(PAYLOAD)                   │
│  INTENT   u64 [8]     Routing hint (non-authorizing)    │
│  TS       u64 [8]     UTC nanoseconds                   │
│  LEN      u32 [4]     Payload length                    │
│  SIG      [64]        Ed25519(domain ‖ header ‖ payload)│
├─────────────────────────────────────────────────────────┤
│  PAYLOAD  var         Canonical JSON or CipherEnvelope  │
└─────────────────────────────────────────────────────────┘
```

Total header: 121 bytes

### Implementation

```rust
// sirp/src/capsule.rs

use blake3::Hasher;
use ed25519_dalek::{Signature, SigningKey, VerifyingKey};

/// A Capsule: the atomic unit of SIRP transport
#[derive(Debug, Clone)]
pub struct Capsule {
    pub header: CapsuleHeader,
    pub payload: Payload,
    pub signature: Signature,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CapsuleHeader {
    pub magic: u16,      // 0x5199
    pub version: u8,     // 0x01
    pub flags: u8,
    pub ttl: u8,
    pub cid: [u8; 32],   // BLAKE3(payload)
    pub intent: u64,     // Routing hint
    pub timestamp: u64,  // UTC nanoseconds
    pub payload_len: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Payload {
    Canonical(serde_json::Value),
    Encrypted(CipherEnvelope),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipherEnvelope {
    pub nonce: [u8; 24],
    pub aad: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

impl Capsule {
    pub const MAGIC: u16 = 0x5199;
    pub const VERSION: u8 = 0x01;
    pub const DOMAIN: &'static [u8] = b"sirp.cap.v1";

    /// Create a new capsule
    pub fn new(
        recipient: &Did,
        payload: serde_json::Value,
        signing_key: &SigningKey,
    ) -> Result<Self, CapsuleError> {
        // Canonicalize payload
        let payload_bytes = json_atomic::canonize(&payload)?;
        let cid = blake3::hash(&payload_bytes);

        // Compute intent (routing hint from action)
        let intent = Self::compute_intent(&payload);

        let header = CapsuleHeader {
            magic: Self::MAGIC,
            version: Self::VERSION,
            flags: 0,
            ttl: 64,  // Default hop limit
            cid: *cid.as_bytes(),
            intent,
            timestamp: Timestamp::now().as_nanos(),
            payload_len: payload_bytes.len() as u32,
        };

        // Sign with domain separation
        let sig_material = Self::signature_material(&header, &payload_bytes);
        let signature = signing_key.sign(&sig_material);

        Ok(Self {
            header,
            payload: Payload::Canonical(payload),
            signature,
        })
    }

    /// Create an encrypted capsule
    pub fn new_encrypted(
        recipient: &Did,
        payload: serde_json::Value,
        recipient_public_key: &x25519_dalek::PublicKey,
        signing_key: &SigningKey,
    ) -> Result<Self, CapsuleError> {
        // Canonicalize
        let payload_bytes = json_atomic::canonize(&payload)?;

        // Encrypt with X25519 + ChaCha20-Poly1305
        let envelope = encrypt_payload(&payload_bytes, recipient_public_key)?;
        let envelope_bytes = serde_json::to_vec(&envelope)?;

        let cid = blake3::hash(&envelope_bytes);
        let intent = Self::compute_intent(&payload);

        let header = CapsuleHeader {
            magic: Self::MAGIC,
            version: Self::VERSION,
            flags: 0x01,  // ENCRYPTED flag
            ttl: 64,
            cid: *cid.as_bytes(),
            intent,
            timestamp: Timestamp::now().as_nanos(),
            payload_len: envelope_bytes.len() as u32,
        };

        let sig_material = Self::signature_material(&header, &envelope_bytes);
        let signature = signing_key.sign(&sig_material);

        Ok(Self {
            header,
            payload: Payload::Encrypted(envelope),
            signature,
        })
    }

    /// Verify capsule integrity
    pub fn verify(&self, public_key: &VerifyingKey) -> Result<(), CapsuleError> {
        // 1. Verify CID
        let payload_bytes = match &self.payload {
            Payload::Canonical(v) => json_atomic::canonize(v)?,
            Payload::Encrypted(e) => serde_json::to_vec(e)?,
        };

        let computed_cid = blake3::hash(&payload_bytes);
        if computed_cid.as_bytes() != &self.header.cid {
            return Err(CapsuleError::CidMismatch);
        }

        // 2. Verify signature with domain separation
        let sig_material = Self::signature_material(&self.header, &payload_bytes);
        public_key.verify(&sig_material, &self.signature)
            .map_err(|_| CapsuleError::InvalidSignature)?;

        Ok(())
    }

    fn signature_material(header: &CapsuleHeader, payload: &[u8]) -> Vec<u8> {
        let mut material = Vec::with_capacity(Self::DOMAIN.len() + 57 + payload.len());
        material.extend_from_slice(Self::DOMAIN);

        // Header bytes (first 57 bytes)
        let header_bytes: [u8; 57] = unsafe {
            std::mem::transmute_copy(header)
        };
        material.extend_from_slice(&header_bytes);

        material.extend_from_slice(payload);
        material
    }

    fn compute_intent(payload: &serde_json::Value) -> u64 {
        // Intent = first 64 bits of BLAKE3("namespace.action")
        let action = payload.get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let hash = blake3::hash(action.as_bytes());
        u64::from_le_bytes(hash.as_bytes()[..8].try_into().unwrap())
    }

    /// Get content address
    pub fn cid(&self) -> ContentAddress {
        ContentAddress::from_bytes(&self.header.cid)
    }
}
```

### Capsule Invariants

| ID | Guarantee |
|----|-----------|
| **CP-I1** | Any byte change invalidates SIG |
| **CP-I2** | CID MUST equal BLAKE3(PAYLOAD) |
| **CP-I3** | TTL decremented on relay; drop at zero |
| **CP-I4** | Replay defense by (sender_did, CID) |
| **CP-I5** | INTENT is hint only; never authorizes |

**Critical:** INTENT guides queue priority but NEVER authorizes effects. Authorization lives in Gate receipts (Paper IV).

---

## V. Discovery

Discovery maps DIDs to network endpoints using an identity-bound DHT.

```rust
// sirp/src/discovery.rs

use libp2p::kad::{Kademlia, KademliaConfig, store::MemoryStore};

/// Identity-bound DHT for peer discovery
pub struct Discovery {
    kad: Kademlia<MemoryStore>,
    local_did: Did,
}

/// A signed peer descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerDescriptor {
    pub did: Did,
    pub endpoints: Vec<Endpoint>,
    pub relay_did: Option<Did>,
    pub timestamp: Timestamp,
    pub pubkey: PublicKey,
    pub kid: String,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Endpoint {
    Udp(SocketAddr),
    Quic(String),
    WebSocket(String),
    Tcp(SocketAddr),
}

impl Discovery {
    /// Create a new discovery instance
    pub fn new(did: Did, signing_key: &SigningKey) -> Result<Self, DiscoveryError> {
        let peer_id = did_to_peer_id(&did)?;

        let config = KademliaConfig::default();
        let store = MemoryStore::new(peer_id);
        let kad = Kademlia::with_config(peer_id, store, config);

        Ok(Self {
            kad,
            local_did: did,
        })
    }

    /// Publish our peer descriptor
    pub async fn publish(&mut self, descriptor: PeerDescriptor) -> Result<(), DiscoveryError> {
        // Verify descriptor is self-signed
        descriptor.verify()?;

        // Key = BLAKE3(DID public key bytes)
        let key = blake3::hash(descriptor.pubkey.as_bytes());

        // Value = canonical descriptor
        let value = json_atomic::canonize(&descriptor)?;

        self.kad.put_record(
            libp2p::kad::Record {
                key: key.as_bytes().to_vec().into(),
                value,
                publisher: None,
                expires: None,
            },
            libp2p::kad::Quorum::Majority,
        )?;

        Ok(())
    }

    /// Resolve a DID to endpoints
    pub async fn resolve(&mut self, did: &Did) -> Result<PeerDescriptor, DiscoveryError> {
        // Derive key from DID
        let pubkey = did.public_key()?;
        let key = blake3::hash(pubkey.as_bytes());

        // Query DHT
        let record = self.kad.get_record(key.as_bytes().to_vec().into()).await?;

        // Parse and verify descriptor
        let descriptor: PeerDescriptor = serde_json::from_slice(&record.value)?;

        // Verify signature
        descriptor.verify()?;

        // Check DID matches
        if descriptor.did != *did {
            return Err(DiscoveryError::DidMismatch);
        }

        Ok(descriptor)
    }
}

impl PeerDescriptor {
    pub fn verify(&self) -> Result<(), DiscoveryError> {
        let canonical = json_atomic::canonize(&PeerDescriptorContent {
            did: &self.did,
            endpoints: &self.endpoints,
            relay_did: &self.relay_did,
            timestamp: &self.timestamp,
        })?;

        self.pubkey.verify(&canonical, &self.signature)
            .map_err(|_| DiscoveryError::InvalidSignature)
    }
}
```

**Rule:** Descriptors MUST be signed by the DID key. Newest valid by timestamp is preferred.

---

## VI. Transport Abstraction Layer (TAL)

TAL provides carrier flexibility without identity drift.

```rust
// sirp/src/transport.rs

/// Transport Abstraction Layer
pub struct TransportLayer {
    drivers: Vec<Box<dyn TransportDriver>>,
    preferred_order: Vec<TransportKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Udp,       // Lowest latency
    Quic,      // Multiplexed, encrypted
    WebSocket, // Firewall-friendly
    Tcp,       // Fallback
}

#[async_trait]
pub trait TransportDriver: Send + Sync {
    fn kind(&self) -> TransportKind;

    async fn connect(&self, endpoint: &Endpoint) -> Result<Connection, TransportError>;

    async fn send(&self, conn: &mut Connection, capsule: &Capsule) -> Result<(), TransportError>;

    async fn recv(&self, conn: &mut Connection) -> Result<Capsule, TransportError>;
}

impl TransportLayer {
    pub fn new() -> Self {
        Self {
            drivers: vec![
                Box::new(UdpDriver::new()),
                Box::new(QuicDriver::new()),
                Box::new(WebSocketDriver::new()),
                Box::new(TcpDriver::new()),
            ],
            preferred_order: vec![
                TransportKind::Udp,
                TransportKind::Quic,
                TransportKind::WebSocket,
                TransportKind::Tcp,
            ],
        }
    }

    /// Send capsule using best available transport
    pub async fn send(
        &self,
        capsule: &Capsule,
        endpoints: &[Endpoint],
    ) -> Result<Receipt, TransportError> {
        // Try transports in preference order
        for kind in &self.preferred_order {
            let driver = self.drivers.iter()
                .find(|d| d.kind() == *kind)
                .ok_or(TransportError::NoDriver(*kind))?;

            // Find compatible endpoint
            let endpoint = endpoints.iter()
                .find(|e| self.endpoint_matches_transport(e, *kind));

            if let Some(ep) = endpoint {
                match driver.connect(ep).await {
                    Ok(mut conn) => {
                        match driver.send(&mut conn, capsule).await {
                            Ok(()) => {
                                // Wait for receipt
                                return self.wait_for_receipt(&mut conn, capsule).await;
                            }
                            Err(e) => {
                                log::warn!("Send failed on {:?}: {}", kind, e);
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Connect failed on {:?}: {}", kind, e);
                        continue;
                    }
                }
            }
        }

        Err(TransportError::AllTransportsFailed)
    }
}
```

### Constraints

- **No fragmentation:** SIRP does not fragment capsules
- **MTU guidance:** Keep header + LEN ≤ 1200 bytes over UDP
- **Session identity:** TAL drivers MUST authenticate peer_did at setup

---

## VII. Cryptographic Receipts

Routing outcomes become durable evidence.

```rust
// sirp/src/receipt.rs

/// Receipt types for SIRP operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Receipt {
    #[serde(rename = "sirp.receipt.relay.v1")]
    Relay(RelayReceipt),

    #[serde(rename = "sirp.receipt.delivery.v1")]
    Delivery(DeliveryReceipt),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayReceipt {
    pub capsule_cid: ContentAddress,
    pub sender_did: Did,
    pub receiver_did: Did,  // This relay node
    pub ts_received: Timestamp,
    pub metrics: RelayMetrics,
    pub outcome: RelayOutcome,
    pub next_hop_did: Option<Did>,
    pub canon_cid: ContentAddress,
    pub kid: String,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayMetrics {
    pub latency_ingress_ms: u32,
    pub verification_cost_us: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelayOutcome {
    Forwarded,
    DroppedTtl,
    ReplayDrop,
    RejectSig,
    Queued,
    DroppedBackpressure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReceipt {
    pub capsule_cid: ContentAddress,
    pub sender_did: Did,
    pub receiver_did: Did,
    pub ts_received: Timestamp,
    pub outcome: DeliveryOutcome,
    pub canon_cid: ContentAddress,
    pub kid: String,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryOutcome {
    Delivered,
    Rejected { reason: String },
}

impl Receipt {
    /// Verify receipt signature
    pub fn verify(&self, public_key: &VerifyingKey) -> Result<(), ReceiptError> {
        let (canonical, signature) = match self {
            Receipt::Relay(r) => {
                (json_atomic::canonize(r)?, &r.signature)
            }
            Receipt::Delivery(d) => {
                (json_atomic::canonize(d)?, &d.signature)
            }
        };

        public_key.verify(&canonical, signature)
            .map_err(|_| ReceiptError::InvalidSignature)
    }

    /// Get the capsule CID this receipt is for
    pub fn capsule_cid(&self) -> &ContentAddress {
        match self {
            Receipt::Relay(r) => &r.capsule_cid,
            Receipt::Delivery(d) => &d.capsule_cid,
        }
    }
}
```

### Outcomes

| Outcome | Meaning |
|---------|---------|
| **FORWARDED** | Relay accepted, passed to next hop |
| **DELIVERED** | Final recipient received |
| **DROPPED_TTL** | TTL exhausted |
| **REPLAY_DROP** | Duplicate within window |
| **REJECT_SIG** | Invalid signature |
| **QUEUED** | Accepted, awaiting processing |
| **DROPPED_BACKPRESSURE** | Rejected due to load |

---

## VIII. The Node

A complete SIRP node implementation:

```rust
// sirp/src/node.rs

/// A SIRP network node
pub struct Node {
    did: Did,
    signing_key: SigningKey,
    discovery: Discovery,
    transport: TransportLayer,
    replay_cache: ReplayCache,
    receipt_store: ReceiptStore,
}

impl Node {
    pub async fn new(did: Did, signing_key: SigningKey) -> Result<Self, NodeError> {
        let discovery = Discovery::new(did.clone(), &signing_key)?;
        let transport = TransportLayer::new();
        let replay_cache = ReplayCache::new(Duration::from_secs(300));  // 5 min window
        let receipt_store = ReceiptStore::new();

        Ok(Self {
            did,
            signing_key,
            discovery,
            transport,
            replay_cache,
            receipt_store,
        })
    }

    /// Send a capsule to a recipient
    pub async fn send(&mut self, capsule: Capsule) -> Result<Receipt, NodeError> {
        // 1. Resolve recipient endpoints
        let recipient_did = capsule.recipient_did()?;
        let descriptor = self.discovery.resolve(&recipient_did).await?;

        // 2. Send via transport layer
        let receipt = self.transport.send(&capsule, &descriptor.endpoints).await?;

        // 3. Store receipt for future reference
        self.receipt_store.store(&receipt)?;

        Ok(receipt)
    }

    /// Receive and process incoming capsules
    pub async fn receive(&mut self, capsule: Capsule) -> Result<Receipt, NodeError> {
        // 1. Verify capsule
        let sender_pubkey = self.resolve_public_key(&capsule.sender_did()?).await?;
        capsule.verify(&sender_pubkey)?;

        // 2. Check TTL
        if capsule.header.ttl == 0 {
            return Ok(self.create_receipt(&capsule, DeliveryOutcome::Rejected {
                reason: "TTL exhausted".to_string(),
            })?);
        }

        // 3. Check replay cache
        let cache_key = (capsule.sender_did()?, capsule.cid());
        if self.replay_cache.contains(&cache_key) {
            return Ok(self.create_relay_receipt(
                &capsule,
                RelayOutcome::ReplayDrop,
            )?);
        }
        self.replay_cache.insert(cache_key);

        // 4. Determine if we are the final recipient
        let recipient = capsule.recipient_did()?;
        if recipient == self.did {
            // Final delivery
            let receipt = self.create_receipt(&capsule, DeliveryOutcome::Delivered)?;
            self.handle_payload(&capsule).await?;
            Ok(receipt)
        } else {
            // Relay to next hop
            self.relay(capsule).await
        }
    }

    async fn relay(&mut self, mut capsule: Capsule) -> Result<Receipt, NodeError> {
        // Decrement TTL
        capsule.header.ttl -= 1;

        if capsule.header.ttl == 0 {
            return Ok(self.create_relay_receipt(&capsule, RelayOutcome::DroppedTtl)?);
        }

        // Resolve next hop
        let recipient = capsule.recipient_did()?;
        let descriptor = self.discovery.resolve(&recipient).await?;

        // Forward
        match self.transport.send(&capsule, &descriptor.endpoints).await {
            Ok(receipt) => Ok(self.create_relay_receipt(
                &capsule,
                RelayOutcome::Forwarded,
            )?),
            Err(_) => Ok(self.create_relay_receipt(
                &capsule,
                RelayOutcome::DroppedBackpressure,
            )?),
        }
    }

    fn create_receipt(
        &self,
        capsule: &Capsule,
        outcome: DeliveryOutcome,
    ) -> Result<Receipt, NodeError> {
        let receipt = DeliveryReceipt {
            capsule_cid: capsule.cid(),
            sender_did: capsule.sender_did()?,
            receiver_did: self.did.clone(),
            ts_received: Timestamp::now(),
            outcome,
            canon_cid: ContentAddress::default(),  // Computed below
            kid: self.signing_key.kid(),
            signature: Signature::default(),  // Signed below
        };

        let canonical = json_atomic::canonize(&receipt)?;
        let canon_cid = ContentAddress::from_blake3(blake3::hash(&canonical));

        let mut receipt = receipt;
        receipt.canon_cid = canon_cid;
        receipt.signature = self.signing_key.sign(&canonical);

        Ok(Receipt::Delivery(receipt))
    }
}
```

---

## IX. CLI Usage

```bash
# Start a SIRP node
logline sirp start \
  --did "did:logline:agent:alice" \
  --keyfile alice.key \
  --port 9000

# Output:
# SIRP node started
# DID: did:logline:agent:alice
# Endpoints: udp://0.0.0.0:9000, quic://0.0.0.0:9001
# Discovery: bootstrapped to 3 peers

# Send a capsule
logline sirp send \
  --to "did:logline:agent:bob" \
  --payload '{"action": "transfer", "amount": 1000}'

# Output:
# Capsule sent
# CID: b3:7f3a9b2c4d5e6f7a8b9c0d1e2f3a4b5c...
# Waiting for delivery receipt...
# DELIVERED at 2026-02-05T14:23:07Z
# Receipt CID: b3:8f4a9c3d...

# Verify a receipt
logline sirp verify-receipt \
  --receipt receipt.json \
  --capsule capsule.json

# Output:
# Capsule CID: MATCH
# Signature: VALID
# Timestamp: 2026-02-05T14:23:07Z (within acceptable skew)
# Receipt verification: PASS

# List pending receipts
logline sirp receipts \
  --status pending \
  --since "1h"

# Output:
# Pending receipts (last 1h):
#   b3:1a2b... → did:logline:agent:charlie  QUEUED
#   b3:3c4d... → did:logline:agent:david    FORWARDED

# Resolve a DID
logline sirp resolve "did:logline:agent:bob"

# Output:
# DID: did:logline:agent:bob
# Endpoints:
#   udp://203.0.113.5:9000
#   quic://relay.example.com:443/bob
# Relay: did:logline:relay:gamma
# Last updated: 2026-02-05T13:00:00Z
# Signature: VALID
```

---

## X. Security Properties

| Threat | Mitigation |
|--------|------------|
| **Header tampering** | Domain-separated signature over header ‖ payload |
| **Payload exposure** | Encrypted capsules reveal only INTENT hint |
| **Replay attacks** | (sender_did, CID) cache + time windows |
| **Downgrade** | Explicit VER; mismatch rejected |
| **Key rotation** | DIDs rotate via descriptors; receipts bind to kid |
| **TOCTOU** | Authorization bound in Gate receipts, not SIRP |

---

## XI. Conformance

| Test | Requirement |
|------|-------------|
| **CT-V-01** | Capsule roundtrip: serialize → TAL → parse → verify |
| **CT-V-02** | Anti-replay: same (sender_did, CID) → REPLAY_DROP |
| **CT-V-03** | TTL enforcement: decrement per hop, drop at zero |
| **CT-V-04** | Receipt validity: offline verification passes |
| **CT-V-05** | Discovery integrity: invalid signature rejected |

```rust
#[cfg(test)]
mod conformance {
    #[test]
    fn ct_v_01_roundtrip() {
        let capsule = Capsule::new(
            &did!("bob"),
            json!({"action": "test"}),
            &signing_key,
        )?;

        // Serialize
        let bytes = capsule.to_bytes()?;

        // Parse
        let parsed = Capsule::from_bytes(&bytes)?;

        // Verify
        parsed.verify(&signing_key.verifying_key())?;

        assert_eq!(capsule.cid(), parsed.cid());
    }

    #[test]
    fn ct_v_02_anti_replay() {
        let mut node = Node::new(did!("alice"), signing_key).await?;

        let capsule = Capsule::new(&did!("alice"), json!({}), &other_key)?;

        // First receive: OK
        let receipt1 = node.receive(capsule.clone()).await?;
        assert!(matches!(receipt1, Receipt::Delivery(_)));

        // Second receive: REPLAY_DROP
        let receipt2 = node.receive(capsule).await?;
        assert!(matches!(receipt2, Receipt::Relay(r) if r.outcome == RelayOutcome::ReplayDrop));
    }

    #[test]
    fn ct_v_03_ttl() {
        let mut capsule = Capsule::new(&did!("bob"), json!({}), &key)?;
        capsule.header.ttl = 1;

        let mut relay_node = Node::new(did!("relay"), relay_key).await?;
        let receipt = relay_node.receive(capsule).await?;

        // TTL was 1, after decrement it's 0, should be dropped
        assert!(matches!(receipt, Receipt::Relay(r) if r.outcome == RelayOutcome::DroppedTtl));
    }
}
```

---

## XII. The Invariant Connection

| Invariant | SIRP Implementation |
|-----------|---------------------|
| **I1** Integrity | Capsules, descriptors, receipts canonical and signed |
| **I2** Legality | Delivery never authorizes effects |
| **I3** Attribution | DIDs and kid bind actors across artifacts |
| **I4** Reproducibility | Same payload + receipts → same verification |
| **I5** Observability | DROPPED, REPLAY outcomes are metrified |

---

## XIII. Constants

| Constant | Value |
|----------|-------|
| MAGIC | 0x5199 |
| VER | 0x01 |
| INTENT | First 64 bits of BLAKE3("namespace.action") |
| Domain string | "sirp.cap.v1" |
| Default TTL | 64 hops |
| Replay window | 300 seconds |

---

## XIV. Conclusion

> **SIRP makes intention deliverable without identity loss.**

In a network where meaning matters more than location:

- **Capsules** carry proof-ready content
- **Receipts** convert forwarding into facts
- **Discovery** ties cryptographic persons to changing edges
- **TAL** abstracts the wire without losing accountability

The result is a network where movement of meaning is:
- **Auditable** (every hop receipted)
- **Portable** (identity independent of topology)
- **Economically accountable** (receipts enable settlement)

When packets become artifacts, routing becomes governance.

---

## The Equation

```
Capsule + Receipts = Verifiable Delivery

Movement becomes fact.
```

---

*Next: [Hardware as Text and Power](07_Hardware_as_Text_and_Power.md)*
