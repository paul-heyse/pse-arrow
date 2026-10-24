---
title: "Remaining test architecture: preparation, evidence and lifecycle composition"
date: 2026-10-24
doc_role: review
doc_owner: docs/plans/53-test-architecture-target.md
doc_topics: [design, validation, preparation, storage, environment, numerical-execution]
---

# Remaining test architecture: preparation, evidence and lifecycle composition

## Judgment and authority

**Proposed:** retain Plan 53's overall target, but revise its remaining work around the
contracts that complete workflows consume. The strongest improvements are better composition
of existing owners: stage-specific preparation, an immutable collected invocation contract,
selective protected evidence reads, effect-shaped lookup and exclusion, and failure-independent
settlement. Adding another runner, general cache, result store or lifecycle manager would
increase the undertaking without supplying the missing guarantees.

This is a **Plan 53 led plan-improvement review**, covering A04's remaining acceptance and
53d–53g. Completed P00/A01–A03/B/C work is a foundation, inspected where remaining changes
adapt its consumers. Relevant Plan 52f–52h/52k changes are considered for integration; linked
amendments are proposed only where a shared responsibility justifies them. The operator
explicitly excluded 52c/C04. It is neither a prerequisite nor a proposed followup here.

The operator clarified that **52k will be complete before remaining Plan 53 implementation**.
Recommendations therefore consume the complete one-entry-point target, including its loaded
revision, native contracts, attachment, derived references and outcome-correlation owners.
They do not create interim interfaces or transfer unfinished 52k delivery into Plan 53.
Where this review identifies an additional shared contract correction, it remains an explicit
linked amendment, rather than assumed unfinished work in 52k.

The [Plan 53 coordinator](../plans/53-test-architecture-target.md) retains package, inquiry and
DTA disposition authority; the [Plan 52 coordinator](../plans/52-agent-effectiveness-and-engineering-workflows.md)
retains its own status. Detailed affected owners are
[53a](../plans/53a-scientific-assurance-and-references.md),
[53d](../plans/53d-build-discovery-and-generation.md),
[53e](../plans/53e-resource-and-evidence-lifetimes.md),
[53f](../plans/53f-effect-scoped-scheduling.md),
[53g](../plans/53g-qualification-and-publication.md),
[52f](../plans/52f-evidence-analysis-and-trace-correlation.md),
[52g](../plans/52g-agent-and-operator-workflows.md),
[52h](../plans/52h-qualification-and-publication.md) and
[52k](../plans/52k-unified-entry-point-and-derived-interfaces.md).
The [source solution review](../design_review/reviews/design_review_test-architecture-target_2026-10-23.md)
retains R01–R08/S01–S09 reasoning. Recommendations below are not adopted scope, completion
judgments, a refreshed diagnostic audit or production changes. Subsequent plan revision can adopt the recommendations through the existing owners.

## Inspected basis and evidence limits

**Interface-checked:** inspection began at `450c954b0`, including concurrent 52k changes to
capability catalogue/guidance/server and coherence/observation/presentation/reference code and
tests. During inspection, `bfa3f75a5` and `09af64fbc` committed loaded-interface and presentation/
correlation work. The decisive preparation, resource, requirement and result owners used below
were unchanged by those commits. Reference generation and actual-host checks were still in
progress at that observation. This records an inspected baseline, not a commit pin or a demand
for ongoing churn reconciliation. Recorded 52k qualification remains with its owner.

The selected standard is Core 3.4, Efficient Architecture Heuristics 1.0, Test Assessment 1.2,
Process Simulator 1.5 and the pse-arrow binding, through
[standard.toml](../design_review/design_principles/standard.toml). AP-01–AP-03 inform ownership
and composition; AP-04–AP-05 inform identity, requirement and state meanings; AP-06 informs
local reasoning and independent assurance; AP-07 informs necessary work, movement, live state
and coordination. Relevant DP/PS criteria are cited with recommendations. Efficiency does not
replace the other judgments. No exhaustive checklist or architecture acceptance verdict is
claimed.

Three independent read-only contributions examined preparation, lifetimes/scheduling and exact
library capabilities. The coordinator inspected their decisive consumers, retained scientific
reports and combined consequences. An independent assessment of the assembled recommendations
is recorded in the publication evidence below.

