---
id: llf.paper.tdln.v1
title: "Paper IV — TDLN: The Policy Compiler"
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
gates_required: ["G1", "G2", "G3", "G4"]
thesis: "Intention, when normalized under a signed constitution, becomes a logical atom: canonical, proven, governable, and ready to execute."
hash: ""
signer: ""
---

# Paper IV — TDLN: The Policy Compiler

**Deterministic Translation of Natural Language**

*Normative keywords per RFC 2119/8174 (MUST/SHOULD/MAY) apply.*

---

## The Story

**September 2024. An AI trading system. A regulatory investigation.**

The system was supposed to "avoid trades that might manipulate the market." The developers interpreted this as: flag trades above $1 million. The regulators interpreted it as: detect coordinated patterns regardless of size.

Six months of trades were executed under the wrong interpretation. $340 million in fines. The investigation's central question: **"What did 'might manipulate' actually mean in this system?"**

No one could answer. The policy existed as a comment in the code:

```python
# Avoid trades that might manipulate the market
if trade.amount > 1_000_000:
    flag_for_review(trade)
```

The gap between human intent and machine execution was invisible. The interpretation was buried in an engineer's decision from months ago. There was no proof that this interpretation matched the policy. There was no way to verify what "should have" happened.

**Now imagine a different architecture.**

The policy is written in TDLN:

```
ruleset market_safeguards@v1.0

@policy manipulation_detection
@description "Detect trades that might manipulate the market"
when trade.amount > 1000000: flag_review
when trade.pattern IN ["wash", "layering", "spoofing"]: flag_review
when trade.velocity > context.market.avg_velocity * 3: flag_review
```

This policy compiles to a canonical AST with a deterministic CID:

```
canon_cid: b3:4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a...
```

The compilation produces a proof binding the source text to the AST. The proof is signed. If regulators ask "what did this policy mean?", the answer is the AST. If they ask "did the system follow the policy?", the answer is: compare the decision receipts to the policy hash.

**The investigation becomes a hash comparison.**

This is TDLN.

---

## I. The Problem

Natural language is ambiguous. Code is rigid. The gap between human intent and machine execution is where systems fail.

Traditional approaches:
- Developers interpret requirements (lossy)
- Code encodes interpretation (drift)
- Tests verify code, not intent (mismatch)
- Production reveals the gap (incident)

**The translation from "what I meant" to "what the machine did" is invisible, unverifiable, and unreproducible.**

---

## II. The Thesis

> **Intention, when compiled under a signed ruleset, becomes a canonical, proof-carrying bundle.**

TDLN defines:
1. A typed **policy language** without ambiguity
2. A **canonicalization function** (ρ) that normalizes ASTs
3. A **translation proof** binding source to output
4. A **Gate** that evaluates bundles deterministically

**The compiler is governance. The proof is accountability.**

---

## III. Install It Now

```bash
# Add to your Rust project
cargo add tdln

# Or install the CLI
cargo install logline-cli
```

```rust
use tdln::{Compiler, Gate, PolicySet, SemanticUnit};

fn main() -> Result<(), tdln::Error> {
    // Load a policy file
    let source = r#"
        ruleset transfer_policy@v1.0

        @policy kyc_required
        @description "Require KYC for large transfers"
        when amount > 1000: require context.user.kyc_verified == true

        @policy manager_approval
        @description "Require manager approval for very large transfers"
        when amount > 10000: require confirmed_by IN ["manager", "director"]
    "#;

    // Compile with proof generation
    let compiler = Compiler::new("nv-gate.v0.51")?;
    let (policy_set, proof) = compiler.compile_with_proof(source)?;

    println!("Policy set hash: {}", policy_set.hash());
    println!("Proof canon_cid: {}", proof.canon_cid);

    // Evaluate a transfer intent
    let intent = SemanticUnit::transfer(1500, "alice", "bob");
    let gate = Gate::new(&policy_set)?;

    let decision = gate.evaluate(&intent, &context)?;
    println!("Decision: {:?}", decision);

    Ok(())
}
```

---

## IV. The Architecture

### Actors

