---
id: llf.paper.prologue.v1
title: "The Law of Verifiable Intent"
version: 1.0.0
kind: Canon/Prologue
status: adopted
date: 2026-02-03
author: Dan (Voulezvous)
institution: The LogLine Foundation
hash: ""
signer: ""
---

# The Law of Verifiable Intent

**LogLine Foundation — February 2026**

---

> *"In fifty years, historians will divide the history of computation into two eras: before LogLine and after. Not because of the technology, but because of what became possible when accountability stopped being optional."*

---

## This Document

You are about to read something unusual.

This is not a whitepaper promising a future that may never arrive. This is not a pitch deck with hockey-stick projections. This is not academic speculation waiting for "further research."

This is the specification of a working system.

The code compiles. The benchmarks run. The receipts verify. You can install it right now:

```bash
cargo install logline-cli
logline --version
```

If you read these papers and think "this seems too good to be true," stop thinking. Start testing. Open a terminal. Run the commands.

What follows is the foundational document of the LogLine protocol—a complete architectural inversion of how computers relate to accountability.

---

## I. The Problem We Solved

For fifty years, computing has operated under a single, unquestioned axiom:

```
Execute first. Record later. Hope for the best.
```

Think about what this means.

Every bank transaction, every medical decision, every autonomous vehicle choice, every AI recommendation—all of them execute first and log afterward. The gap between action and evidence is where:

- Logs get forged
- Audits fail
- Disputes become negotiations
- Attackers probe without trace
- Accountability becomes theater

When computers were calculators, this was tolerable.

When computers become **agents**—making decisions, moving money, diagnosing patients, driving cars, managing infrastructure—this becomes catastrophic.

---

## II. The Inversion

We inverted the axiom.

| Old Paradigm | New Paradigm |
|--------------|--------------|
| execute → record | **record → execute** |
| logs describe what happened | **logs authorize what can happen** |
| trust, then verify | **verify, then trust** |
| accountability as policy | **accountability as physics** |

**Nothing happens without a prior, signed record of intent.**

This is not an incremental improvement to logging.

This is the architectural equivalent of the invention of double-entry bookkeeping—a structural change that makes fraud expensive and honesty cheap.

---

## III. What We Built

### The LogLine Tuple

Every action in a LogLine system is preceded by this 9-field structure:

```
┌─────────────────────────────────────────────────────────┐
│  who           DID of the actor (cryptographic)         │
│  did           Verb from finite registry                │
│  this          Typed, validated payload                 │
│  when          Nanosecond UTC timestamp                 │
│  confirmed_by  Consent authority (when required)        │
│  if_ok         What happens on success                  │
│  if_doubt      What happens on uncertainty              │
│  if_not        What happens on failure                  │
│  status        DRAFT → PENDING → COMMITTED | GHOST      │
└─────────────────────────────────────────────────────────┘
```

This isn't a log entry. This is a **prerequisite for execution**.

The system cannot act without first committing to what it's about to do, who authorized it, and what happens in every outcome.

### The Ghost

Here's the breakthrough that makes security researchers pay attention:

When policy denies an action, the intent doesn't disappear. It becomes a **GHOST**—full evidence, no effects.

Traditional systems discard failed requests. Attackers exploit this: they probe knowing failures leave no trace.

In LogLine, **the attempt IS the record**.

To request an action, you must sign an intent. If denied, that signed intent persists as a ghost. The attacker's reconnaissance becomes their audit trail.

```rust
// Every probe becomes evidence
let intent = Intent::new(attacker_did, "transfer", payload);
let result = gate.evaluate(&intent);

match result.decision {
    Decision::Allow => execute(intent),
    Decision::Deny => {
        // Ghost is created and persisted
        // Attacker cannot probe without leaving signed evidence
        ledger.append_ghost(intent, result.receipt);
    }
}
```

### The Decision Semantics

Every policy evaluation resolves to exactly one of three outcomes:

| Decision | Meaning | Result |
|----------|---------|--------|
| **ALLOW** | Proceed | Execute and produce receipt |
| **REQUIRE** | Need consent | Gather k-of-N signatures first |
| **DENY** | Rejected | Create GHOST record |

