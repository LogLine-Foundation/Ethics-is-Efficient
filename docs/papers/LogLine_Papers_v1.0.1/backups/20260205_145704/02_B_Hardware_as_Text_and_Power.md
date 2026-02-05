---
id: llf.paper.text-power.v1
title: "Paper B — Hardware as Text and Power"
version: 1.0.1
kind: Canon/Foundation
status: adopted
date: 2026-01-31
author: Dan (Voulezvous)
institution: The LogLine Foundation
lineage: []
gates_required: ["G1", "G2", "G3", "G4"]
thesis: "Text is the only substrate where intention becomes enforceable constraint."
hash: ""
signer: ""
---

# Paper B — Hardware as Text and Power

**The Substrate of Verifiable Governance**

---

> *"Whoever controls the text controls the system."*

---

## A Thought Experiment

Consider two files:

**File 1: policy.md**
```markdown
# Transfer Policy

Users must be KYC verified before transferring more than $1000.
Transfers above $10,000 require manager approval.
```

**File 2: policy.ll**
```yaml
policy "transfer_authorization":
  when amount > 1000:
    require context.user.kyc_verified == true
  when amount > 10000:
    require confirmed_by in ["manager", "director"]
```

Both express the same intent. But there's a crucial difference.

File 1 is **advice**. A human reads it, interprets it, implements something that hopefully matches. The policy and the implementation drift. Disputes require lawyers.

File 2 is **law**. It compiles directly into constraints that execute. The policy IS the implementation. Disputes require hash comparison.

```bash
# File 2 compiles to executable constraint
logline compile policy.ll -o policy.wasm

# The hash of the policy IS its identity
b3sum policy.ll
# b3:7f3a9b2c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a...

# Same policy text → Same hash → Same behavior
# Forever. Everywhere. No interpretation.
```

This is what we mean by "hardware as text."

The text is not describing the hardware. The text IS the hardware—the hardware is just one of many possible materializations.

---

## I. The Three Powers

Every modern system involves three kinds of power that must be aligned:

### Silicon Power

Raw computation. Throughput. Parallelism. Energy efficiency.

Silicon is **amoral and obedient**. It executes whatever can be encoded, with perfect fidelity and zero judgment. It's a furnace: powerful, useful, and completely indifferent to whether it's heating a home or burning it down.

```rust
// Silicon doesn't know what it's doing
// It just executes instructions
fn transfer(from: Account, to: Account, amount: u64) {
    from.balance -= amount;  // Could be theft
    to.balance += amount;    // Could be fraud
    // Silicon doesn't care. It just runs.
}
```

### Human Power

Authority to declare commitments and be accountable. Meaning. Priorities. Trade-offs. Exception-handling.

Human power is **legitimate but non-deterministic**. Humans are the source of policy, but terrible at consistent enforcement. We negotiate, we forget, we make exceptions, we get tired.

```
Human: "Only verified users can transfer large amounts."
Monday: Enforced.
Tuesday: Enforced.
Wednesday: "Just this once, the CEO asked..."
Thursday: "Well, we did it yesterday..."
Friday: What policy?
```

### AI Power

Amplification. Translation at scale. Compression of complexity.

AI power is **multiplicative**. It doesn't create intent—it amplifies whatever intent is given. Weak governance at the input means scaled failure at the output. Strong governance means scaled capability.

```
AI + Bad Policy = Bad at Scale
AI + Good Policy = Good at Scale
AI + No Policy = Chaos at Scale
```

---

## II. The Problem: Policy as Advice

Most systems place policy above computation as **advice**:

- Documentation that developers "should" read
- Best practices that teams "should" follow
- Human review that "should" catch errors
- Runtime checks that "should" enforce rules
- Post-incident enforcement that "should" deter violations

Every "should" is a gap. Every gap is a vulnerability.

**The Iron Law of Systems:**

> If a forbidden state can be represented, it will eventually be reached.

"Can happen" becomes "will happen." Exceptions become normal. Shortcuts become habits. Audits become theater. Incident response becomes the real governance.

```
Year 1: "We have a policy against that."
Year 2: "We mostly follow the policy."
Year 3: "We follow the policy when convenient."
Year 4: "What policy?"
Year 5: Incident. Investigation. "How did we get here?"
```

---

## III. The Solution: Text as Law

"Hardware as text" means two things:

1. **Commitments exist as canonical textual artifacts**
2. **Artifacts compile into execution constraints**

Not "text that advises hardware." Text that IS hardware.

```
Traditional:
  Human Intent → Code → Hardware → Maybe Policy Checked → Effects

LogLine:
  Human Intent → Canonical Text → Compilation → Constraints → Execution
       ↓              ↓              ↓            ↓            ↓
    Signed         Hashed        Proven       Enforced    Receipted
```

### Why Text?

Text is the only substrate that is simultaneously:

| Property | Why It Matters |
|----------|----------------|
| **Human-writable** | Humans can author policy |
| **Machine-executable** | Machines can enforce policy |
| **Cryptographically signable** | Authorship is verifiable |
| **Content-addressable** | Identity is deterministic |
| **Version-controllable** | History is preserved |
| **Universally portable** | Runs anywhere |

