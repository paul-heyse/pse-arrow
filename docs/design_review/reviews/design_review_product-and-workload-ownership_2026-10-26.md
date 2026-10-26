---
title: Product and workload ownership decision review
date: 2026-10-26
standard: core-3.4/process-simulator-1.5
doc_role: review
doc_owner: docs/plans/56-design-principle-alignment.md
doc_topics: [design, storage, deployment, performance]
---

# Product and workload ownership

**ADR-0229: Accept at Proposed design evidence. ADR-0230: Not Accept as a whole while
D04's active-capture premise remains unresolved.** The separable D01/D02/D03/D05/D06
contracts are acceptable at Proposed design evidence; this judgment does not accept a larger
cohort cap or certify their implementation. D03 retains the existing cap and frozen membership
contract until D04 is settled. Independent work need not wait for that probe.

The proposed boundaries follow independent reasons for change and remove concrete repeated work.
Scientific authority remains in the registry; private development storage becomes an xtask
responsibility. Product deployment and allocation acquire an executable Rust owner. Bounded
installation, compact outcomes, shared result scans and incremental exchange address complete
operations rather than only transport shape. Durable capture is a useful additional responsibility,
but its coherent active behavior cannot be inferred from the existence of SSI primitives.

## 1. Review contract and evidence

| Field | Scope |
|---|---|
| Tier / purpose | Design / target; bounded joint decision review |
| Focus | Combined design principles and SurrealDB library fit |
| Reviewer | Independent delegated design reviewer; 2026-10-26 |
| Source baseline | `c5ae90b8435ce71b2b6b552cc4f2e7b96dd66aba` plus the uncommitted ADR-0229/0230 drafts; ADR-0230's capture-lifetime amendment was re-read during review |
| Diagnostic input | [Living audit](../audit/README.md), publication `f9b96810a84409d9d32ff2a1f62b029ac151ae2e`, audited source `682fc3830f8a370f0995544af5adf821e4be9ab9` |
| Standard | Core 3.4, efficiency heuristics 1.0, process-simulator 1.5, test-assessment 1.2 for the proposed evidence controls; [binding](../design_principles/binding/pse-arrow.md) |
| Disposition | [Plan 56](../../plans/56-design-principle-alignment.md); execution in [56d](../../plans/56d-publication-and-workload-fit.md) and [56e](../../plans/56e-product-deployment-and-resource-ownership.md) |

This engagement consumes the existing diagnosis rather than restarting a broad audit. Relevant
diagnoses are DA-019/020/036/037 for product/deployment ownership and DA-034/035/044/045 for
physical work. Completed 56a/56b/56c changes intervene: exact model/physical interpretation,
scientific permission and kind-owned facts, canonical declarations and attributed settlement,
v9 inline/sealed placement, native recovery and closed readmission. Those are preserved inputs,
not findings reopened by this review. Their recorded tests retain their original scope.

Decisive sources inspected include `pse-schema::catalog`, `xtask/src/codegen.rs`, runtime
`boundary_documents.rs` and `workflow/native_deployment.rs`, operations `canonical_studies.rs`
and `canonical_study_workbench.rs`, codegen `surreal_execution.rs` and
`surreal_study_workbench.rs`, runtime `study_workbench.rs`, `project_closure.rs` and
`project_portability.rs`, selected profile/receiver realization in `scripts/surreal_server.py`,
and `.config/agent-capacity.toml`. Concurrent process-observation and supplier-record edits are
excluded from implementation judgment. No product test, native probe, build or benchmark was
run by this reviewer. This is not whole-product or scientific qualification.

The functional target is reliable interactive and many-case scientific work with exact, readable
results and recoverable interruption. The relevant growth case is 10,000 study points across
creation, summary and cohort reads. The 100,000 admission ceiling remains unqualified. All current
numeric profiles, sixteen reference lanes, fixture lane distinctions, override precedence and
uncapped ordinary workstation behavior are preservation constraints. No new crate, engine,
numerical method, daemon or general validation language is selected.

## 2–3. Responsibilities, contracts and semantic authority