| Actor | Role |
|-------|------|
| **Author** | Proposes intent in NL or DSL |
| **Compiler** | Transforms intent → AST + proof |
| **Gate** | Evaluates under signed policy set |
| **Executor** | Realizes effects with valid receipt only |
| **Ledger** | Records everything before execution |

### Boundaries

```
┌─────────────────────────────────────────────────────────┐
│  NATURAL LANGUAGE / DSL                                 │
│  (Ambiguous, human-authored)                            │
├─────────────────────────────────────────────────────────┤
│  COMPILER                                               │
│  (Translation + Proof Generation)                       │
│  Free text ENDS here                                    │
├─────────────────────────────────────────────────────────┤
│  CORE AST                                               │
│  (Canonical, typed, deterministic)                      │
├─────────────────────────────────────────────────────────┤
│  GATE                                                   │
│  (Policy evaluation + Receipt generation)               │
├─────────────────────────────────────────────────────────┤
│  EXECUTOR                                               │
│  (Effects only with valid receipt)                      │
└─────────────────────────────────────────────────────────┘
```

**The Hard Boundary:** The executor MUST NOT interpret free text. Data is not instructions.

---

## V. The Type System

```rust
// tdln/src/types.rs

/// TDLN's type system - no ambiguity, no surprises
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    // Base types
    Bool,
    Int,      // Arbitrary precision, no floats
    String,
    Time,     // UTC, RFC3339, nanosecond precision

    // Composite types
    Tuple(Vec<Type>),
    Array(Box<Type>),
    Map(Box<Type>),  // Keys are always strings

    // Special types
    Capability(CapabilityType),
    Did,
}

/// Capabilities form a lattice
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {
    pub kind: CapabilityKind,
    pub resource: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityKind {
    Read,
    Write,
    Emit,
    Call,
}

impl Capability {
    /// Check if this capability subsumes another
    /// write:ledger.append ⊑ write:ledger:* ⊑ write:*
    pub fn subsumes(&self, other: &Capability) -> bool {
        if self.kind != other.kind {
            return false;
        }

        // Wildcard matching
        if self.resource == "*" {
            return true;
        }

        if self.resource.ends_with(":*") {
            let prefix = &self.resource[..self.resource.len() - 1];
            return other.resource.starts_with(prefix);
        }

        self.resource == other.resource
    }
}

/// The capability lattice
pub struct CapabilityLattice {
    granted: Vec<Capability>,
}

impl CapabilityLattice {
    pub fn check(&self, required: &Capability) -> bool {
        self.granted.iter().any(|g| g.subsumes(required))
    }
}
```

### Time

- Normalized to UTC
- RFC3339 format
- No ambiguous timezones
- Nanosecond precision

```rust
// tdln/src/time.rs

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TdlnTime {
    nanos: u64,  // Nanoseconds since Unix epoch, UTC
}

impl TdlnTime {
    pub fn now() -> Self {
        Self {
            nanos: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        }
    }

    pub fn to_rfc3339(&self) -> String {
        // Always UTC, always full precision
        chrono::DateTime::from_timestamp_nanos(self.nanos as i64)
            .to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
    }

    pub fn from_rfc3339(s: &str) -> Result<Self, TimeError> {
        let dt = chrono::DateTime::parse_from_rfc3339(s)
            .map_err(|_| TimeError::InvalidFormat)?;
        Ok(Self {
            nanos: dt.timestamp_nanos_opt().unwrap() as u64,
        })
    }
}
```

---

## VI. The Policy Language

### Grammar (EBNF)

```ebnf
ruleset     := 'ruleset' IDENT '@' SEMVER
policy      := '@policy' IDENT description? condition+ composition?
description := '@description' STRING
condition   := 'when' expression ':' action
expression  := term (OP term)*
term        := literal | context_ref | func_call | '(' expression ')'
action      := 'require' expression | 'deny' | 'allow' | 'flag_review'
composition := 'compose' aggregator '(' IDENT (',' IDENT)* ')'
aggregator  := 'all' | 'any' | 'majority' | 'weighted'
OP          := 'AND' | 'OR' | '==' | '!=' | '>' | '<' | '>=' | '<=' | 'IN'
```

