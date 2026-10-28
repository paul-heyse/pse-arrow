---
title: "Plan58: model-driven Rust execution and hard cutover"
date: 2026-10-28
doc_role: review
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [architecture, design, libraries, validation]
---

# Plan58: model-driven Rust execution and hard cutover

## Combined judgment and scope

**Proposed:** amend Plan58 toward a Rust-owned system whose durable execution model lives in canonical SurrealDB, whose native workers perform scientific effects, and whose development transition uses integrated replacements rather than compatibility bridges. Remove obsolete mechanisms and requirements instead of translating the Python operating layer into Rust.

The colleague's [review](../functional_reviews/external_rust_pivot_review/python-subsumption-review-2026-10-28.md) and [evidence appendix](../functional_reviews/external_rust_pivot_review/python-subsumption-review-2026-10-28-evidence.md) strongly support eliminating Python from compiler, host, fixture, orchestration and product paths. Their proposed native catalogue, admission daemon, full Python replacement surface and interim FastMCP proxy do not follow automatically from that evidence. Several are unnecessary under the operator's selected target.

This is the authorized **plan-improvement review** of the remaining [Plan58 series](../plans/58-rust-owned-system.md), not a completion audit, broad diagnostic audit or implementation. The inspected source and plan baseline is `9d6604cfb`; production source under crates/python/scripts/xtask has not changed since the external review's `bd1c4891e`. User-supplied external reports and cdylib/nextest/lifecycle overviews were untracked inputs when inspected. They were not modified. Core 3.4, efficiency heuristics 1.0, test assessment 1.2 and process-simulator 1.5 apply through `standard.toml`.

**Interface-checked:** source paths, active consumers and documented library mechanisms. **Proposed:** all amendments below. External timings remain **Measured (sandbox)** only; estimated per-build savings are not current-host measurements. No test, compiler, database service, workload or performance campaign ran for this review. Documentation closeout does not qualify the proposed runtime.

### Operator directions controlling this judgment

During this engagement the operator selected integrated cutovers, accepting temporary incompleteness during development rather than transitional Python bridges. CLIs remain optional, RMCP conditional, and PyO3 implementation deferred. Subsequent clarification sharpened the target:

- Live use of an old function is not a reason to preserve Python, its mechanism or even its requirement.
- Prefer typed construction, ownership and real connections over duplicate declarations and runtime guard systems.
- Bespoke build observation is not a desired capability; scientific calculation observation is.
- Durable structured storage and migration infrastructure are essential long-term foundations. Regenerable development state should not impose full production recovery and historical-migration qualification on every change.
- Explore SurrealDB as the model governing durable execution, not just a persistence adapter beneath application-owned coordination.

These directions modify earlier research advice that treated every existing operational guarantee as a carry-forward obligation. Preservation recommendations below are tied to retained outcomes, not inherited mechanism lists. No current database deletion or reset is authorized by this document.

## Architectural argument

The largest improvement is a change in responsibility, not language syntax. Authored scientific meaning, durable execution facts and host effects need explicit owners; they need not each create a ledger, process, acknowledgement, generated DTO family or validation pass.

For repeated studies, long-running execution and later daily analyses, use the canonical data model for definitions/revisions, requests, dependencies, realized points, attempts, assignments, state transitions and result references. Rust operations and database transitions should realize that same meaning. A native worker consumes selected executable intent and publishes its actual outcome; it should not maintain another durable interpretation of readiness and completion.

This is an extension of existing capabilities, not a proposal to invent a workflow engine from scratch. Generated study functions already own readiness, claim, settlement, cancellation and conclusion. The readiness module executes shared Rust study policy over bounded facts inside the database snapshot. Their existence is a useful foundation, not proof that every remaining coordination path is already necessary or optimal.

Host process existence, foreign solver workspaces and actual native completion remain physical facts. Their integration should be a narrow performing owner, not a competing workflow model. OS containment and Rust lifetimes can remove many application-level fixture/allocation checks. Compile-time construction cannot prove that an independent process did not crash; observe such uncertainty at its actual boundary rather than repeatedly reconstructing it throughout the system.

AP-01/AP-03 favor removing intermediary owners; AP-02/AP-05 favor capability-bearing connections instead of parallel permission declarations; AP-04 favors an execution model that actually governs behavior; AP-06 favors ordinary Rust tests without unrelated infrastructure; AP-07 favors delegation of complete database/tool capabilities and deletion of repeated scans, transcoding and per-compiler supervision. These are separate judgments from numerical/model adequacy: existing physical semantics and independent scientific outcome checks retain their meaning.