Library research used Context7 followed by installed or exact-version primary sources. Existing
strengths matter: nextest executable identity is already authenticated; C01/C03 already own
selection and actual Arrow leaves; B04 supplies scientific closures; installed Python admission
already checks bytes/features; native evidence analysis already parses sources once per
analysis; FastMCP lifespans, cached TypeAdapters, protected native result windows and separate
read/action cancellation already exist. None is recommended as missing functionality.

**Tested, narrow observation:** a supplied assessment projection with one passing selected gate
and 250 ordinary source-digest entries, passed to the current `operations.bounded`, retained the
gate but returned only 100 source entries and `complete=False`, with the explicit source-map
omission. This confirms truthful final truncation and motivates a more useful owner query; it
does not demonstrate false Pass or actual-host behavior. The probe imported the installed host
libraries under `scripts/pse-env -- .venv/bin/python`; it ran no compiler, service or scientific
workload and wrote no repository source.

**Inspected historical evidence:** the retained A04 Arrow report has 1,208 checks: 1,191 passed,
two failed, fourteen inconclusive and one not-applicable. Its three fixture outcomes are one
failed and two inconclusive. Reading those artifacts does not refresh their source applicability
or constitute a new scientific run. No benchmark, full suite, service reset or repository-health
qualification was performed for this review. Structural work removed by a proposal is distinct
from an unmeasured latency or capacity benefit.

## Recommended amendments

<a id="PI01"></a>
### PI01 — Finish the existing preparation route; remove work at its actual owner

**Proposed — missing-obligation reconciliation and alignment improvement.** Amend 53d D01/D02/D04,
with completed 52k bindings and 52h/53g as consumers. Preserves R05 and DTA-024/032/034/037/039.

The D01 foundation is partly stale. `.config/diagnostic-tools.toml` already pins nextest 0.9.146;
`cargo_tools.selected_identity` checks version, resolved executable and digest; Rust capture,
prepared inventory and archives consume it. Retain this owner and qualify the understood
machine-metadata shapes and executable/version drift. An exact supported pin plus explicit
shape checks can suffice; another generic schema/version registry has no demonstrated benefit.

The ordinary unprepared route still merits correction. `select.main` resolves correctness;
without the native observer, `test_run.run_rust` separately lists and then runs the original
nextest command. Feed that route through `ordinary_rust_capture` and consume its retained Cargo
metadata, binaries metadata and exact catalogue. Keep local preparation independent of canonical
observer acquisition. This removes repeated Cargo/build-discovery invocations; it does not
establish that every such invocation currently recompiles dependencies.

Complete generation inputs before sharing generation results: `physical::load` consumes
`packages/reference/fixture-projection.toml` and its selected packages, while the `generation`
input scope omits packages. Start with a known sufficient package scope. In aggregate codegen
comparison, remove only the physical/Rust-contract work already covered by the relations route.
Keep independent recipes, ordered mutating bootstrap, Python candidate/published checks, actual
API stubs, native ABI checks and missing/extra/untracked-output guards.

Move scope selection into `validation.sources` before byte access; retain separate full provenance
capture where promised. Preserve modes, symlinks and consumed resolved targets, additions,
deletions, absence and `.envrc.local` meanings. Unknown scope remains broad. This is a scoped
capture API at the existing owner, not an incremental file-index service.

**Alternative and challenge:** retaining separate preparation is appropriate for genuinely
different interactive, support-enabled or validated closures. A full-codegen-only replacement
would unnecessarily activate native capabilities for otherwise independent checks. Matching
feature spelling cannot establish equivalent actual graphs.

**Closure:** one real ordinary selection prepares once and executes retained metadata; source,
config, tool and feature changes refuse or prepare; actual Arrow target/host leaves and catalogue
association remain correct. A package-only edit invalidates generation; scoped capture never
opens an unrelated fixture file; aggregate comparison covers each retained root without duplicate
production. Delete the replaced fallback after its callers migrate. No new persistent catalogue
cache is required. Criteria: AP-01/02/03/06/07, DP-09/10/13/15/16, H5/H12/H13/H28.

<a id="PI02"></a>
### PI02 — Give Python preparation stages independent currency and useful operations