### Constraints (Enforced at Compile Time)

- **No loops** - Guaranteed termination
- **No recursion** - Guaranteed termination
- **No floats** - Exact arithmetic only
- **All functions are total** - Always terminate
- **All functions are pure** - No side effects

```rust
// tdln/src/parser.rs

pub struct Parser {
    lexer: Lexer,
    ruleset_id: Option<String>,
}

impl Parser {
    pub fn parse(&mut self, source: &str) -> Result<Ruleset, ParseError> {
        self.lexer = Lexer::new(source);

        let ruleset = self.parse_ruleset()?;

        // Enforce constraints
        self.check_no_loops(&ruleset)?;
        self.check_no_recursion(&ruleset)?;
        self.check_no_floats(&ruleset)?;
        self.check_total_functions(&ruleset)?;
        self.check_pure_functions(&ruleset)?;

        Ok(ruleset)
    }

    fn check_no_loops(&self, ruleset: &Ruleset) -> Result<(), ParseError> {
        for policy in &ruleset.policies {
            for condition in &policy.conditions {
                if self.contains_loop(&condition.expression) {
                    return Err(ParseError::LoopDetected {
                        policy: policy.name.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    fn check_no_floats(&self, ruleset: &Ruleset) -> Result<(), ParseError> {
        for policy in &ruleset.policies {
            for condition in &policy.conditions {
                if self.contains_float(&condition.expression) {
                    return Err(ParseError::FloatDetected {
                        policy: policy.name.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}
```

---

## VII. The Core AST

### Policy Bit

The atomic unit of policy - a semantic transistor:

```rust
// tdln/src/ast.rs

/// A PolicyBit is the atomic unit of policy - a semantic transistor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyBit {
    pub node_type: String,  // Always "policy_bit"
    pub id: ContentAddress, // Deterministic CID
    pub name: String,
    pub description: Option<String>,
    pub condition: Expression,
    pub action: Action,
    pub fallback: bool,     // Fail-closed default
    pub requires_cap: Vec<Capability>,
}

impl PolicyBit {
    /// Compute deterministic ID from content
    pub fn compute_id(&self) -> ContentAddress {
        let canonical = json_atomic::canonize(&PolicyBitContent {
            name: &self.name,
            condition: &self.condition,
            action: &self.action,
        }).unwrap();
        ContentAddress::from_blake3(blake3::hash(&canonical))
    }

    /// Evaluate this policy bit against a context
    pub fn evaluate(&self, context: &Context) -> Result<BitResult, EvalError> {
        let condition_result = self.condition.evaluate(context)?;

        if condition_result {
            Ok(BitResult::Triggered(self.action.clone()))
        } else {
            Ok(BitResult::NotTriggered)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Allow,
    Deny,
    Require(Expression),  // Must evaluate to true
    FlagReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BitResult {
    Triggered(Action),
    NotTriggered,
}
```

### Policy Composition