<a id="I01"></a>
## I01 — Make integrated responsibility cutovers the execution unit

**Amend T02, B03–B04, C04, F02–F05, G01–G03 and H.** Plan58 already forbids permanent fallbacks, but its package boundaries can still invite incremental porting of helper files and preservation of each old workflow between commits. Replace that interpretation with coherent cutover sets: the new owner, its actual callers, configuration, generators, tests and old-path deletion form one integrated unit.

Temporary broken development paths may be recorded while a unit is implemented. They receive no acceptance credit. Do not build compatibility DTOs, a Rust server proxying FastMCP, a Python admission fallback or a second registry to keep that intermediate state looking finished. Independent cutover sets may progress separately; a hard cutover is not a requirement for one repository-wide mega-commit or one new monolithic crate.

Give each set an explicit retained outcome, native owner, removed mechanisms and final public behavior. Reuse a prior black-box test temporarily where it supplies independent evidence; retaining a test harness is not retaining the old production implementation. H qualifies the assembled target after functional scope exists.

<a id="I02"></a>
## I02 — Put the durable execution model in SurrealDB

**Amend A05, C02–C05, D01–D04, E01–E03 and the coordinator target.** Strengthen the target from “shared Rust operations with a store” to canonical model-driven orchestration for durable workflows. Definitions, dependency state, attempts, assignments and scientific outcomes should have one authoritative representation whose guarded operations govern execution and later analysis.

`crates/pse-codegen/src/codegen/surreal_studies.rs`, `pse-study-module` and `pse-model::study_policy` already supply bounded readiness/policy/transition composition. Prefer extending a complete store-side operation—select/recheck/claim, or advance readiness and return executable intent—over a client loop that fetches candidates, independently interprets prerequisites, retries stale updates and mirrors status in memory. The current worker's `work_once_scoped` performs study paging, readiness advancement, candidate discovery and finalization; `work_study_candidate` evaluates scope, prepares inputs, reevaluates with seed evidence and then claims. A bounded domain command must distinguish progress/continuation through an examined scope from an exhaustive no-work result. It can return the next applicable action (preparation needed, settled/refused, claim granted or idle), with exact durable premises. These are illustrative domain outcomes, not a mandatory universal enum or new catalogue. Preserve preparation/resource admission before execution claim, and recheck the prepared evidence against current dependencies/cancellation before granting that claim. Use ordinary SurrealQL first; use Surrealism when shared bounded Rust policy earns the module boundary. Verify current permissions and controller limits before expanding module capabilities.

Keep expensive preparation and native solver calls outside database transactions. Record actual realized points, useful calculation observations and result references at the intended analytical granularity. Do not serialize live Rust object graphs, native pointers or Tokio handles as the durable execution model; choose progress retention from its scientific/operational use rather than mirror every implementation event by default. Notifications may wake workers; authoritative state resolves lag, restart and missed events. Original intent/operation identity handles an uncertain command acknowledgement without another application scheduler journal.

One-off pure computation can remain a direct native operation. This recommendation does not force a database round trip around every local function. Its value is most concrete for persisted studies and repeated analysis, where the execution model is also the long-term queryable history.

**Closure:** create/revise a study definition with a dependency before activation, interrupt the worker/client and resume selection from canonical state without reconstructing a second workflow truth. Current `append_dependencies` refuses new edges after activation/cancellation; live graph mutation is not implied. Two workers cannot both acquire the same applicable assignment. A later analysis reads the same recorded scientific lineage rather than a separately reconstructed history. No measured throughput claim follows.

<a id="I03"></a>
## I03 — Separate durable infrastructure from development recovery burdens

**Amend T01, B04, D03, F02/F04, G03 and H03/H04.** Keep structured schemas, versioned interpretation, transactional updates, meaningful identities and a deliberate migration interface. Do not make exhaustive old-data migration, complete backup/restore and every interrupted-maintenance recovery path prerequisites to this development pivot.

For explicitly owned regenerable development/test deployments, prefer a fresh schema and reconstruction from authored inputs/reference material to porting resumable Python registry converters or maintaining compatibility with every historical store format. Define disposable placement at creation; it does not grant permission to erase arbitrary existing stores. Preserve uniquely authored inputs and independent reference evidence outside automatic disposal.