No other substrate has all six properties.

Silicon isn't human-writable. Voice isn't machine-parseable. Images aren't content-addressable. Binary isn't version-controllable.

**Text is the unique intersection.**

---

## IV. The Three Equations

When text is treated as law, three things become true:

### Equation 1: The File is the Identity

```
Same Meaning → Same Bytes → Same Hash

Identity = BLAKE3(canonical_bytes)
```

This is not metaphor. This is how the system works:

```rust
// logline-core/src/identity.rs

/// Compute the identity of any artifact
pub fn identity<T: Serialize>(artifact: &T) -> ContentAddress {
    // 1. Canonicalize to deterministic bytes (Paper II)
    let bytes = json_atomic::canonize(artifact);

    // 2. Hash the bytes
    let hash = blake3::hash(&bytes);

    // 3. That hash IS the identity
    ContentAddress::from_blake3(hash)
}

// Two policies with same meaning = same identity
let policy_a = Policy::parse("when x > 10: DENY");
let policy_b = Policy::parse("when x > 10: DENY");

assert_eq!(identity(&policy_a), identity(&policy_b));
// Both are: b3:7f3a9b2c...
```

If someone gives you a policy hash, you can verify:
- You have the exact same policy
- It hasn't been modified
- Any execution using it is traceable

### Equation 2: The Compiler is Governance

```
Policy + Compiler → Constraints

Compilation = Governance
```

The policy text doesn't just describe what should happen. It compiles into constraints that make violations **structurally impossible**.

```rust
// Traditional: Check at runtime (can be bypassed)
fn transfer(amount: u64, user: &User) -> Result<()> {
    if amount > 1000 && !user.kyc_verified {
        return Err("KYC required");  // Can be bypassed by removing this line
    }
    // ... transfer
}

// LogLine: Capability exists only if policy allows
fn transfer(amount: u64, cap: TransferCapability) -> Result<()> {
    // TransferCapability can only be created if policy allows
    // There's no "if" to bypass - the type doesn't exist without authorization
    execute_with_capability(cap, amount)
}

// The capability creation is gated by policy
impl TransferCapability {
    pub fn request(user: &User, amount: u64, policy: &Policy) -> Option<Self> {
        let decision = policy.evaluate(&Context {
            user: user.clone(),
            amount,
        });

        match decision {
            Decision::Allow => Some(TransferCapability { /* ... */ }),
            Decision::Require => None, // Need consent first
            Decision::Deny => None,    // Rejected
        }
    }
}
```

The difference is fundamental. In the traditional model, security is a check that can be removed. In LogLine, security is a type that can't be forged.

### Equation 3: The Runtime is Proof

```
Execution → Receipt

Receipt = Proof of Governance
```

Every execution produces a cryptographic receipt proving:
- Which policy was active
- What inputs were provided
- What decision was made
- Who authorized it

```rust
// Every execution produces proof
let receipt = runtime.execute(intent, policy)?;

// The receipt is verifiable by anyone
assert!(receipt.verify(&public_key)?);

// The receipt binds to the policy
assert_eq!(receipt.policy_hash, policy.identity());

// Disputes collapse into verification
fn resolve_dispute(my_receipt: &Receipt, their_receipt: &Receipt) -> Winner {
    if my_receipt.receipt_cid != their_receipt.receipt_cid {
        // Different receipts - check chain order
        return check_chain_order(my_receipt, their_receipt);
    }
    // Same receipt - no dispute
    Winner::Both
}
```

---

## V. Non-Representability

This is the most important concept in the entire paper.

**Traditional security:** Detect and prevent violations at runtime.

**LogLine security:** Make violations impossible to express.

### The Core Lemma

> If a violation cannot be encoded in the Intermediate Representation,
> then no valid plan can schedule it,
> thus execution cannot observe it.

The forbidden state doesn't get caught. The forbidden state **cannot be typed**.

### Example: Transfer Without KYC

**Traditional (representable, therefore reachable):**

```rust
// The violation CAN be expressed
fn evil_transfer(amount: u64) {
    // Just... don't check KYC
    ledger.transfer(from, to, amount);
    // The type system allows this
}
```

**LogLine (non-representable, therefore unreachable):**

```rust
// The violation CANNOT be expressed
fn transfer(amount: u64, cap: VerifiedTransferCapability) {
    // VerifiedTransferCapability can only exist if:
    // 1. User is KYC verified
    // 2. Policy evaluation returned ALLOW
    // 3. Receipt was generated

    // There's no way to call this function without the capability
    // The capability can't be forged
    // The violation is structurally unreachable
    ledger.transfer_with_proof(cap, amount);
}

// Trying to bypass...
fn evil_transfer(amount: u64) {
    // We need a VerifiedTransferCapability but...
    // - Can't construct it directly (private fields)
    // - Can't get one from policy.evaluate() (returns None for unverified)
    // - Can't forge one (signature verification)

    // The attack surface doesn't exist.
}
```

