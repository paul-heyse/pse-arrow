---
name: design-review
description: Review a proposed design document or an active codebase against the Data Model–Based Design Charter (DM-01–DM-60, gates G1–G7), producing an evidence-grounded review document under docs/design_review/reviews/
allowed-tools: Read, Glob, Grep, Bash, Write, Edit, Agent
user-invocable: true
model-baseline: claude-5 (2026-08)
---

# Design Review — Data Model–Based Design Charter

Review a **proposed design** (document) or an **existing implementation** (code) against the doctrine in `docs/design_review/design_principles/`, and produce one review document.

**How you conduct the analysis is yours to decide.** Where you start, what you read and in what order, which tools you use, whether you delegate breadth — all of it is your call. What this skill fixes is the other end: what the review has to establish, what a claim in it must be backed by, and how the result is organized. Everything in [REFERENCE.md](REFERENCE.md) beyond the principle index is a lens offered because it has proven useful, not a step you owe anyone.

## The standard

| File (under `docs/design_review/design_principles/`) | Role |
|---|---|
| `DATA_MODEL_DESIGN_CHARTER.md` | Normative. DM-01–DM-60, gates G1–G7, dimensions, evidence vocabulary, §E–§G tests. |
| `AGENT_DESIGN_DIRECTIVE.md` | The governing objective and required approach, compressed. |
| `DESIGN_REVIEW_TEMPLATE.md` | The output shape (§1–§11). |

The charter is the standard; without it there is nothing to review against, so if it is missing, stop and say so. [REFERENCE.md](REFERENCE.md) §1 indexes the DM IDs with their MUST/SHOULD levels for citation accuracy — it is a lookup table, not a substitute for the charter.

You are judging **meaning and authority**, not vocabulary, technology, or volume. Charter §G exists because the most common way this review goes wrong is rating a design highly for fluently using the doctrine's own words.

## Arguments

