---
title: "26: Testing architecture"
status: draft
date: 2026-10-05
adrs: [ADR-0051, ADR-0092, ADR-0119, ADR-0122, ADR-0143, ADR-0145]
review_sources: ["../design_review/reviews/design_review_testing-architecture_2026-10-05.md"]
scenario_sources: ["../design_review/reviews/design_review_testing-architecture_2026-10-05.md#revealing-changes"]
---

# 26: Testing architecture

## Purpose, baseline and ownership

Make testing a small composition of production contracts, independent expectations and
library-owned execution. Remove repeated policy, calculation, validation and setup wherever
they establish no distinct claim. Correct production foundations that obstruct that target.
This is a **hard design pivot**, including breaking internal APIs and changing policies;
historical formats, suite names, enumeration compatibility and case counts do not constrain it.

The basis is the [testing-architecture review](../design_review/reviews/design_review_testing-architecture_2026-10-05.md),
F01–F09 and S01–S08, under Core 3.3 / ProcessSimulator 1.4. The inspected production baseline
is `9c953ba490731232bd9eca8b979f8cbee5092989`. The review is an uncommitted source artifact at
authoring time. Its source observations and the focused foundation assessment below are
**Implemented/source-inspected** or **Interface-checked**; the target and all work packages
are **Proposed**. No product tests, builds or measurements were run while writing these plans.

This coordinator owns the combined design, cross-plan dependencies, F01–F09 dispositions and
the series completion decision. Companions own their designs, package progress and execution
evidence. Architecture sections and ADRs retain their existing authority; a plan records the
proposed change, not a second production specification.

| Companion | Consumes and delivers | Progress owner |
|---|---|---|
| [26a: Completed results and transport](26a-completed-results-and-transport.md) | Consumes numerical completion and joined reports; delivers immutable coherent trajectory results and shared bounded export | Its package table |
| [26b: Test composition and independent evidence](26b-test-composition-and-independent-evidence.md) | Consumes intact model selection and declaration-owned invariant derivation; delivers focused independent mechanism evidence and retired duplicate machinery | Its package table |
| [26c: Test execution and qualification evidence](26c-test-execution-and-qualification-evidence.md) | Consumes categorized tests and production result boundaries; delivers actual-effect fixtures, one report composition and scoped evidence reuse | Its package table |

Plan 26 does not reopen Plan 25n or adopt AF findings. Plan 25k remains the owner of Plan 25's
paused full product campaign. Authoring this series neither resumes that campaign nor authorizes
production implementation. When both series' qualification is authorized on the same final
tree, execute overlapping scopes once and link the same scoped evidence from their respective
owners. Plan 26's own completion checks can run without claiming Plan 25 qualification.

## Shared target and assessed foundations

| Responsibility | Owner and target use |
|---|---|
| Model meaning and preparation | Authored declarations and production admission/preparation; tests select actual definitions rather than rewriting packages |
| Local layout/value validity | Generated construction and raw admission mint checked batches; consumers borrow certificates without redundant rescans |
| Bundle validity | Production invariant derivation and actual candidate admission; a field certificate does not establish keys, references or domain completeness |
| Scientific permission | Original-model assessment and numerical completion; consumer projections derive from one retained decision |
| Independent correctness | Focused tests use analytic values, distinct numerical/reference calculations or deliberately specified bad inputs capable of rejecting a wrong production result |
| Execution | nextest/pytest own collection, selection and scheduling; recipes own environments/features/profiles |
| Evidence | One thin owner composes actual selection, terminal results, relevant inputs and native identity, without redefining scientific validity |

The existing local certificates, original assessment, completion composition, native fault
controls and `RunResult` table retention are suitable foundations. Reuse them. The trajectory's
publicly mutable completed projections are unsuitable: seal them before retaining exports.
Its whole-collection encoder is reusable after completion/header coherence is corrected; lazy
successful retention avoids repeated work without adding a per-relation encoder hierarchy.

The registry's generic integrity producer is also suitable, but its current invariant catalog
does not expose generated versus authored origin. Add that distinction at the production
producer; do not infer it from test-owned name prefixes. Catalog-wide binding remains valuable
and different from independent family value semantics. Historical semantic fixture branches
do not establish a current production rule; preserve the actual loading/admission predicates.