**Proposed — alignment and capability improvement.** Amend 53d D03 and coordinator inquiry I06, integrating through the
completed canonical operation owner. Preserves R05 and DTA-033; does not reopen strict producer
qualification.

`python_extension.sync` unconditionally composes dependency synchronization, Maturin develop and
stub generation. Separate dependency-environment ensure/sync, extension prepare/install, stub
generate/check and execution admission. Keep explicit combinations convenient, but let a caller
request the responsibility whose premises changed. Descriptions and any derived interface follow
the canonical definitions rather than another CLI grammar.

Use different currency premises. Environment currency concerns the selected interpreter,
locked dependencies and installation selection; installed uv 0.13.5 supplies `sync --check`.
Extension currency concerns the supported actual Cargo/Maturin selection, consumed Rust/
declaration/configuration/tool/native inputs and the installed artifact. Stub currency additionally
consumes `python_stubs::run`, its module/exception/class declarations, generator dependencies,
Ruff/configuration, actual binary and output state. Execution admission independently observes
import origin, features and bytes in parent and worker. A missing receipt or unknown consumed
input prepares conservatively; it never creates a plausible matching receipt from current state.

Coordinator inquiry I06 must explicitly qualify this ordinary editable dev/native correctness domain before skipping
work. Include membership/absence, effective configuration, interpreter/PyO3 selections and
admitted native inputs; do not turn a conservative whole-host scan or all-Python hash into the
claimed precise domain. Pure Python-only changes can affect test/result provenance without
invalidating Rust preparation. Preserve real mutable replacement windows and C03 graph assurance.

The existing `producer_deployment.bind_python_artifact` handles editable RPATH transformation:
raw Cargo and installed bytes may legitimately differ. Bind the actual installed/imported
artifact and reuse relevant transformation semantics, without importing the entire strict capture
campaign. Current Maturin 1.15.0 is verified locally; exact-release upstream source retrieval was
unavailable in this research, so no recommendation relies on newer main-branch options.

**Alternative and challenge:** always preparing remains the safe fallback. A single large key is
simpler but either loses the pure-Python benefit or omits a stage dependency. A binary-only stub
key misses declaration/tool changes. Do not assume the external server package-reuse contract
solves editable extension freshness.

**Closure:** pure Python edit preserves extension and unrelated stubs; Rust/config/tool/native
changes prepare; stub-only changes regenerate/check stubs; parent-to-worker byte replacement
refuses. Interactive unvalidated, correctness, native and explicit diagnostic/strict paths retain
their meanings. Old endpoint schema compatibility follows completed 52k's restart/snapshot
contract; a new extension is not evidence its loaded handler was refreshed. Criteria:
AP-01–AP-07, DP-03/04/09/10/18/19/23/24, H12/H14/H18/H27.

<a id="PI03"></a>
### PI03 — Seal the complete collected invocation contract once

**Proposed — strengthen E01's physical design.** Amend 53e E01 and coordinator inquiry I09 with C02, F01/F02 and
native/canonical result consumers. Preserves R07, DTA-038 and withdrawn DTA-041's live protections.

E01 moves selected identities to an immutable header, but current
`test_resources.invocation`, `publish_selection` and `publish_python_requirements` also retain a
full requirement map in mutable state; `test_run.finish` copies it into terminal evidence.
Moving selection alone would leave another catalogue-sized rewrite on registration/settlement.

Bind collected identities, executable association, resolved requirements and their roles in one
immutable collected descriptor, or explicitly bound immutable parts with one publication meaning.
Requirements here describe semantic demand; actual acquired service/generation bindings stay
with invocation/fixture ownership as in PI07. Keep membership, producer/composer references, setup
observations and terminal references mutable.
Use an indexed validated view of immutable selection within the owning operation, instead of
re-reading/scanning it per selected-test/effect lookup. Actual parent/executable, current phase
and pre-effects admission checks stay fresh.

**Alternative and challenge:** a selection-only split is smaller but leaves the same structural
amplification in requirements. Python requirements can precede actual collection and nested
collectors can be authenticated subsets: preserve an explicit uncollected staging phase and seal
only complete selected correspondence. Collected-empty must not mean uncollected; immutable
publication must not create a window allowing undeclared effects.

