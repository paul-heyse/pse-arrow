---
title: Resolved model and incremental preparation
status: proposed
date: 2026-10-21
adrs: []
doc_role: plan
doc_owner: docs/plans/49-library-native-remediation.md
doc_topics: [design, preparation, libraries, validation]
---

# Resolved model and incremental preparation

This companion develops the preparation target of
[Plan 49](49-library-native-remediation.md). The coordinator owns rule decisions,
source-finding dispositions, **all package status**, combined dependencies and qualification.
The packages below explain changes and acceptance; they are not another progress ledger.
The coordinator's recorded rule decisions govern this target; implementation remains Proposed.

The basis is HEAD `e08b1f1dd9b89484ab7471f32b66f8e9c2a59a05`, the integrated
[in-use-library review](../design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md)
and its [recommendation audit](../design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md).
The review's original product observations use `a791c533e`; its dirty integrated revision
is an authorized planning input, not accepted implementation. Local source inspection
confirms the preparation mechanisms described below. No product checks or probes were run
to author this plan. The selected criteria are Core 3.4, Efficient Architecture 1.0 and
Process Simulator 1.5. Existing authority remains blueprint §5.3, §7.1–§7.4, §14.3–§14.4
and §16.2 until the decision route changes it.

## Target and foundation assessment

The target carries resolved physical meaning through preparation and lowers it once into
library-owned mathematics. `pse-modeling` owns specialization, lexical scope and resolved
model relationships; `pse-quantity` owns physical operation admission; `pse-ids` owns durable
identity frames; `pse-compiler` owns mathematical lowering and pure incremental products.
Symbolica owns the arithmetic representation after admission, as developed in
[49d](49d-symbolica-mathematical-execution.md). Salsa owns dependency tracking and memo reuse,
while the project owns finite retention policy, canonical revision meaning and actual leases.
Neither library takes over physical types, authored obligations or durable permission.

Today `SpecializedModel::bound_bodies` expands shared dispatch bodies back into named syntax.
`workspace/modeling/executable.rs::projection` parses `s_<hex>` references, constructs
expression-member graphs, and repeatedly searches model collections. `executable/grouped.rs`
then creates per-output `BodyInput` values and re-normalizes names for rendered body framing.
`typed_math.rs::Request` still accepts source-bearing `Expr`, string-keyed formals and
environments. The existing physical witness and occurrence inventories are valuable
foundations: their receiving authority and exact attribution survive the representation change.
They must be translated deliberately rather than replaced by a weaker expected-result type.

Retention is independently defective at ordinary defaults. `workspace.rs::trim_queries`
evicts query values and rebuilds the entire database when `retention_usage` exceeds the
generation bounds. Heavy granular queries already have LRU; `granular.rs::configure` is
not missing an eviction mechanism. The original ThreeUnits probe B2 reported about 54.5k
retained entries and 606 MB of **estimated known retained bytes**, above the default 16,384
entries/256 MiB, and 143 bodies competing for an LRU of 64. Those numbers are not RSS.
The target repairs the policy/default mismatch and preserves backdating before attributing
all repeated preparation to a need for finer specialization.

This is a hard representation pivot, not a second AST pipeline. The resolved product replaces
reconstruction from names and rendered text. Current-identity indexes and retention can land
first because neither needs the final representation. The generated handle migration can
also precede it. The compiler annotation/view products establish the semantic boundary
consumed by runtime work without importing runtime effects into Salsa.

## Resolved specialization and identity

Use a `typed-index-collections::TiVec<SymbolIx, ResolvedSymbol>` for one immutable symbol
table per specialized product. A symbol entry records its semantic identity, complete physical
contract, role and diagnostic source. Resolved expressions distinguish a symbol reference,
lexical binder, parameter, exact literal and operation/application. A binder has a scope-local
identity distinct from a referenced model symbol; a parameter stays distinct from a free
derivative coordinate. Finite membership retains declared entity kind, tuple order and absence.
The representation records original guard and validity dependencies before simplification.
It is a physical/semantic specialization product, not project arithmetic bytecode.

The type checker keeps its source-facing `TypeLookup` interface and lexical `TypeScope`.
Specialization supplies a borrowed authoritative table view with a boundary name-to-entry map;
inner operations use resolved entries and do not rebuild `BTreeMap<String, Type>` at each
contribution/coordinate. Choose this borrowed design now. An `rpds` persistent overlay is
unnecessary unless an actual independently retained overlay needs it; the existing checker
can keep its own persistent environments without forcing their use into this product.