- **target** (required): a design document path; a code scope (directory, crate, module, glob, file list); or both — a document plus the code claiming to implement it.
- **focus** (optional): principle groups `1`–`12`, specific DM IDs, or gate IDs to emphasize. Focus shifts where depth goes; it does not suppress a MUST-level defect found outside it.
- **depth** (optional): `compact` (short assessment in the directive's style), `standard` (default), `deep` (adds adversarial journeys, a constructed counter-design, and verification detail).
- **slug** (optional): kebab-case filename descriptor; infer it if omitted.

## Output

```
docs/design_review/reviews/design_review_{slug}_{YYYY-MM-DD}.md
```

Follow `DESIGN_REVIEW_TEMPLATE.md` §1–§11, scoped to the change. Drop sections irrelevant to the scope with a one-line note rather than filling them with invented requirements; charter §A and the directive both say the review machinery should not cost more than the decision warrants. [REFERENCE.md](REFERENCE.md) §4 sketches how the sections tend to compress by mode and depth.

Two additions the template does not carry:

- **Method and coverage** (in §1). What you looked at, what you did not, what you could not verify and why. A reader has to be able to tell "no defect here" from "not inspected" — otherwise the review's silences read as assurance they haven't earned.
- **Applicability note** (in §7). Which principle groups bore on this scope and which did not, with the reason tied to scope. The charter permits "not applicable" only for a stated reason; unapplied is not the same as irrelevant.

Sections §6 (gates), §7 (findings), and §11 (decision) carry the review's weight and stay tabular at every depth.

Close by reporting to the user: scope and coverage, gate results, the top few findings in severity order, the decision, and the file path.

## What the review has to establish

1. **Gates G1–G7, settled independently.** Pass, fail, unresolved, or not-applicable — each on its own evidence. They are not averaged, not offset by a strong dimension score, and not softened because the design is otherwise impressive.
2. **A verdict per applicable principle, from three options.** *Satisfied* — the mechanism is identified and you can say where it is enforced and what it rejects. *Violated* — you can describe a concrete situation in which the required meaning is lost, ambiguous, contested, or unenforced. *Unresolved* — the design neither establishes nor precludes the requirement; the decision hasn't been made. Unresolved is the honest verdict for most document-stage gaps, and the charter is explicit that it is not a pass. Resist the pull to upgrade it because the surrounding design is good, or to downgrade it to a violation for rhetorical weight.
3. **Findings that survive contact with the design.** See *Finding quality* below.
4. **A decision that follows from the gates**, per the calibration table below.
5. **Claims labeled at the strength the evidence supports** — charter §D's vocabulary, applied honestly.

### Claim strength is bounded by what you actually have

This is the constraint most worth getting right, because it determines whether the review's conclusions mean anything.

**Reviewing a document**, you have claims, not behavior. You can establish whether the design is *specifiable* and *decidable* — whether two competent implementers reading only this would build the same semantics, and whether each invariant names a place it is enforced and something observable when it is violated. You cannot establish correctness. Evidence labels therefore top out at **Proposed**, or **Interface-checked** where you actually inspected the interface or library surface named. A performance claim in a design document is a hypothesis (DM-39) unless it cites a measurement with its conditions; say that rather than letting it pass.

**Reviewing code**, you have behavior. Docstrings, names, type aliases, and design notes are claims *about* the code; the path that executes is the evidence. Cite `file:line` with the actual expression rather than a paraphrase, and only cite what you have read yourself — that applies equally to anything a subagent hands you, which is a lead until you've looked. **Tested** and **Measured** require naming the test or benchmark and its conditions; "there are tests" is not Tested.

**Reviewing both together**, the additional question is where the document's semantics and the implementation's diverge, and which of them is authoritative. A design document maintained independently of the code it describes is itself a second authority under DM-02 — worth saying plainly, along with the reconciliation path.

## Judgment calibration

**Severity ordering.** Rank findings by: correctness and authority defects (G1–G5 failures, MUST-level gaps on in-scope behavior) → semantic duplication and extension difficulty (charter §E) → measured performance and cost. Let that ordering drive §11's priority column. A finding that moves none of the three is an observation; label it as one or cut it.

**Proportionality cuts both ways.** The directive is explicit that a design should not become an unnecessary platform to satisfy a checklist. Under-specification is a finding; so is machinery — a registry, compiler, generator, IR layer, service, plugin system — that has no demonstrated consumer or costs more than the risk it addresses (DM-56, DM-57, DM-58). Recommending *removal* is a legitimate and sometimes the strongest outcome. A specialized algorithm behind a complete contract is aligned; charter §F is the placement test, and "make it declarative" is not the default answer. Touching many files is not a defect — the same meaning independently re-expressed across subsystems is.

**Unattacked guarantees are asserted, not verified.** Trying to break a claimed guarantee is usually what separates a review that helps from one that summarizes. The situations that tend to reward the attempt: two authorities disagreeing; an invalid value reaching an operation that assumes validity; a retry double-applying an effect; an interruption leaving a partial output indistinguishable from a committed one; a rewrite or cache hit changing the answer; a backend accepting an operation and lowering it differently. Whichever you attempt, record guarantees you did not attack as asserted, and say so in the Method note.

**The simpler alternative is part of the output.** Template §8 wants a real third row, and charter §C asks for the comparison. Constructing one takes work and is easy to skip; when the simpler alternative wins, that is usually the review's headline.

## Finding quality

Each §7 row carries six fields. The middle three are what make it a finding rather than a preference:

| Field | What makes it adequate |
|---|---|
| Finding | One substantive defect, stated as a claim that could turn out to be wrong. |
| Principle IDs | Only IDs the argument actually uses. If deleting a citation wouldn't change the reasoning, it was decoration. |
| Evidence or gap | Code: `file:line` and the expression. Document: the section, the quoted claim, and what is absent at that exact point. "Not addressed anywhere" is a legitimate gap when stated precisely. |
| Consequence | A concrete situation — inputs or state → wrong, ambiguous, or unrecoverable outcome. If this sentence can't be written, there is no finding yet. |
| Proposed correction | A direction with a rough surface area. Not a patch, not a signature. |
| Verification | The check that would show the correction landed and catch the regression (DM-60). |

Group by **cause**, not symptom: several instances of one structural cause are one finding citing several instances, ranked by the cause's severity rather than the instance count.

[REFERENCE.md](REFERENCE.md) §3 has worked adequate/inadequate pairs, including the over-construction finding that tends to go unwritten.

## Decision calibration (§11)

| Situation | Decision |
|---|---|
| No MUST gap and no failed or unresolved gate on in-scope behavior | Accept |
| Only SHOULD deviations, each with a §10 exception record | Accept scoped design with documented deviations |
| MUST gap or failed gate on behavior the design claims to support | Revise — or Accept with the supported scope explicitly narrowed to exclude it |
| Unresolved gate on in-scope behavior | Not Accept. Record it as unresolved and name the decision the author has to make |
| Competing authority, silent semantic loss, or an unbacked capability claim at the core | Reject or Revise, whatever the rest looks like |

Low code volume, elegance, and performance do not offset lost meaning, competing authority, hidden effects, inconsistent revisions, invalid reuse, or unsupported behavior.

## When the review isn't finished yet

Worth a pass before writing the file — each of these has a way of being true right up until you check:

- A finding somewhere has no concrete consequence written out, and is really a preference.
- A citation is doing no work in the argument that carries it.
- Something cited hasn't actually been read at the grain it's cited at.
- A gate verdict drifted toward the overall impression of the design rather than its own evidence, or *unresolved* got rounded to *pass*.
- An evidence label outruns what was inspected.
- No simpler alternative was constructed, so §8's third row is a placeholder.
- Over-construction went unexamined while under-specification got all the attention.
- Strengths are stated as praise rather than as what would break without them.
- The coverage note doesn't distinguish what was skipped from what was clean.
- The author would learn the design is imperfect but not *where it is unsafe*.

## Failure modes in the output

A review with any of these is worse than none, because it presents a checklist as assurance:

- All 60 principles enumerated regardless of scope.
- Dimension scores computed while gates went unevaluated, or a score offsetting a failed gate.
- Alignment asserted from vocabulary — *canonical*, *typed*, *contract*, *declarative*, *zero-copy*. Charter §G lists the hidden defect to check behind each attractive claim.
- Findings with no consequence.
- Platform machinery recommended for a scope that hasn't demonstrated the need.
- *Tested* or *Measured* claimed without naming the test or benchmark.
- File or line counts offered as evidence of semantic duplication.
- Unverified secondhand evidence, from a subagent or from the design's own prose.

## Edge cases

| Situation | Suggested handling |
|---|---|
| Charter missing or unreadable | Stop and report. |
| Template or directive missing | Proceed against the charter; note the substitution. |
| Target ambiguous — document or code unclear | Ask. What the review can establish depends on it. |
| Scope larger than the depth supports | Narrow to the semantically load-bearing part, state the selection rule, say what was excluded. |
| Document describes code that doesn't exist yet | Document-stage claims only; a document's confidence is not evidence about an implementation. |
| Document and code disagree | Report the divergence as a finding against the pair, and name which is meant to be authoritative. |
| A claim can't be verified with what's available | Record it as asserted, name the check that would settle it, put it in §9. Don't guess a verdict. |
| Nothing wrong found | Say so, with the coverage statement and what was examined. Don't manufacture findings. |
| Scope is a mechanical refactor or pure performance change | Say which burdens of proof apply and compress the semantic sections accordingly. |