```rust
/// Compose multiple policies with aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyComposition {
    pub node_type: String,  // Always "policy_composition"
    pub id: ContentAddress,
    pub composition_type: CompositionType,
    pub policies: Vec<ContentAddress>,  // IDs in canonical order
    pub aggregator: Aggregator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompositionType {
    Sequential,  // Evaluate in order, short-circuit
    Parallel,    // Evaluate all, aggregate
    Conditional, // If-then-else
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Aggregator {
    All,       // All must pass
    Any,       // At least one must pass
    Majority,  // >50% must pass
    Weighted { threshold: u32, weights: HashMap<ContentAddress, u32> },
}

impl PolicyComposition {
    pub fn evaluate(
        &self,
        bits: &HashMap<ContentAddress, PolicyBit>,
        context: &Context,
    ) -> Result<CompositionResult, EvalError> {
        let results: Vec<_> = self.policies.iter()
            .map(|id| {
                let bit = bits.get(id).ok_or(EvalError::MissingPolicy(*id))?;
                bit.evaluate(context)
            })
            .collect::<Result<_, _>>()?;

        match self.aggregator {
            Aggregator::All => {
                let all_pass = results.iter().all(|r| !matches!(r, BitResult::Triggered(Action::Deny)));
                Ok(if all_pass { CompositionResult::Allow } else { CompositionResult::Deny })
            }
            Aggregator::Any => {
                let any_pass = results.iter().any(|r| matches!(r, BitResult::Triggered(Action::Allow)));
                Ok(if any_pass { CompositionResult::Allow } else { CompositionResult::Deny })
            }
            Aggregator::Majority => {
                let pass_count = results.iter()
                    .filter(|r| !matches!(r, BitResult::Triggered(Action::Deny)))
                    .count();
                Ok(if pass_count * 2 > results.len() {
                    CompositionResult::Allow
                } else {
                    CompositionResult::Deny
                })
            }
            Aggregator::Weighted { threshold, ref weights } => {
                let total_weight: u32 = self.policies.iter()
                    .filter(|id| {
                        let idx = self.policies.iter().position(|x| x == *id).unwrap();
                        !matches!(results[idx], BitResult::Triggered(Action::Deny))
                    })
                    .map(|id| weights.get(id).copied().unwrap_or(1))
                    .sum();
                Ok(if total_weight >= threshold {
                    CompositionResult::Allow
                } else {
                    CompositionResult::Deny
                })
            }
        }
    }
}
```

### Semantic Unit

The complete compiled intent:

```rust
/// A SemanticUnit is the complete compiled intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticUnit {
    pub node_type: String,  // Always "semantic_unit"
    pub kind: String,       // "transfer" | "deploy" | "evaluate"
    pub slots: HashMap<String, Value>,
    pub inputs: Vec<Parameter>,
    pub policies: Vec<PolicyRef>,
    pub hal_ref: ContentAddress,
    pub rule_set_id: String,
    pub policy_set_hash: ContentAddress,
    pub source_hash: ContentAddress,
    pub ast_cid: ContentAddress,
    pub canon_cid: ContentAddress,
}

impl SemanticUnit {
    pub fn transfer(amount: u64, from: &str, to: &str) -> Self {
        Self {
            node_type: "semantic_unit".to_string(),
            kind: "transfer".to_string(),
            slots: [
                ("amount".to_string(), Value::Int(amount as i64)),
                ("from".to_string(), Value::String(from.to_string())),
                ("to".to_string(), Value::String(to.to_string())),
            ].into_iter().collect(),
            inputs: vec![],
            policies: vec![],
            hal_ref: ContentAddress::default(),
            rule_set_id: String::new(),
            policy_set_hash: ContentAddress::default(),
            source_hash: ContentAddress::default(),
            ast_cid: ContentAddress::default(),
            canon_cid: ContentAddress::default(),
        }
    }
}
```

---

## VIII. Canonicalization (ρ)

The function ρ normalizes an AST to canonical form.