Canonical body framing encodes operation tags, lexical scopes, local coordinate/parameter
roles, exact literal facts and complete consumed physical/function/provider premises. Instance
SymbolIds map to canonical local roles outside the reusable body frame. Two physically
equivalent instances can therefore share one program while preserving separate instance
bindings, validity and occurrence attribution. A changed renderer cannot change a body hash;
a changed physical premise, finite membership, guard order or derivative contract must.
The dispatch-group hash remains a structural grouping key, not a substitute for this body key.

Catalogue each derived relationship in `pse-ids` rather than recreating a `named_id` preimage
at a consumer. The specialized product records closure-to-conservation-row and other reached
derivations explicitly. Consumers distinguish a nonnumerical target from a broken required
relationship; a failed lookup cannot become `continue`. New frames and resolved body framing
use explicit domain/version separation under blueprint §5.3. Existing durable artifacts keep
their recorded interpretation or receive an explicit incompatibility/regeneration outcome.
No current frame silently changes meaning. The coordinator owns RC01 and the necessary ADR
and architecture route before this hash/representation contract changes.

`pse.expr_dsl` currently has `VersionOnly` metadata in `pse-schema::model::extension`.
Extend its declaration with grammar/version and field-role meaning, then generate collectors,
selectors and binding inventories from it. This is an explicit metadata evolution, coordinated
with the boundary/schema owners; merely attaching today's marker supplies neither distinction.
Generate relation/artifact membership from `catalog/products.rs::declare_profiles` and use
generated `RELATION_ID`/`NAME` handles in non-result consumers. Result declared-indexable fields
consume the same declaration through the result companion; no companion creates its own list.

## Dependency access, reuse and retention

Build one dependency-to-consumer graph per selected projection from identities. Its universe
includes free inputs, derivative-rate symbols, expression members and equation/observation
outputs, including non-expression-valued rates. Numerical dependence and validity/guard
dependence remain separate edge roles. Forward `petgraph::visit::Bfs`/`Dfs` from rate nodes
replaces the per-row fixed point for dynamic-row discovery. The current member-only graph
cannot supply that operation: it has no node for some rate symbols. Blindly wrapping it in
`Reversed` would also reverse the required direction. Topological construction uses the
expression subgraph, retaining cycle diagnostics and deterministic source order.

Build row, output, contribution and symbol lookups once from existing identity maps. Reuse
these indexes for per-output closure and accumulator construction rather than scanning every
row or re-rendering all symbols per contribution and coordinate. Preserve `MemberReads`' guard
closure and diagnostic lineage while changing its keys. The aim is necessary selected-input
work plus reached dependency traversal, not a new whole-model transitive-closure matrix.

Split retention policy into finite inventory/input/interned metadata bounds, query-family count
capacities, and a finite allowance for the current admitted live root and the memos needed for
backdating. Use Salsa 0.28.4 `set_lru_capacity` per expensive query family, with capacities for
granular checking separately from projection, `specialized` and `semantic_body`. Retain the
live root's body values and catalog-firewall results while its lease is active; admit its
extent before retaining it. Disposable query results can evict under their family capacities.
This is project policy over a **count LRU**, never a claim of native byte-weighted LRU.

Propose ordinary defaults of 131,072 retained entries and 1 GiB known retained bytes, with
granular declaration-family capacities of 4,096 and 256 values for each live-root query family
(`specialized`, projection/firewalls and semantic bodies), replacing the single capacity of 64.
These finite starting values cover the observed reference counts; PRE01 adjusts them if the
actual retained representation and targeted B2 check require it, under the enclosing allocation
allowance. They are a policy proposal, not a measured capacity guarantee or reservation.
Salsa supplies no per-memo pinning here. Admit the active-root set and temporary construction
working set together, and raise the relevant count capacities within their finite allowance
before queries can displace required live/backdating values. If that cannot fit, refuse or retire
an eligible root explicitly. Retaining an escaped result Arc alone does not retain Salsa's old
memo value for backdating; the implementation must keep that value inside the query cache.
Root admission must either supply the finite live allowance, retire an unpinned retained root,
or refuse retention/rotate explicitly while preserving returned immutable products. Routine
publish must not rotate the supported reference workload. Actual non-evictable metadata or
input growth can still require generation rotation; boundedness must cover repeated edits,
deletion and root switching rather than only one successful solve.

