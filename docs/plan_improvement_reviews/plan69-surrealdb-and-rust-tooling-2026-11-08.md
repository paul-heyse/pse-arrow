---
title: "Plan69: SurrealDB and Rust tooling leverage"
status: proposed
date: 2026-11-08
doc_role: review
doc_owner: docs/plans/69-intent-first-cli-and-codex-workflows.md
doc_topics: [current-work, design, libraries, storage, validation]
---

# Plan 69 SurrealDB and Rust tooling leverage

## Judgment and baseline

**Proposed.** Close the schema rollout authority gap by composing the existing maintenance
owner with a frozen, versioned SurrealKit execution plan. Use the existing v4 fixture and
module preparation routes for live product evidence. Improve source-change preview by replacing
textual anchors with structural ones and carrying native SurrealQL diagnostics through the
existing envelope. Keep ordinary test selection compile-free until exact Nextest inventory
or execution is requested. The corpus also supports focused query diagnostics and bounded
source browsing where those capabilities replace a recurring manual handoff.

Inspected main `e7bcacac8`, [Plan 69 and 69a–69g](../plans/69-intent-first-cli-and-codex-workflows.md#companion-plans-and-dependencies),
and the 16 working-copy guides identified in the [coverage appendix](#coverage-appendix).
Applied core 3.4, efficiency heuristics, the test-assessment companion, the pse-arrow binding,
and process-simulator profile 1.5. The selected transport, operation, and evidence contracts
come from blueprint §21.7 and §§23–24. Plan 66 owns scientific meaning and qualification,
Plan 67 supplies artifact custody and fixtures, and Plan 68 supplies v4 schema, transport,
query, and deployment contracts. The coordinator retains the O01–O08 dispositions from
the functional review and I01–I08 from the prior CLI review. Recommendation IDs below
are scoped to this document. [Prior CLI review](plan69-cli-library-leverage-2026-11-07.md) ·
[Design standard](../design_review/design_principles/standard.toml) ·
[Native host](../authoritative_design/sections/native-host-and-engineering-tools.md#section-21-7) ·
[Surreal substrate](../dev/surreal-substrate.md)

**Interface-checked.** The current checkout selects SurrealDB 4.0.0, SurrealKit
1.0.0-rc.1 at `625e264c4e134def842bfe8ac9d2f918a42bb6c3`, Cargo metadata 0.23.1,
nightly-2026-09-29, native semantic RA 0.0.358 and standalone syntax 0.0.360.
The structural stack uses ast-grep 0.51.0 and tree-sitter 0.27.0 with Rust grammar
0.24.2. The generated Rust stack uses syn 3.0.5, quote 1.0.47 and prettyplease
0.3.0. The syn guide's 3.0.9 addendum is capability discovery. The installed
3.0.5 implementation controls a recommendation. Rustdoc JSON uses format 61
and rustdoc-types 0.61.0. The official Surreal launcher, optional components,
SDK examples, and live deployment can each select different versions or features.
[Workspace declarations](../../Cargo.toml) · [SurrealKit pin](../../.config/surrealkit.toml) ·
[Toolchain pin](../../rust-toolchain.toml)

<a id="I01"></a>

## I01 — Delegate rollout execution and retain authenticated activation

**Missing accepted obligation.** E03 has a passive Viewer catalog observer and an activation
consumer. The live path still needs the production owner of the maintenance listener and
exact pending and settled nonce. The consumer checks the original physical instance, namespace,
database, predecessor and input hash, compares an observed generated-target witness and
module digest, and reconciles an existing activation receipt by nonce. The issued intent,
settled database work, and activated product marker are distinct states. Resolve the
existing E03 beta.6 reference against the rc.1 pin when revising the plan.
[E03](../plans/69e-source-model-and-live-schema-operations.md#e03--inspect-preview-apply-and-recover-live-schema) ·
[Activation consumer](../../crates/pse-operations/src/canonical_maintenance.rs) ·
[Plan 68d receiving](../plans/68d-surrealdb-v4-developer-and-test-receiving.md)

**Proposed.** Keep the passive preview under the existing authenticated read boundary. Attach
the exact frozen manifest, SQL identity and prepared tool to the PSE preview, with
effective template bindings fixed or ambient substitution disabled. The native owner
binds predecessor, target interpretation and module. Use `rollout lint` for manifest/chain
validation. At apply, the maintenance owner authenticates and rechecks the premises,
coordinates admission and drain, and executes the selected `start` and `complete` phases.
Use authored assertions and `await_migrations` for actual named v4 backfills. Use installed
catalog readback, the existing module witness, and an index readiness check where the change
builds an index. Publish only compact operation, manifest, phase, and witness references.
Compare the existing pinned subprocess route first with the Rust `Rollout` API. The latter
would add an SDK/feature closure and needs a concrete contract advantage. A generic SQL
wrapper would have to recreate the same migration sequencing and recovery semantics.
[Pinned rollout implementation](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/rollout.rs) ·
[Pinned Rust integration](https://github.com/surrealdb/surrealkit/tree/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src) ·
[Existing tool owner](../../crates/pse-host/src/surrealkit.rs)

**Interface-checked.** rc.1 `sync --dry-run` runs setup before its dry-run branch.
`rollout plan --dry-run` can create local state directories and reports a proposal from
local inputs. Keep both outside passive live preview. More importantly, `repair_inner`
can advance `running_complete` and reconstruct the managed catalog without checking
every completion step. The journal's running and completed writes surround separate
SQL execution. After an interrupted reply, inspect the original rollout and
independently establish required phase effects, including applicable data assertions and
background completion. Read the relevant persisted status through the owner since rc.1
`rollout status` and `Rollouts::pending` also run setup, and `rollout up --dry-run`
acquires a lock. Where effect evidence is inconclusive, retain the original
unresolved operation and its recovery reference. A catalog match alone can miss an
unfinished data step. Rehearse a stale predecessor or module, interruption before a
required step, and interruption after effect before receipt. Verify zero writes for
stale refusal and one original activation across recovery in both human and machine output.
DP-03, DP-04, DP-14, DP-19, H18 and H24 motivate the authority and recovery split.
[Pinned sync implementation](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/sync.rs) ·
[Pinned rollout recovery](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/rollout.rs)

<a id="I02"></a>

## I02 — Reuse v4 preparation and distinguish the receiving checks

**Missing accepted evidence.** The previous fixture inspection refused a v3-shaped Study
module receipt. B02 and D01–D03 still need the assembled v4 journeys. The current
`scripts/pse fixture prepare study-module` keys its immutable artifact by exact source,
selected server, tools and build inputs and reuses a matching receipt before building.
Use that route and check the selected extension digest, capabilities, and live `ext::`
readiness. The v4 CLI's version and readiness probes can diagnose version and listener
availability. Product issuer, extension identity and native settlement come from the PSE
owners. Compare reusing a current matching receipt with a single build through the same owner.
[69b](../plans/69b-codex-workstation-and-preparation.md) · [69d](../plans/69d-routine-product-and-data-operations.md) ·
[Preparation](../../crates/pse-host/src/study_module.rs) ·
[Installed extension checks](../../crates/pse-operations/src/canonical_study_module.rs) ·
[SurrealDB CLI readiness](https://surrealdb.com/docs/reference/cli/db/commands/isready)

**Proposed.** Use one small owned fixture for each distinct claim. D01–D03 can share the
prepared module and cover target ambiguity/issuer refusal, original write acknowledgement,
protected reopen, and Study publication with narrowly selected native checks. B03 still needs
the early invalid-socket refusal, bounded owner diagnostics and actual drain. SurrealKit
`test --exec` can provision a logical database and pass its identity and scoped credentials
to a child for database contract checks. Use the current native fixture when the assertion
depends on the captured product process, canonical initializer or original product receipts.
Add an external test adapter only if several concrete tests can consume the scoped setup
without duplicating the native process owner. The pinned external command is split on
whitespace, inherits most environment variables, and on interruption kills its direct
child. Keep supervision and descendant drain with the existing owner.

For E02, separately establish generated source publication, generated SurrealQL validity,
and old-row behavior. A focused rollout can insert a project under the predecessor, apply the
new optional field, read the old row as absent, and round-trip an explicit description.
The runner's normal `--schema-from both` builds sync and replay separately and seeds after
schema construction. It supplies parity evidence and needs an explicit predecessor row
inside the rollout or an ordered fixture for the historical-row claim. Check `NONE` versus
`NULL` against the generated field's actual type. Add a named row migration only if the
owner requires stored-value transformation. Preserve the guarded E02 tracked-file apply
journey as its own check. Retain compact case outcomes and release the fixture by its owner.
AP-06, DP-23 and H20 favor these distinct assertions over repeating complete journeys.
[Pinned tester](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/tester/runner.rs) ·
[Optional-field fixture](../../xtask/src/codegen/source_change/fixtures/optional-project-description.patch) ·
[v4 optional and migration semantics](../libraries/surrealDB/SurrealDB_v4.md)

<a id="I03"></a>

## I03 — Carry structural anchors and original diagnostics through source preview

**Interface-checked.** E02 `Case::anchor` and `locate` use first-substring searches for
declaration boundaries and the field. A lookalike in a comment, nested expression, or another
declaration can produce a credible but incorrect byte range. Separately, `surrealql-validate`
already parses with surrealdb-parser-next 4.0.0 and joins native diagnostic messages and
offsets into one string. Neither issue requires a new language service.
[Source preview](../../xtask/src/codegen/source_change.rs) ·
[Current SurrealQL validator](../../xtask/src/surrealql_tools.rs)

**Proposed.** Bind each supported source case to a unique structural location before
preparation. In xtask, use the existing pinned tree-sitter or syn input to locate the
`canonical_projects` registry tuple and its direct column expression, and the
`PackResultSelection` declaration and direct field in the bounded
`pse_contract::projection_struct!` token tree. Ordinary Rust field ASTs do not
describe the latter DSL. A byte-native CST avoids offset reconstruction. If syn parses
the relevant grammar, account for any stripped BOM or shebang before interpreting spans.
Refuse missing, ambiguous, or malformed candidates. Keep semantic rename and SSR
for cross-file identity and references, feeding the existing proposal postimage handoff.
Keep syn/quote/prettyplease in generated output, where syntax construction and deliberate
reformatting are useful. The existing guard still owns revision, digest, publication and
recovery. Check comment/string lookalikes, duplicate declarations, and an independently
known UTF-8 byte range. Use the actual registry and macro forms in the fixtures.
Carry a bounded projection of the existing affected report alongside package names.
Its direct owners, widening reasons and generator route explain why a selected check
runs and keep conservative reach distinct from known consumers.
[Pinned syntax API](https://docs.rs/ra_ap_syntax/0.0.360/ra_ap_syntax/) ·
[Pinned ast-grep matcher](https://docs.rs/ast-grep-core/0.51.0/ast_grep_core/trait.Matcher.html) ·
[Pinned syn parser](https://docs.rs/syn/3.0.5/syn/fn.parse_file.html)

Use the existing configuration-bound rustdoc API comparison only when a supported
change raises a distinct public Rust surface question. Its format-61 decoder and
older-format public-API adapter have separate checks. Retain the typed comparison
and its missing-item accounting. The rename's JSON name, Protobuf tag, presence,
historical decode and behavior belong to the existing generated contracts and
receiving checks. [API capture owner](../../crates/pse-host/src/api.rs) ·
[Structural comparison](../../crates/pse-host/src/api/comparison/structural.rs) ·
[Pinned rustdoc schema](https://docs.rs/rustdoc-types/0.61.0/rustdoc_types/)

For the changed SurrealQL postimages that a preview needs to explain, preserve native
`Diag.kind`, primary range and label, and optional `extra` range and label in the A03
projection. Carry a bounded staged path and digest and a separate related finding for
another file. Check a UTF-8 offset and a secondary delimiter label. Distinguish unreadable
input from syntax failure and restrict excerpts under the existing redaction policy.
Use the existing parser before adding a subprocess or LSP dependency.
`surreal db validate --stdin` is a useful independently pinned parse probe. SurrealQL
LSP analysis is a separate candidate if a named semantic question survives the parser
and installed checks. Its 0.6.3 headless JSON check uses UTF-16 ranges and a different
builtins revision. SurrealKit offline `compile` applies definitions to an embedded
engine and emits schema document v2. The prepared remote-only tool build omits the
embedded memory backend. Revisit a separately prepared candidate only for an unresolved
normalization or compatibility question. Keep schema-document and generated-target hashes
in their own namespaces. DP-01, DP-15, DP-21 and PS-13 apply to these evidence boundaries.
[Parser diagnostic source](https://github.com/surrealdb/surrealdb/blob/284fe93afc851c3baec2c262c01289602021a70d/surrealdb/parser-next/src/parse/mod.rs) ·
[LSP 0.6.3 check contract](https://github.com/surrealdb/surrealql-language-server/blob/5aa6a12a29b7def0e7df662c09ac9203e66f5033/AGENTS.md) ·
[CLI validation](https://surrealdb.com/docs/reference/cli/db/commands/validate) ·
[Pinned SurrealKit compile](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/main.rs) ·
[A03](../plans/69a-cli-contracts-configuration-and-presentation.md#a03--original-outcomes-and-safe-recovery)

<a id="I04"></a>

## I04 — Explain candidate, admission, and execution from their existing owners

**Implemented.** C01 already uses Cargo declarations for target eligibility, the pinned
unit graph for actual host/target and feature-bearing planned units, and a conservative
changed-source view for wider reach. The semantic analyzer supplies guarded likely tests.
Nextest owns the executable inventory, exact filters, reruns and verdicts. The capture
owner already selects only needed images and ties release to drain. A package graph,
source test browse, and test inventory answer different questions.
[C01–C02 and capture evidence](../plans/69c-ordinary-testing-and-managed-results.md#analyzer-and-capture-boundaries) ·
[Affected graph](../../crates/pse-host/src/affected.rs) ·
[Cargo unit graph](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#unit-graph) ·
[Nextest listing](https://nexte.st/docs/machine-readable/list/)

**Proposed.** Make the existing selection and catalog projections say which claim is
available and why. Distinguish affected packages, a source-derived candidate, an admitted
target, an enumerated test and a verdict. Give doc-only candidates the existing
`just doctest` route, with applicability and owner clear. Cargo's `Target.doctest` is
eligibility metadata. A rust-analyzer runnable or acquired rustdoc Format 2 example can
help locate source, while execution belongs to rustdoc. Check a doctest-only target
without changing the current Cargo-equivalent root predicate. Preserve conservative widening
on ambiguity or incomplete resolution. An optional dependency absent in a single
resolution and an integration helper outside a source match are revealing controls.
Avoid a general test browse feature until there is a named consumer.

Keep the existing traversal for C01. Guppy `PackageGraph` and `CargoSet` are credible
for richer bounded package/feature explanations when an identified consumer can
replace traversal while preserving optional, host, build and dev edges. Its simulated
activation needs comparison with actual Cargo. Cargo tree is useful as an explanation
route. Hakari would introduce a second feature-unification mechanism alongside the
current pinned Cargo workspace policy. Retain native unification. Guppy summaries,
determinator, timings, and persistent build analysis require an actual comparison or
performance question before retaining artifacts. Preserve Cargo event framing and
original child completion. `Message::parse_stream` has permissive text fallback and
should not silently become the strict observation parser. AP-04, DP-15, H26 and PS-10
support the distinctions and low-cost defaults.
[Cargo metadata target API](https://docs.rs/cargo_metadata/0.23.1/cargo_metadata/struct.Target.html) ·
[Pinned message parser](https://docs.rs/cargo_metadata/0.23.1/src/cargo_metadata/messages.rs.html) ·
[Dependency tooling](../libraries/rust_analyzers/rust_dependency_tooling.md) ·
[Qualification lanes](../dev/validation-assessment.md)

<a id="I05"></a>

## I05 — Make database access-path diagnosis an owner-backed read

**Proposed, conditional enhancement.** If a routine bounded project or Study read needs
access-path explanation, give the existing typed graph neighborhood an authenticated
diagnostic route under D01/D03. Reuse Plan 68c's anchor recheck, bounds, continuation,
typed explanation and index observations. Separate static
`EXPLAIN FORMAT JSON`, index construction status from `INFO FOR INDEX`, and explicit
`EXPLAIN ANALYZE` in a controlled scope. Preserve issuer, selected database, request
shape, row cap, and current readiness. Compare against the existing xtask diagnostic
route and move its routine consumer only when the new route removes state-path or
request-file staging. Require a foreign-issuer refusal, a bounded result, and a
constructing-index example. `INFO FOR INDEX` requires database actor View, so qualify
the exact scoped role and preserve its refusal. Keep the stable CLI projection small
because raw explain output can change. Plan 68c owns the query shape, plan interpretation and
performance claims. AP-01, DP-18 and H26 favor a passive bounded diagnosis as the
default. [Plan 68c](../plans/68c-surrealdb-v4-query-improvements.md) ·
[Typed explanation owner](../../crates/pse-operations/src/canonical_explanations.rs) ·
[EXPLAIN contract](https://surrealdb.com/docs/reference/query-language/statements/explain) ·
[Pinned index authority](https://github.com/surrealdb/surrealdb/blob/284fe93afc851c3baec2c262c01289602021a70d/surrealdb/core/src/exec/vm/ir_lower/info.rs)

The current bounded project read and publication owners remain the baseline. Native
SDK streams, subscriptions and CDC merit investigation only when a named consumer
needs a continuous or larger projection and the chosen transport can preserve cursors,
partial results, cancellation, transaction completion and drain. A04 transfer progress
can use actual owner writer events if available. SDK arrival, accepted bytes, durable
publication and authoritative settlement remain distinct. Standard SQL imports, exports
and replication are potentially useful maintenance capabilities with different custody
and authority from issuer-bound project packs. The v4 HTTP import handler can commit
successful statements independently and report partial application. DP-04, DP-19,
H21 and PS-12 constrain any
such addition. [69d exchange](../plans/69d-routine-product-and-data-operations.md) ·
[69g presentation](../plans/69g-output-policy-and-owner-driven-progress.md) ·
[v4 transport and limits](../plans/68b-surrealdb-v4-schema-transport-and-authority.md) ·
[Pinned import handler](https://github.com/surrealdb/surrealdb/blob/284fe93afc851c3baec2c262c01289602021a70d/surrealdb/server/src/ntw/import.rs)

For C03–C04, indexed rows or incrementally maintained summaries could serve a named
cross-invocation question if the existing bounded history no longer suffices. Retain
the existing expiry owner under current use. Changefeed retention governs mutation
replay and durable-session TTL governs session expiry. Developer result lifetime
depends on owner settlement, holds and exact disposal. A future database projection
needs a compact schema, a named consumer, a retention policy and a failure boundary
that keeps original custody with the issuing owner. DP-05, H16 and H27 support
preserving the existing C04 disposition. [C04](../plans/69c-ordinary-testing-and-managed-results.md#c04--retention-and-authenticated-release) ·
[Plan 68d history decision](../plans/68d-surrealdb-v4-developer-and-test-receiving.md#d04--preserve-the-history-disposition)

<a id="I06"></a>

## I06 — Reuse immutable source coordinates in bounded structural browsing

**Interface-checked.** Structural browsing already filters captures, bounds retention,
handles cancellation and detects Tree-sitter match-limit overflow. Its locator computes
line starts from the entire original text for each emitted match or capture. Compute one
coordinate index per queried source snapshot and pass it through the existing locator
boundary. This removes repeated work without changing the source identity, retained
matches, or edit authority. Compare one UTF-8 source with multiple captures and a
multi-line edit range against the old coordinates. DP-09 and H26 support the local reuse.
No performance magnitude follows from this source inspection.

Keep native range pushdown conditional. Tree-sitter 0.27.0's
`set_containing_byte_range` requires all matched nodes to lie inside its range. The
current host checks captured nodes. An uncaptured parent with an in-range capture can
distinguish the behaviors. If a later consumer needs a narrower native query, preserve
captured-node containment, recovery, completeness and limit behavior in the comparison.
Ast-grep's traversal and filtering primitives are useful for bounded candidate
discovery, with change application and stale checks remaining in the preview owner.
[Structural query implementation](../../crates/pse-host/src/source_query/native.rs) ·
[Source coordinates](../../crates/pse-host/src/source_query.rs) ·
[Pinned cursor contract](https://docs.rs/tree-sitter/0.27.0/tree_sitter/struct.QueryCursor.html#method.set_containing_byte_range)

<a id="I07"></a>

## I07 — Give selected SurrealKit preparation an exact recovery route

**Interface-checked.** The existing verifier reports the same install command after
a missing or invalid release. The install command uses its default generation while
verification keeps using an explicit `PSE_SURREALKIT_BIN` override. Install also
verifies an existing same-generation directory and returns its failure. A bad override
or a corrupt existing generation can therefore lead to repeated ineffective preparation.
[Current preparation owner](../../crates/pse-host/src/surrealkit.rs)

**Proposed.** If E03 selects this tool, receive its exact prerequisite under B02 and
A03. Distinguish absent default release, rejected override, and invalid existing
generation. Return explicit preparation for the absent default, selection correction
for the override, and a protected-owner repair or an unresolved state for corrupt
bytes. Preserve previously usable selections and qualify repeated verify after each
recoverable case. Retain immutable receipt checks for source, lock and executable.
The official launcher and `--version --json` can help diagnose a selected executable
but carry less provenance. Compare with keeping the verifier as a specialist route
if E03 selects another prepared mechanism. DP-04, DP-21 and H24 motivate the distinction.
[Pinned version output](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/output.rs) ·
[Plan 69b preparation](../plans/69b-codex-workstation-and-preparation.md)

## Ordering and evidence limits

I01 and I02 address existing completion obligations. Exact fixture preparation can
unblock the product journeys independently of E03's issuer and settler. Settle the
required E03 recovery postconditions before exposing a CLI apply route. I03 can improve
E02's receiving evidence without blocking either live owner. I04 clarifies an already
implemented lane. I05 requires the named product diagnostic use and I06 can be evaluated
as a local reuse improvement. Apply I07 when the rollout path selects the existing
tool. Completed F01/F02 and parser-owned catalog remain useful
constraints for every added command, and A02–A04 govern selection, diagnostics, and output.

**Tested.** On current main, `direnv exec . scripts/pse inspect test-selection pse-host --lib
--admitted --format json` completed with zero failures against a zero-failure baseline.
It returned the admitted library target and reported the exact test population unavailable
because explicit build/list had not been requested. Passive help for `inspect test-selection`
and `source-change preview` also exited zero and exposed the current handoffs. No test
binaries, module preparation, rollout, or live qualification were needed for these
observations. Existing Plan 69 records supply the wider admission and fixture evidence
under their original conditions. The new proposals are unmeasured and require their
listed receiving controls.

## Coverage appendix

Coverage includes the original guides and addenda. Section ranges below group related
capabilities and retain their disposition. Git blob identities identify the exact reviewed
working-copy bytes. The two untracked source guides are identified by path and hash.

### SurrealDB and SurrealKit

| Guide and reviewed Git blob | Sections and dispositions |
|---|---|
| `docs/libraries/surrealDB/SurrealDB_CLI.md` · `d77f16553616942740752d44f0dbca310034d662` | **§§1–2, 13–15 and synthesis.** Launcher selection, dispatch, exit/output contracts and feature availability inform B02/F01 and A03. Retain PSE's selected receipt and argv boundary. **§3.** Use offline validate and explicit readiness/version diagnosis as I02–I03 describe. Start, interactive SQL, direct storage, imports/exports, upgrades and MCP remain deliberate specialist routes. **§§4–6.** Cloud provisioning/backups require a Cloud owner. Extension build, signing and deployment stay with 68a/68b. The LSP is conditional semantic evidence. **§7.** Use the Kit dispositions below. **§§8–10.** Sync snapshot/CDC, physical `skv` and standalone migration tools have separate identities and failure domains. A formatting route needs a recurring source use. **§§11–12.** Upstream language/benchmark/diagnostic tools serve supplier investigations. Docker, Helm and companion-product deployment remain separate. |
| [SurrealDB v4](../libraries/surrealDB/SurrealDB_v4.md) · `50a4588d4f7129947ad7d00099c4c41fce49d34f` | **Base §§1–3, 6.** Execution, identity, schema and transaction behavior stay in 68a/68b/68e, with exact receiving in I01–I02. **§§4–5, 7.** Graph, search and maintained state inform I05 through 68c. ANN, hybrid retrieval, materialized views and new reactive delivery need specific consumers. **§§8–9.** Retain bounded native gRPC, caller authority, extension and API restrictions. SQL/GQL compatibility does not establish equivalent application semantics. **§§10–13.** Backend, migration, raw data movement, and deployment remain with 67/68. Cloud, MCP, Agent Memory and ML have no Plan 69 adoption case. **§§14–15.** Synthesis and sources. **Addendum §§1–5.** Physical graph, maintained analytics, access plans, indexes and resource scheduling remain in 68c, with bounded CLI access and readiness in I01/I05. **§§6–9.** CBOR fidelity, compatibility, named migrations, sequence allocation and DB programming authority stay with 68b/68e. Sequence gaps and restart semantics make allocations unsuitable as original receipt identity. **§§10–12.** SQL/GraphQL/SDK and streaming need exact transport, authorization and use. **§§13–15.** Numeric, collection, null, index, record-format, capability and import differences remain explicit in versioned owners. Comments and experimental features have no current adoption case. |
| `docs/libraries/surrealDB/SurrealKit.md` · `9ee8ea04801b8cdca64dc2f4c7622b31515083c8` | **Base §§1–2, 11–13 and source map.** Orient against the exact rc.1 pin. **§§3–5.** Use scoped sync, frozen rollout and module order for staged/owned targets, with live preview and settlement under I01. **§§6–8.** Keep seed and declarative database fixtures in 68d. Use type generation only for a named consumer and test behavior on predecessor data separately from parity. **§§9–10.** Keep the standalone remote closure. Embedding, macro, template and watch integrations need an identified consumer. **Addendum §§1–3.** Offline engine, schema document v2 and external emitters are conditional. Schema v2 omits extension digest and capabilities. Per-file publication is insufficient evidence for original PSE recovery. **§§4–6.** MCP has no current receiving requirement. Native migration barriers apply to named rewrites. Compatibility raising belongs to 68. **§§7–8.** Use chain lint and phase history within I01. Catch-up, repair, discard and abandon have different effects and authority. **§§9–10.** `--exec` is conditional database fixture delegation. Embedded selection, prepared schema SQL, capture and Effect APIs need a concrete consumer. **§§11–13.** Bind effective variables, treat setup and raw apply as effects, and qualify command-specific JSON and packaging under I01/I07. **§§14–15 and scope note.** Synthesis, limits and primary sources. |

### Rust analyzers and Cargo tooling

| Guide and reviewed Git blob | Sections and dispositions |
|---|---|
| [ast-grep-core](../libraries/rust_analyzers/ast-grep-core.md) · `97040c0005396b76144d50fe69b4e1136a9a265f` | **Base §§1–8, addendum A–F and H.** Existing patterns, rule composition, constraints, capture semantics, templates and edits serve bounded discovery and preview in I03/I06. Candidate filters and pruning can reduce traversal when they preserve completeness. Compare original ranges for nested edits and reject conflicting edits under the guarded owner. Mutable reparsing supplies no transactional publication. **Base §§9–16, addendum G and I.** Grammar adapters, injection, parser caching and adoption inform the existing integrations. Extra grammars and retained node ownership need concrete demand. |
| [cargo_metadata](../libraries/rust_analyzers/cargo_metadata.md) · `7059bd3097e8664a8e93929064ce7deacf3c957d` | **Base §§1–5, 8–13, addendum §§2–7, 9.** Retain typed package identity, declarations, target eligibility and scoped graph interpretation under I04. Command options, defaults, platform conditions and incomplete artifact edges qualify that interpretation. Prefer indexed joins only where existing work warrants them. **Base §§6–7, addendum §§1, 8.** Retain original build events and structured diagnostics under their owners. Experimental libtest messages and richer artifact metadata have no separate C01 consumer. |
| [ra_ap_hir](../libraries/rust_analyzers/ra_ap_hir.md) · `328566e61cfd5b24969ae82eb9f736721e276a06` | **Base §§1–3, 5–7.** Existing snapshot-bound semantic identity, types, trait resolution and relationships. **Base §4, addendum §§1–2.** Existing source and macro provenance, with generated rename conditional on a usable mapping. **Base §8, addendum §§3–5.** Focused type, predicate, closure, layout and compiler observations. **Base §§9–12, addendum §§6–8.** Preserve database, cache and lifetime boundaries. |
| [ra_ap_ide](../libraries/rust_analyzers/ra_ap_ide.md) · `0ee7819c5cdf64c3866e43866a51cd037d91ce11` | **Base §§1–3, 8–10, addendum §§1, 6–7.** Existing snapshot and relationship evidence. **Base §5, addendum §2.** Existing rename, SSR and structured previews, with I03 receiving. **Base §7, addendum §5.2–5.3.** Runnables, cfg and update intent remain passive candidates in I04. **Base §§4, 6, addendum §§3–4, 5.1, 5.4.** Completion/presentation, static indexes, outlines, layout and predicate checks require a matching UI, bulk or diagnostic use. |
| [ra_ap_syntax](../libraries/rust_analyzers/ra_ap_syntax.md) · `3703e3b3ed88f41df0eca94e379303ab1ad5bc9e` | **Base §§1–7, addendum §§1–7.** Lossless and incremental CST, typed navigation, syntax text, literal mapping, attributes and editing help original-byte source work. Keep standalone and semantic-family versions separate. Syntax pointers are snapshot-scoped. Use edits for proposals, with formatting and macro/semantic conclusions under their own owners. **Base §§8–10 and source map, addendum §§8–9.** Existing semantic SSR remains the cross-file mechanism. |
| [rust-analyzer](../libraries/rust_analyzers/rust-analyzer.md) · `ca713b5d0290b61df6f18ee95989f4e14056722c` | **Base §§1, 3–5, 9–12, addendum §§13–15, 21.** Preserve existing workspace, cfg, security and snapshot setup. **Base §§2, 6–7, addendum §§18.3–18.4, 19.1.** Local syntax, guarded edits, run candidates and separate compiler evidence inform I03/I04. **Base §8, addendum §§16–17, 18.1–18.2, 18.5, 19.2–19.3, 20.** LSP, alternate builds, indexes, UI and documentation tools need distinct consumers. |
| [Rust compiler internals](../libraries/rust_analyzers/rust-compiler-internals.md) · `eb198cc22b007fc9f2ec303a92bca67992564c4b` | **Base §§1–8, addendum A–E.** Dataflow, ownership, MIR transformations, phase interactions and integration belong to the focused pinned compiler route. No separate acquisition for source location or test admission. |
| [Rust compiler internals continued](../libraries/rust_analyzers/rust-compiler-internals-continued.md) · `f572ea6c88239097a1b9d31c8cd27d667a51d1dd` | **Base §§1–8, addendum scope and §§1–4, 6–11.** Richer edge, initialization, drop, shim, dependency, coroutine, borrow and diagnostics evidence remains on that focused route. **Addendum §5.** Instrumented coverage has a different claim and owner from candidate or admitted population. |
| [Rust dependency tooling](../libraries/rust_analyzers/rust_dependency_tooling.md) · `6f8d5279c80048c085958d990012938cd194b6c4` | **Base §§1–7, addendum A–B, D, F–H.** Cargo metadata/tree/unit graphs serve distinct I04 questions. Guppy and Hakari are alternative traversal/simulation and feature-unification mechanisms with different completeness and policy. **Addendum C, E.** Summaries, determinator, timings and build-analysis persistence are conditional on a named comparison, with compact retention. |
| [rustdoc-types](../libraries/rust_analyzers/rustdoc-types.md) · `d6dd562cff7e49efbf149a5719f232b7ea876962` | **Base §§1–10, 12–14 and synthesis, addendum A–H.** Existing format-checked, configuration-bound API capture for I03. Keep visibility, IDs, omissions and wire contract separate. **Base §11.** Serializer, indexing, hashing and hosted archive integration need an actual acquisition or storage consumer. |
| [rustdoc](../libraries/rust_analyzers/rustdoc.md) · `576d40fbf62b1e2ea4796504f34bde3ac2d60d24` | **Base §§1–4, 8–9, 11–13, addendum scope, §§1–2, 6.** Existing contextual API capture. **Base §§5–6, addendum §§3, 5.3–5.4.** Doctest preprocessing, execution, and documentation lints are separate in I04. **Base §§7, 10, addendum §§4, 5.1–5.2, 5.5–5.6.** HTML, search and customization remain publication concerns. |
| [syn, quote and prettyplease](../libraries/rust_analyzers/syn_quote_prettyplease.md) · `f55bf64d5476a5d96700e5ec0006a6d505c6426b` | **Base §§1–4, 6–10, addendum §§1–8, 10.** Grammar-specific parsing, speculative token recognition, span diagnostics, derive modeling, interpolation and compositional generation fit existing codegen and a bounded I03 selector. Check syntax evolution against the pin. **Base §5, addendum §9.** Keep formatting in generated output and original bytes in authored previews. Token strings lose source fidelity and hygiene. |
| [tree-sitter-rust](../libraries/rust_analyzers/tree-sitter-rust.md) · `d50d53d90d4cdf689c702f9cd75a37721318e902` | **Base §§1–7, 9–12, addendum §§1–3, 5–6, 9.** Keep bounded structural queries, typed captures, recovery and coordinates under the existing I03/I06 owners. Distinguish range intersection, containment and match-limit loss. Macro token trees, attributes, Unicode and grammar-version gaps require realistic probes for supported cases. **Base §8, addendum §§4, 7–8.** Grammar customization, generated typed wrappers, alternate deployments, and editor assets require named consumers. |