```rust
// tdln/src/canonicalize.rs

/// The canonicalization function ρ
/// Guarantees: Idempotence, Confluence, Stability
pub fn rho(ast_raw: &Ast, ruleset: &RulesetConfig) -> Result<CanonResult, CanonError> {
    // Step 1: Normalize keys (lexicographic order)
    let ast = normalize_keys(ast_raw)?;

    // Step 2: Normalize slots (canonical symbols)
    let ast = normalize_slots(ast, &ruleset.synonym_table)?;

    // Step 3: Normalize conditions (CNF form)
    let ast = normalize_conditions(ast, NormalForm::Cnf)?;

    // Step 4: Simplify boolean expressions
    let ast = simplify_bool(ast)?;

    // Step 5: Rewrite IDs (deterministic CIDs)
    let ast = rewrite_ids(ast)?;

    // Step 6: Serialize to canonical bytes (Paper II)
    let bytes = json_atomic::canonize(&ast)?;

    // Step 7: Compute canonical CID
    let canon_cid = ContentAddress::from_blake3(blake3::hash(&bytes));

    Ok(CanonResult {
        ast,
        canon_cid,
        bytes,
    })
}

/// Boolean simplification rules
fn simplify_bool(ast: Ast) -> Result<Ast, CanonError> {
    ast.transform(|expr| {
        match expr {
            // A ∧ ⊤ → A
            Expr::And(a, b) if *b == Expr::True => *a,
            // A ∧ ⊥ → ⊥
            Expr::And(_, b) if *b == Expr::False => Expr::False,
            // A ∨ ⊤ → ⊤
            Expr::Or(_, b) if *b == Expr::True => Expr::True,
            // A ∨ ⊥ → A
            Expr::Or(a, b) if *b == Expr::False => *a,
            // ¬¬A → A
            Expr::Not(box Expr::Not(box a)) => a,
            // Keep as is
            other => other,
        }
    })
}

/// Convert to Conjunctive Normal Form
fn normalize_conditions(ast: Ast, form: NormalForm) -> Result<Ast, CanonError> {
    ast.transform_conditions(|cond| {
        match form {
            NormalForm::Cnf => to_cnf(cond),
            NormalForm::Dnf => to_dnf(cond),
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_idempotence() {
        let ast = parse_ast("when x > 10: deny")?;
        let ruleset = RulesetConfig::default();

        let result1 = rho(&ast, &ruleset)?;
        let result2 = rho(&result1.ast, &ruleset)?;

        // ρ(ρ(AST)) = ρ(AST)
        assert_eq!(result1.canon_cid, result2.canon_cid);
        assert_eq!(result1.bytes, result2.bytes);
    }

    #[test]
    fn test_confluence() {
        // Different source representations, same meaning
        let ast_a = parse_ast("when x > 10 AND y < 5: deny")?;
        let ast_b = parse_ast("when y < 5 AND x > 10: deny")?;  // Reordered

        let ruleset = RulesetConfig::default();

        let result_a = rho(&ast_a, &ruleset)?;
        let result_b = rho(&ast_b, &ruleset)?;

        // Same meaning → same canon_cid
        assert_eq!(result_a.canon_cid, result_b.canon_cid);
    }
}
```

---

## IX. Translation Proof

Every compilation produces a proof:

```rust
// tdln/src/proof.rs

/// Proof that translation was performed correctly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationProof {
    pub proof_type: String,  // Always "translation"
    pub ruleset_id: String,
    pub source_hash: ContentAddress,
    pub ast_cid: ContentAddress,
    pub canon_cid: ContentAddress,
    pub steps: Vec<TranslationStep>,
    pub compiler_hash: ContentAddress,
    pub compiler_kid: String,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationStep {
    pub step: String,
    pub input_hash: ContentAddress,
    pub output_hash: ContentAddress,
    pub rule_applied: Option<String>,
}

impl TranslationProof {
    /// Verify the proof
    pub fn verify(
        &self,
        source: &str,
        public_key: &VerifyingKey,
    ) -> Result<(), ProofError> {
        // 1. Verify source hash
        let computed_source_hash = ContentAddress::from_blake3(
            blake3::hash(source.as_bytes())
        );
        if computed_source_hash != self.source_hash {
            return Err(ProofError::SourceMismatch);
        }

        // 2. Re-execute ρ and compare
        let compiler = Compiler::load(&self.compiler_hash)?;
        let (_, reproduced) = compiler.compile_with_proof(source)?;

        if reproduced.canon_cid != self.canon_cid {
            return Err(ProofError::CanonCidMismatch {
                expected: self.canon_cid.clone(),
                reproduced: reproduced.canon_cid,
            });
        }

        // 3. Verify step chain
        self.verify_step_chain()?;

        // 4. Verify signature
        let canonical = json_atomic::canonize(self)?;
        public_key.verify(&canonical, &self.signature)
            .map_err(|_| ProofError::InvalidSignature)?;

        Ok(())
    }

    fn verify_step_chain(&self) -> Result<(), ProofError> {
        for i in 1..self.steps.len() {
            if self.steps[i].input_hash != self.steps[i - 1].output_hash {
                return Err(ProofError::BrokenStepChain { step: i });
            }
        }
        Ok(())
    }
}
```

---

## X. The Gate

The Gate is the policy evaluator—the semantic transistor.