Deliver a minimal native versioned-schema/migration owner as part of the cutover, rather than only documenting a future interface. Reuse the registry/generation and appropriate native migration facilities; do not introduce a second schema declaration or revive the entire beta experiment. The implementation must declare and apply ordered compatible transformations, check predecessor applicability at migration/store construction, fence incompatible writes and publish the resulting schema/version state. An established typed store capability carries those premises; ordinary operations do not repeat the whole admission proof. Specify the persistent-deployment boundary: which schema/version is supported, where structured migrations are declared/applied, who opens/closes write admission, and what transactional/identity semantics survive process restart. Initial migration readiness should have an explicit predecessor/successor relation and a representative adjacent-version transformation on disposable data, including incompatible predecessor refusal and interruption leaving an explicit pending/closed-admission state. This is focused evidence for the infrastructure, not a complete historical migration or recovery campaign. Production backup/restore, supported historical migrations and disaster-recovery qualification become explicit release obligations before deployments hold non-regenerable or shared operational data. They need an owner and trigger, not implementation in every development fixture.

The initial source assessment identified missing full database-operator coverage in B/D/F. Under the clarified target, the remedy is **not** automatically to port the entire Python operator. Name the native owner and deliver current lifecycle/credential/readiness behavior that the chosen deployment actually consumes; implement the minimal durability/migration infrastructure and explicitly retire unnecessary development recovery machinery. Comprehensive historical data conversion and operational disaster-recovery qualification can follow later; the basic infrastructure itself is not deferred. Canonical SurrealDB/MX remains selected; a separate embedded evidence store is not required.

<a id="I04"></a>
## I04 — Use construction and ownership instead of a fixture permission system

**Amend B03/B04, C04, F02/F04 and H03.** A Rust test needing canonical data should receive a real typed store/fixture capability. A test needing a native worker should receive the corresponding owned capability. Private constructors, lifetimes, owned child handles and borrowed resources can make misuse impossible or local, eliminating duplicated test-name/effect/role tables and many guards.

Nextest groups schedule work; they are not credentials or capabilities. Do not replace `test_requirements.py` with another global Rust registry plus a check that a group name grants canonical effects. Two current demand classes map to the same scientific group, so that translation would lose meaning while keeping the wrong mechanism.

For disposable fixtures, failed scientific assertions need not imply permanent retention of a database, success-gated reclamation or a resumable global terminal-composer protocol. Preserve meaningful diagnostics outside the disposable store and let one native fixture owner settle its owned resources on success or failure. Do not infer scientific success from `Drop`; use the test runner's actual assertions/outcome. Their result and resource cleanup are different observations, but need not be reconciled by a second evidence framework.

Retain runtime enforcement only where uncertainty enters: external actor authentication, externally supplied identifiers, process death, shared-service interference or a persistent consumer requiring retention. Prefer private per-run stores/scopes and direct ownership to cross-run shared fixture machinery. Shared persistent services, if selected, need their genuine borrow/stop contract; that is a reason to isolate their adapter, not impose it on all fixtures.

<a id="I05"></a>
## I05 — Remove per-compiler Python and custom build observation

**Amend B03/B04 and F02/F05.** Use the real pinned sccache executable/native service capabilities for Rust and C/C++ compilation. Select immutable tool/configuration once at explicit setup or build establishment; remove repeated hashing, ledger rewriting, manager queries, polling and group settlement around every compile unit. Do not replace the Python wrapper with the same Rust wrapper by default.

Critically reassess the current rule that cache-service death invalidates an otherwise valid compiler result. Compiler input selection, correct returned output and native compiler containment are meaningful; cache service identity is not automatically scientific correctness. Propose any change to that rule explicitly rather than retain expensive monitoring as a hidden prerequisite.

Pinned sccache0.19 can lazily start a server; `SCCACHE_NO_DAEMON` changes detachment and is not a never-start guarantee. Its client-side mode also has logging/distribution qualifications. Choose a source-backed managed native service/ordinary client configuration and define intended cache lifetime and compiler placement. If a retained requirement truly lacks native support, isolate that exact gap rather than assume a startup check or “100-line shim” solves it. No new application admission daemon or cache tier is justified by this issue.