| Responsibility | Owned decision and operation | Required consumed contract |
|---|---|---|
| Scientific registry | Scientific identities, relations, physical meaning and permission vocabulary | Generation and runtime use the same declarations; private development changes do not redefine science |
| Owner-local documents and existing generator | Non-scientific enums, document versions, bounds and output-family publication | Directional contracts, closed choices and unknown-construct refusal remain; xtask aggregates contributions |
| Private evidence backend in xtask | Evidence/knowledge storage, queries and development publication | No raw canonical scientific client; normal product closure does not enable it |
| Product deployment/allocation owner | Validate and materialize one versioned allocation and receiver association | Host realization consumes admitted values; conformance selects a production profile |
| Leaf process observations | Authenticated observed facts and explicit uncertainty | Admission owns borrowers; each lifecycle owner retains its own decision policy |
| Canonical study installation | Immutable population identity, exact bounded page receipts, node/edge closure and activation | Existing scientific admission, original fences/clocks and attributed effect knowledge |
| Capture and protected result provider | Immutable captured membership/facts, exact manifests and shared lazy scans | Exact capture replay; atomic retained protection; unavailable members never become later successes |
| Offline copy and selected exchange | Controlled immutable progress, useful hydration units and final publication | Source/copy/destination/semantic integrity remain distinct; explicit recovery and manifest-last visibility |

Current source supports the diagnosis rather than the replacement's implementation. The catalogue
assembles evidence and knowledge alongside product declarations; runtime's boundary document list
names their types, and `pse-runtime/Cargo.toml` enables operations `evidence-query`. Moving only the
document list would leave the actual dependency cause. ADR-0229 correctly requires the complete
private backend and all generated consumers to migrate together, while preserving product
observability and scientific IPC. An owner-local enum is sufficient for a deployment choice;
forcing it into a universal scientific vocabulary adds no meaning.

Current `NativeAllocation::validate` already expresses relational allocation constraints.
`surreal_server::reference_execution` independently reads conformance settings and materializes
headroom, observer and admission values. The target must make the Rust validation operation usable
before setup effects, without starting a canonical store or native receiver. Schema annotations
alone cannot provide this relational enforcement. AST import discovery must include package
parents, generated modules, configurations, executable/provider identities and declared dynamic
imports. An isolated frozen import probe checks the actual closure before publication.

**Physical semantics.** No unit/property definition, dimension, basis, reference state or validity
envelope changes here. Captured results retain existing quantity/unit identifiers, original
completion permission and exact manifest selection. Compact outcome facts are projections of the
admission owner, never independent judgments of solver success. Unavailable/refused/not-yet-usable
members remain explicit captured facts. The physical-semantics table therefore has no newly
introduced quantity row.

**Well-posedness and numerical stages.** Variable roles, structural analysis, formulation,
derivatives, scaling, solver selection and independent post-solve checks remain with their existing
scientific owners. No new formulating/evaluating/solving stage exists in this decision. Installation
and copying cannot confer scientific usability. This bounded review does not requalify those
neighboring numerical mechanisms.

## 4–5. Revealing changes and complete operations

| Scenario | Change class | Assessment and preservation |
|---|---|---|
| S1: change a private corpus/enum | Instance / representation | Only its xtask declarations, generated family and consumers change; product/foundation output stability and normal Cargo closure distinguish a real partition |
| S2: change one production profile | Policy | One typed materialization drives setup, supervisor and worker; conformance consumes selection; all existing numeric values and distinct scope meanings survive |
| S3: unrelated Python script changes | Mechanism input | Receiver identity remains stable; a newly required generated or dynamic import changes it or refuses before publication |
| S4: interrupt population installation | Composition / failure | Resume exact installed frontier, retain original issued authority, refuse conflicting replay and incomplete activation; dependencies close after endpoints exist |
| S5: live study completes while cohort pages are read | Concurrent observation | Every page reflects one captured cut, including earlier unavailable members; exact manifests stay protected through scan drain |
| S6: retire captured results, lose capture ACK or abandon caller | Lifecycle | Atomic protection prevents a retirement gap; exact readback/release and explicit retention classes govern uncertainty and cleanup |
| S7: copy or export several large revisions | Mechanism / growth | Immutable compact selection survives; hydrate/encode/release useful units; output allowance precedes avoidable work; final manifest follows all acknowledged content |
| S8: PID reused or launcher dies with descendants | Observation / failure | Observations preserve authentication and uncertainty; owner policies distinguish launcher death from physical drain |

`surreal_execution::append_many` currently preflights and then calls `append` in a `FOR` loop.
One transaction therefore does not remove repeated writer/frontier work. The proposed grouped
native mutation retains each immutable logical acknowledgement while sharing applicable admission
and frontier realization. Empty demanded sets need no file; buffering must never defer an already
demanded acknowledgement or change live-prefix durability.