The runner already deduplicates recipe leaves and retains failures, interruption and provenance.
Preserve those mechanisms while removing overlapping selections, native enumeration and report
parsing. Its global source snapshot is useful context, but is unsuitable as every claim's
reuse key. The selected replacement is explicit recipe-level input scopes, conservatively
broad within a subsystem, without automatic test-impact analysis.

### Intentional changes and preservation

Completed trajectory Rust fields become read-only accessors and clone-shared immutable state.
Test fixture discovery stops requiring a valid/violating YAML pair for each generated instance.
The ordinary comprehensive configuration runs common Rust obligations once in the linked graph;
default-feature absence behavior receives a focused, explicitly different selection. Python
markers describe actual responsibilities/effects. Per-test Git attribution is deleted.
Evidence formats may change without old-format readers or transfer shims.

Preserve explicit force-validation in every correctness recipe, typed failure and incomplete
evidence, native memory caps, actual resource leases, physical units/conventions, original-model
checks, callback fault controls, lifetime tests and independent scientific references. Preserve
runtime relational enforcement. A test is removable because its distinct failure is covered or
impossible under the consumed construction contract, not merely because a validator exists.

Keep tests of constructor/admission owners and bypass paths. Do not repeatedly test their
intrinsic guarantees at every consumer. Distinct mode behavior and independently authored
semantic rules still need distinct evidence. Upstream-only library probes belong in capability
research only when a current question warrants them.

## Decisions, authority routes and implementation order

### P0 — Record the policy and boundary decisions

Before governed implementation, create a decision record for testing responsibility and scoped
qualification, including owned effects, one freshness owner, crate retirement and the removal
of compatibility-only obligations. Supersede ADR-0051 through the established route: its body
explicitly requires the overlapping governance check that this design removes. Carry forward
its complete regeneration-equivalence guarantee and current generated-path ownership. Preserve
ADR-0092's useful evidence distinctions; do not supersede it merely to change a helper.

The decision route requires a bounded review of the selected governance/crate changes. The
principal review's **Revise** judgment is diagnosis, not acceptance of a newly selected remedy.
Schedule that review against the concrete target, without a second whole-codebase audit or
another scientific campaign. Record any separately required hashing/registry/Python boundary
decision where the chosen production change alters such a contract.

Amend blueprint §14.2, §19.2, §21 and §24.1/§24.3 as applicable through the design route and
revision row. Update the qualification guide, AGENTS cleanup/selection wording, recipe docs and
reference relationship documentation together with the governed changes. Accepted ADR bodies
remain immutable. Adding a third-party library does not require an ADR or exact pin.