Retire bespoke `build_observation.py` and its consumers as a deliberate feature removal. Ordinary compiler diagnostics, exits and supplied Cargo/nextest/sccache outputs suffice; do not port live build-stage, wait-lock or freshness telemetry merely because current activity tools consume it. Preserve scientific calculation/progress observation. This is a target change, not a claim that Cargo timings reproduce every removed field or that cached builds are already measured as trivial.

**Closure:** real cache hit/miss and ordinary compiler failure use native tool results; bootstrap works without the old wrapper. Assess service loss and selected logging/distribution modes only against the newly accepted contract. No benchmark is required to justify deleting interpreter hops; numerical speedup remains unmeasured.

<a id="I06"></a>
## I06 — Delegate test execution to nextest; keep only real contract gaps

**Amend F02/F04/F05 and H.** Use nextest's inventory, filters, static groups/overrides, result/JUnit surfaces and selective setup where they fit. Remove per-selected-test TOML projection, repeated inventories and parallel hand-maintained CLI grammars. Use supported metadata APIs rather than embedding nextest's internal runner.

The new overview targets0.9.150; the current pin is0.9.146. Setup/wrappers exist at the pin but remain experimental. Setup occurs after discovery and supplies `NEXTEST_ENV`; it cannot prepare prerequisites needed to compile or list the harness. Setup has no general teardown hook. Prefer a native enclosing fixture owner over an extra supervisor per test. Distinguish setup outcomes from product/scientific testcases in reporting. `NEXTEST_RUN_ID` correlates one run's internal discovery/execution, not arbitrary separate list/run commands.

Use libtest-mimic for correctness case catalogues only where it removes a custom loop. Keep exact-case lookup and execution-free listing; do not introduce repeated whole-catalogue preparation. Statistical measurement remains with Criterion or the scientific measurement owner, not a nextest success count. Preserve doctest coverage separately.

Arrow correctness selection still needs actual target/host feature closure. Reuse one resolution within an invocation initially; a manifest/lock/argv-only persistent cache misses Cargo configuration, environment, target, toolchain and source override inputs. Do not replace one repeated resolver with an incorrectly keyed cache. Pure scopes must continue to avoid unrelated validation roots.

<a id="I07"></a>
## I07 — Delete Python projections without first building a replacement Python product

**Amend A04/A07, F03/F04 and G03/G04.** Retire facade, codec, generated Python document/row and stub machinery after retained native consumers move. Do not make public pyclasses, frozen wrappers for every document, pythonize or Python OTel integration intermediate requirements when Python delivery is deferred.

Keep Rust-owned domain declarations and external contracts that still have consumers. Remove Python output declarations, generated registration, manifests and self-check expectations together. Finiteness, version, unknown-field and physical-context enforcement must exist where real input enters; exposing a Rust type alone does not prove those checks exist. Prefer validated Rust construction and borrowing over rewalking internally established values.

There is a concrete cutover obligation: `python/pse/parity/tests/support.py` imports `pse`, generated types and test fixtures and constructs `pse.Runtime`. Keeping that harness unchanged would retain a required extension. Move candidate execution to native scientific operations and exchange explicit cases/results with the isolated Python reference side. Preserve independent oracle calculations, tolerances and physical interpretation; test a deliberately wrong candidate result. This does not require a public product CLI or continued Python API.

<a id="I08"></a>
## I08 — Replace broad “Wave 0” with explicit deletion/replacement decisions

**Amend T02, F02–F05 and G01–G03.** Several external no-port claims identify useful simplifications but active consumers:

| Candidate | Recommendation and actual responsibility |
| --- | --- |
| Unused generated private/attrs modules | Verify retained consumers, then remove generator emission, manifest and dynamic self-check expectations with outputs. A self-check loading every emitted module is not a product requirement to keep it. |
| `check_generated.py` | Strong deletion candidate; remove remaining presentation/configuration references after confirming generation-check ownership. Do not introduce another checker. |
| `grpc_wire_schema.py` | `xtask/src/codegen/grpc_wire.rs` calls it. Replace that Rust-through-Python parser with `syn`/existing Rust generation, regenerate, then remove it in one cutover. |
| `native-engine-boundaries.py` | Invoked by justfile. Move its retained dependency contract to Rust ownership/closure checking; delete the script and obsolete declarations. |
| `test_requirements.py` | Replace scheduling with nextest and capabilities with construction (I04/I06), not a renamed global guard table. |
| `build_observation.py` | Retire intentionally under I05; no one-to-one replacement is required. |
| `resource_conversion.py` | Still reachable from explicit registry conversion actions. Prefer development reconstruction under I03 and retire converter/fences/actions together; do not silently discard persistent user records. |
| SurrealKit beta tooling/schema copies | Retire disposable experiment paths if no retained migration consumer; include pending-state/operator branches and exposed recipes. A schema differing from today's schema is not by itself proof that a historical migration fixture is invalid. |
| `python/stubs`, `IDAES_NAMES`, unused resource-conversion/probe assets | Remove active configuration/declaration consumers and tests for retired meaning in the same change. Confirm actual reachability; no blind pathname-based deletion. |
| Codex/runtime glue | Retire unneeded local integration or move useful host-specific tooling out of the product closure. No generic replacement agent framework. |

