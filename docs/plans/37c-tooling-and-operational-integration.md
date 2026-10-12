---
title: SurrealKit tooling and operational integration
status: in-progress
date: 2026-10-12
adrs: []
review_sources: [docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md]
doc_owner: docs/plans/37-surrealdb-lifecycle-and-integration.md
doc_topics: [current-work, design, storage, libraries]
doc_retention: while-dependent
---

# Plan 37c — Tooling and operational integration

## Target and reusable foundations

[Plan 37](37-surrealdb-lifecycle-and-integration.md) owns package status, rule decisions, findings and assembled acceptance. This companion develops C0–C3 and I1–I7. Confirmed RC09 selects SurrealKit 1.0.0-beta.6; the separate production SDK/server baseline remains 3.3.0. RC05–RC07 retain patched WS, local-first support and truthful loss/retry. OP01 selects clean target rebuild with regenerated data/evidence, without legacy migration or compatibility readers.

**Implemented, source-inspected:** pse-schema catalog/substrate.rs declares durable schema; pse-codegen codegen/surreal.rs and related generators lower DDL/codecs/digest. CanonicalStore owns explicit initialization/verification; ordinary open refuses unknown schema. scripts/surreal_server.py owns exact service/maintenance identities, independent metadata, connection customization and recovery. scripts/test_resources.py/resource_store.py own resources, borrows, retention, exact removal and settled outcomes. Existing authenticated HTTP MCP uses selected-service VIEWER access.

These foundations remain. Tooling is an adapter under them, not another scientific/schema/deletion authority. Its standalone process avoids silently importing an unpatched `Surreal<Any>` client into production. Library integration remains eligible only if a later concrete comparison justifies deliberate SDK/patch integration.