**Closure:** changing membership rewrites neither full selection nor full requirements;
interrupted collection/publication, nested subsets, header mismatch, duplicate/late registration,
undeclared effects and terminal freeze retain meaningful refusals. Preserve invocation→identity→
resource ordering, original deadlines and uncertain-owner fences. Criteria: AP-01/03/04/05/06/07,
DP-03/05/09/10/19/24, H12/H15/H18/H19/H27.

<a id="PI04"></a>
### PI04 — Make E05 one protected reader cutover with demand-shaped views

**Proposed — alignment and useful reader capability.** Amend 53e E01/E04/E05 and coordinator inquiry I09 with 52f F01,
52g investigation/workflow readers and completed 52k consumers. Preserves R07 and DTA-040's
legitimate aggregate/reuse semantics; does not introduce a second evidence authority.

`validation_result.read_assessment(gate=...)` parses the full v5 record; `selected_result` carries
all assessment fields other than scope/checks, including `source_files`. Canonical
`assessment_read` bounds that already-materialized result afterward. The narrow observation above
shows truthful omission, but the instruction to narrow a query offers no source-field selector.
A small returned result is not yet a small read or a complete useful answer.

Define compact mandatory summary/source-cut references, selected gate/failure views, explicit
source detail and explicit full export at the native reader. Hydrate only requested immutable
detail under its original protection. Keep native assurance, selected/unreported populations,
detail availability and presentation completeness separate. Middleware is a final encoded-size
safeguard; it should not choose which essential claims survive dictionary traversal.

Coordinator inquiry I09 must enumerate actual readers and custody operations before changing formats. Besides the
three named E05 readers, current consumers include `validation_receipts._reuse_guarded`,
`validation.run_gates`/final summary, `capabilities.qualification_worker`,
`case_measure.functional_reference`, `evidence_sources.assessment_snapshot`, archive report
production, report sealing/manifests/origin references, and FileStore invocation readers in
`run_inspection`/`evidence_projection`. Classify their distinct report meanings; one giant shared
wire type is unnecessary. Migrate producers and consumers through one versioned reader boundary.

For new 52f native questions, make optional sections and their bounds explicit. Current
`EvidenceAnalysis::ask` limits primary rows, but separately obtains contexts and other sections
through `LIMIT 10001` queries without the primary rows' truncation treatment. Section selection
must precede acquisition. Full semantic denominators and ambiguity checks remain necessary even
when display detail is narrow. Full source exports remain legitimate explicit operations.

**Alternative and challenge:** retaining v5 is simplest but keeps growing writes/full hydration;
a permanent dual store or event journal adds unwarranted lifecycles. A reused gate may point to
an older protected origin. Publish and protect that detail before its root pointer; missing detail
is unavailable/refused, not an empty passing record. A root digest cannot replace native graph
or codec semantics.

**Closure:** selected-gate reads avoid unrelated source/detail reads and preserve mandatory
outcomes; large subsidiary sections report their own completeness/continuation; full export
preserves original diagnostics. Test detail-before-root interruption, corrupt/dangling references,
old-origin protection, stored-version migration or explicit reuse refusal, same-cut native/
canonical comparison and offline server-down inspection. Criteria: AP-02/03/04/06/07,
DP-02/08/10/19/21/23/24, H5/H8/H10/H15/H16/H26.

<a id="PI05"></a>
### PI05 — Apply one settlement clock across the composed ownership join

**Proposed — concrete lifecycle correction and integration amendment.** Amend 53e E02 and its
Rust/Python/native bridge consumers. Preserves R06/R07, DTA-042 and independent assurance/cleanup outcomes.

`FixtureLifetime::drain_connections` already shares a task-local clock but short-circuits on a
sibling error. `CanonicalStore::remove_isolated_fixture` and `testing::resource_bridge` cross
further stages; the bridge can create a fresh 50-second wait and Python drain a fresh 45-second
clock. Fixing only each loop can therefore retain a renewed timeout in the combined operation.

Establish the original settlement clock at the outer fixture join, propagate it through existing
bridge/operation owners, preserve necessary cache-eviction/disconnection ordering, and attempt
independent siblings while retaining individual failures. Logical Python `Runtime.close` releases
a caller; it does not establish native receiver drain. Observer detachment, helper exit and native
physical settlement remain separate facts.

