---
id: ADR-0074
title: Retain native execution assemblies and exact local evidence
status: proposed
date: 2026-09-19
deciders: [paul-heyse]
level: decision
principles: [DM-09, DM-24, DM-26, DM-32, DM-38, DM-42]
blueprint: [§3.3.3, §5.4, §14.3, §14.3.1, §14.3.2, §24]
review: docs/design_review/reviews/design_review_integrated-native-performance_2026-09-19.md
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A new transformation, mutable provider or library upgrade changes evidence preservation or assembly lifetime.
verification: Plan 11 I00-I15 targeted units and source-qualified deletion receipts; complete integration after I17, performance after I18.
---

# ADR-0074: Retain native execution assemblies and exact local evidence

## Context

The integrated native-performance review identifies repeated preparation and invalid
binding/extent assumptions. Plan 11 carries the unresolved Plan 10 acceptance scope.

## Scope

Amend blueprint §3.3.3, §5.4, §14.3–§14.3.2 and §24 through dedicated revision 42.
This authorized design amendment uses `PSE_DESIGN_EDIT=1`; formal decision/design PR
metadata remains `adr`/`needs-review`. The integrated review is Revise, not acceptance.

## Drivers

Exact semantic ownership, fresh attempt services, bounded retained work, and efficient
implementation with complete final qualification.

## Options

- Rebuild each snapshot and rescan each boundary: repeats established work.
- Infer equivalence from names, pointers or digests alone: insufficient evidence.
- Retain immutable capabilities and exact evidence with explicit selection: selected.

## Outcome

Retain compatible model contexts and actual functions/planners/rules. Bind immutable
revision/role views and derive fresh query/attempt state. Native binding and coercion
precede schema reconstruction. Registry-owned schemas and prepared validators retain
local evidence through supported transformations; relational obligations remain separate.
Allocation accounting follows known backing owners through wrapping and slicing;
unknown foreign capacity requires an explicit extent contract or a reserved copy.

Producer/invariant/index reuse is permitted immediately with complete dependencies;
additional automatic dependency frameworks remain separately evidence-gated. Native
relational composition and finite checked ports replace universal stage/tuple capture.
Bounded completion retention, demand, exact Delta snapshots, resource partitions,
settlement and supported scalar programs follow Plan 11. Hash framing and ADR-0047's
accepted ordered/guarded numerical semantics do not change.

For I04, grouped native preparation separates owner-retained intrinsic facts from
lexical, selected-input and live admission. Pre-fold volatility remains an attempt
dependency; actual factory outputs and physical reset families remain admitted.
For I05, producer-local allocation ownership replaces global pointer discovery,
safe per-allocation leases survive escaping views, and typed completion distinguishes
validity, abandonment, resource refusal, partial streams and uncertain effects.
For I06, native logical plans expose relational work before execution. Finite
multi-output algorithms expose checked relation ports over the existing operation
completion; existing algorithms default to atomic bundles. Changed inputs, demand
or epochs cannot reuse their completion. No second scheduler or tuple transport remains.

### Consequences

Implemented: I13-I15 retain invocation-owned member-write completion evidence, use
native returned commit outcomes for known success, and reserve reconciliation for
uncertain outcomes and retries. Exact mapping and local-value evidence do not
replace publication-wide obligations or certify untouched history. Model-result
retention admits only fully completed immutable results with exact owner and input
selection; attempt scopes isolate in-flight work. Runtime query/output/load budgets
share the existing resource owner. Python retains the direct Arrow C stream and
moves exhaustive generated-contract lint to explicit development checks, with a
cheap compiled registry fingerprint check at import. The detailed implementation
and development receipts are tracked in the
[I13-I15 execution checkpoint](../plans/11-i13-i15-execution.md).