`canonical_studies` creates individual run identities while batching point metadata, and summary
reads select descriptors with `LIMIT 4`. Compact admission facts and bounded run-and-point pages
remove distinct costs. A page receipt must acknowledge exact run identities, point contents and
frontier atomically; node completion cannot stand in for edge closure. Fresh bounded work may have
its own authority, but resumption never renews an expired issued page.

Current workbench capture returns full frozen membership under 128 members, and KPI analysis
constructs per-member snapshots and a union chain. One multi-attempt manifest and relation-shaped
provider can consume that same existing exact membership without a larger cap. D04's durable cut
adds a different capability. Compact records bound transport and hydration, but the short capture
still examines and writes its complete selected universe. Its real transaction work, conflict
behavior and temporary residency must fit the admitted active workload.

Current `ResolvedClosure` retains aggregate document bytes and source packages. Incremental sinks
remove that residency while retaining necessary protections and deterministic ordering. Holding
compact exact manifests until final publication is justified; holding every hydrated package is
not. Backup likewise retains independent integrity purposes while eliminating only repeated work
over unchanged protected bytes. A resumed destination requires renewed premise checks.

## 6. Architectural fitness and independent gates

Verdicts concern the proposed target. They do not certify the current implementation.

| Foundation | ADR-0229 and independent D contracts | D04 capture |
|---|---|---|
| AP-01 | Satisfied: scientific, private development, deployment and physical work have coherent owners | Satisfied: canonical capture owns the cut and its lifetime |
| AP-02 | Satisfied: versioned documents, exact receipts and explicit failure preserve consumed meaning | Satisfied as specified after the lifetime amendment |
| AP-03 | Satisfied: existing roots/generator/shared primitives compose without a new service | Satisfied: capture composes with one protected provider |
| AP-04 | Satisfied: profiles, observations, installation phases and permission are distinct governing concepts | Satisfied: frozen unavailable facts and original permission remain authoritative |
| AP-05 | Satisfied: one allocation validator and exact install transitions expose constraints | Satisfied at Proposed level: amended atomic protection/replay/release/classification contract |
| AP-06 | Satisfied: private change and local policy controls avoid unrelated product startup | Satisfied: bounded canonical owner exposes its actual dependency |
| AP-07 | Satisfied qualitatively: shared work, bounded units and selective hydration remove observed amplification | Unresolved: exact active capture feasibility, including its complete transaction and retention work, is unprobed |

| Gate | Independent scope | D04 capture / reason |
|---|---|---|
| G1 Authority | Pass: registry, allocation and owner policies remain scoped | Pass at Proposed level: capture copies facts rather than granting new permission |
| G2 Semantic fidelity | Pass: enums, identities, units, unavailable and partial states preserved | Pass as specified; no live-page reinterpretation |
| G3 Validity | Pass: relational validation, exact page closure and refusal precede use | Unresolved pending exact-engine feasibility controls |
| G4 Hidden behavior | Pass: setup, capture, copying and publication are explicit effects | Pass: durable capture is effectful, rather than a pure browsing getter |
| G5 Consistency/recovery | Pass at Proposed level: original settlement and manifest-last semantics retained | Unresolved until coherent capture/protection is exercised under mutation and interruption |
| G6 Transformation/reuse | Pass: original scientific and immutable premises survive projections | Pass as specified; frozen absence cannot be refreshed from live state |
| G7 Truthful claims | Pass: Proposed claims and unqualified ceiling are explicit | Unresolved for the selected capability; source primitives do not qualify its composition |
| G8 Library leverage | Pass: existing generation, parsing, relational engine and DataFusion capabilities retained | Pass in mechanism selection; native SSI composition is the candidate, not bespoke MVCC |
| G9 Architectural fitness | Pass at Proposed design level | Unresolved through AP-07; no averaging against independent strengths |
| PS-G1 Physical consistency | Pass for preserved boundary contract; no new physical computation | Same preservation; quantity/permission controls remain required |
| PS-G2 Well-posedness | Not applicable: formulation and solver admission unchanged | Not applicable for the same reason |
| PS-G3 Numerical integrity | Pass for preserved permission/outcome contract; no numerical qualification claimed | Same preservation; capture cannot upgrade solver or completion status |

## 7. Findings and their relationship

<a id="f01"></a>
### F01 — Active immutable capture has an unresolved exact-engine premise

