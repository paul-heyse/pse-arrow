---
title: "Plan68: SurrealDB v4 cutover"
status: in-progress
date: 2026-11-07
adrs: [ADR-0257]
doc_role: plan
doc_owner: docs/plans/68-surrealdb-v4-cutover.md
doc_topics: [current-work, design, deployment, storage, libraries, validation]
review_sources: [docs/functional_reviews/surrealdb-v4-cutover-2026-11-07.md]
---

# SurrealDB v4 cutover

## Current status

**Implemented.** The fresh-default cutover under [ADR-0257](../adr/0257-rebuild-canonical-storage-on-surrealdb-v4.md)
is complete. GitHub and workstation `main` use the exact SurrealDB 4.0.0 family.
The workstation launcher selects the verified server release through the normal
`server select` route. Fresh catalog installation, selected original-operation
recovery, and the developer fixtures were exercised on that release. [68e](68e-surrealdb-v4-fresh-generation-and-acceptance.md#e03--select-open-writes-and-release)
owns the deployment and receiving evidence.

The plan remains **in progress** for three kinds of work after the cutover:

- Finish or explicitly defer the scheduled schema, access-path, graph, ranking
  and resource investigations. The existing v4 paths are usable today. A claimed
  benefit needs its own matched workload and evidence. [68b](68b-surrealdb-v4-schema-transport-and-authority.md#packages) · [68c](68c-surrealdb-v4-query-improvements.md#packages)
- Reconcile the remaining platform qualification and matched build/cache and
  allocation observations. Current receiving evidence covers named cases. The
  earlier workspace `just hygiene` run reported eight failed recipes against the
  zero target, and broader requalification was stopped at the maintainer's direction.
  [68a](68a-surrealdb-v4-supplier-runtime-and-extension.md#packages) · [68d](68d-surrealdb-v4-developer-and-test-receiving.md#packages)
- Identify any historical protected original that a current reader still needs.
  Preserve it through its existing reader or an owned one-time transfer where
  required. Release historical v3 generations only after the custody owners
  verify their consumers, holds and physical settlement. [68e](68e-surrealdb-v4-fresh-generation-and-acceptance.md#packages)

## Dependencies on Plan 66

Plan66 supplies selected consuming routes and owns their domain semantics and
acceptance. The platform cutover has already landed. Only the following work
depends on the corresponding consumer being ready.

| Plan68 work | Plan66 dependency and when to act |
| --- | --- |
| B03 and C02 protected-value and access-path verification | When [66b](66b-contracts-generation-and-interpretation.md#packages) supplies the generated interpretation for a selected result family and [66e E02–E03](66e-protected-data-and-result-delivery.md#packages) supplies its protected reader, exercise the actual permissions, original Arrow/Proto values, and bounded result pages before crediting v4 pushdown or lower read work for that family. Existing workbench, readiness and private evidence paths can be compared now. |
| C03–C04 expanded graph or ranked discovery | The existing private knowledge corpus can support the bounded predicate and ranking comparisons today. Wider product graph or discovery adoption needs an actual scoped graph, result or evidence consumer from [66e E07/E09](66e-protected-data-and-result-delivery.md#packages) or [66f F04](66f-developer-operations-clients-and-evidence.md#packages), with a useful query and expected authorized results. |
| D03 expanded original-evidence coverage | The selected regeneration, original-report projection and rerun evidence is in place. Extend the receiving matrix as [66f F02–F04](66f-developer-operations-clients-and-evidence.md#packages) supplies additional independently acquired reports or rerun consumers and [66e E07](66e-protected-data-and-result-delivery.md#packages) supplies their factual projections. D03's matched cache and allocation comparison can proceed on the existing lane. |
| E02 assembled scientific acceptance | [66l L02–L03](66l-required-population-and-assembled-acceptance.md#packages) owns the aqueous fixture drain and the wider scientific population. Use its settled workflows to extend receiving evidence for the v4 paths they actually consume. |

B05's private evidence count-guard experiment, C05's spill and mixed-load
qualification, and historical inventory and custody can proceed with their
existing owners and inputs. A protected artifact identified during Plan66 work
enters E01 through its named reader and custody owner.

## Target and baseline

**Proposed.** Cut over every active SurrealDB route to one exact 4.0.0
release family after Plan67 completes. Prepare and prove the replacement
privately, regenerate the catalog in fresh owned state, and make one
authoritative selection. Keep Plan66 moving. Its available scientific and
developer routes supply the receiving tests. Its unfinished consumers gate
only the additional functionality that actually needs them.

The pre-cutover comparison baseline was main `6da5f6289` and the November 7
working tree with patched SurrealDB 3.3.4, persistent SurrealMX 1.0.0, protocol
0.13.1 and Surrealism 0.6.4. The completed Plan67 regeneration and working-report
policy governs the comparison. The persistent history pilot was withdrawn in
`b3ee69f14`. The functional review's isolated memory probes provide candidate
designs and document the conditions of their supplier evidence.
[Functional review](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md) ·
[Plan66 baseline](66-integrated-simulation-target.md) ·
[Plan67 decisions](67-storage-lifecycle-and-build-reuse.md#decisions-and-boundaries)

Use core 3.4, efficient architecture 1.0, test assessment 1.2,
process simulator 1.5, and the repository binding. Preserve DP-09's
complete reuse identity, DP-15's qualified library boundary and the
independent assurance attached to scientific fixtures under PS-13.
[Selected standard](../design_review/design_principles/standard.toml)

## Shared design and authority

- **One release family and deployment.** Select server `v4.0.0` at
  `284fe93afc851c3baec2c262c01289602021a70d`, MX 1.1.0, SDK/value family
  4.0.0, protocol 0.14.1 and Surrealism 0.7.0. The tagged lock is supplier
  evidence. The PSE preparation owner adds exact features, patches,
  compiler and immutable receipts. Keep persistent MX and the existing
  bounded native client. Supplier support for explicit SSI, gRPC streaming,
  extension resource limits or import is independently qualified against
  the PSE ownership contract where consumed.
  [Tagged lockfile](https://github.com/surrealdb/surrealdb/blob/v4.0.0/Cargo.lock) ·
  [Native source and patch owner](../../scripts/surrealdb-source/source-build.md)
- **Cover actual entry points.** Walk direct and generated producers,
  host tool preparation/select/import, native canonical launch, initialized
  or reopened service, connection-only clients, maintenance and recovery,
  private evidence/knowledge stores, validation/codegen, SurrealKit and
  Rust fixtures. Record the selected source recipe and each route's
  readiness, effects, custody and settlement. Examine feature-selected,
  test-only and WASI closures as well as ordinary use. Retired artifacts
  and historical evidence citations are provenance inputs. Identify and
  remove any old production fallback when its last caller moves.
- **Regenerate through current owners.** Fresh v4 level-4 stores allow
  one generated schema and rebuilt indexes. Preserve a named protected
  original through its existing portable reader whenever it can be
  consumed that way. Convert once only for an actual v4 reader that needs
  it. Preserve exact membership, protection and original interpretation.
  Keep product writes and test evidence under their existing owners.
  Use the frozen v3 generation for pre-write rollback. Recover forward
  after v4 admits writes.
  [Migration contract](https://surrealdb.com/docs/build/migrating/from-old-surrealdb-versions/3x-to-4x) ·
  [Native deployment ownership](../dev/surreal-substrate.md)
- **Decision route.** Before changing production source or selection, use
  the major-family ADR and applicable scoped design review to approve the
  release and contract delta. Address Plan66's explicit v4 exclusion
  through the same decision and a cross-plan amendment. That amendment
  gives Plan68 the platform cutover while leaving the scientific scope
  and its status with Plan66. ADR-0257 records the scoped fresh-default
  decision. Plan68e records authoritative selection separately from the
  private receiving evidence.
  [Decision rules](../../.claude/rules/decisions.md#when-an-adr-is-required) ·
  [Existing MX rationale](../adr/0214-surrealmx-canonical-store.md)

## Ownership and sequence

| Companion | Owned result and dependencies |
| --- | --- |
| [68a](68a-surrealdb-v4-supplier-runtime-and-extension.md) | Source-built server and extension, persistent profile, replay and physical resource contracts. Supplies exact generation inputs and engine qualification to B–E. |
| [68b](68b-surrealdb-v4-schema-transport-and-authority.md) | Generated schema and installed-definition witness, typed transport, permissions and original interpretation. Supplies the receiving boundary to C–E. |
| [68c](68c-surrealdb-v4-query-improvements.md) | Required query equivalence and bounded optimizer, read-only, lexical, graph and resource investigations on existing owners. Adopts only verified improvements. |
| [68d](68d-surrealdb-v4-developer-and-test-receiving.md) | Locked SurrealKit, fixture parity, fresh working evidence and rerun, and matched Plan67 build and footprint checks. |
| [68e](68e-surrealdb-v4-fresh-generation-and-acceptance.md) | Fresh reconstruction, named protected originals, actual receiving, selection and rollback rehearsal, forward recovery and v3 retirement. |

Start after Plan67's completed custody, fixture, build and retirement
contracts are identified. Capture the matched v3 inputs before replacing
the supplier or releasing its preparation. After the decision route, A's
candidate build and B's client/schema work can proceed from a common exact
release tuple. D can prepare the tool in parallel and consume B's usable
definitions when its fixtures run. E can classify state and named original
consumers while the private foundation is built. Required C query
equivalence runs before activation. Optional C comparisons use the
qualified interfaces and can finish after cutover.

Use the available Plan66 aqueous Study protected-read and independent
IPC/Proto interpretation, selected fit and trajectory receivers, native
lifecycle controls and developer evidence/rerun as initial end-to-end
slices. A regression on any supported affected route blocks authoritative
selection. An unimplemented additional Plan66 capability remains with its
owner. Refresh the affected route list if Plan66 delivers another
consumer before cutover.
[Plan66 receiving population](66l-required-population-and-assembled-acceptance.md#packages)

## Opportunity dispositions

The coordinator owns these dispositions. Companion package tables own
implementation state. Each optional candidate must return an adoption or
deferral result against its stated acceptance conditions. Preserve qualified
v4 behavior if a candidate fails semantic equivalence or brings no useful
improvement.

| Review reference | Disposition | Owning intervention |
| --- | --- | --- |
| O01 | Open | B05 keeps the production decimal guard and owns the shared named-type and bounded gRPC comparison. B01 already uses named field migration for the replay checkpoint. |
| O02 | Open after adoption | C01 adopted the current read-only knowledge snapshot. C03 added bounded graph diagnostics and retains the existing traversal while inline predicates and any wider graph projection await their scoped comparison. |
| O03 | Open for broader receiving | D03 exercised regenerated evidence and original-report readback, alongside the baseline failed-rerun control. Broader consumers and matched build/retention observations remain. Persistent history is deferred pending a named recurring consumer and retention decision. |
| O04 | Resolved | D01–D02 received the locked SurrealKit RC.1 and managed sync/rollout lane on the exact v4 server with settled disposal. |
| O05 | Open for matched evidence | C02 exercised selected v4 catalog and count paths, and C03 exercised the graph reader. Compare equivalent v3/v4 plans and work before crediting an index or executor benefit, and extend protected-result coverage when the named readers are ready. |
| O06 | Open for resource qualification | A02 exercised selected serializable writes and original-operation recovery. C05 owns live spill, mixed-load and physical allocation observations. A narrower publication witness requires an actual contention case and absent/future predicate evidence. |
| O07 | Open for comparison | C04 retains existing scoped lexical discovery. Compare bounded ranking against the current corpus. Persistent views, vector/hybrid and geospatial search need a named workload and evidence of usefulness and custody. |
| O08 | Open for broader reader receiving | B02–B03 landed native typed gRPC and exercised selected wire/value contracts. Additional independent scientific readers depend on their Plan66 families. Alternative encodings or incremental delivery are deferred pending a named fidelity or memory gap. |
| O09 | Open for remaining qualification | A03 landed and exercised the pure ABI 4 extension. B04 exercised selected HTTP authority and original envelopes. Remaining paired controls stay with A/B. Transaction-capable exports, typed bindings and invoker routes require a specific consumer and authority assessment. |
| O10 | Open for historical custody | E exercised fresh initialization and selected original-operation recovery on MX and completed default selection. Inventory, protected-original custody and historical release remain. Separate KV or distributed topology requires its own capacity, durability or failure-domain objective. |
| O11 | Deferred | A bounded protected reverse lookup or an actual observer with continuity requirements can reopen materialized references or changefeed delivery. Preserve existing occurrence identity and synchronous authority. |
| O12 | Deferred | A named external SQL client or UI that replaces meaningful work can reopen a separate read-only interface and its isolation tests. |

[Stable review opportunity anchors](../functional_reviews/surrealdb-v4-cutover-2026-11-07.md#O01)

## Verification and outcome

Each companion schedules affected compile checks, targeted controls and
replacement deletion. Use explicit force-validation for every actual
Arrow-consuming correctness closure. Run relevant assembled verification
and `just hygiene` once after functional integration. E owns the
authoritative cutover evidence. Report actual commands, mode, scope,
failure count against zero, and physical settlement. Compare matched
results, access paths, work, build units and allocations before claiming
benefits. The plan closes after E verifies the v4 selection, named
originals, recovery and release, and each scheduled optional investigation
has a disposition. Record what was built, mistakes corrected and deliberate
deviations, and transfer enduring contracts to their architectural owners
before retirement.
[Qualification guide](../dev/validation-assessment.md)
