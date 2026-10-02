---
title: "25: Coordinated design remediation"
status: in-progress
date: 2026-09-30
adrs: []
review_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md]
scenario_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s01, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fs01]
---

# 25: Coordinated design remediation

## State, purpose and reading order

**25a, 25b, 25c, 25d, 25e and 25f implementation are complete.**
The [25c Outcome](25c-process-composition-and-conservation.md#outcome-recorded-after-implementation)
owns its composite focused evidence, final compilation and limits.
The [25d Outcome](25d-mathematical-realization-and-response.md#outcome-recorded-after-implementation)
owns selected implicit meaning, exact library rational/export fidelity, demand-aware derivatives
and shared fitting/public Root response. Required E/F/H/I/J prerequisite slices are implemented
and tested; subsequent E/F completion is recorded below. ADR-0144 remains proposed pending its decision PR.
The [25e Outcome](25e-declared-analyses-and-qualification.md#outcome-recorded-after-implementation) owns declared execution, composed qualification, endpoints, supervised shooting and the required preserving operational transition, with composite focused evidence. I4 is complete. ADR-0145/0146 remain proposed pending their decision PRs.
The [25f Outcome](25f-studies-diagnostics-and-continuation.md#outcome-recorded-after-implementation) owns the shared study definition/policy, typed diagnostics, admitted physical bindings, durable receipt recovery and generated study boundaries. **Implemented/Tested, 2026-10-01:** F05/F06/F07/F08/F17/F28/FU09 functional corrections are resolved with bounded [25f verification](25f-studies-diagnostics-and-continuation.md#verification). Required G/H/I/J slices are consumed; their wider scope, including F13/F24, remains open. The next work is [25g's remaining evolution scope](25g-durable-contract-evolution.md#packets). Real PostgreSQL and mixed-operation journeys, aggregate qualification and measurements remain 25k work.
F02/F33/FU04/FU05 are resolved with scoped 25c evidence; 25e completes the remaining E-owned structural/closure obligations of F16.
The [25a Outcome](25a-physical-values-and-contextual-contracts.md#outcome-recorded-after-implementation)
records its implementation, focused verification and limits. It resolves the physical findings
identified below; compound findings retain their remaining owners. Integrated qualification
remains 25k work, and ADR-0135/0136 remain proposed pending their decision-PR route.
The [25b Outcome](25b-scientific-knowledge-and-applicability.md#outcome-recorded-after-implementation)
records its scientific contracts and composite focused verification. F11 and FU01–FU03 are
resolved; 25e completes E3 result qualification for F12. ADR-0140/0141 remain proposed.
The series integrates the [original review](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md) and [follow-up](../design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md):
35 original findings, 12 additional findings and 10 remedy qualifications.

Read this coordinator, then the relevant lettered plans. Each packet describes its concrete
implementation vision—input, admitted product, preserved meaning, producer/consumer behavior,
failure cases, replacement and acceptance—while leaving local code organization to the implementer.
Conceptual product names describe required contracts; they are not a demand for a particular
struct spelling or a new type for every paragraph.

This document owns the overall lifecycle, cross-plan decisions and **current finding dispositions**.
Each lettered plan owns its packet progress and eventual implementation evidence. Reviews remain
historical evidence; packet completion and accepted ADRs do not by themselves resolve a finding.
There is no separate backlog or duplicate status table in the child plans.

### Target architecture

Authored knowledge and model definitions become context-bound admitted products. Physical
meaning survives conversion, expression composition, connection and numerical lowering.
Scientific operations produce independent evidence; one qualification owner decides result/seed
permission. Studies consume those decisions through a pure occurrence policy. Durable storage
preserves recorded meaning through explicit evolution. Generated public interfaces expose the
same contracts. Reuse shares immutable mathematics while retaining source attribution and actual
resource ownership.

This is a single production path after migration. It adds explicit missing meanings where the
reviews found them and deletes competing or inactive authorities. It does not add a universal
workflow engine, new solver mathematics, generic ontology or architecture-scoring framework.

| Plan | Concrete final product/responsibility |
|---|---|
| [25a: Physical values and contextual contracts](25a-physical-values-and-contextual-contracts.md) | Canonical admitted values, semantic intermediates, coordinate maps and owner-relative transfers |
| [25b: Scientific knowledge and applicability](25b-scientific-knowledge-and-applicability.md) | Complete conserved claims, reaction projections and coherent parameter/applicability selections |
| [25c: Process composition and conservation](25c-process-composition-and-conservation.md) | State ports, one connection occurrence, indexed balances and independently assessed conservation |
| [25d: Mathematical realization and response](25d-mathematical-realization-and-response.md) | Selected implicit operations, faithful exports, derivative requirements and qualified root response |
| [25e: Declared analyses and qualification](25e-declared-analyses-and-qualification.md) | Declared execution, structural/route assessments, candidate decisions and endpoint coverage |
| [25f: Studies, diagnostics and continuation](25f-studies-diagnostics-and-continuation.md) | Diagnostic envelopes, admitted bindings, occurrence-based studies and pure transitions |
| [25g: Durable contract evolution](25g-durable-contract-evolution.md) | Verified recorded contracts, separate read/write/migration products and preserved catalog continuity |
| [25h: Authoring and admission ownership](25h-authoring-and-admission-ownership.md) | Context-complete document products, checked expression occurrences and explicit validation contexts |
| [25i: Identity, reuse and resource ownership](25i-identity-reuse-and-resource-ownership.md) | Distinct identity roles, shared immutable products, fresh attribution and completion/allocation owners |
| [25j: Generated boundaries and library consolidation](25j-generated-boundaries-and-library-consolidation.md) | Owned decoded/admitted request boundaries and mechanically generated public transport |
| [25k: Integrated qualification and closure](25k-integrated-qualification-and-closure.md) | One evidence campaign for the complete target, followed by architecture and disposition closure |

### Grounding and boundary

At authoring start, HEAD was 5b580315ed98681285ccd91b96bd665d9f4ed162. Production source,
packages, tests, Cargo declarations and architecture matched the follow-up's production baseline
at 31001ee49da1547a5c2257515291385e9f7e4294. Concurrent Plan 24 and documentation work was
preserved. The reviews' source observations informed the plan; decision-sensitive premises were
checked selectively, not re-audited wholesale.

Plan 23's Outcome remains historical qualification. Plan 24 is consulted for domain distinctions
and incompatibilities, but knowledge-base construction, procurement, database execution and
production adoption are excluded. The consulted model/correspondence notes are available;
final committed kernel-gap/schema-delta reports were not found at authoring. No plan waits for
Plan 24 to finish. Its weaker dimension-only expression checking and open-world admission are
not imported into production.

## Shared decisions and execution discipline

### One contract owner per meaning

| Meaning | Semantic owner and downstream contract |
|---|---|
| Quantity conversion, kind/reference/basis and field-facet purposes | A; F binds targets, H admits rows/expressions, G interprets recorded declarations, J transports values |
| Composition completeness, reaction extent/source, parameter identity/coherence and applicability evidence | B; C formulates physical contributions; E applies declared result-use policy |
| Independent process state, connection identity, local inventory and original-flux closure | C; E decides structural/result/endpoint qualification without re-authoring balances |
| Implicit selection, fidelity, numerical derivative requirements and square response | D; E consumes them in routes/procedures; I frames their relevant semantic dependencies |
| Mode-qualified structural assessment, route decision, authored execution and scientific permission | E; F projects and schedules, G persists, J exposes |
| Detailed failure identity and operation/effect-sensitive retry; point occurrence/dependency policy | F; E retains scientific decision authority, store adapters retain effects/locks |
| Recorded interpretation, consumer projection, exact writer admission and explicit migration | G; schema meaning stays with its declaring operation; no transform-on-open mutation |
| Checked occurrence and complete interpretation context | H; I can reuse that admitted product without weakening its obligations |
| Identity roles, immutable allocation owners, revision attribution and native-work lifetime | I; F distinguishes binding content from occurrence identity |
| Public wire generation and mechanical adapters | J; request admission/defaults remain with the consuming Rust owner |

A physical compatibility operation is not Arrow field equality. A content hash is not collection
equality. An operation's scientific usability is not its job completion. A model's residual
relation is not automatically a selected function. These distinctions appear in the actual
products and consumers, not merely explanatory metadata.

### Direct target cutover

Every functional packet compiles affected owners, runs focused behavioral controls and deletes
the replaced code, callers, obsolete tests and fixtures as soon as the replacement works.
No compatibility APIs, unused alternate mechanisms, feature-flagged old production path or
tests retained solely to exercise deleted code remain. Independent scientific oracles and
historical-format fixtures survive only when they establish a continuing target contract.

Generated declaration changes are regenerated with the functional change. Formatters, lint,
full integration, solver/Python campaigns and performance qualification **do not run at packet
or lettered-plan boundaries**. The whole series has one final campaign in 25k after all functional
scope. That is the maintainer's explicit execution choice for this design pivot.

J1/J2 are migration work across the series: execute each complete operation slice alongside
its producer's interface change, including generated adapters and callers. Their late position
in the order below means completion of the full inventory, not permission to leave old consumers
or shims until then. Schema changes likewise use G3 as each producing packet lands. No packet
can claim its deletion obligation while retaining a replaced downstream path.

The plans define behavioral controls, not an exhaustive file/function script. Use existing
recipe-owned compile/targeted-test commands; no broad journey may be hidden under a unit label.
A new mechanism's focused tests accompany it. Author coupled fixtures with their owner, then
execute them in K3. Record checkpoint state/decisions/next steps without per-command rerun logs.

### Authority changes precede the affected implementation

The following decision briefs are selected by these plans. Allocate real ADR identifiers through
the existing route when beginning their implementation; no numbers or acceptance were invented
during authoring. Each affected packet includes its decision/architecture work before changing
the governed contract. Ordinary fixes within accepted contracts do not acquire unnecessary ADRs.

| Decision family | Selected change | Owning packets and route |
|---|---|---|
| Physical meaning | Semantic anonymous intermediates, explicit coordinate mappings, transfer orientation, schema-owned facet purposes | A1–A4: D5/ADR-0124 and metadata changes use ADR plus design review; §8/§9.8; R-51 trigger |
| Scientific/process operations | Composition claims, reaction projection, coherent parameter selection, state connections and conservation | B/C: package changes where sufficient; short ADR for new kernel contracts; §9/§10/§12/§13 and ADR-0127 rationale |
| Mathematical realization | Function selection versus relation, honest export fidelity, demand-aware capabilities | D1–D3: short ADR reconciling ADR-0100/0105; D6/§7.5 truth; changed D decision uses full route |
| Root response | Qualified square response independent of optimization KKT scope | D4/E2: amend ADR-0118 scope, §15.5.1/§25 |
| Execution/qualification | Mode-aware structure, declared execution, explicit lexicographic choice, incumbent and endpoint policy | E: ADR-0106/0111 and relevant new contracts; §15/§16.6/§19; C supplies closure |
| Study/diagnostic boundary | Detailed codes with separate projections, pure occurrence policy and typed failures | F: §19.3/§23.2; new durable/request relations use their existing decision routes |
| Durable evolution | Four compatibility operations, quiescent explicit migrations, independent histories and orphan reconciliation | G: supersede ADR-0114 Outcome 23, ADR plus design review, §20.5/§20.6 and R-35 |
| Identity/reuse | Role-typed frames, all changed preimage versions, immutable products and fresh provenance | I: hashing ADR plus review; §5.1/§5.3/§14.3/§14.4/§18.8; ordinary lifetime fixes need no new decision |
| Authoring/public interface | Checked occurrences, explicit validation context and generated owned boundary | H/J: short kernel ADR where applicable; Python-boundary ADR plus review; §4.6/§7.7/§21/§22 |

Accepted ADRs are not edited in place. Enduring architecture changes use their owners and a
blueprint revision row. Scheduling these decisions is not evidence of implementation acceptance.
The final target review in K5 complements, rather than postpones, the required decision reviews.

### Library and evolution choices

Retain library-owned symbolic/numerical/graph operations. Select the reviewed library rational,
cache, vocabulary, codec and generator utilities through their integration owners. Capability/API
details are checked at the pinned release during implementation; no library or license is
excluded. A new dependency pin is an implementation manifest choice, not a reason to leave
semantic design undecided.

Durable evolution preserves old recorded meaning and immutable identities without keeping old
production engines. PostgreSQL upgrades are explicitly quiescent and guarded from runtime
admission through the entire multi-transaction sequence; intermediate schema states cannot open
as ready. Artifact opening remains read-only. Destructive reclamation is scoped to attributable,
explicitly selected and rechecked maintenance candidates, never inferred from catalog absence.

## Dependency-ordered implementation

Packet tables in the lettered plans define their required inputs. The following is one valid,
serial-friendly order covering every packet; **rows are not synchronization barriers**. An
independent packet can start once its own prerequisites and required decision route are satisfied.

| Order | Packets | Available target capability after this group |
|---|---|---|
| 1 | [A1](25a-physical-values-and-contextual-contracts.md#a1), [B1](25b-scientific-knowledge-and-applicability.md#b1), [F1](25f-studies-diagnostics-and-continuation.md#f1), [H1](25h-authoring-and-admission-ownership.md#h1), [H3](25h-authoring-and-admission-ownership.md#h3), [I1](25i-identity-reuse-and-resource-ownership.md#i1), [I2](25i-identity-reuse-and-resource-ownership.md#i2), [I4](25i-identity-reuse-and-resource-ownership.md#i4) | Existing-invariant corrections and early value, diagnostic, identity/context contracts |
| 2 | [A2](25a-physical-values-and-contextual-contracts.md#a2), [H2](25h-authoring-and-admission-ownership.md#h2), [E1](25e-declared-analyses-and-qualification.md#e1), [F2](25f-studies-diagnostics-and-continuation.md#f2), [G1](25g-durable-contract-evolution.md#g1) | Intermediate meaning, checked occurrences, structural facts, admitted bindings and evolution contracts |
| 3 | [A3](25a-physical-values-and-contextual-contracts.md#a3), [A4](25a-physical-values-and-contextual-contracts.md#a4), [H4](25h-authoring-and-admission-ownership.md#h4), [G3](25g-durable-contract-evolution.md#g3), [J3](25j-generated-boundaries-and-library-consolidation.md#j3) | Physical wrappers/transfers, real predicates/deletions, migration mechanism and generator utilities |
| 4 | [B2](25b-scientific-knowledge-and-applicability.md#b2), [B3](25b-scientific-knowledge-and-applicability.md#b3), [C1](25c-process-composition-and-conservation.md#c1), [D1](25d-mathematical-realization-and-response.md#d1), [G2](25g-durable-contract-evolution.md#g2), [I3](25i-identity-reuse-and-resource-ownership.md#i3) | Reaction/parameter products, state ports, implicit meaning, portable reads and complete reuse |
| 5 | [B4](25b-scientific-knowledge-and-applicability.md#b4), [C2](25c-process-composition-and-conservation.md#c2), [D2](25d-mathematical-realization-and-response.md#d2), [D3](25d-mathematical-realization-and-response.md#d3), [G4](25g-durable-contract-evolution.md#g4), [I5](25i-identity-reuse-and-resource-ownership.md#i5) | Applicability, connections, faithful math/demand, artifact/orphan lifecycle and worker ownership |
| 6 | [C3](25c-process-composition-and-conservation.md#c3), [D4](25d-mathematical-realization-and-response.md#d4) | Indexed local balances/storage and qualified root response |
| 7 | [C5](25c-process-composition-and-conservation.md#c5), [E2](25e-declared-analyses-and-qualification.md#e2) | Temporal conservation and authoritative execution/defaults |
| 8 | [C4](25c-process-composition-and-conservation.md#c4), [E3](25e-declared-analyses-and-qualification.md#e3) | Executable recycle initialization and composed scientific acceptance |
| 9 | [E4](25e-declared-analyses-and-qualification.md#e4) | Endpoint obligations with settled closure and applicability |
| 10 | [E5](25e-declared-analyses-and-qualification.md#e5), [F3](25f-studies-diagnostics-and-continuation.md#f3) | Admitted shooting/result facts and the shared occurrence policy |
| 11 | [F4](25f-studies-diagnostics-and-continuation.md#f4), [J1](25j-generated-boundaries-and-library-consolidation.md#j1) | Durable executor cutover and completion of owned boundary declarations |
| 12 | [F5](25f-studies-diagnostics-and-continuation.md#f5) | General study operations and truthful provenance |
| 13 | [J2](25j-generated-boundaries-and-library-consolidation.md#j2) | Completion of generated consumer migration |
| 14 | [K1](25k-integrated-qualification-and-closure.md#k1), [K2](25k-integrated-qualification-and-closure.md#k2), [K3](25k-integrated-qualification-and-closure.md#k3), [K4](25k-integrated-qualification-and-closure.md#k4), [K5](25k-integrated-qualification-and-closure.md#k5) | Readiness, final format/static checks, integrated qualification, measurement and architectural closure |

The important joins are E1 → D4 → E2, C5 → E3/E4, G3 → durable schema cutovers, and
H/A/I ownership → wider reuse. D3 implements the shared demand operation before E2 migrates
its callers. F1 is available without the study engine. This avoids hidden whole-plan cycles.

**Parallel implementation ownership:** if later execution uses agents, assign packets and actual
edit scope before they write. Shared contract changes are coordinated by their semantic owner;
consumers adapt to that contract rather than creating a second definition. Preserve concurrent
work in the existing checkout. No worktree is required for this documentation series.

## Finding dispositions

Rows remain **open** unless the stated obligation has implementation and evidence. The listed
work is a complete resolution scope, not a deferral. “Owns closure” identifies who collects contributions for a compound
finding; each contributing obligation has one implementing packet. Sxx refers to the original
review's scenarios; FSxx refers to the follow-up's. Existing definitions are linked in the reviews,
not copied into a second scenario registry.

| Finding | Obligation | Disposition | Decision/work packets | Scenario and closure boundary |
|---|---|---|---|---|
| [F01](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f01) | Complete physical typing inside production laws and unit intermediates | resolved | [A2](25a-physical-values-and-contextual-contracts.md#a2), [A3](25a-physical-values-and-contextual-contracts.md#a3), [A4](25a-physical-values-and-contextual-contracts.md#a4) | S01/S09: All affected production consumers, including datum translation and reactor report typing, use admitted physical meaning. **Resolved:** migrated consumer inventory, retained-admission and independent scientific/source controls in the [25a Outcome](25a-physical-values-and-contextual-contracts.md#outcome-recorded-after-implementation). |
| [F02](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f02) | State/transport groups, connection occurrences and executable tears | resolved | [C1](25c-process-composition-and-conservation.md#c1), [C2](25c-process-composition-and-conservation.md#c2), [C4](25c-process-composition-and-conservation.md#c4) | S02/S07: **Implemented/Tested:** aggregate state/transport ports, occurrence topology/tears and admitted conditional execution replace scalar material wiring ([25c evidence](25c-process-composition-and-conservation.md#verification)). The scientific coupled recycle journey remains 25k. |
| [F03](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f03) | Datum-free energy transfer and owner-relative direction | resolved | [A4](25a-physical-values-and-contextual-contracts.md#a4) | S01/S09: Transfer signs are applied once; enthalpy-reference translation remains explicit. **Resolved:** actual indexed/inherited/child owner, direction/reflection, conservation and datum-translation controls in the [25a Outcome](25a-physical-values-and-contextual-contracts.md#outcome-recorded-after-implementation). |
| [F04](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f04) | One declared case execution consumed by all entrypoints | resolved | [E2](25e-declared-analyses-and-qualification.md#e2) | S04/S06: **Implemented/Tested:** one retained DeclaredExecution serves direct execution, initialization, inspection, conformance, Python and study/durable preparation; route/procedure and compatible defaults/overrides survive ([25e evidence](25e-declared-analyses-and-qualification.md#verification)). |
| [F05](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f05) | One study policy and general run-point definition | resolved | [F3](25f-studies-diagnostics-and-continuation.md#f3), [F4](25f-studies-diagnostics-and-continuation.md#f4), [F5](25f-studies-diagnostics-and-continuation.md#f5) | S04/S06: **Implemented/Tested, 2026-10-01:** both executors consume one admitted definition and pure occurrence policy over existing solve/simulation/fit/horizon owners ([25f verification](25f-studies-diagnostics-and-continuation.md#verification)). Full mixed-operation journeys remain 25k. |
| [F06](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f06) | Detailed diagnostic authority and explicit projections | resolved | [F1](25f-studies-diagnostics-and-continuation.md#f1) | S08/S10: **Implemented/Tested, 2026-10-01:** typed code, rule, causes, observations and source attribution survive purpose-specific Rust/durable/Python projections; retry also requires known-safe effects ([25f verification](25f-studies-diagnostics-and-continuation.md#verification)). |
| [F07](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f07) | Typed failures for every terminal durable attempt | resolved | [F4](25f-studies-diagnostics-and-continuation.md#f4) | S08/S10: **Implemented/Tested, 2026-10-01:** pre-context failures and stale/superseded attempts retain typed diagnostic envelopes and actual attempt lifecycles, with repeated callbacks enriching one history record ([25f verification](25f-studies-diagnostics-and-continuation.md#verification)). Real store/restart journeys remain 25k. |
| [F08](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f08) | Physical values, contextual target binding and generated transport | resolved | [A1](25a-physical-values-and-contextual-contracts.md#a1), [F2](25f-studies-diagnostics-and-continuation.md#f2), [J2](25j-generated-boundaries-and-library-consolidation.md#j2) | S09: **Implemented/Tested, 2026-10-01:** F2/J2 complete A1's physical-value/context contribution with typed quantity/target admission before binding identity, canonical member replay and generated closed transport ([25f verification](25f-studies-diagnostics-and-continuation.md#verification)). Explicit-ID stability remains conditional on the admitted context. |
| [F09](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f09) | Admitted shooting and shared completion | resolved | [E5](25e-declared-analyses-and-qualification.md#e5), [I4](25i-identity-reuse-and-resource-ownership.md#i4) | S04/S06: **Implemented/Tested:** shooting uses admitted start/handle supervision, immutable composed permission and completion-owned CPU capacity; raw solve and simulation-procedure bypasses are removed ([25e evidence](25e-declared-analyses-and-qualification.md#verification)). |
| [F10](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f10) | Actual terminal endpoint obligations and coverage | resolved | [E4](25e-declared-analyses-and-qualification.md#e4) | FS05: **Implemented/Tested:** separate actual endpoints retain terminal state/mode/pre-change inputs; conserved prefixes qualify independently of unavailable later samples and fixed-domain integrals ([25e evidence](25e-declared-analyses-and-qualification.md#verification)). |
| [F11](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f11) | Phase-independent versus phase-specific parameter context | resolved | [B3](25b-scientific-knowledge-and-applicability.md#b3) | S01: Implemented/Tested: separate keyed shapes remove the vapor sentinel and preserve PC-SAFT liquid/vapor applicability; focused selection and potential controls pass ([evidence](25b-scientific-knowledge-and-applicability.md#verification)). Full qualification remains 25k. |
| [F12](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f12) | Applicability evidence and explicit data-use permission | resolved | [B4](25b-scientific-knowledge-and-applicability.md#b4), [E3](25e-declared-analyses-and-qualification.md#e3) | S08: B4 evidence and data-use permission remain independently visible ([25b evidence](25b-scientific-knowledge-and-applicability.md#verification)); **Implemented/Tested:** E3 composes Unknown/extrapolation qualifiers without waiving physical/domain obligations ([25e evidence](25e-declared-analyses-and-qualification.md#verification)). |
| [F13](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f13) | Directional evolution, independent catalog identity and orphan lifecycle | open | [G1](25g-durable-contract-evolution.md#g1), [G3](25g-durable-contract-evolution.md#g3), [G4](25g-durable-contract-evolution.md#g4) | S05/FS10: **Implemented/Tested, 2026-10-01:** required G3 slices have independent support identities/histories and exact preserving transitions from 25e through appended V5, retaining historical-unavailable meaning without rewriting old payloads ([25e evidence](25e-declared-analyses-and-qualification.md#verification), [25f verification](25f-studies-diagnostics-and-continuation.md#verification)). G1 directional interpretation, remaining G3 planning/report surfaces and G4 orphan lifecycle remain open; real DB transition qualification remains 25k. |
| [F14](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f14) | Portable predicates derived from recorded witnesses | open | [G2](25g-durable-contract-evolution.md#g2) | S11/FS10: Old domains remain unchanged; opening is read-only |
| [F15](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f15) | Complete body/view/package reuse with correct attribution | open | [I3](25i-identity-reuse-and-resource-ownership.md#i3), [I5](25i-identity-reuse-and-resource-ownership.md#i5) | S07/FS08/FS11: I3 owns closure including I5 durable-worker evidence; production reuse replaces coarse roots and private LRU |
| [F16](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f16) | Structural assessment and truthful closure verdicts | resolved | [E1](25e-declared-analyses-and-qualification.md#e1), [C5](25c-process-composition-and-conservation.md#c5), [E3](25e-declared-analyses-and-qualification.md#e3) | S02/FS04/FS05: C5 original physical closure descriptors/evaluation are complete ([25c evidence](25c-process-composition-and-conservation.md#verification)); **Implemented/Tested:** E1/E3 complete original mode-qualified structural admission and truthful closure permission, including missing required evidence and combined opt-ins ([25e evidence](25e-declared-analyses-and-qualification.md#verification)). |
| [F17](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f17) | Exhaustive purpose-specific outcome projections | resolved | [F1](25f-studies-diagnostics-and-continuation.md#f1), [E3](25e-declared-analyses-and-qualification.md#e3), [F4](25f-studies-diagnostics-and-continuation.md#f4) | S10: **Implemented/Tested, 2026-10-01:** exhaustive diagnostic/result/seed/retry projections consume E3's single scientific decision while preserving typed assessment causes and effect facts ([25f verification](25f-studies-diagnostics-and-continuation.md#verification)). |
| [F18](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f18) | Owned Rust documents, typed getters/reasons and generated Python | open | [J1](25j-generated-boundaries-and-library-consolidation.md#j1), [J2](25j-generated-boundaries-and-library-consolidation.md#j2) | S05/S09: J2 owns closure across every exported request/result/error/default, including flow selection |
| [F19](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f19) | One grammar and checked expression occurrences | open | [H2](25h-authoring-and-admission-ownership.md#h2) | S06: Quoted logic and attributed syntax errors use admitted AST occurrences |
| [F20](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f20) | Delete inactive authorities and front doors | open | [H4](25h-authoring-and-admission-ownership.md#h4) | S01/S11: Raw Inputs/queries, unused structure/projection, duplicate invariants and inert requirement policy are all addressed |
| [F21](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f21) | Capability-owned lexicographic choice | resolved | [E1](25e-declared-analyses-and-qualification.md#e1) | S03: **Implemented/Tested:** native versus staged priorities follow adapter capability and auto/explicit policy; unsupported explicit selection refuses and partial-incumbent permission cannot advance a priority ([25e evidence](25e-declared-analyses-and-qualification.md#verification)). |
| [F22](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f22) | Qualified root sensitivity without a dummy objective | resolved | [D4](25d-mathematical-realization-and-response.md#d4), [E2](25e-declared-analyses-and-qualification.md#e2) | S04/FS06/FS07: **Implemented/Tested:** shared qualified physical square response serves fitting and public Root sensitivity without objective/dual/KKT fields; invalid neighborhoods or unavailable optional analysis preserve the qualified base root ([25d evidence](25d-mathematical-realization-and-response.md#verification)). Broader E2 route/default ownership remains open. |
| [F23](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f23) | Explicit qualified-incumbent policy | resolved | [E3](25e-declared-analyses-and-qualification.md#e3) | S10: **Implemented/Tested:** default result refusal preserves lawful seeds; feasible/gap opt-ins retain original qualification, stop, objective-sense/bound-origin and independent closure/applicability evidence ([25e evidence](25e-declared-analyses-and-qualification.md#verification)). |
| [F24](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f24) | One closed-vocabulary mechanism and unknown-member contract | open | [F1](25f-studies-diagnostics-and-continuation.md#f1), [J3](25j-generated-boundaries-and-library-consolidation.md#j3) | S10: **Implemented/Tested, 2026-10-01:** F1's semantic projections and source-owned study/diagnostic vocabulary generate closed boundary documents ([25f verification](25f-studies-diagnostics-and-continuation.md#verification)). J3's wider generated/handwritten enum mechanics and strum consolidation remain open. |
| [F25](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f25) | Exact library rationals and truthful export scope | resolved | [D2](25d-mathematical-realization-and-response.md#d2) | FS06: **Implemented/Tested:** arbitrary Symbolica Rational, canonical versioned preimages, explicit finite native conversion and truthful Exact/Relaxed selected export ([25d evidence](25d-mathematical-realization-and-response.md#verification)); D6 permits non-evaluating transport through proposed ADR-0144. |
| [F26](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f26) | Hex, generator graph and collection validators | open | [I1](25i-identity-reuse-and-resource-ownership.md#i1), [J3](25j-generated-boundaries-and-library-consolidation.md#j3) | S05: Three child obligations: I1 hex; J3 topological ordering and equality-preserving cardinality/uniqueness |
| [F27](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f27) | Visible profile-chain spawn/panic outcomes | open | [I5](25i-identity-reuse-and-resource-ownership.md#i5) | FS11: No swallowed panic or silent replay; actual parallelism and failure are reported |
| [F28](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f28) | Persisted route and admission explanation | resolved | [E1](25e-declared-analyses-and-qualification.md#e1), [E5](25e-declared-analyses-and-qualification.md#e5), [F5](25f-studies-diagnostics-and-continuation.md#f5) | S03/S10: **Implemented/Tested, 2026-10-01:** F5 attaches retained declared route/admission facts to the shared study definition and occurrence outcomes, with typed refusals, actual start provenance and scientific permissions ([25e evidence](25e-declared-analyses-and-qualification.md#verification), [25f verification](25f-studies-diagnostics-and-continuation.md#verification)). |
| [F29](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f29) | Representation-specific structural admission | resolved | [E1](25e-declared-analyses-and-qualification.md#e1), [D2](25d-mathematical-realization-and-response.md#d2) | S03/FS06: D2 export fidelity and D4 complete original square matching are complete ([25d evidence](25d-mathematical-realization-and-response.md#verification)); **Implemented/Tested:** E1 completes representation-specific Root/equality/native-feasibility admission using the existing original witness ([25e evidence](25e-declared-analyses-and-qualification.md#verification)). |
| [F30](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f30) | Canonical floats, role-typed frames and request identities | open | [I1](25i-identity-reuse-and-resource-ownership.md#i1), [G3](25g-durable-contract-evolution.md#g3) | S09/FS08: I1 owns frame/role migration; G3 handles persisted operational field changes without rehashing history |
| [F31](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f31) | Explicit validation context and safe implementation construction | open | [H3](25h-authoring-and-admission-ownership.md#h3) | S06: No first-wins ambient state; local and concurrent construction cycles refuse |
| [F32](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f32) | Demand/default/start rules, accelerators and supervision | open | [D3](25d-mathematical-realization-and-response.md#d3), [E2](25e-declared-analyses-and-qualification.md#e2), [I4](25i-identity-reuse-and-resource-ownership.md#i4), [I5](25i-identity-reuse-and-resource-ownership.md#i5), [J2](25j-generated-boundaries-and-library-consolidation.md#j2) | S04/S06: D3 demand/admitted adapter minima are complete ([25d evidence](25d-mathematical-realization-and-response.md#verification)); **Implemented/Tested:** E2 Rust defaults/typed starts and I4 supervision plus their required Python consumers are complete ([25e evidence](25e-declared-analyses-and-qualification.md#verification)). I5 accelerator preservation and broader J2 generated defaults remain open. |
| [F33](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f33) | Indexed control-volume boundaries and supported material bases | resolved | [C3](25c-process-composition-and-conservation.md#c3) | S02/FS04: **Implemented/Tested:** shared indexed balances retain component/phase/element/total semantics and unit-specific laws ([25c evidence](25c-process-composition-and-conservation.md#verification)); element/total storage without explicit inventory projection refuses. Full journeys remain 25k. |
| [F34](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f34) | Declared facet purposes and resolved unit compatibility | open | [A1](25a-physical-values-and-contextual-contracts.md#a1), [H4](25h-authoring-and-admission-ownership.md#h4) | S09: A1 owns meaning/projection policy; H4 migrates real admission comparators. A1 declarations, generated projections and selected-unit admission are complete ([evidence](25a-physical-values-and-contextual-contracts.md#outcome-recorded-after-implementation)); broader H4 admission consolidation remains open. |
| [F35](../design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#f35) | Specification, datum and structural-attribution corrections | open | [A3](25a-physical-values-and-contextual-contracts.md#a3), [A4](25a-physical-values-and-contextual-contracts.md#a4), [E1](25e-declared-analyses-and-qualification.md#e1), [E5](25e-declared-analyses-and-qualification.md#e5), [H4](25h-authoring-and-admission-ownership.md#h4), [I1](25i-identity-reuse-and-resource-ownership.md#i1) | S01/S08/S09: A3 physical-partial/A4 datum contributions are complete ([25a evidence](25a-physical-values-and-contextual-contracts.md#outcome-recorded-after-implementation)); **Implemented/Tested:** E1 solve-refusal model paths and E5 numerical-source inventory/obsolete identity-owner correction are complete ([25e evidence](25e-declared-analyses-and-qualification.md#verification)). H4 unused projection and I1 identity-owner consolidation remain open. |
| [FU01](../design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fu01) | Executed reaction source derived from checked chemistry | resolved | [B2](25b-scientific-knowledge-and-applicability.md#b2) | FS01: Implemented/Tested: both reactors consume authoritative coefficients; missing products, inert extras and extent/heat mismatches have focused controls ([evidence](25b-scientific-knowledge-and-applicability.md#verification)). The callback and forwarding source tables are deleted; full qualification remains 25k. |
| [FU02](../design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fu02) | Known composition distinguished from unknown and complete-empty | resolved | [B1](25b-scientific-knowledge-and-applicability.md#b1) | FS02: Implemented/Tested: explicit composition completeness, independent charge evidence and guarded sparse lookups refuse unsupported conservation while retaining catalog Unknown and Complete-empty roles ([evidence](25b-scientific-knowledge-and-applicability.md#verification)); full qualification remains 25k. |
| [FU03](../design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fu03) | Per-pair parameterization selection, variants and coherence | resolved | [B3](25b-scientific-knowledge-and-applicability.md#b3) | FS03: Implemented/Tested: mixed-source ternary NRTL, directional/symmetric selections, joint-fit closure and allowed subsystems distinguish fitted/missing/predictive zero ([evidence](25b-scientific-knowledge-and-applicability.md#verification)); full qualification remains 25k. |
| [FU04](../design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fu04) | Global time analysis separated from local storage | resolved | [C3](25c-process-composition-and-conservation.md#c3) | FS04: **Implemented/Tested:** mixer → stored CSTR → separator stays algebraic outside declared local storage in steady/integrated/simultaneous representations ([25c evidence](25c-process-composition-and-conservation.md#verification)); native coupled journeys remain 25k. |
| [FU05](../design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fu05) | Authored reusable temporal conservation | resolved | [C5](25c-process-composition-and-conservation.md#c5) | FS05: **Implemented/Tested:** reusable original-inventory/flux conservation checks matching/wrong/unauthorized transfers, drift and mode/terminal settlement on Diffsol/IDAS ([25c evidence](25c-process-composition-and-conservation.md#verification)). Event-bearing simultaneous requests refuse; E retains result permission. |
| [FU06](../design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fu06) | Implicit selected-function meaning and exact-export evidence | resolved | [D1](25d-mathematical-realization-and-response.md#d1), [D2](25d-mathematical-realization-and-response.md#d2) | FS06: **Implemented/Tested:** compiler-owned relation/branch/operational selection survives realization and export; checked affine/sign-restricted square-root equivalence or conservative Relaxed fidelity, with explicit local selector/bound refusal ([25d evidence](25d-mathematical-realization-and-response.md#verification)). No general uniqueness proof is claimed. |
| [FU07](../design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fu07) | Demand-aware implicit derivatives | resolved | [D3](25d-mathematical-realization-and-response.md#d3) | FS07: **Implemented/Tested:** shared inner/outer requirement algebra and reverse nested demand compile stable C1 First and C2 Second honestly; unsupported smoothness/selection orders refuse ([25d evidence](25d-mathematical-realization-and-response.md#verification)). |
| [FU08](../design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fu08) | Complete context for incremental document reuse | open | [H1](25h-authoring-and-admission-ownership.md#h1) | FS08: Final-source admission agrees with clean loading after policy/identity changes |
| [FU09](../design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fu09) | Study occurrence identity independent of binding equality | resolved | [F3](25f-studies-diagnostics-and-continuation.md#f3), [F4](25f-studies-diagnostics-and-continuation.md#f4) | FS09: **Implemented/Tested, 2026-10-01:** explicit occurrence keys and revised durable rows permit equal binding content in distinct experiments. Revision/lease fencing preserves retry identity; exact repeated member receipts are idempotent, conflicting descriptors refuse, and inventory supersession requires a known-safe original effect ([25f verification](25f-studies-diagnostics-and-continuation.md#verification)). Real duplicate-occurrence/recovery DB journeys remain 25k. |
| [FU10](../design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fu10) | Finite canonical conversion | resolved | [A1](25a-physical-values-and-contextual-contracts.md#a1) | S09: Input and converted magnitudes checked before constructing a physical value. **Resolved:** source/converted nonfiniteness, multiply/add overflow, affine and near-limit controls through inline/column admission in the [25a Outcome](25a-physical-values-and-contextual-contracts.md#outcome-recorded-after-implementation). |
| [FU11](../design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fu11) | Native completion owns staged-step CPU admission | open | [I4](25i-identity-reuse-and-resource-ownership.md#i4) | FS11: Abandonment cannot release capacity before native completion/cleanup |
| [FU12](../design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fu12) | Value rebind allocations have actual live owners | open | [I2](25i-identity-reuse-and-resource-ownership.md#i2) | FS11: Shared provenance and new binding payload are retained/charged correctly |

For F13, catalog continuity, semantic compatibility and orphan repair are separate obligations;
migration support alone cannot close the finding. For F18, generating a shape is insufficient
without moving admission/defaults and all boundary consumers. For F32/F35 the explicit child
assignments above prevent one small correction from closing the whole bundled finding.
The coordinator may mark a functional correction resolved only after every assigned child has
its relevant bounded evidence; integrated K qualification remains a separate open obligation.
There are no silently omitted or indefinitely “related” findings.

### Follow-up remedy constraints

| Refinement | Required treatment in the implementation plans |
|---|---|
| R1 | A supports lawful direct/reciprocal coordinate maps and semantic intermediates; F resolves complete contextual overlays; named-policy renames are not promised stable |
| R2 | B separates phase scope, parameter identity, empirical knowledge and permission; preserves unknown and explicit unrestricted claims |
| R3 | C defines independent state/transport and admitted conditional execution, not merely a graph or scalar bundle |
| R4 | C/E preserve closure kind, mode-qualified structure and actual endpoint obligations; later required observations do not disappear |
| R5 | D uses F_x X_p = -F_p with physical/scaled validity and selector stability; root regularity is not optimization KKT sufficiency |
| R6 | F separates scientific decisions, seed demand/capability, dependency intent and operational effects, including NotNeeded seeds |
| R7 | F preserves detailed code/cause and derives purpose-specific projections; it never reverses a many-to-one class map |
| R8 | G separates recorded interpretation, consumer projection, exact write admission and explicit migration; opening stays read-only |
| R9 | H/J admit meanings and expression occurrences with context; generated shape and lexical deletion counts alone prove little |
| R10 | H/I separate complete admission inputs, immutable mathematics, revision attribution, binding state and allocation lifetime before wider reuse |

These qualifications modify proposed remedies while retaining the problem each original finding
identified. No original review was silently rewritten, and no prior positive assessment is
promoted into fresh whole-system qualification.

## Verification, authoring assessment and completion

**Proposed acceptance** is specified in each packet and assembled in 25k. Functional packet
evidence is local and bounded. K separately establishes integrated scientific/behavioral
adequacy, architecture under the selected core 3.3/process-simulator 1.3 standard, and measured
reuse/resource behavior. No counts of deleted helpers or adopted libraries substitute for these.

The authoring effort used three read-only agents for domain/contract design and cross-author
challenge, with one coordinating document writer. The challenge refined operational-selector
derivative validity, terminal event input-side semantics, unknown versus complete-empty
composition, pair-fit coherence/predictive zeros, facet-purpose distinctions, collection equality,
frame versioning, construction cycles and migration/runtime exclusion. These are design decisions,
not executed counterexamples.

No production code, accepted ADR, generated tree or review evidence was changed in authoring.
No product tests, builds, benchmarks, full integration or static qualification campaign ran.
Only the new series and its plan-index navigation are authored here. Selected library mechanisms
are Interface-checked where the plans cite the reviewed/pinned evidence; proposed behavior is
not labelled Tested.

The implementation is complete only when:

- All 43 functional packets have delivered their target consumers and deletion obligations.
- Required decision routes and architecture corrections are complete.
- K1–K5 have established their named evidence against the repository's zero target, with
  limitations stated honestly.
- Every finding's full obligation is resolved by evidence, or an actual changed conclusion is
  explicitly reviewed and recorded. Writing a plan or accepting an ADR does not resolve it.
- Enduring meaning has moved to its proper owner before completed plans/reviews retire.

No material architectural choice is intentionally deferred to an “investigate later” packet.
Routine struct/function layout, exact focused test names and compatible dependency pin selection
remain implementation judgment. If implementation evidence falsifies a selected design, update
the owning decision and dependent plans explicitly rather than retaining a compatibility path.

## Outcome (recorded after implementation)

### What was built

Not implemented. The current artifact is the Proposed coordinated design and work sequence.

### A mistake made and corrected

Record an actual implementation correction when closing the series.

### Deviations from the plan, deliberate

None in production. During plan authoring the maintainer requested more concrete implementation
visions; every functional packet was expanded to state products, dataflow and final consumer behavior.