`Database::memory_usage` exposes aggregate ingredient metadata/fields/heap estimates, including
evicted memos. It exposes no public resident-value count; zero heap bytes do not prove an
evicted value. Keep `RetainedInputs`' shared-Arc/generation census with the aggregate report
until a replacement has explicit allocation ownership. `#[salsa::input(heap_size = ...)]`
callbacks are a bounded accounting alternative, not an automatic deletion of the census:
independent callbacks can double-count shared roots or omit generation-owned maps.

Replace projection's direct `catalog.checked(db)` reads with tracked selected-package
`physical_scope(selected)` and `function_contracts(selected)` accessors. Their equality
and retained old values allow backdating after unrelated catalog edits. Keep actual consumed
premises, negative/absence dependencies and provider environment in the tracked inputs.
Move observation demand for **existing** symbols out of structural `Request` identity;
demand introducing a new let member still changes structure and must remain represented.
Numerical case values stay outside Salsa and continue through value rebind.

## One compiler policy product per mode

Publish immutable numerical-annotation lowering keyed by admitted model, analysis mode and
consumed policy inputs. It carries typed bound/nominal/scale/start facts and their sources,
plus explicit mode differences; it does not erase a legitimate dynamic-versus-steady distinction.
Publish compiler view choice with the premise that permits promotion, so runtime consumers
do not reimplement that predicate. These are pure compiler products hosted by Salsa.