```rust
// tdln/src/gate.rs

/// The Gate evaluates policies and produces receipts
pub struct Gate {
    policy_set: PolicySet,
    hal: HardwareAbstractionLayer,
}

/// Decision outcomes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decision {
    Allow,
    Deny { reason: DenyReason, policy_id: ContentAddress },
    Require { signers: Vec<Did>, quorum: Quorum, expires: Timestamp },
    Ghost { reason: GhostReason },
    PlanInvalid { reason: String },
}

impl Gate {
    pub fn new(policy_set: &PolicySet) -> Result<Self, GateError> {
        Ok(Self {
            policy_set: policy_set.clone(),
            hal: HardwareAbstractionLayer::default(),
        })
    }

    /// Evaluate a semantic unit and produce a receipt
    pub fn evaluate(
        &self,
        unit: &SemanticUnit,
        context: &Context,
    ) -> Result<(Decision, PowerReceipt), GateError> {
        // 1. Check HAL constraints
        if let Err(e) = self.hal.check_constraints(unit) {
            return Ok((
                Decision::PlanInvalid { reason: format!("HAL violation: {}", e) },
                PowerReceipt::plan_invalid(unit, e),
            ));
        }

        // 2. Check capability lattice
        let required_caps = self.extract_required_capabilities(unit);
        let granted_caps = context.capabilities();

        for cap in &required_caps {
            if !granted_caps.check(cap) {
                return Ok((
                    Decision::Deny {
                        reason: DenyReason::CapabilityMissing {
                            required: cap.clone(),
                            granted: granted_caps.granted.clone(),
                        },
                        policy_id: ContentAddress::default(),
                    },
                    PowerReceipt::deny(unit, "capability_missing"),
                ));
            }
        }

        // 3. Evaluate policy bits
        let mut triggered_actions = Vec::new();

        for policy_ref in &unit.policies {
            let policy = self.policy_set.get(policy_ref)?;
            let result = policy.evaluate(context)?;

            if let BitResult::Triggered(action) = result {
                triggered_actions.push((policy_ref, action));
            }
        }

        // 4. Determine final decision
        let decision = self.aggregate_decisions(&triggered_actions, context)?;

        // 5. Generate receipt
        let receipt = PowerReceipt {
            kind: "receipt.power.v1".to_string(),
            trace_id: Ulid::new(),
            rules_applied_id: self.policy_set.ruleset_id.clone(),
            policy_set_hash: self.policy_set.hash(),
            decision: decision.clone(),
            capabilities_granted: granted_caps.granted.clone(),
            capabilities_required: required_caps,
            hal_ref: self.hal.hash(),
            safeguards: self.evaluate_safeguards(context),
            ethics_efficiency: self.compute_ethics_efficiency(context),
            inputs_hash: unit.compute_inputs_hash(),
            compiler_hash: unit.compiler_hash(),
            signature: Signature::pending(),
            issued_at: Timestamp::now(),
        };

        Ok((decision, receipt))
    }

    fn compute_ethics_efficiency(&self, context: &Context) -> EthicsEfficiency {
        let mut score = 1.0;
        let mut penalties = Vec::new();

        // Check isolation
        if context.requires_isolation() && !context.isolation_applied() {
            score -= 0.25;
            penalties.push("Isolation bypassed in high-risk context");
        }

        // Check circuit breaker
        if context.breaker_at_threshold() && !context.mitigation_applied() {
            score -= 0.25;
            penalties.push("Circuit breaker at threshold, no mitigation");
        }

        // Check shadow validation
        if context.shadow_anomaly() && !context.human_verified() {
            score -= 0.25;
            penalties.push("Shadow validation anomalous, no human check");
        }

        EthicsEfficiency {
            score: score.max(0.0),
            rationale: if penalties.is_empty() {
                "All safeguards active".to_string()
            } else {
                penalties.join("; ")
            },
        }
    }
}
```

### Power Receipt

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerReceipt {
    pub kind: String,
    pub trace_id: Ulid,
    pub rules_applied_id: String,
    pub policy_set_hash: ContentAddress,
    pub decision: Decision,
    pub capabilities_granted: Vec<Capability>,
    pub capabilities_required: Vec<Capability>,
    pub hal_ref: ContentAddress,
    pub safeguards: SafeguardStatus,
    pub ethics_efficiency: EthicsEfficiency,
    pub inputs_hash: ContentAddress,
    pub compiler_hash: ContentAddress,
    pub signature: Signature,
    pub issued_at: Timestamp,
}

