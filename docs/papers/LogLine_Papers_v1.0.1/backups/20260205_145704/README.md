# LogLine SecurityOS

### A Protocol Suite for Verifiable Accountability in Autonomous Systems

---

<p align="center">
  <em>"We will not execute what we cannot explain,<br>
  and we will not explain what we cannot replay."</em>
</p>

---

## The Problem

For fifty years, computing has operated on a broken premise:

```
execute → record → hope
```

Actions happen. Logs are written. We hope they're accurate, complete, and unmodified. When disputes arise, we argue about what really happened.

This worked when computers were tools. It fails when computers are agents—making decisions, moving money, taking actions on our behalf.

## The Solution

LogLine inverts the architecture:

```
record → consent → execute → receipt
```

**Nothing happens without a prior record of intent.**
**Nothing executes without policy evaluation.**
**Nothing completes without a cryptographic receipt.**

The result: accountability becomes a property of the system, not a layer applied to it.

---

## Quick Start

| Document | Purpose |
|----------|---------|
| [**MANIFESTO**](MANIFESTO.md) | The vision in 2 pages |
| [**ABSTRACT**](ABSTRACT.md) | Academic-style summary |
| [**Prologue**](00_Prologue_Ethics_is_Efficient.md) | One-page technical thesis |

---

## The Protocol Stack

```
┌─────────────────────────────────────────────────────────────┐
│                     FOUNDATIONS                              │
├─────────────────────────────────────────────────────────────┤
│  A: Ethics is Efficient     │  B: Hardware as Text          │
│  (Economic argument)        │  (Substrate theory)           │
├─────────────────────────────────────────────────────────────┤
│                       PROTOCOLS                              │
├──────────┬──────────┬──────────┬──────────┬────────┬────────┤
│    I     │    II    │   III    │    IV    │   V    │   VI   │
│ LogLine  │  JSON✯   │  LLLV    │  TDLN    │ SIRP   │ Chip   │
│ Protocol │ Atomic   │ Retrieval│ Policy   │ Network│ Code   │
├──────────┴──────────┴──────────┴──────────┴────────┴────────┤
│                       INVARIANTS                             │
│  I1 Integrity │ I2 Legality │ I3 Attribution │ I4 Replay │ I5 Obs │
└─────────────────────────────────────────────────────────────┘
```

---

## The Papers

### Foundations (Why & Where)

| # | Paper | Summary |
|---|-------|---------|
| 00 | [Prologue](00_Prologue_Ethics_is_Efficient.md) | One-page thesis and system invariants |
| A | [Ethics is Efficient](01_A_Ethics_is_Efficient.md) | Economic rationale: accountability reduces total cost |
| B | [Hardware as Text](02_B_Hardware_as_Text_and_Power.md) | Substrate theory: signed text becomes structural power |

### Specifications (How)

| # | Paper | Summary |
|---|-------|---------|
| I | [LogLine Protocol](03_I_The_LogLine_Protocol.md) | The 9-field tuple. Ghost records. Threat model. |
| II | [JSON✯Atomic](04_II_JSON_Atomic.md) | Deterministic canonicalization. Same meaning = same bytes. |
| III | [LLLV](05_III_LLLV_Ledger_and_Proof_Vectors.md) | Proof-carrying retrieval. Evidence capsules. |
| IV | [TDLN](06_IV_TDLN_Deterministic_Translation_of_Natural_Language.md) | Policy compilation. Consent protocol. Gate mechanics. |
| V | [SIRP](07_V_SIRP_Secure_Intent_Routing_Protocol.md) | Network transport. Capsules. Cryptographic receipts. |
| VI | [Chip as Code](08_VI_Chip_as_Code.md) | Computational realization. Hardware as backend. |

### Legal

| # | Document | Purpose |
|---|----------|---------|
| 98 | [Declaration](98_Declaration_of_Invention_LogLine.md) | Formal claim of authorship and priority |
| 99 | [Announcement](99_Announcement_LogLine_SecurityOS.md) | Public disclosure statement |

---

## The Core Innovation: The LogLine Tuple

Every action in a LogLine system is preceded by this 9-field structure:

```
┌─────────────────────────────────────────────────────────────┐
│  who          │ DID of the actor (cryptographic identity)   │
│  did          │ Canonical verb from allowed registry        │
│  this         │ Typed payload (validated against schema)    │
│  when         │ Timestamp (nanosecond UTC)                  │
│  confirmed_by │ Consent authority (when required)           │
│  if_ok        │ Success commitment                          │
│  if_doubt     │ Uncertainty protocol                        │
│  if_not       │ Failure commitment                          │
│  status       │ DRAFT → PENDING → COMMITTED | GHOST         │
└─────────────────────────────────────────────────────────────┘
```

**The GHOST is the breakthrough.** When an action is denied by policy or times out waiting for consent, it doesn't disappear—it persists as a ghost record. Full evidence, no effects.

The attacker's reconnaissance becomes their audit trail.

---

## The Five Invariants

| ID | Name | Guarantee |
|----|------|-----------|
| **I1** | Integrity | Every mutable effect has a preceding tuple and success receipt |
| **I2** | Legality | DENY or unmet REQUIRE → GHOST only (no effects possible) |
| **I3** | Attribution | Identities and consents are cryptographically verifiable |
| **I4** | Reproducibility | Deterministic replay reconstructs state exactly |
| **I5** | Observability | Ghost rate and decision transitions are metrified |

These are not aspirations. They are structural constraints. Violations are architecturally impossible.

---

## Decision Semantics

All policy evaluations resolve to one of three outcomes:

| Decision | Meaning | Result |
|----------|---------|--------|
| **ALLOW** | Proceed immediately | Receipt on success |
| **REQUIRE** | Gather k-of-N consent first | Gated by `confirmed_by` |
| **DENY** | Rejected by policy | GHOST record |

---

## Verification

```bash
# Compute BLAKE3 hashes for all papers
for f in *.md; do echo "$(b3sum "$f" | cut -d' ' -f1)  $f"; done

# Verify against manifest
cat manifests/canon_v1.0.1.json
```

---

## Historical Context

In 1494, Luca Pacioli codified double-entry bookkeeping. The innovation was mechanical: every transaction recorded twice. Fraud required falsifying two coordinated entries. Honesty became cheaper than deception.

Within a century, the merchants who adopted it dominated Mediterranean trade.

LogLine is the same pattern for computation. We make accountability structural. We make verification cheaper than argument. We make honesty the dominant strategy.

---

## Citation

```bibtex
@techreport{logline2026,
  title     = {LogLine SecurityOS: A Protocol Suite for Verifiable Accountability},
  author    = {Voulez, Dan},
  year      = {2026},
  month     = {February},
  institution = {The LogLine Foundation},
  version   = {1.0.1}
}
```

---

## License

- **Protocol Specifications:** Open
- **Documentation:** CC BY 4.0
- **Reference Implementations:** See individual repositories

---

<p align="center">
  <strong>The LogLine Foundation — 2026</strong><br>
  <br>
  <em>"Receipts or it didn't happen."</em>
</p>