**Principles:** AP-07, DP-15/19/20/22; G3/G5/G7/G9; S5/S6. **Evidence:** Interface-checked
SSI patch and current native selector; Proposed replacement. The pinned
`scripts/surrealdb-source/mx-1.0-controls.patch` selects
`with_serializable_snapshot_isolation` and contains write-skew/empty-range phantom controls.
Those establish an implementation route and primitive challenge, not a coherent bounded
SurrealQL capture of actively changing scientific facts plus retained manifests.

If the actual capture cannot complete under active changes and selected limits, enlarging the cap
would advertise a capability with no qualified route. Probe the exact installed patched engine,
native query and ownership path: mutate active facts, capture one cut with unavailable members,
change live state between bounded reads, then challenge retirement, cancellation and release.
Preserve original clocks and drain charges; do not substitute terminal-only selection or repeated
attempts until the study stops changing. A passing short feasibility probe permits reassessment of
the mechanism; it does not supply 10,000-point qualification or measured benefit. Failure requires
settling the engine/transaction realization before dependent cap changes. Plan 56d D04 owns this
obligation; independent D packages remain separable.

<a id="f02"></a>
### F02 — Durable capture requires exact lifetime, not references alone

**Principles:** AP-02/04/05, DP-03/19/24; G3/G5; S6. The first draft selected durable member and
manifest references without explicitly closing the capture-to-protection handoff or lost-response
cleanup. A concurrent retirement could otherwise remove selected payloads before lazy acquisition;
an unknown capture ACK could strand or incorrectly release obligations.

The coordinator amended ADR-0230 during review. The inspected amendment binds immutable request
identity and exact replay/conflict, installs capture facts and all selected protections in the
same native transaction or consumes already-held protection, retains original-grant readback,
authenticates idempotent release, waits for scan custody to drain, and requires retention/backup
classification for interrupted installs and abandoned captures. **The draft contract gap is
addressed at Proposed evidence.** This is not implementation closure: the native realization,
retirement/restore consumers and negative controls remain in D04 and the coordinated interpretation
change. No additional ledger or universal lease framework is needed.

F02 supplies the complete operation whose feasibility F01 must challenge. Settling the words does
not settle the engine composition; conversely, a snapshot-only happy-path probe cannot close the
owned protection/recovery obligation.

## 8–9. Library mechanisms and alternatives

Context7 was resolved first (`/surrealdb/surrealdb`) and queried for SDK transaction/stream ownership.
It returned current main-branch lifecycle material, useful for orientation but insufficient for
this pinned integration. The supplied references were consulted at SurrealDB §§6.1, 8.3–8.4 and
9.4–9.5; tooling §§3.1–3.2 and 6.1–6.3; SurrealMX §§1–2; SurrealKV §§1–2; and the tooling idea's
native capability section. Direct storage APIs in standalone SurrealMX/SurrealKV are not selected
as a parallel canonical store or record interpreter.

| Alternative | Fit and decision |
|---|---|
| Universal catalogue plus universally enabled evidence feature | Retains the observed product dependency cause; reject as the partition remedy |
| New evidence crate/service | Adds distribution/lifecycle machinery without a distinct present need; existing xtask composition is simpler |
| Owner-local serde documents through existing generation | Selected: keeps meaning with its owner and preserves directional Rust/schema/Python projections; actual output and Cargo boundaries still require controls |
| Python-only deployment validation | Retains independently maintained relational meaning; selected Rust operation must remain callable before effects |
| Single append loop inside one transaction | Preserves atomicity but repeats shared work; native bulk `INSERT`/`INSERT RELATION` plus grouped frontiers is selected |
| Live keyset pages | Efficient transport but can mix observation times; unacceptable as immutable capture |
| Terminal-only capture | Simpler stable universe but removes supported active-study semantics; not the selected scope |
| Long remote transaction across pages | Can retain a view, but pins engine versions through caller lifetime and broadens cleanup; short durable capture is preferred if F01 passes |
| SDK `Query::stream_items` | Not selected: pinned `vendor/surrealdb/src/method/query.rs` explicitly rejects payload-owned streaming queries; provisional rows also require final statement/transaction success |
| Short immutable capture and bounded pages | Candidate selected by the draft, with F01 unresolved; adds justified durable lifetime and recovery obligations |
| Incremental protected export / controlled copy | Selected: uses existing scientific/file owners, releases hydrated units, and preserves distinct integrity checks and final visibility |