The [capability investigation](../design_review/evidence/surrealdb-lifecycle-and-integration-2026-10-12/library-capabilities-and-tooling.md) supplies exact-release links and paper corrections. Further authoring research checked the [beta.6 schema preparer](https://github.com/surrealdb/surrealkit/blob/v1.0.0-beta.6/crates/surrealkit/src/schema_state.rs), [runner](https://github.com/surrealdb/surrealkit/blob/v1.0.0-beta.6/crates/surrealkit/src/tester/runner.rs), [actors](https://github.com/surrealdb/surrealkit/blob/v1.0.0-beta.6/crates/surrealkit/src/tester/actors.rs) and [manifest](https://github.com/surrealdb/surrealkit/blob/v1.0.0-beta.6/Cargo.toml). **Interface-checked**, not executed qualification. No tools were installed, stores started or probes run during authoring.

## C0. Beta input and command qualification

Establish the minimal standalone adapter contract before enabling consumers. Declare beta.6 once in repository-owned tooling configuration and acquire its executable through an owned tool location with exact release/source/checksum and locked build resolution. Its manifest declares Rust1.95 and a compatible SDK3.3 range; record actual acquired resolution rather than infer it from the release name. Use the repository's pinned toolchain/environment routes; do not override the checkout toolchain or add an independent production SDK universe.

Codegen supplies a stable managed schema input path/module. Staging exact generated output is a build input, not a handwritten second schema. Split generated definitions from explicit activation/identity DML at the owning generator if needed: the beta parser normally accepts managed DEFINE statements, rejects ordinary non-DEFINE input and refuses managed DEFINE NAMESPACE/DATABASE. Namespace/database ownership stays operational.

Inspect/qualify the complete actual generated surface: functions and nested statements, assertions, enums/types, permissions, bytes/decimal, enforced endpoints, indexes and FOR UPDATE bodies. The preparer can insert OVERWRITE and replace IF NOT EXISTS; test resulting definitions and behavior against direct generated initialization. Neither parse success nor text equality is a semantic oracle. Keep selected-store identity and activation marker writes outside unconstrained sync.

The repository SCHEMA_DIGEST includes interpretation, DDL and codec tokens; beta file/frozen-SQL SHA256 serves a different purpose. Preserve both with their owners. A beta sync success cannot independently activate the canonical interpretation. Ordinary open still refuses an unknown store.

Define one explicit selected endpoint/namespace/auth configuration per adapter invocation and scrub ambient SURREALDB/legacy DATABASE connection variables and credentials. Resolve supported commands separately: beta sync consumes a module/target matrix; seed uses base configuration/folder; tests re-resolve configuration/overrides. Do not forward global target flags and assume equal selection. Reject unsupported multi-target/ambient dispatch before effects. Avoid watch mode on canonical state and shared auto-pruning.

**Bounded qualification:** use authenticated disposable fixtures to compare direct generated installation, beta sync and frozen rollout replay; verify representative functions/permissions/references and exact digest/selection refusal. Exercise wrong target, empty source, renamed managed paths, unsupported statement and credential confusion. Record which beta transformation is admitted. If required generated semantics do not fit, adapt the narrow input/tool boundary and rerun those controls; do not silently use stable0.7, disable meaningful checks or permit broad pruning. A material newly changed contract returns to the coordinator before adoption.

C0's output is a concrete generated-input layout, supported command dispatch table, acquisition/provenance contract and passing qualification requirement. It blocks tool consumers, not unrelated transport/policy repairs. Recheck changed generated definitions after A declarations land.

## C1. Owned declarative database tests

Use beta declarative expectations for schema/functions/permissions and relationship behavior where they replace repetitive fixture assertions. Transport race/cancellation, service recovery and scientific correctness remain under their actual test owners. Tests consume production-generated schema, not a miniature alternative canonical schema.

Select **namespace-auth execution in a fresh namespace per tooling run**. Beta.6 suite_names uses the configured namespace under namespace auth; root auth instead generates independent namespaces. Parity uses the same naming path. Therefore register and provision the enclosing namespace before any beta setup/suite/parity effect, using an authentic selected service identity, owner nonce, credentials and outcome/evidence policy. Do not infer ownership from a name prefix or report contents; parity reports omit namespace/database names.

Give the beta process only namespace-scoped credentials and the exact configured namespace; use keep-db and external checked cleanup. Restrict actor/suite inputs to that scope and refuse injected root credentials, unrelated tokens or alternate external targets. The default bootstrap actor named root respects namespace auth; explicit ActorKind::Root and ActorKind::Headers instead attempt root signin in beta.6. Reject those explicit modes before setup, unless C0 qualifies a narrow adapter that preserves namespace-only authority. Qualify every admitted actor mode rather than assuming all variants support namespace credentials. Namespace auth bounds effects even when generated child names are private. Test explicit actor overrides and malformed setup, not only default suites.

Extend the existing fixture resource owner with an authenticated enclosing namespace kind and recoverable exact child inventory, rather than build a second fixture manager. Current remove_database handles individual DBs and is not already a namespace owner. Parent registration records whether creation occurred and retains uncertainty. Descendant/borrower/evidence checks govern cleanup. After proven terminal outcome/drain, enumerate owned children and perform checked logical removal, then checked namespace/account cleanup. Missing or ambiguous acknowledgment stays protected; do not mark capacity released because a helper returned success.

Beta cleanup currently discards statement checks and warns on errors; keep-db avoids treating that best-effort path as disposal authority. Interrupted preparation and parity must fall within pre-registered ownership. Preserve selected failure/unknown fixtures and reports according to the existing outcome policy. OP01 permits regenerating old evidence; it does not make a newly failed/unknown run an automatic cleanup success. If tool interruption leaves server effects unproved, retain the namespace for exact reconciliation or an authorized whole-owner drain/rebuild; CLI process exit alone is not server settlement.

Use schema_from=both for useful fresh sync/replay contract tests: suite override precedes CLI then defaults, Both creates separate jobs, and parity runs when replay jobs exist, sync is enabled and parity is enabled. Replay refuses an absent chain: C1's replay/parity closure therefore depends on C2's initial generated frozen chain, while sync-only fixtures can proceed after C0 and namespace ownership. Generating that chain is separate from qualifying C2's persistent maintenance execution. Definition/catalog agreement does not prove data behavior, scientific meaning or removal. The beta sync test builder uses the default module; arrange generated test input accordingly rather than assuming its full CLI module matrix applies.

**Proposed controls:** registration refusal prevents all creation; exact selected namespace contains normal/parity/setup-failure children; namespace credentials cannot affect another fixture; report omission does not hide ownership; successful test with refused removal remains charged; failed/unknown execution retains evidence; authenticated drained success disposes exactly its scope; crash/restart discovers registered residue. Exercise direct and replay installations with meaningful permission readback.

## C2. Frozen forward rollouts and exclusive maintenance

Create the initial baseline and forward chain from fresh target declarations, not an imported historical migration catalog. Codegen supplies managed schema; required data steps belong to the explicit operation owner. Use beta's frozen changed SQL beside manifests with SHA256, ordered catch-up and fork/predecessor/hash checks. Commit/ship frozen files through their producer; they are generated outputs with one source, not another editable schema. Existing frozen steps remain immutable; changed declarations generate new forward steps. The initial chain supplies C1's replay/parity prerequisite without requiring a live persistent rollout first.

Retain explicit activation. A rollout can establish compatible definitions/data but cannot independently publish the repository interpretation digest or change issuing-store identity. Verify schema, codec interpretation, target scientific obligations and operation settlement before activation. No legacy data conversion/backfill is in scope. Forward data-preserving tests use fresh synthetic target records and changes compatible with the target's current interpretation. Incompatible future changes use an explicit fresh reset unless a separate product decision authorizes migration; this package does not require historical readers or multiversion compatibility.

Use existing require_maintenance_owner, watch_maintenance_owner, maintenance_endpoint, drain_maintenance_listener and recover_maintenance. Closed application admission, B1 dependency quiescence, an exact live owner and exclusive listener supply write authority, excluding other tooling writers as well. Never use beta LockKeepAlive as the actual fence: lost renewal only warns/returns and renewal errors warn. Its TTL coordinates cooperating tool users, not every canonical writer.

Run the CLI as an owned child whose lifetime is tied to maintenance ownership. Owner loss stops further tool effects, closes/drains the maintenance listener and leaves admission closed until exact recovery. Timeout/child exit does not prove submitted SQL failed. Preserve exact rollout phase/step/error and settle uncertain work before reopening; do not blindly replay a data step. Use beta's progress/repair where it supplies the contract and retain only the operational authority it cannot supply.

**Proposed controls:** fresh baseline, two ordered changes and catch-up; missing/falsified frozen file; fork/out-of-order refusal; failed completion/repair; interruption after submitted SQL; maintenance owner loss while beta heartbeat still appears healthy; competing tool/application writer refusal; activation only after admitted definitions/data/interpretation. Test expected data independently of the tool's catalog parity. Remove private bookkeeping only where the qualified beta route replaces it.

## C3. Clean rebuild and operational cutover

Implement the authorized fresh-target reset/reseed route. Identify the selected store and authentic owner, close admission, drain owned activities/workers/transports, and refuse while unrelated borrowers still exist. Stop/drain the owned service before filesystem reset where required. Reset only explicitly selected owned data/metadata; a large/old path, matching name or absent process alone is not deletion authority.

Create fresh deployment/store identities and credentials; old capabilities cannot attach to the regenerated store. Install final generated contracts and seed from authoritative definitions/fixtures, then explicitly activate and reopen only after validation. Rebuild actual managed workers and Python extension against the target. Regenerate scientific/evidence fixtures used by Q1. Do not translate old IDs, retain old readers or create compatibility aliases.

Remove replaced historical schema interpreters, historical-read codecs, backfill/migration adapters and tests/fixtures as their target callers are replaced and targeted controls pass. Preserve only mechanisms actually consumed by the new forward rollout and normal target contracts. The source review and existing plan history remain unchanged during execution; H1 later reconciles affected owners after acceptance.

**Proposed controls:** reset refusal for wrong identity/live unrelated borrower; interrupted reset leaves admission safely closed; fresh setup opens only the exact generated target; old token/store handle refuses; rebuilt worker and installed Python boundary use fresh artifacts; representative science can be regenerated and read. No preservation snapshot/old-data migration gate is required by OP01. User-supplied source documents and another owner's workspace are outside store reset.

## Bounded capability investigations

The following are required decision packages, not a commitment to adopt every capability. Each records the actual consumer, source/version evidence, simplest alternative, selected outcome, dependent implementation and observable revisit trigger. Source inspection may settle no adoption. Probes are discretionary and fixture-owned; quantitative claims require representative measurement. Coordinator package status is authoritative.

| Packet | Question, evidence and bounded outcome | Adoption or revisit condition |
|---|---|---|
| I1 | Map concrete A0 relationships to typed REFERENCE/REJECT/custom actions versus current exact guards. Check string/bytes invisibility, permissions, cascade direction and both creation/deletion orders on the resolved backend. | Adopt only structural meaning that removes redundant machinery without losing policy/provenance/borrower settlement; never substitute references for scientific sufficiency or time-sensitive authority. |
| I2 | Inventory actual repeated polling and repeat-safe follow-up effects. Compare current calls with LIVE/changefeeds/ASYNC, finite attempts, maintenance dependence, cursor gaps/resync and reconnect subscription loss. | A concrete consumer and bounded retry/resync owner must justify integration. Otherwise keep explicit operations; notification is a hint, not canonical truth. Do not transfer the 3.3.1 retry-budget change to3.3.0. |
| I3 | Find a real large protected read and compare current native pages/blocks with gRPC incremental rows and dedicated WS/HTTP/embedded alternatives. Include value fidelity, provisional StatementEnd finality, buffering, original clocks, unknown writes and physical drain. | Retain current patched WS for this plan unless a concrete replacement is separately confirmed. No measured memory/speed claim from streaming API presence; no direct multiprocess RocksDB substitute. |
| I4 | Assess future forensic history, payload storage and actual reclaim decisions: VERSION currently disabled, bytes already integrated, buckets/files experimental and cross-store, REMOVE logical versus deferred key/file reclaim, compaction after deletion. | Keep bytes/explicit retention as comparator. Add history/files/compaction only for a defined consumer and complete backup/publication/reclaim contract; measure physical outcomes only if capacity decisions need them. No TTL or routine per-run compaction mandate. |
| I5 | Inspect actual candidate-selection/retention/neighborhood query shapes and indexes/server functions. Use INFO/EXPLAIN; ANALYZE executes and must use reads/owned fixtures. Compare LIGHTWEIGHT recordless edges and INLINE write/overflow costs with attributed existing relationships. | Adopt fitting access paths that remove meaningful work. Lightweight edges cannot carry receipt properties/events/indexes; inline caches do not optimize arbitrary string joins. No blanket graph rewrite or stable JSON interchange promise. |
| I6 | Compare exact released-server/network needs with owned native fixtures and testcontainers; assess selected-service HTTP MCP/domain operations, optional Studio/metrics/OTel and maintenance wakeups. | Override container image/backend/caps/readiness if adopted; default v2.2 memory/root is not3.3 RocksDB qualification. Keep authenticated VIEWER inspection. Domain writes invoke canonical owners. Add diagnostics/timer only for a consumer; no mandatory collector or scheduler, and a timer never authorizes deletion. |
| I7 | Identify workload-specific restart sufficiency and remote/other extension boundaries. Distinguish seed, partial trajectory, backup and VERSION from numerical resume. Assess search/vector/GQL/Postgres wire/WASM/SurrealML/hosted/cloud leads only for a simulator consumer. | Record truthful retry/local-first limits under RC06/RC07. Durable resume, remote binding and new query/runtime features need a later concrete product/contract decision; eligible libraries are not rejected for license or current mechanism alone. |

The [scientific decisions](37-investigation-evidence/scientific-closure-and-replay.md) and [operational decisions](37-investigation-evidence/operational-capabilities.md) now supply the source-led outcomes, concrete consumers, alternatives and observable revisit triggers. I2 retains explicit durable readiness and bounded polling despite a real LIVE wakeup consumer. I3 retains patched WS and existing protected pages/blocks. I6 retains native owned fixtures and authenticated VIEWER inspection. I1/I4/I7 retain explicit conflict guards, owned bytes/history-disabled and truthful local retry. I5's actual access-path inventory governs selective fixture checks and reinspection when A declarations change; no blanket graph/index rewrite is selected.

Some questions can conclude during source investigation; others require new target fixtures after A/B/C foundations. Capture unresolved selection premises as gates on their dependent work, not a reason to hold independent corrections. Any adopted additional contract changes D0 and the affected plan before implementation. Deferred decisions follow the existing register route with owner/check/observable trigger/next review; no competing backlog is introduced.

## Coverage limits, acceptance and handoff

Both papers' corrections remain relevant: features already present are not new gaps; versioned history is not TTL; logical disposal is not returned space; at-least-once attempts are not eventual successful effects; streaming rows can be provisional; references cannot see opaque dependencies; beta selection flags do not guarantee common dispatch. These qualifications constrain every adopted package.

The review did not qualify full numerical models, derivative/scaling/closure chains, power-loss recovery, remote deployment, all queries or representative performance. Q1 qualifies affected target consumers and operational failure journeys, not that entire breadth. Capture wider unexamined scope with the precise owner/trigger or support limit. Do not turn absence of a probe into a product finding or promise.

During implementation, run targeted tooling/unit/fixture controls with authentic ownership and relevant scope-aware environment routes. At overall functional scope end Q1 integrates scientific/transport/tooling behavior and runs the necessary static/manual checks once. Report actual outcomes against zero failures; never equate a beta test report, exited CLI, empty cgroup or successful REMOVE with all broader drain/settlement/physical recovery claims.

Transfer new enduring instructions to the substrate/agent-environment/architecture owners through D0/H1. Existing execution plans remain unedited until Plan37 is implemented and accepted, then receive only the reconciliation needed before returning to their remaining work.

## Initial qualified adapter evidence

The [beta boundary evidence](37-investigation-evidence/beta-tooling-qualification.md)
records the actual exact-release acquisition, admitted generated/fixture/setup
surface, complete-schema comparison, namespace isolation/disposal, independent
review corrections and explicit permission/qualification limits. Initial CLI
support is sync, initial plan, frozen replay and sync/both tests; seed/custom SQL
and extra actor modes remain unsupported until qualified. The coordinator owns
package completion and the remaining persistent maintenance boundary.

The separate maintenance foundation uses the existing exclusive service owner
and durable unknown-operation fence. Its disposable reservation-revocation
control observes an actual running beta step and live lock, then proves exact
child/listener drain, ordinary admission refusal and preservation of unknown SQL
through status and a supported repair refusal. It does not qualify process death,
successful interrupted-step settlement or persistent deployment rollout. See the
same evidence owner for the actual failure, correction and qualification limits.