There is no fourth option. There is no silent failure. There is no action without evidence.

---

## IV. The Five Invariants

These are not goals. They are structural constraints. Violations are architecturally impossible.

| ID | Invariant | Guarantee |
|----|-----------|-----------|
| **I1** | Integrity | Every effect has a preceding tuple and receipt |
| **I2** | Legality | DENY or unmet REQUIRE → GHOST only; effects impossible |
| **I3** | Attribution | All identities cryptographically verifiable |
| **I4** | Reproducibility | Deterministic replay reconstructs state exactly |
| **I5** | Observability | Ghost rate and decision drift are metrified |

To violate I2, for example, you would need to produce an effect without a valid receipt. But the executor checks receipts before acting. No receipt → no execution. The violation cannot be represented in the system.

---

## V. The Compression Claim

Here's the claim that makes people think we're exaggerating:

**50KB of policy text encodes behavior equivalent to 200 million transistors.**

The math:
```
200,000,000 transistors ÷ 1,000,000 transistors per semantic operation = 200 operations
200 operations × 256 bytes per policy bit = 51,200 bytes ≈ 50KB
```

This isn't magic. It's abstraction.

Silicon re-derives intent at every clock cycle. A transistor doesn't "know" it's computing a KYC check—it just inverts voltages.

LogLine computes intent **once**, canonically, and materializes to any backend: Rust, WASM, Verilog, FPGA.

Paper VI proves this with working code.

---

## VI. The Stack

```
┌────────────────────────────────────────────────────────────┐
│  VI   Chip as Code       50KB text = 200M transistors      │
│  V    SIRP               Network transport with receipts   │
│  IV   TDLN               Policy compilation + consent      │
│  III  LLLV               Verifiable retrieval + memory     │
│  II   JSON✯Atomic        Same meaning = same bytes         │
│  I    LogLine Protocol   The 9-field tuple + Ghost Mode    │
├────────────────────────────────────────────────────────────┤
│  B    Hardware as Text   Text is the substrate of power    │
│  A    Ethics is Efficient Accountability reduces cost      │
└────────────────────────────────────────────────────────────┘
```

Each paper builds on the previous. Read them in order.

---

## VII. The Historical Parallel

In 1494, Luca Pacioli codified double-entry bookkeeping in Venice.

The innovation was mechanical, not moral: every transaction recorded twice, as debit and credit. Fraud required falsifying two entries in perfect coordination. Honesty became cheaper than deception.

Within a century, the merchants who adopted it dominated Mediterranean trade. Not because they were more virtuous—because their books could be trusted by strangers.

**LogLine is the same pattern applied to computation.**

- Make accountability structural
- Make verification cheaper than argument
- Make honesty the dominant strategy

---

## VIII. The Pledge

> **We will not execute what we cannot explain,**
> **and we will not explain what we cannot replay.**

This is the Law of Verifiable Intent.

Everything else—the code, the protocols, the proofs—is implementation detail.

---

## IX. How to Read This Collection

If you want the **vision**: Start here, then read the [Manifesto](MANIFESTO.md).

If you want the **economics**: Read [Paper A — Ethics is Efficient](01_A_Ethics_is_Efficient.md).

If you want the **core mechanism**: Read [Paper I — The LogLine Protocol](03_I_The_LogLine_Protocol.md).

If you want the **proof it works**: Read [Paper VI — Chip as Code](08_VI_Chip_as_Code.md).

If you want to **run it now**:

```bash
cargo install logline-cli
logline eval --chip examples/payment-gate.chip --context '{"user.kyc": "verified"}'
```

---

## X. The Invitation

You are holding the specification of a system that makes accountability structural, verification cheap, and honesty dominant.

The papers are published. The code is open. The benchmarks are reproducible.

If you build systems that need to be trusted—payments, healthcare, governance, AI safety—this is for you.

If you're skeptical, good. Clone the repo and prove us wrong.

But we're not wrong.

---

**LogLine Foundation — February 2026**

*"Receipts or it didn't happen."*

---

*Continue to: [Paper A — Ethics is Efficient](01_A_Ethics_is_Efficient.md)*