The streaming refusal is decisive over the generic reference's supported gRPC streaming statement.
Likewise, the local SSI patch is decisive over generic online isolation descriptions. No source
statement or library name establishes current latency, memory, transaction capacity or speedup.
Replacing the engine would require requalifying the consumed snapshot/protection/recovery contract;
the public scientific consumers should not acquire engine keyspace knowledge.

## 10. Verification obligations and limits

| Claim | Evidence now | Settling evidence still required |
|---|---|---|
| Product/private responsibility split fits expected change | Proposed design plus inspected dependency cause | Scientific output stability under private change; actual supported normal Cargo feature closure excludes private backend |
| One allocation meaning, complete frozen receiver | Proposed | Malformed cross-field refusal before effects; profile propagation with unchanged values; isolated imports including parents/generated/dynamic inputs; unrelated-script stability |
| Process facts preserve owner policy | Proposed | PID reuse, uncertainty, launcher death with surviving descendants, repeated drain and report custody controls |
| Exact bounded installation/publication | Proposed | Replay/conflict/lost ACK, stale fences, demanded empty/partial appends, interrupted nodes/edges, refused variants and incomplete activation |
| Coherent active capture | Unresolved | F01 exact-engine probe, then original permission/quantity, concurrent retirement and protection/drain controls |
| Supported many-case physical route | Proposed | 10,000-point creation/summary/cohort journey and relevant physical-work measurements; 100,000 remains unqualified |
| Bounded copy/exchange | Proposed | Corrupt/changed source and destination, cancellation/stall/resume, allowance exhaustion, exact contents and actual retained hydration units |

Tests/benchmarks by this reviewer: **not_run**. Failure target remains zero; no passing count is
claimed. No touched unit/property model requires a new conformance suite because none changes;
existing scientific qualification is not renewed. Existing selected stores and 56c fixtures are
preserved. Changed canonical interpretation must be coordinated once across declarations,
writers/readers, retention and backup classification, before activation on owned disposable state.

## 11. Rule impacts and disposition

These retain the coordinator's existing identities. Selection by the operator is distinct from
ADR status and from implementation acceptance. Current state remains at Plan 56.

| Impact | Current rule / proposed change | If the rule were kept |
|---|---|---|
| <a id="rc02"></a>RC02 | ADR-0116 Outcome 2 and affected blueprint D1 wording: permit owner-local non-scientific enums/document families through existing generation; keep scientific registry canonical | E01/minimal F01 need a different effective partition before claiming closure |
| <a id="rc03"></a>RC03 | Prior I21 placement / blueprint §3.2: move complete private evidence/knowledge backend and generation contributions to xtask, preserving product capabilities | Another actual dependency seam is required; universally enabled gating is insufficient |
| <a id="rc04"></a>RC04 | Blueprint §3.2 composition roots: worker/project product binaries move to existing runtime; offline recovery stays with operations and fixtures/exploration with xtask | Demonstrate a genuinely product-only xtask closure instead |
| <a id="rc05"></a>RC05 | Capacity/conformance authority: separately typed production profiles in existing capacity configuration, Rust-owned materialization, conformance as consumer | Remove mirrors while explicitly resolving the remaining conformance-to-production dependency |

ADR-0230 additionally changes canonical installation/capture interpretation at blueprint
§19.3/§20.1/§20.4/§20.5/§20.6. Its coordinated revision must include new durable family lifecycle
classification. No new numerical, host-capacity, scientific-registry or compatibility exception is
recommended. ADR-0116 is amended through the decision route rather than edited as an accepted record.

## 12. Decision and next consequential evidence

**Behavioral/semantic adequacy:** ADR-0229 and D01/D02/D03/D05/D06 pass at Proposed design level.
They explicitly preserve original scientific and effect authority. D04 remains unresolved for
coherence, owned protection and recovery under actual active concurrency, despite its now adequate
lifetime specification.

**Architectural fitness:** ADR-0229 and the independent D scope satisfy AP-01–AP-07 qualitatively.
D04's AP-07/G9 remains unresolved. The combined ADR-0230 therefore receives **Not Accept** at this
review point; this is not an Accept-scoped exception to a MUST. Its independent contracts may
proceed within authorized work while the capture mechanism and cap remain unaccepted.

The next decision is evidence-led: reassess F01 after the short exact-native active-capture probe.
If it passes, retain F02's complete protection/replay/drain/classification contract through
implementation and then qualify the complete 10,000-point journey. No successful local probe or
accepted ADR alone closes Plan 56d/56e, the broader Plan 56 series or measured benefit obligations.