A live consumer identifies where to cut, not a reason to preserve the function. Direct Rust generators, libraries and ordinary tools are the replacement route wherever retained behavior needs one. These recommendations require no Python compatibility phase.

<a id="I09"></a>
## I09 — Keep owner-local typed operations; decline universal dispatch

**Amend T02, A01, F01 and G02/G04.** Retained project request families can stay typed and dispatchable within their domain. Do not unify all native scientific, developer and agent operations into one global catalogue merely to generate optional faces. Retire Python `WorkerFamily`, import eligibility, retained-session and host-envelope projections whose consumers disappear.

Decline the proposed interim RMCP server proxying a slim FastMCP child. Remove Python MCP in the native cutover, accepting feature loss; a reduced RMCP adapter can follow an actual need. Direct Rust applications are real consumers even without a public CLI. Progressive disclosure, guidance and edit previews are not mandatory retained semantics under the operator's chosen priorities.

<a id="I10"></a>
## I10 — Move necessary telemetry; remove unnecessary observation requirements

**Amend B/F and G, with E/A calculation consumers.** Python currently resolves `OTEL_*`/local configuration and emits host/build/assessment observations. Rust `Host::from_environment` consumes serialized settings; it is not an existing complete native environment resolver. Implement the useful resolver below scientific dependencies rather than deleting it on a false duplication premise.

Retain scientific calculation/progress, diagnostics and meaningful persisted execution outcomes. Drop build telemetry and derivative build-specific alerts/queries rather than porting their producers. Review assessment and pressure observations individually for actual operational value; no second SDK, new monitoring stack or per-primitive instrumentation follows from the migration.

Keep optional telemetry independent of scientific admission and local computation. Collector removal/direct backend ingestion is a separate choice because current alerts depend on collector queue/failure signals. Remove those requirements intentionally if unwanted; do not claim signal equivalence. No pyo3-opentelemetry adoption is needed with deferred Python delivery.

<a id="I11"></a>
## I11 — Use a lean native host; decline a mandatory admission daemon

**Amend B01–B05, C04 and F02.** Reuse native process facts, managed unit operations and actual Rust ownership. Treat the configured unbounded developer mode as a reason to simplify ordinary host admission—not to port every slot record or pressure-policy branch. Retain explicit scientific resource demands and any actually selected reference/timing exclusion; do not impose new host caps or serial execution.

A resident admission daemon adds IPC, availability, authentication, restart and uncertain-launch recovery. No evidence establishes its necessity here. Use it only if a named persistent coordination workload demonstrates that it removes more machinery than it adds. OS-manager containment plus a foreground owned operation is the preferred baseline. Independent observers remain usable when admission is unavailable.

Replace Python controller filenames and unit-name conventions with the smallest actual typed artifact/manager capability consumed. Do not add a generalized registry to retain implementation-name drift guards. A small bootstrap closure cannot import the current DataFusion-bearing `pse-testkit` merely to share fixture helpers; place minimal physical primitives below those dependencies or use existing lightweight ones.

<a id="I12"></a>
## I12 — Consolidate compatible mechanics, not all publication semantics

**Amend B01/B04, F03/F04 and E.** Use library filesystem/private staging/atomic publication facilities where needed. A selected-executable descriptor update, immutable payload publication and a scientific transaction commit need different visibility and uncertainty semantics. The current sources demonstrate replacement versus no-replace contracts; a universal “publish generation” protocol would hide those differences.

Conversely, delete generation publication, source hashing and loaded-extension consistency machinery whose only purpose was frozen Python executable code or generated mirrors. “Everything becomes compile time” is too broad: actual external artifacts, input files, remote requests and database records still cross trust boundaries. Establish their validity once at the appropriate boundary and carry it structurally; do not retain repeated checks simply because the old operating layer had them.