Predicted-start eligibility/fallback belongs to pure `pse-model::study_policy`; executors
apply its decision and record the actual start dependency. PRE05 establishes the shared
compiler/view contract and pure-policy input/output shape. The
[runtime companion's RUN05](49e-runtime-and-native-execution.md) owns migration of the
steady, implicit and dynamic consumers and the two start executors. RC16 governs the
annotation/view authority change. Runtime progress does not redefine compiler semantics.

## Packages and migration

| Package | Required input | Delivered behavior and acceptance boundary |
|---|---|---|
| PRE01 | Coordinator RC03 decision; current retained-input ownership | Finite family-specific retention/live-root policy, fitting ordinary defaults and selected catalog firewall; B2 reuse plus bounded churn |
| PRE02 | Registry relation/artifact declarations; metadata evolution decision for grammar/role | Generated handles/membership and expression inventories; all affected consumers migrate with missing coverage refused |
| PRE03 | Existing identity maps and current projection | Complete dependency-to-consumer indexes including rates; scan replacements preserve dynamic rows, guards and attribution |
| PRE04 | Coordinator RC01/frame decision; PRE02 expression contract; PRE03 access contract | One resolved specialization product, canonical framed bodies/derived relationships and complete lowering cutover |
| PRE05 | Existing per-mode semantics and RC16 decision | Compiler-owned annotations/view choice and pure start-policy contract; working runtime consumer migrations are a separate join |
| PRE06 | PRE01 observations and PRE04 semantic keys | Bounded cache/lane/incremental inquiries with explicit adoption decisions; no requirement to build every candidate |
| PRE07 | PRE01–PRE05 functional cutovers and selected PRE06 outcomes | Preparation migration coverage and common conformance handed to combined qualification |

**PRE01 — make normal reuse viable first.** Change `WorkspaceLimits`, `configure`, granular
query capacities and `trim_queries` together. Add selected-package tracked accessors and retain
their backdating values. Exercise no-edit prepare, unrelated and reached edits, deleted members,
root switches and oversized live roots in `workspace/modeling/reuse_tests.rs` and granular
retention tests. Port the B2 observation into a durable product-level regression rather than
copying its scratch instrumentation. Require no ordinary publish rotation, 143/143 live-body
reuse on the recorded reference scenario and `specialized` backdating under the selected new
defaults. Replace obsolete default/rotation expectations once these tests pass. This package
does not wait for resolved representation or claim to fix all cross-lane caches.

**PRE02 — remove convention-based coverage.** Change registry declarations and generators;
run `just codegen` as part of the eventual implementation. Migrate artifact membership,
non-result analysis dispatch, serde/selector inventories and relation-policy consumers together.
Test a newly declared member relation, an explicitly empty required relation, unsupported
expression metadata and grammar/role disagreement. Remove hand-maintained lists only after
the generated route reaches each consumer. A spelling mismatch must fail construction or
compilation rather than silently skip coverage. Coordinate result fields with 49b and schema
generation with the boundary companion; generated outputs are never hand edited.

**PRE03 — replace incidental repeated scans.** First build local identity indexes without
changing body framing. Create rate/free/member/row nodes and numerical/guard adjacency once,
then migrate dynamic-row selection, row lookup, contribution membership and type access.
Tests cover an unexpressed rate feeding a let chain, guard-only reads, disconnected symbols,
shared accumulators, cycles and cancellation. Preserve the exact square-equality refusal for
dynamic systems. Delete replaced fixed-point loops and repeated `find`/symbol-render rebuilds
after their callers use the indexes. PRE04 replaces the temporary name-resolution edge adapter.

**PRE04 — cut over specialization through consumers.** Introduce the resolved model and
catalogued frames, migrate specialization producers, `typed_math`/portable reconstruction,
`projection`/`grouped` lowering, derived-row consumers and body-retention keys together.
Retain immutable source occurrence inventories with explicit mapping from resolved nodes.
Give 49d the reusable resolved region/body and instance-binding contract before migrating its
construction path. Local pause points can be implemented private types and tests; they do not
authorize a second selectable production route. When replacement targeted tests pass, delete
`s_`/hex reconstruction, renamed-rendered framing and `bound_bodies` expansion callers, their
mechanism-specific tests and fixtures. Keep scientific/domain tests as independent expectations.

**PRE05 — publish decisions once.** Build table-driven mode tests around compiler annotation
and view outputs, preserving authored source, source precedence and intentional differences.
Exercise predicted-start eligibility/fallback as a pure policy operation without a runtime.
Supply the working products to RUN05; do not count its consumer migration as
complete from an agreed interface. Remove old lowering/promotion/start-policy copies in the
same changes that migrate their actual consumers. Shared files require an assigned edit owner.

**PRE06 — settle consequential optional simplifications.** Audit `BasisKey`, `view_key` and
process-pointer keys against their consumed dependencies; replace incidental address identity
with complete semantic keys where current source permits, preserving receiver-local permission.
For pure preparation flights, select `quick_cache` coalescing only if caller-owned cancellation
is acceptable and weighted retention replaces existing machinery; retain detached completion
for native drain. Shared cloned Salsa handles are eligible only for one immutable revision;
retain detached historical lanes because any setter cancels readers. Consider per-instance
tracked specialization only if PRE01/PRE03 leave reached structural edits doing material
unnecessary whole-root work; root accumulation/closure stays over original terms. Consider
input `heap_size` census replacement only after shared-root accounting equivalence. No new
persistence layer is selected: a discardable canonical-premise cache is permissible in principle,
but needs a concrete restart-reuse benefit and version/retention owner before adoption.

## Proposed verification and completion

PRE07 requires independent expectations for alpha-renaming and renderer invariance; local-slot
sharing across two instances; changed physical/binder/validity premises causing different
identities; and explicit old-frame incompatibility. Exercise value-only rebind and existing/new
observation demand separately. Re-run shared derivative consistency, conservation closure,
physical inference and original-domain conformance after the resolved lowering cutover and
49d adoption; representation equality alone can preserve the same bug in both routes.

During implementation use `just check-package pse-modeling`, `just check-package pse-compiler`
and targeted `just unit-package` filters selected from resolved identity, projection, granular,
reuse, physical premise and scientific witness tests. Include `pse-ids`/`pse-schema` checks when
their declarations change. Actual Arrow-consuming correctness closures retain force-validation.
No command listed here has been run for this proposal. Zero is the failure baseline.

The coordinator selects assembled integration/Python/native journeys and once-at-scope-end
`just hygiene`, governance, docs and applicable native/powerset checks. Local targeted passes
will be **Tested** only for their command/mode/conditions; they do not establish combined
scientific qualification. Expected reductions in rendering, scans and repeated preparation are
**Proposed** mechanism benefits. Quantitative timing/retained-memory gains require measured
before/after workloads at recorded defaults; historical B2 does not measure the new design.

Coverage explained here: R5/R6/R8/R9 and the compiler/pure-policy part of R13; DA-001–DA-003,
DA-006, DA-013, DA-027 identity, DA-029, DA-030 in part, DA-031/DA-032 and F09. DA-022's
accounting contribution stays bounded by known-byte/shared-root scope. Remaining uncertainty
is confirmation of the proposed retention capacities, optional cache/incremental simplification value. Those are explicit package decisions, not an open feasibility gate
on the resolved-model pivot. Enduring contracts move to their architecture owners through the
coordinator's decision route before retirement.