### Why This Matters

In traditional systems, security is an arms race:
- Attacker finds bypass
- Defender adds check
- Attacker finds new bypass
- Forever

In LogLine, security is structural:
- Violations cannot be typed
- Therefore cannot be compiled
- Therefore cannot execute
- End of story

---

## VI. The Control Planes

The system operates across five planes, each with distinct responsibilities:

```
┌─────────────────────────────────────────────────────────┐
│  HUMAN PLANE                                            │
│  Authorship + Accountability                            │
│  Writes policies, signs artifacts, bears responsibility │
├─────────────────────────────────────────────────────────┤
│  TEXT PLANE                                             │
│  Law + Canonical Form                                   │
│  Policies, schemas, manifests - all signed, all hashed  │
├─────────────────────────────────────────────────────────┤
│  AI PLANE                                               │
│  Translation under Constraint                           │
│  MAY translate, MAY optimize, MUST NOT be final signer  │
├─────────────────────────────────────────────────────────┤
│  SILICON PLANE                                          │
│  Execution                                              │
│  Runs what was compiled, produces receipts              │
├─────────────────────────────────────────────────────────┤
│  LEDGER PLANE                                           │
│  Memory + Evidence                                      │
│  All receipts, all ghosts, immutable and verifiable     │
└─────────────────────────────────────────────────────────┘
```

Power flows downward. Proof flows upward.

---

## VII. The Power Boundary

A "power boundary" is where one entity's authority ends and another's begins.

In traditional systems, power boundaries are:
- API endpoints (can be bypassed)
- Authentication (can be stolen)
- Authorization (can be misconfigured)
- Contracts (require lawyers to interpret)

In LogLine, the power boundary is the **text artifact** itself:

| Property | Verification |
|----------|--------------|
| **Canonical** | hash(policy_text) = hash(executing_policy) |
| **Signed** | Ed25519 signature verified |
| **Compiled** | IR version pinned and attested |
| **Receipted** | Every effect references policy_hash |
| **Non-representable** | Violations fail at compile time |

```rust
// Verify power boundary
fn verify_power_boundary(
    policy_text: &str,
    compiled_wasm: &[u8],
    execution_receipt: &Receipt,
) -> Result<(), BoundaryViolation> {
    // 1. Policy text hash matches
    let policy_hash = blake3::hash(policy_text.as_bytes());

    // 2. Compiled module attests to same policy
    let wasm_attestation = extract_attestation(compiled_wasm)?;
    if wasm_attestation.policy_hash != policy_hash {
        return Err(BoundaryViolation::PolicyMismatch);
    }

    // 3. Receipt references same policy
    if execution_receipt.policy_hash != policy_hash.into() {
        return Err(BoundaryViolation::ReceiptMismatch);
    }

    // 4. Signatures verify
    verify_signatures(policy_text, compiled_wasm, execution_receipt)?;

    Ok(())
}
```

---

## VIII. Consequences

When text becomes law, three things happen:

### Zero Trust by Construction

You don't trust the server. You don't trust the network. You don't trust the operator.

You verify the receipt.

```rust
// I don't trust the bank's claim that they processed my payment
// I verify the receipt they gave me
let receipt = bank.get_receipt(payment_id)?;
let policy = registry.get_policy(receipt.policy_hash)?;

// Verify the receipt is valid
receipt.verify(&bank_public_key)?;

// Verify the policy is what I agreed to
assert_eq!(policy.identity(), expected_policy_hash);

// Now I trust the outcome - not the bank, the math
```

### Latency Collapse

Pre-compiled governance means decisions are made at policy time, not runtime.

Traditional:
```
Request → Parse → Check Policy → Check Auth → Check Limits → Execute
         [  All this happens at runtime, on every request  ]
```

LogLine:
```
Request → Execute Pre-compiled Constraints → Receipt
          [Decision already made at compile time]
```

The runtime just executes what was already decided. Latency drops by 10-100x.

### Freedom as Consequence

This seems paradoxical: more constraints → more freedom?

Yes.

When you know the rules are enforced for everyone:
- You don't negotiate
- You don't worry
- You don't second-guess
- You just act

Hard constraints remove uncertainty. Freedom is the presence of trustworthy constraint.

---

## IX. The Conclusion

Silicon provides execution—raw, amoral, obedient.

Humans provide intention—legitimate, meaningful, inconsistent.

AI provides scale—multiplicative, amplifying whatever it's given.

**Text provides power.**

Text is the only substrate where intention becomes enforceable constraint. Where a file's hash is its identity. Where compilation is governance. Where execution is proof.

Whoever controls the text controls the system.

The power boundary is not negotiation between parties.
The power boundary is **compilation**—policy text becoming executable constraint.

In LogLine, the text doesn't describe the hardware.
The text IS the hardware.
The hardware is just a rendering.

---

*Next: [Paper I — The LogLine Protocol](03_I_The_LogLine_Protocol.md)*