<a id="I13"></a>
## I13 — No live generation or mandatory new foreign ABI

**Amend A01, F03 and G04.** Native domain/schema generation belongs to explicit build/development publication when needed. Ordinary execution, imports, reads and help should not regenerate code or reconstruct a registry to validate its own already-built declarations. Use compiler types/macros/Serde and explicit external schema emission rather than a runtime mirror language.

`cdylib` produces a foreign-loadable artifact; it does not generate an API or ownership contract. The current `_native` is a PyO3 extension with CPython assumptions, not the general independently usable Rust library. Keep normal linked Rust APIs as the immediate target. A future binding can adapt coarse native operations; do not add a C ABI, plugin system, second control cdylib or stable-ABI framework without an actual foreign consumer.

<a id="I14"></a>
## I14 — Reframe acceptance around the lean target

**Amend H01–H04 and F04.** Preserve supported scientific outcomes and genuine external interaction contracts, not a requirement to reproduce every old recovery/fixture/build-observation failure mode. Test construction/type restrictions where they now eliminate invalid states, and exercise the remaining external uncertainty at that boundary. Avoid replacing thousands of mocked interpreter calls with thousands of mocked Rust manager calls.

The initial Plan58 native author→prepare→execute→inspect route remains useful. Add a durable-model journey that survives worker replacement and supports a later query over the same persisted lineage. For disposable development fixtures, verify creation, owned cleanup and reconstruction without a historical conversion/recovery framework. Future persistent-deployment acceptance separately verifies supported migration, backup and recovery before that support is claimed.

Existing Python black-box assertions can temporarily challenge a native replacement without remaining in its production path. Do not retain the old implementation as the expected-answer oracle. Keep independent Python science references isolated and explicitly decouple their candidate execution (I07). Scientific failures, setup failures, missing reports and actual resource leaks are different observations; their representation can be simple and native without a universal evidence system.

## External proposal coverage and dispositions

References are the numbered clusters in external §3; evidence A–E2 supplies their source detail. This table covers substantive scope without adopting its line counts or effort estimates as goals.

| Cluster | Disposition in this review |
| --- | --- |
| 1 Generated mirrors | Adapt: retire consumerless projection; no mandatory pyclass replacement (I07/I08/I13). |
| 2 Facades/export/codec | Adopt removal; preserve remaining input semantics structurally, not all drift guards (I07/I12). |
| 3 Product CLIs/supervision | Reuse native owners; CLI optional, no required clap migration (I09/I11). |
| 4 Durable control | Native actor-facing capability; privileged native-system methods are not authority-equivalent (I02/I07). |
| 5 MCP/catalogue | Adopt Python MCP removal; decline universal catalogue, mandatory RMCP and interim proxy (I01/I09). |
| 6 Database operator | Adapt to durable-model infrastructure plus current needed native lifecycle; do not port full development recovery by default (I02/I03/I11). |
| 7 SurrealKit experiment | Conditional retirement with its pending-state consumers; future migration library only where useful (I03/I08). |
| 8 Registries/stores/publication | Replace cross-language ownership with typed construction and current canonical model; no mandatory embedded evidence store or universal publication protocol (I02–I04/I12). |
| 9 Host admission | Lean native manager/capability owner; daemon not selected; remove unnecessary ordinary-mode bookkeeping (I04/I11). |
| 10 Operation observation | Reuse physical primitives where needed; scientific observation retained, build observation retired (I05/I10/I11). |
| 11 Compiler wrapper | Prioritize deletion/native tool delegation; explicitly settle changed cache-lifetime contract (I05). |
| 12 Build/provider preparation | Native existing owners, Cargo freshness and independent bootstrap; avoid both opaque Python receipts and a new umbrella build system (I05/I06/I11). |
| 13 Test orchestration | Nextest delegates runner responsibilities; typed fixtures replace duplicate guard systems (I04/I06). |
| 14 Assessments/measurement | Retain useful native outcomes and independent science; no wholesale port of orchestration or arbitrary measurement frameworks (I06/I14). |
| 15 Telemetry | Native resolver and useful scientific producers; build observation deliberately lost (I10). |
| 16 Docs/ADR/workspace | Rust parsing/generation in actual owner, not another transport loop; parser/library choice needs consumed Markdown/front-matter semantics (I08/I13). |
| 17 Agent/runtime glue | Retire or isolate useful host-specific integration; no mandatory product agent framework (I08/I09). |
| 18 Oracles/consumer edge | Independent Python references retained; facade-bound candidate harness must change; optional NumPy/PyArrow consumers do not block native delivery (I07/I14). |