**Alternative and challenge:** sequential attempts with collected errors are often sufficient.
Use existing Tokio `join!` or dynamic all-results composition where a stalled independent sibling
must not prevent others progressing; fail-fast `try_join!` is not equivalent. Python
`AsyncExitStack` can continue exits but cannot supply the owner's deadline or native drain.
Do not let a generic task group cancel all remaining cleanup on the first failure. An escaped
reader remains protected even after other siblings settle; dependent unsafe work must not proceed.

**Closure:** one sibling fails/stalls, another settles, the original deadline expires, repeated
cancellation occurs, and an escaped reader/late effect stays protected. Drained is published only
for complete required physical settlement. Frozen assurance is never rewritten by cleanup.
Criteria: AP-02/03/05/06/07, DP-18/19/20/21, H21/H22/H23/H24/H25.

<a id="PI06"></a>
### PI06 — Give service access and scratch reclamation genuinely selective discovery

**Proposed — refine E03/E04's access paths and transition obligations.** Affects 53e E03/E04,
53f F02 and maintenance/result-origin consumers. Preserves R07/R08 and DTA-041/042/043 limits.

`FileStore.active_ids(limit=64)` enumerates and sorts the entire active directory before slicing.
Referenced, pinned and borrowed reports remain active, and `reclaim_reports` filters candidates
later. Merely excluding all-protected reports inside that loop does not remove full-population
work. It can also leave useful scratch behind a bounded prefix of ineligible entries.

Maintain narrow service/generation access and nonempty scratch-candidate projections under the
existing authority. An all-protected report still seals and protects its origins but never enters
routine scratch discovery. Recheck authoritative eligibility before deletion. Define activated
index coverage and the legacy/unknown boundary: a missing marker cannot establish service quiet
when writer coverage is unknown.

Include write cost in the design. `_save` currently republishes the active marker on each active
save; adding two more unconditional marker writes would amplify borrows and releases. Maintain
projection entries on relevant membership transitions, retaining protective ordering and crash
recovery. Presence is a discovery aid, not authority or a lease.

**Alternative and challenge:** retaining a scan is reasonable for an explicitly small population;
retained evidence can grow independently of the selected service, so that premise does not follow
from a local workstation deployment. A universal index service or database replacement is not
justified. Maintenance racing fixture creation or reference acquisition must still see protection
before effects; authority departure must precede marker removal. Stale/corrupt/ambiguous state
resolves through authority or refuses safely.

**Closure:** selected-cohort discovery, interrupted marker/authority transitions, generation
replacement, all-protected referenced reports, stale markers and unknown legacy writers retain
safe behavior. Candidate errors do not prevent independent eligible candidates being considered.
Criteria: AP-04/05/07, DP-03/16/19/20, H5/H9/H14/H23/H25/H27.

<a id="PI07"></a>
### PI07 — Model exclusion and quiet scope alongside effects, without inventing capacity policy

**Proposed — semantic refinement before F01/F02 scheduling.** Affects 53f F01–F03, 53e E01/E03
and C02's requirement consumers. Preserves R08, DTA-043–046 and accepted RC02.

Current `Requirement` owns effects, fixtures, capacity and roles. Timing/store exclusions remain
separately expressed in scheduler configuration. Canonical effects or scientific capacity do not
mean service-wide quiet. Add only the exclusion/quiet distinctions actual cases need to the same
requirement owner; bind the actual service/generation at invocation/fixture ownership and derive
per-run scheduling from those meanings. Do not turn host placement widths into semantic effects.

Inventory the assertion before choosing an invocation-owned service. For example,
`fixture_connections_preserve_deployment_database` protects a deployment database's existence;
that claim alone does not require a private service per test. Keep database-local work scoped to
its database. Where service-wide quiet is genuinely required, one relevant invocation-owned
service and a two-run interference control can supply the declared scope. E03 index absence alone
cannot authorize quiet.

**Alternative and challenge:** retaining conservative barriers is appropriate until the claim
is resolved. Separate services per test, blanket concurrency relaxation and a universal capacity
taxonomy add obligations without the selected benefit. Current functional slots, resident-store
slots, compiler limits, memory ceilings and measured demand are distinct. F03 should consume
current role declarations, not copy historical machine numbers into another policy owner.