impl PowerReceipt {
    /// Verify the receipt
    pub fn verify(&self, public_key: &VerifyingKey) -> Result<(), ReceiptError> {
        let canonical = json_atomic::canonize(self)?;
        public_key.verify(&canonical, &self.signature)
            .map_err(|_| ReceiptError::InvalidSignature)
    }
}
```

---

## XI. Consent Protocol

When `required_cap ⊒ threshold_cap`:

```rust
// tdln/src/consent.rs

/// Consent receipt for high-privilege operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentReceipt {
    pub kind: String,  // "receipt.consent.v1"
    pub parent_trace_id: Ulid,
    pub who: Did,
    pub confirmed_by: Did,
    pub capabilities: Vec<Capability>,
    pub scope: ConsentScope,
    pub expiry: Timestamp,
    pub nonce: ContentAddress,
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentScope {
    pub canon_cid: ContentAddress,
    pub ruleset_id: String,
}

impl ConsentReceipt {
    /// Check if consent is valid for a given operation
    pub fn is_valid_for(
        &self,
        operation: &SemanticUnit,
        now: Timestamp,
    ) -> Result<(), ConsentError> {
        // Check expiry
        if now > self.expiry {
            return Err(ConsentError::Expired);
        }

        // Check scope matches
        if self.scope.canon_cid != operation.canon_cid {
            return Err(ConsentError::ScopeMismatch);
        }

        // Check capabilities cover requirements
        let required = operation.required_capabilities();
        for cap in &required {
            if !self.capabilities.iter().any(|c| c.subsumes(cap)) {
                return Err(ConsentError::InsufficientCapabilities);
            }
        }

        Ok(())
    }

    /// Verify signature
    pub fn verify(&self, public_key: &VerifyingKey) -> Result<(), ConsentError> {
        let canonical = json_atomic::canonize(self)?;
        public_key.verify(&canonical, &self.signature)
            .map_err(|_| ConsentError::InvalidSignature)
    }
}
```

---

## XII. Hardware Abstraction Layer (HAL)

The HAL declares what effects are permitted:

```rust
// tdln/src/hal.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareAbstractionLayer {
    pub target: Target,
    pub memory: MemoryConfig,
    pub io: IoPermissions,
    pub side_effects: SideEffectPolicy,
    pub time: TimeConfig,
    pub forbid: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoPermissions {
    pub read: Vec<String>,
    pub write: Vec<String>,
    pub emit: Vec<String>,
    pub call: Vec<String>,
}

impl HardwareAbstractionLayer {
    /// Check if an operation is allowed by this HAL
    pub fn check_constraints(&self, unit: &SemanticUnit) -> Result<(), HalError> {
        // Check memory limits
        if unit.estimated_memory() > self.memory.max_pages * 65536 {
            return Err(HalError::MemoryExceeded);
        }

        // Check I/O permissions
        for read_op in unit.read_operations() {
            if !self.io.read.iter().any(|p| matches_pattern(p, &read_op)) {
                return Err(HalError::ReadNotAllowed(read_op));
            }
        }

        for write_op in unit.write_operations() {
            if !self.io.write.iter().any(|p| matches_pattern(p, &write_op)) {
                return Err(HalError::WriteNotAllowed(write_op));
            }
        }

        // Check forbidden operations
        for forbidden in &self.forbid {
            if unit.uses_resource(forbidden) {
                return Err(HalError::ForbiddenResource(forbidden.clone()));
            }
        }

        Ok(())
    }
}

fn matches_pattern(pattern: &str, resource: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.ends_with("*") {
        return resource.starts_with(&pattern[..pattern.len() - 1]);
    }
    pattern == resource
}
```

**The Rule:** Executor MUST refuse operations outside HAL.

---

## XIII. CLI Usage

```bash
# Compile a policy file
logline tdln compile policy.tdln -o policy.ast.json