External Wave0 becomes verified deletion or integrated replacement (I08); Wave1's hot paths are high priority (I05/I06/I11); Wave2 does not require building a deferred Python product (I07); Wave3 becomes coherent ownership cuts, not one monolithic daemon (I01–I04/I11); Wave4's three mandatory faces/proxy are declined (I09); Wave5 delegates native tools without recreating an umbrella runner (I06/I14). A blanket new-Python lint is not recommended: revise ownership/instructions and native dependency checks through the existing decision route, avoiding another checker framework.

## Coherent delivery order and adoption consequences

First settle the retained outcome/guarantee reductions and the durable execution model in T01/T02. Then prioritize native bootstrap/compiler and fixture/host inversions where they unblock all work. Cut the relevant host, service, fixture and configuration consumers together; use direct operations rather than transitional servers. Complete native scientific and retained-data callers, retire mirror/MCP operating machinery, and qualify the selected model-driven target. Optional frontends, new adaptive features and transport expansions must not delay that correction.

Proposed rule changes include removing mandatory Python-boundary/MCP behavior, reducing build-observation and cache supervision promises, replacing fixture permission/retention protocols with construction and disposable ownership, and separating development reconstruction from future persistent deployment qualification. These need precise operator disposition and the existing ADR/design/instruction routes before dependent implementation. This review does not itself amend those contracts or Plan58's tables.

No new scope is imported from superseded Plan57. Existing original numerical limits, source meaning, maintained arithmetic and independent assessment remain relevant baseline contracts; this review is not a solver redesign. New scientific/analysis capabilities separable from the cutover can remain deferred without weakening native completeness.

## Sources and evidence limits

The external review and all evidence sections were read across the review team and checked against decisive current source. The newly supplied nextest and cdylib overviews were read end to end; Context7 was followed by pinned source where release details mattered. Useful primary references:

- [Nextest 0.9.146 setup](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.146/site/src/docs/configuration/setup-scripts.md), [wrappers](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.146/site/src/docs/configuration/wrapper-scripts.md), [groups](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.146/site/src/docs/configuration/test-groups.md), [custom harnesses](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.146/site/src/docs/design/custom-test-harnesses.md).
- [Sccache 0.19 command/startup source](https://github.com/mozilla/sccache/blob/v0.19.0/src/commands.rs), [client-side architecture](https://github.com/mozilla/sccache/blob/v0.19.0/docs/Architecture.md), [configuration source](https://github.com/mozilla/sccache/blob/v0.19.0/src/config.rs).
- [Rust linkage](https://doc.rust-lang.org/reference/linkage.html), [FFI](https://doc.rust-lang.org/nomicon/ffi.html).
- [SurrealDB functions](https://surrealdb.com/docs/surrealql/statements/define/function), [transactions](https://surrealdb.com/docs/surrealql/transactions), current generated study functions and constrained Rust readiness module.

Source inspection covered the affected Plan58 responsibilities, generators and consumers sufficiently to locate the proposed changes; it was not a proof that every apparently unused module is unreachable. Actual manager, cache-failure, new migration and native replacement behavior was not tested. Database performance is a hypothesis to assess where placement decisions require it, not a measured premise. The colleague's population/tool counts and effort estimates are not adoption targets. Its header's appendix path points at a different location; this review links the actual supplied file without editing it.

## Independent assessment and final synthesis

A fresh design-reviewer context assessed the remaining footprint, external proposal coverage and current source. It supported publication after requiring review metadata, explicit delivery of migration infrastructure during cutover (not just future interface design), preparation-dependent claim premises and clarification that dependency edits occur before study activation. Those corrections are incorporated. The assessment found the fourteen amendments and all eighteen external-cluster dispositions coherent for subsequent plan revision; it did not certify implementation, numerical behavior or performance.

Earlier contributor advice to preserve the complete database operator, fixture terminal registry and build observations was reconsidered against the operator's later directions. This synthesis instead recommends essential durable infrastructure now, disposable-development simplification, structural fixture capabilities and deliberate removal of build observation. No researcher conclusion substitutes for the selected functional target. The Plan58 documents and their completion conditions remain unchanged pending adoption of these amendments.