I07-I12 extend these owners with explicit value/obligation demand and terminal
contracts, chunked compiler construction, affected-key rule epochs, immutable semantic
indexes and trusted canonical evidence. Hash-only framing uses the existing FrameSink
without changing canonical bytes. Scalar numerical execution is a derived guarded
layout over admitted operations; attempt workspaces own cancellation and mutable slots.
Exact Delta selections are independent of current-head observation. Maintenance
generation, load capability, retained settings and finite native cache budgets remain
explicit admission inputs. Their implementation and targeted unit verification are
recorded in the I07-I12 execution checkpoint; final qualification remains open.

Delete replaced paths and callers in their implementation packages. No legacy API,
fallback engine or historical-data migration is introduced. Capability, complete-view
and consumed-input identities are distinct; current authorization is never cached proof.

I16-I19 use the existing assessment runner for one functional campaign followed by
separately identified measurements. Required product and quality checks retain a zero
failure baseline. Dependency policy findings remain advisory under ADR-0066, while
failed tool invocations remain failures. R-20 API-reference lint is explicitly
unsupported until implemented. Native/Python selected identities, interruptions,
source drift and linked continuations are retained without adding campaign controls.
Typed diagnostic projection preserves native and platform aggregate leaves across
external boundaries. Functional helpers default to Contract observation; value
previews are disabled and diagnostic rendering is explicit.

### Compensating controls

Targeted unit tests are sufficient behavioral verification for every implementation
and deletion item. Compilation, static checks and pure generation remain available.
Every integration campaign waits for all I00-I17 implementation/deletion scope; final
integration and performance remain mandatory at I18/I19. No small workflow subset is
reclassified as a unit. Correctness tests retain explicit force-validation.

### Confirmation

The Plan 11 manifest maps cases to exact source, binary, feature mode and oracles.
Source-qualified development receipts and barrier controls reject incomplete or stale
coverage. Measurements use separately named modes and cannot replace functional evidence.

Tested: the I04-I06 checkpoint runs 82 isolated Rust units with
`pse-relations/force-validate`, baseline zero; exact `just unit-package` selections
and receipts are in the execution inventory. Interface-checked: workspace
`just check` and `just clippy` (default and no-default features) pass. Full integrated
functional and performance claims remain Proposed.

Implemented: I07-I12 demand/obligation reuse, compiler indexes, grouped rule epochs,
semantic/canonical reuse, guarded scalar execution and exact Delta generations are
recorded in the [execution checkpoint](../plans/11-i07-i12-execution.md). Its Tested
unit receipts and Interface-checked workspace commands qualify only those development
boundaries. Final integrated qualification still waits for I17; this record remains
proposed.

## Pros and cons

Retained owners avoid repeated work, at the cost of precise invalidation and bounded
lifetimes. Conservative misses remain preferable to invented source equivalence.

## More information

[Plan 11](../plans/11-integrated-native-performance.md), D07 and I00-I15;
[execution inventory](../plans/11-execution-inventory.md). DataFusion/Arrow and Delta
contracts come from the pinned local skills and source. No speedup is claimed.

## Status history

- 2026-09-19 — proposed before I00-I03 behavior changes; acceptance remains open.
- 2026-09-19 — extend the proposed implementation decision to authorized I04-I06;
  targeted units remain sufficient for implementation and deletions.
- 2026-09-19 — extend the proposed decision to authorized I07-I12 before implementation;
  no additional campaign enforcement is introduced and integrated qualification waits
  for all implementation and deletion scope through I17.
- 2026-09-20 — record I07-I12 implementation and scoped development evidence;
  decision acceptance and final integrated qualification remain open.
- 2026-09-20 — extend the proposed decision to authorized I13-I15 before changes;
  targeted units cover implementation and deletions; no new campaign controls.
- 2026-09-20 — record I13-I15 and L03/L15-L17 implementation/deletion closure;
  Tested 57 isolated Rust units with force-validation and 65 Python units, baseline
  zero; Interface-checked normal workspace/static and pure-generation checks. Final
  integration and performance remain open through the complete I17 barrier.

- 2026-09-20 — implement the authorized I16-I19 scope within Plan 11, preserving
  existing campaign enforcement and the I17 implementation/deletion barrier.
  Functional and measurement qualification remain open.