**Closure:** renamed/split/new cases preserve requirements or refuse stale mapping before effects;
known pure KINSOL/Ipopt and B05 cases shed unrelated reservations; genuine timing/native/canonical
cases retain them. Two independent invocations exercise creation, retained access and maintenance
under the chosen claim. No wider canonical concurrency or throughput claim is required.
Criteria: AP-01/03/04/05/07, DP-02/05/18/20, H2/H21/H22/H23/H25.

<a id="PI08"></a>
### PI08 — Make A04's remaining scientific integration work explicit and discriminating

**Proposed — clarify a remaining accepted obligation.** Affects 53a A04 and 53g S01/S05; adjacent
scientific owners enter only to correct the selected consumer boundary. Preserves R01,
DTA-003's resolved reference provenance, PS-07/10/13 and independent expectations.

Do not regenerate the three already reproduced references or treat reproduction as consumer
acceptance. The retained report identifies separate questions: integrated controller's complete
layout omits a consumed guard/numerical input; simultaneous controller reaches compiler live-root
retention admission; PFR encounters closure validity and suspicious derivative comparisons.
`region_composition` and `Workspace::admit_live_root` identify the first two enforcement owners.
The compiler limit is not evidence that the workstation needs more functional slots or a higher
OS memory ceiling.

Give each selected case a bounded next inquiry and repair boundary: authored fixture inputs and
resolved layout correspondence; actual live preparation/retention requirements; and derivative/
validity diagnostic premises. Establish whether a premise has changed before selecting a rerun.
Inspect derivative sampling, scaling, cancellation and domain effects before deciding that a
reported discrepancy is a production derivative error or harmless numerical noise. Do not loosen
expectations or increase budgets merely to obtain a green report.

**Alternative and challenge:** defer the accepted consumer qualification explicitly if its remedy
requires a separately selected broader solver redesign. Keeping A04 simply 'in progress' while
sending every failure to repository health leaves S01 without an actionable completion route.
Conversely, these fixtures do not authorize a full runtime optimization or solver-fleet campaign.
Keep direct original-space, physical and fresh numerical-state checks even when preparation is
shared under PI01/PI02.

**Closure:** the unchanged independent references challenge real selected consumers; layout,
resource-limit, derivative and envelope outcomes remain correctly classified; meaningful wrong-case
controls still discriminate. Any unsupported outcome or broader remedy remains visible and cannot
be replaced by passing generator counts. Criteria: AP-04/05/06/07, DP-03/11/15/21/22/23,
PS-04/06/07/10/12/13, H1/H11/H20.

<a id="PI09"></a>
### PI09 — Offer one finite multi-question evidence inquiry when it serves the investigation

**Proposed — coordinated optional capability improvement.** Fit with 53e E05 reader work and
52f F01/52g investigations, exposed through the completed canonical bindings. This is not an
additional prerequisite for closing the original Plan 53 findings.

`EvidenceAnalysis` already owns a single DataFusion context, a bounded pool and parse-once source
preparation. Its `ask(&mut self, ...)` deliberately serializes changes to a temporary selected
view. Current one-question process boundaries can discard that preparation between related
failure/missing/denominator questions in one investigation.

Expose a bounded list of existing typed questions over one explicit baseline/candidate cut and
execute them sequentially under one native owner. Return per-question outcomes, bounds and source
identities; preserve PI04's selective sections and complete semantic denominators. This is a useful
operation-shaped batch, not an arbitrary SQL/batch executor, durable analysis session or new cache.

**Alternative and challenge:** retain one question per request when inquiries are tiny or need
independent cancellation. Prefer a finite batch before a retained analysis handle that introduces
expiry, reconnection and retention protocols. DataFusion stream execution can release query
resources on drop, but does not by itself bound sorting/aggregation intermediates or process RSS.
Partial question results must not imply the whole investigation completed.

**Closure:** independent expected pairs/denominators match native/offline/canonical answers at the
same source cut; duplicate import does not duplicate outcomes; one analysis preparation serves
several questions, each with explicit failure/completeness. A failed or cancelled question retains
its authentic outcome. Adopt only when the real comparison consumer benefits. Criteria:
AP-03/06/07, DP-08/10/13/20, H8/H12/H16/H24/H27.