# Output:
# Compiled policy.tdln
# canon_cid: b3:4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a...
# Proof written to policy.proof.json

# Verify a translation proof
logline tdln verify-proof \
  --source policy.tdln \
  --proof policy.proof.json

# Output:
# Source hash: MATCH
# Canon CID: MATCH
# Step chain: VALID
# Signature: VALID
# Proof verification: PASS

# Evaluate an intent against a policy
logline tdln evaluate \
  --policy policy.ast.json \
  --intent '{"kind": "transfer", "amount": 5000, "to": "bob"}'

# Output:
# Decision: REQUIRE
# Required signers: manager, director
# Quorum: 1 of 2
# Receipt written to receipt.json

# Show policy as human-readable
logline tdln explain policy.ast.json

# Output:
# Ruleset: transfer_policy@v1.0
#
# Policy: kyc_required
#   When: amount > 1000
#   Action: require context.user.kyc_verified == true
#
# Policy: manager_approval
#   When: amount > 10000
#   Action: require confirmed_by IN ["manager", "director"]
```

---

## XIV. Conformance

| Test | Requirement |
|------|-------------|
| **CT-PRESERVATION** | Type safety preserved through evaluation |
| **CT-PROGRESS** | Well-typed AST reduces to true/false |
| **CT-ρ-IDEMP** | ρ(ρ(AST)) = ρ(AST) |
| **CT-HASH-STABILITY** | Same semantics → same canon_cid |
| **CT-CAP-CHECK** | All ALLOW decisions satisfy capability lattice |
| **CT-TOCTOU** | Hash drift invalidates execution |

```rust
#[cfg(test)]
mod conformance {
    #[test]
    fn ct_preservation() {
        let policy = parse("when amount > 1000: deny")?;
        let context = Context::new().with("amount", Value::Int(500));

        // Type safety preserved: amount is Int, 1000 is Int, comparison valid
        let result = policy.evaluate(&context)?;
        assert!(matches!(result, BitResult::NotTriggered));
    }

    #[test]
    fn ct_progress() {
        let policy = parse("when true: allow")?;
        let context = Context::new();

        // Well-typed AST always reduces to a decision
        let result = policy.evaluate(&context)?;
        assert!(matches!(result, BitResult::Triggered(Action::Allow)));
    }

    #[test]
    fn ct_cap_check() {
        let gate = Gate::new(&policy_set)?;
        let unit = SemanticUnit::transfer(1000, "alice", "bob");
        let context = Context::new()
            .with_capabilities(vec![Capability::read("vault:balance")]);

        let (decision, _) = gate.evaluate(&unit, &context)?;

        // ALLOW only if capabilities sufficient
        if matches!(decision, Decision::Allow) {
            // Verify capability lattice satisfied
            for req in unit.required_capabilities() {
                assert!(context.capabilities().check(&req));
            }
        }
    }
}
```

---

## XV. The Invariant Connection

| Invariant | TDLN Implementation |
|-----------|---------------------|
| **I1** Integrity | canon_cid, proofs, receipts canonical and signed |
| **I2** Legality | deny/timeout → GHOST; violations are PLAN_INVALID |
| **I3** Attribution | who, confirmed_by, kid with Ed25519 |
| **I4** Reproducibility | Deterministic ρ; same ruleset → same decision |
| **I5** Observability | consent, ghost, drift metrics alertable |

---

## XVI. Conclusion

> **The compiler is governance. The proof is accountability.**

TDLN transforms the gap between intention and execution into a verifiable bridge:

- **Intent** enters as natural language or DSL
- **Compiler** produces canonical AST with proof
- **Gate** evaluates deterministically
- **Executor** moves only with valid receipt

The question "what did you mean?" becomes answerable: show the canon_cid.

The question "did the system follow the rules?" becomes computable: compare receipts to policy hash.

Without receipt, it didn't happen.
With receipt, it cannot be disputed.

---

## The Equation

```
Intent + Ruleset + Compiler = Canonical AST + Proof

Compilation is governance.
Proof is accountability.
```

---

*Next: [Paper V — SIRP](06_V_SIRP.md)*