| Step | Required capability and reason | Delivered work and owner |
|---|---|---|
| P0 | Concrete target in this series | Required decisions/review and authority amendments; coordinator owns the decision route |
| Result authority | Existing joined reports and completion; P0 where a public contract changes | [26a A1](26a-completed-results-and-transport.md#a1): immutable projections, required before reuse |
| Result transport | A1 working result and actual leases | [26a A2–A3](26a-completed-results-and-transport.md#a2): shared lazy export with migrated consumers, then migration audit |
| Selection and family provenance | Existing intact selection; production obligation producer | [26b B1–B2](26b-test-composition-and-independent-evidence.md#b1): workload selection and typed invariant derivation identity |
| Independent mechanism evidence | B2's actual provenance and admitted rule interfaces | [26b B3](26b-test-composition-and-independent-evidence.md#b3): family semantics, catalog binding and real enforcement before witness retirement |
| Test support and retirement | Distinct retained claims established; P0 for governance/crate changes | [26b B4–B5](26b-test-composition-and-independent-evidence.md#b4): strongest freshness owner and obsolete machinery deletion |
| Effects and execution | Stable responsibility categories; no dependency on every result package | [26c C1–C3](26c-test-execution-and-qualification-evidence.md#c1): explicit fixtures, owned effects and one selection/report composition |
| Evidence and measurement readiness | C3 truthful invocation records and workload prerequisites | [26c C4–C5](26c-test-execution-and-qualification-evidence.md#c4): relevant-input reuse and selected measurement prerequisites |
| Q1 | All functional packages integrated and replacements deleted | Coordinator's single assembled acceptance stage below |

This is a dependency route, not a barrier after every row. B1, C1/C2 and proven-unused support
retirement can proceed independently of trajectory export. B3 precedes generated witness
deletion; A1 precedes A2; C3 precedes evidence reuse. Production consumer migration belongs
with each replacement, not a later cleanup phase. Shared `justfile`, rule declarations,
test configuration and generated surfaces each have one assigned writer during execution.

## Finding dispositions

These are **scheduled design work**, not resolved findings or implementation authorization.
Each finding closes only when all linked correction and evidence obligations are met.

| Finding | Review scenarios | Disposition | Work owner | Closure condition |
|---|---|---|---|---|
| [F01](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f01) | S01/S04 | scheduled | 26b B1 | Intact production selection replaces strip/restore and all affected callers |
| [F02](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f02) | S05 | scheduled | 26b B4; 26c C3 | Strongest freshness owner retained; weaker check and duplicate execution removed |
| [F03](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f03) | S02/S03/S05 | scheduled | 26c C3 | Common obligations run once; mode-specific coverage and complete terminal reconciliation preserved |
| [F04](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f04) | S01/S03 | scheduled | 26c C1; 26b B1/B5 | No unrelated autouse/resource setup; support boundaries and actual effect classification agree |
| [F05](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f05) | S07 | scheduled | 26c C2 | Git attribution removed; owned effects and isolated negative write controls exercise the intended guarantee |
| [F06](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f06) | S06 | scheduled | 26c C4/C5 | Relevant inputs/prerequisites control reuse/measurement; irrelevant prose does not |
| [F07](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f07) | S01/S02 | scheduled | 26b B5 | Placeholders, empty structural crate, unused support and unneeded probes/compatibility cases retired |
| [F08](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f08) | S04/S08 | scheduled | 26a A1–A3 | Immutable coherent completion and kind-correct bounded export; lifetime/resource and refusal controls pass |
| [F09](../design_review/reviews/design_review_testing-architecture_2026-10-05.md#f09) | S01/S03/S05 | scheduled | 26b B2/B3/B4 | Independent family variation evidence plus every actual catalog binding/enforcement replaces instance duplication |

## Verification and Q1 assembled acceptance

All acceptance below is **Proposed**, not a record of executed commands. During implementation,
compile touched packages and run the named targeted controls in companions. Regenerate through
`just codegen` when declarations/generators change. Delete replaced mechanisms after the
replacement controls pass and callers move. Do not run integrated/static campaigns per packet.

After all functional scope, Q1 owns one relevant assembled selection: trajectory/direct-run/
shooting Python transport and lifetime journeys, intact reference workload selection, real
invariant candidate admission, and execution/evidence continuation controls. Include applicable
existing science oracles without weakening their assumptions/tolerances. Run `just hygiene`
once, fixing and rerunning failed recipes, plus relevant governance, `just docs`, native
and powerset manual obligations. Apply 26c's leaf composition so governance tests/freshness
already covered in that campaign are not rerun through another aggregate. The new design determines the non-overlapping
default/native/Python commands; selected reference parity runs only where its retained current
scientific claim is affected. Missing selected prerequisites fail; they never become skips.

Q1 must establish that common tests have one intended execution configuration; mode-absence
checks remain distinct; one explicit inventory/report composition exists; result projections
agree; lease accounting survives dropped parents; family oracles catch predicate/null/path
faults; and unrelated concurrent edits cannot be blamed on a test. Review the landed target
against the bounded governance/result changes without treating command success as architecture
acceptance. Broader Plan 25 physical/durable qualification remains with 25k.

Required completion is behavioral alignment, coherent authority, deletion and truthful scoped
evidence. Timing/memory comparisons are optional observations unless a quantitative benefit is
claimed. For such a claim, use representative one-table/multi-table trajectory access and
functional selection/preparation workloads with named conditions, recording retained memory
as well as duration. No throughput percentage or whole-product qualification is promised here.

## Checkpoint and closure

Authoring checkpoint, 2026-10-05: the four proposed designs exist; no package has started.
Next execution step, after separate authorization, is P0 and the ready foundation packages.
There is no unresolved target alternative delegated to an implementer. Routine organization
and names remain local choices; a material new source fact reopens its owning design before
dependent work proceeds.

At closure, move enduring contracts and rationale to their owners, report **Implemented**,
scoped **Tested** and any **Measured** claims with commands/conditions and zero failure baseline,
then retire resolved plans/reviews under ADR-0096 once no live reader depends on them.

## Outcome (recorded after implementation)

### What was built

Not implemented. Populate from actual landed behavior and scoped evidence.

### A mistake made and corrected

Not yet applicable; record an actual implementation correction.

### Deviations from the plan, deliberate

None recorded. Changed architectural decisions follow their decision route.