<a id="PI10"></a>
### PI10 — Qualify shared journeys once while retaining separate acceptance meanings

**Proposed — sequencing and qualification amendment.** Coordinate 53g G01/G02 with 52f/52g/52h;
52k's completed target is the starting condition, not another delivery stage for Plan 53.
Preserves S01–S09 and the original plan/finding ownership.

Use a small set of joined journeys whose authentic evidence can be referenced by both plans:
describe→prepare→execute→watch/read; a relevant extension/schema change and endpoint restart;
interrupted evidence publication/reuse; independent drain failure with retained reader;
two-invocation service claims; and explicit comparison of the selected source cut. This connects
improvements that otherwise risk being accepted only through isolated helper tests.

Share an execution only where its actual source, inputs, population, mode and guarantees match.
Keep separate assertions for native behavior, host presentation, scientific adequacy, evidence
custody and independent cleanup. A successful MCP transport is not Pass; native tests do not prove
actual Codex attachment. Use completed 52k's support decisions and actual-host route rather than
retesting an obsolete proposed interface. Do not sum overlapping counts into broader coverage.

A useful order after 52k completion is: (1) reconcile delivered D01/C/B premises and declare the
needed preparation/requirement/reader contracts; (2) deliver PI01/PI02 and PI03, with PI08's bounded
scientific inquiry independent where possible; (3) migrate PI04's readers/custody together, PI05's
composed settlement and PI06's selective projections; (4) refine PI07 using those actual boundaries;
(5) exercise the joined journeys and publish enduring meaning. PI09 can join the comparison
consumer when useful. This is dependency order, not a demand to finish one entire companion
before starting another.

**Alternative and challenge:** independent per-plan campaigns are justified where modes or
premises differ. A single combined green bundle can conceal missing current subjects, source
changes or unexercised scientific/host behavior. Preserve exact applicable reuse through existing
gate/combination owners; missing or failed required populations stay incomplete. Keep scope-end
static/manual obligations with their selected implementation owners, not this review or every
slice. Any separate maintainer waiver retains its own scope.

**Closure:** each accepted obligation has its real producer, consumer, deletion and evidence;
one real run may satisfy several explicitly named assertions without becoming a new aggregate
result authority. Criteria: AP-01/02/03/05/06/07, DP-05/08/19/21/22/23/24, H18/H20/H24/H26/H28.

## Library mechanisms and adoption limits

The recommendations use capabilities already present rather than name libraries as solutions:

- [Nextest 0.9.146 retained-build interfaces](https://raw.githubusercontent.com/nextest-rs/nextest/cargo-nextest-0.9.146/site/src/docs/ci-features/archiving.md)
  accept binaries/Cargo metadata; callers still own current source, auxiliary inputs and exact
  catalogue correspondence. Cargo remains the compiled-artifact freshness owner.
- [uv environment checking](https://docs.astral.sh/uv/reference/cli/#uv-sync--check) can separate
  environment currency from extension installation. Local help confirmed the installed option;
  it does not certify Rust or native-input currency.
- [FastMCP 4.1.3 dependencies](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/server/dependencies.py)
  provide context-manager cleanup and a per-call dependency cache. Existing endpoint lifespans
  and specialist connection tasks remain the right owners for longer-lived resources.
- [MCP 2.3.0 dispatch](https://github.com/modelcontextprotocol/python-sdk/blob/v2.3.0/src/mcp/shared/jsonrpc_dispatcher.py)
  and [migration semantics](https://github.com/modelcontextprotocol/python-sdk/blob/v2.3.0/docs/migration.md)
  do not turn protocol cancellation or bounded/shielded transport cleanup into physical native
  drain. Preserve the existing owned-helper settlement and detached-read policies.
- [Pydantic 2.14.1 serialization](https://github.com/pydantic/pydantic/blob/v2.14.1/docs/concepts/serialization.md)
  supports preserving explicit field selection through `model_fields_set`/`exclude_unset`.
  `exclude_none` or `exclude_defaults` can erase explicit null/empty/default meanings. Existing
  cached TypeAdapters and native codecs should be retained, not replaced by a schema mirror.
- [DataFusion 55.1 DataFrames](https://github.com/apache/datafusion/blob/55.1.0/datafusion/core/src/dataframe/mod.rs)
  support query composition within the existing analysis context. Keep filters/projections in
  that owner and count full denominator populations before display limits where required.
- [Arrow IPC 59.3](https://github.com/apache/arrow-rs/blob/59.3.0/arrow-ipc/src/writer.rs)
  supplies serialization, not custody or window admission. Existing protected native result
  windows/chunks already serve exact scientific detail; no replacement is proposed.

A separate observation is that current `Specialists.close` can stop after the first connection
exception. The independent assessment found no shared implementation or required contract migration
with PI05. If this remains in the completed 52k result, its endpoint lifecycle owner should assess
it separately; it is not a proposed Plan 53 amendment or closure dependency. Request-scoped
FastMCP dependencies and endpoint-scoped connections have different lifetimes.

There is no adoption case here for Docket, a second scheduler, generic dependency interpreter,
new durable analysis sessions, another result database or automatic concurrency increases. A
broader solver replacement, generic tracing infrastructure or operator UI redesign would be a
separate selection unless a concrete in-footprint contract requires it. The existing 52f finite
trace inquiry remains useful, with real propagation distinguished from ID association.

## Combined tradeoffs and rule impacts

PI01/PI02 remove preparation coupling but must not weaken live source or installed-artifact
admission. PI03 and PI07 must agree on the collected requirement descriptor before E/F consumers
migrate. PI04/PI06 reduce reads and discovery only if their publication/protection writes remain
proportionate. PI05 can settle independent work without granting permission for dependent unsafe
steps. PI08 retains scientific acceptance as an independent constraint. PI10 tests these joins;
PI09 is optional and cannot quietly enlarge the mandatory target.

The footprint does not justify a universal invocation/result model: preparation, native
scientific outcomes, host presentation and custody have distinct meanings and owners. Nor does
one semantic authority require every consumer to load the same full physical payload. Typed
values, ordinary modules, immutable descriptors and narrow derived access paths are sufficient
candidate realizations.

No accepted ADR, architecture section, plan status or DTA disposition is changed by this review.
Adopting changes to public host/native documents requires their existing catalogue/generator and
consumer migration; changes to enduring invocation, compatibility or scheduling contracts use
blueprint §24.1/§24.2 and the applicable decision route. Accepted ADRs remain immutable.
Ordinary tooling corrections within those contracts need no placeholder ADR. RC01/RC02,
ADR-0122 feature unification, ADR-0170 actual Arrow validation and the clean-room reference
boundary remain intact. No withdrawn provenance campaign or excluded 52c/C04 work is revived.

## Publication evidence and remaining uncertainty

**Interface-checked — independent combined assessment:** a separate design reviewer assessed the
complete draft and decisive A04/D–G and 52f–52h/52k consumers through `5d13560cb`. The review
supported the combined model, execution fit, scientific boundaries and migration burden. Its
material scope correction was adopted: endpoint specialist teardown is a separate observation,
not a PI05 amendment or Plan 53 dependency. It found no further material contradiction requiring
a new recommendation. This was static assessment, with no edits, builds, tests or probes; it
did not independently revalidate every library citation or retained scientific artifact.

**Interface-checked:** current plans, architecture contracts, selected source owners, consumer
paths, retained A04 Arrow artifacts and version-specific library contracts support the proposed
amendments. Missing generation input edges, repeated route invocations, full-before-selected
reads, catalogue-sized mutable state and short-circuit cleanup are source observations;
architectural benefits and adoption choices remain Proposed.

**Tested:** the single synthetic bounded assessment projection described above preserved the supplied
assurance fields and explicitly reported source-detail omission. It supplies no actual-host,
publication-race, stored-format migration, Python reuse or scientific acceptance qualification.

**Not run:** product builds, suites, benchmarks, new native/service lifecycles, reference
regeneration, actual Codex journeys and whole-repository static/health checks. Existing receipts
retain their original dates and conditions. The preparation-input domain (coordinator inquiry I06),
stored-reader transition (coordinator inquiry I09) and PI08's live scientific fault classification
still require the bounded implementation inquiry and closure evidence identified above. A review recommendation
does not settle those inquiries or mark their packages complete.
