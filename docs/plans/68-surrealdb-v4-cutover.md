---
title: "Plan68: SurrealDB v4 cutover"
status: planned
date: 2026-11-07
adrs: [ADR-0257]
doc_role: plan
doc_owner: docs/plans/68-surrealdb-v4-cutover.md
doc_topics: [current-work, design, deployment, storage, libraries, validation]
review_sources: [docs/functional_reviews/surrealdb-v4-cutover-2026-11-07.md]
---

# SurrealDB v4 cutover

## Target and baseline

**Proposed.** Cut over every active SurrealDB route to one exact 4.0.0
release family after Plan67 completes. Prepare and prove the replacement
privately, regenerate the catalog in fresh owned state, and make one
authoritative selection. Keep Plan66 moving. Its available scientific and
developer routes supply the receiving tests. Its unfinished consumers gate
only the additional functionality that actually needs them.

The inspected source is main `6da5f6289` and the November 7 working tree.
It selects patched SurrealDB 3.3.4, persistent SurrealMX 1.0.0, protocol
0.13.1 and Surrealism 0.6.4. Refresh the inventory at execution start
against completed Plan67 and the then-current Plan66 baseline. The
Plan67 premise is its current regeneration and working-report policy.
The persistent history pilot was withdrawn in `b3ee69f14`. The v4
review's four isolated memory probes are **Tested** in their stated
conditions. They provide design leads. The local v4 source-built
foundation, actual receiving tests and measured benefits are open.
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
  and its status with Plan66. ADR-0257 records the proposed decision.
  The private candidate proceeds under the maintainer's implementation
  instruction. Formal acceptance and authoritative selection remain separate.
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
| O01 | Planned investigation | B qualifies one shared generated named constraint only after finding equivalent field and callable contracts. |
| O02 | Planned | C qualifies protected and graph reads, then trials current read-only snapshots on the existing pure knowledge consumer. Extra graph representation changes follow C's measured gates. |
| O03 | Planned receiving, persistent history deferred | D receives regenerated evidence and current rerun. A named recurring consumer plus a separate retention decision can reopen the held-expiry probe. |
| O04 | Planned | D receives the completed native suite lane on the exact RC tool, with original in-run JSON and disposal evidence. |
| O05 | Planned | C compares the real query paths, count and index readiness. E requires current-result equivalence. |
| O06 | Planned | A qualifies write isolation, retry and engine controls. C qualifies spill and query resources on owned scratch. Narrowing the witness needs measured contention and absent/future predicate coverage. |
| O07 | Planned investigation | C tests exact lexical and bounded ranking/graph candidates. Defer persistent views, vector/hybrid and geospatial search until a named consumer supplies freshness, relevance and custody needs. |
| O08 | Planned cutover, new encodings deferred | B receives native typed gRPC and independent interpretation. Reopen shaped CBOR, alternate encodings or incremental delivery for a named receiver with a fidelity or memory gap. |
| O09 | Planned | A ports the pure extension. B qualifies current control API authority and rollback. Transaction-capable exports, typed bindings and invoker routes require a specific consumer and a fresh authority assessment. |
| O10 | Planned | E regenerates, preserves named originals and proves recovery on MX. Separate KV or distributed topology requires an independently scoped capacity, durability or failure-domain objective. |
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
