# Codebase opportunities for library leverage

**Focused follow-up, 2026-11-03:** the [codebase-wide Proto inquiry](../libraries/proto/briefing.md)
refreshes CBO-195, CBO-042, CBO-066, CBO-110, CBO-094 and CBO-008 at source
`f55951cfe995a689f9b75185b8e89cb362021616`. Their primary pages identify the new evidence
and limits. No IDs are added, retired or closed; this does not refresh unrelated occurrences
or alter the whole-account baseline below. Additional matches are explicitly conditional
in [Proto applicability](../libraries/proto/applicability.md).

**Account state: whole-codebase refresh and fresh discovery, 2026-11-02.** This page is the entry point to the living
account of where library capabilities could improve how pse-arrow fulfils its responsibilities. The
[codebase audit skill](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/leverage-codebase-audit/SKILL.md)
maintains it. Each opportunity has a stable `CBO` ID and one primary home on a topic page.

An opportunity says that a responsibility deserves investigation. It is not a defect finding, it does
not show that a library fits, and it is not a recommendation. Applicability research belongs to
[library assessments](../libraries/README.md). Choosing between options belongs to
[comparative reviews](../reviews/README.md). Both of those collections predate this baseline; see
[Input to the next stage](#input-to-the-next-stage).

Read the [framework](../library-leverage-framework.md), the [base patterns](../patterns.md), the
[shared vocabulary](../references/shared-language.md) and the
[shared additional patterns](../assessment-patterns.md) alongside this account.

## Inspected baseline and method

| Item | Value |
| --- | --- |
| Inspection date | 2026-11-02 |
| Source baseline | `main` at `f8b7d74d8232da58bf485e6e16f5bfde969c2dd7`, after completed Plan63. This is the inspected source revision, not the later documentation publication commit. |
| Included dirty changes | None. The checkout started clean; this pass's assessment edits are output, not product evidence. |
| Host state | No current host/service qualification. Earlier environment observations retain their original dates and may be stale. Passive CLI source discovery/read commands ran; no semantic/API/compiler preparation or scientific execution was needed. |
| Criteria | Framework edition 0.2, base patterns edition 0.2 (`LOP-01`–`LOP-12`), [additional patterns](../assessment-patterns.md) revision 5 (`LOP-X001`–`LOP-X022`), considered on equal footing. Core 3.4, efficient-architecture heuristics 1.0 and process-simulator 1.5 supply design context through `standard.toml`. |
| Prior account | Whole-codebase audit at source `c8244cff5`, published as `4b48509b5` on 2026-10-30: 142 live opportunities. Its claims and the 140 intervening commits directed attention without bounding discovery. Older retirement pointers retain their original provenance. |
| Evidence level | Current mechanisms are *Interface-checked* by source and selected test-body inspection. Every benefit/extension is *Proposed*. No product was built or tested, and no benefit was *Measured*. Inherited upstream comparisons are explicitly dated, not fresh library research. |

**Method.** This was a refresh plus fresh discovery, not a blind audit. Three responsibility passes
covered scientific/authoring, runtime/store/results, and engineering/registry/tests; the coordinator
covered cross-cutting mechanisms, capability gaps and integration. Every live entry was reconsidered
against current mechanisms and consumers, with representative body/test reads and explicit limits on
inherited evidence. Discovery followed complete durable execution, telemetry, specialization/native
assembly and test infrastructure beyond the earlier account's blind spots. A fresh reviewer challenged
the stable draft; its actual scope and integrated corrections are recorded below.

Source/syntax search, outlines and direct or guarded source inspection answer structural questions
without analyzer preparation. The complete CLI capability catalogue was reviewed when selecting this
approach: semantic relationships and lifetime evidence, locked API/Cargo graphs, compiler artifacts,
schema/payload inspection and retained-test evidence remain escalation routes for questions that need
them. No unresolved premise in this pass justified acquiring those heavier worlds. Search completeness
is local to the requested scope; lexical candidates and test relevance do not prove semantic reach or
coverage. One `inspect declaration` invocation omitted required `--at-column` and was rejected before
provider work; that was an argument error, not a product failure.

## Disposition of the prior account

The opportunity index and topic pages own the current occurrences. Retirement means the stated
opportunity lost its premise or was resolved, not that every imaginable improvement in its area is
exhausted. Anchored pointers preserve incoming research links and immutable prior descriptions; no ID
is reused. The original 2026-10-30 retirements remain dated pointers rather than newly reassessed live
opportunities. This refresh retains 140 of the previous 142 occurrences, retires two and adds four: **144 live
opportunities**, each with one primary topic home.

| Disposition | Count | IDs / basis |
| --- | --- | --- |
| Refreshed in place | 140 | Previous live occurrences except the two below; narrowed or corrected where current counterevidence required it |
| Retired | 2 | CBO-012: local value checks and cross-relation plans have distinct obligations. CBO-064: delivered source/document discovery removes the platform-wide gap; no separate store-query need was established. |
| New | 4 | CBO-180 durable execution; CBO-181 composed telemetry inquiry; CBO-195 domain/protobuf mappings; CBO-196 public API schema projection |

The unused ID intervals between CBO-164 and these additions were reserved for concurrent discovery
and are not missing occurrences; no substantive ID was reused.

## System picture

pse-arrow is a clean-room process-systems core in Rust, with authored `.pse` scientific packages,
generated relation surfaces and isolated independent Python references. The
[architecture overview](../../authoritative_design/sections/architecture-overview.md) assigns operations
to libraries by fit. Symbolica and Numerica own algebra, derivatives and evaluators. Class-specific native
solvers own iteration and factorization. Salsa owns pure semantic reuse. Arrow and DataFusion own columnar
and relational boundaries. SurrealDB owns canonical persistence. The native host owns process and service
control.

**Read as strong at this baseline, with limits.** The claim here concerns reimplementation, not depth: solver and library work is delegated rather than rebuilt, and depth stays open in the entries named. Solver adapters drive libraries through their own APIs, such as HiGHS IIS, ranging and presolve, SCIP expressions and exact mode, and POUNCE retained symbolics. The HiGHS and IBEX statements are adapter-level, since the HiGHS solve body was read representatively and IBEX sources were not available. Depth questions are in [CBO-120](native-solver-integration.md#cbo-120), [CBO-122](native-solver-integration.md#cbo-122) and [CBO-150](capability-gaps.md#cbo-150). Derivatives are library-owned, with `SymbolicComposition` as a good example. Pure and effectful code are separated, with `pse-modeling` free of stores and solvers. The host's cgroup, `AddRef`, `BindsTo` and pidfd use is the mechanism rather than a supervisor rebuilt beside it, with open entries on the same subject at [CBO-004](cross-cutting-mechanisms.md#cbo-004) and [CBO-048](developer-tooling.md#cbo-048). Honest withholding, typed availability and original-space qualification are contracts worth keeping. Many generic-algorithm suspicions dissolved once bodies were read, in the code that was read closely. A strength recorded here was read, not tested, and remains open to later comparison.

## Where opportunities cluster

These are reading routes, not ranked recommendations. Value remains Proposed and library fit remains
unassessed at this refreshed baseline.

1. **Prepaid and retained resource accounting.** Admission predicts supplier allocations before they happen,
   while retained visitors account built values and sharing. Existing supplier estimates and generated
   `HeapUsage` already carry parts of the work. The open question is which remaining mirrors and visitors
   can be delegated without weakening prepay or confusing host and foreign heaps.
   [CBO-001](cross-cutting-mechanisms.md#cbo-001), [CBO-086](cross-cutting-mechanisms.md#cbo-086),
   [CBO-021](mathematics-and-compilation.md#cbo-021), [CBO-125](native-solver-integration.md#cbo-125).
2. **Durable operations and observation.** Canonical capture, study progression, worker ownership,
   effect settlement and recovery compose across several owners. Local polling or per-round-trip changes
   need the complete operation's authority and crash semantics. Telemetry likewise spans runtime sinks,
   exporters, collector/backend services and passive readers. Its parts existing does not establish a
   useful composed investigation surface.
   [CBO-071](persistence-and-transport.md#cbo-071), [CBO-073](persistence-and-transport.md#cbo-073),
   [CBO-016](execution-lifecycle-and-observation.md#cbo-016),
   [CBO-013](execution-lifecycle-and-observation.md#cbo-013).
3. **Store-language and wire boundaries.** Stored and inline SurrealQL, generated declarations and Rust
   envelopes share meaning across representations. The new code-intelligence protocol also supplies
   fresh conversion/description boundaries to examine. Generated shape does not by itself remove policy
   translation, validation or unknown-field obligations.
   [CBO-046](registry-generation-and-python-boundary.md#cbo-046),
   [CBO-010](persistence-and-transport.md#cbo-010), [CBO-067](persistence-and-transport.md#cbo-067).
4. **Dense scientific data and narrow reads.** Matrix-like solver results pass through relation rows,
   IPC blocks and retained projections. Some consumers need one scalar or selected field; others really
   need complete rows. Any columnar or decoder delegation must preserve field checks, prepay and exact
   row coordinates rather than assume universal narrow consumption.
   [CBO-075](result-data-paths.md#cbo-075), [CBO-076](result-data-paths.md#cbo-076),
   [CBO-077](result-data-paths.md#cbo-077), [CBO-078](result-data-paths.md#cbo-078).
5. **Consumers and maintained forms.** Relation projections, engine doors and a durable body tier need
   present consumers or a current promise to justify their upkeep. A source search without a caller
   remains bounded evidence, especially for public library APIs. New source/document ownership and
   code discovery capabilities can resolve an old tool gap without making the curated knowledge graph
   a full code index.
   [CBO-153](registry-generation-and-python-boundary.md#cbo-153),
   [CBO-154](registry-generation-and-python-boundary.md#cbo-154),
   [CBO-099](modeling-and-preparation-workflows.md#cbo-099).
6. **Incremental preparation and supplier seams.** Salsa, compiler fragment reuse and compact native
   assemblies already exist. Remaining opportunities concern specific duplicated trackers or preparation
   lifetimes, supplier global state, arithmetic closure and upstream deltas. The refreshed source does
   not support a blanket claim that specialization is uncached or native callbacks all hydrate maps.
   [CBO-107](mathematics-and-compilation.md#cbo-107), [CBO-108](mathematics-and-compilation.md#cbo-108),
   [CBO-096](modeling-and-preparation-workflows.md#cbo-096),
   [CBO-098](modeling-and-preparation-workflows.md#cbo-098),
   [CBO-116](native-solver-integration.md#cbo-116),
   [CBO-164](execution-lifecycle-and-observation.md#cbo-164).
7. **Author and engineer workflows.** Lossless authoring edits, located/multiple diagnostics, branch
   interpretation, property providers, surrogate products and richer uncertainty workflows have distinct
   consumers. Existing numerics and structured diagnostics are counterevidence to broad absence claims;
   some additional activities lack recorded demand, and higher-index DAE reduction is explicitly outside
   the design target.
   [CBO-113](authoring-and-identity.md#cbo-113), [CBO-114](authoring-and-identity.md#cbo-114),
   [CBO-135](numerical-workflows.md#cbo-135), [CBO-140](capability-gaps.md#cbo-140),
   [CBO-141](capability-gaps.md#cbo-141), [CBO-152](capability-gaps.md#cbo-152).
8. **Assurance and shared effects.** Source-bound review lists, injected fault windows, schedule
   exploration, decoder fuzzing, test identities and reusable fixture assembly each protect different
   obligations. Current native test selection, same-build artifact capture and completed JUnit
   reconciliation are strengths to keep; tool presence or a test body does not establish that the
   complete population was exercised.
   [CBO-049](developer-tooling.md#cbo-049), [CBO-060](developer-tooling.md#cbo-060),
   [CBO-157](test-and-assurance.md#cbo-157), [CBO-158](test-and-assurance.md#cbo-158),
   [CBO-159](test-and-assurance.md#cbo-159), [CBO-163](test-and-assurance.md#cbo-163).

**Interactions.** Supplier moves carry prediction/accounting obligations. Removing genuinely unconsumed
forms can reduce representation-change burden, but cannot remove forms with live scientific consumers.
Study protocol and dense result production can affect the same workflow through independent causes;
no combined cost was measured. Functional gaps need demand and scientific validity before library fit
can turn them into comparative recommendations.

**Relation to other accounts.** The [design-principle audit](../../design_review/audit/README.md)
judges alignment and retains its own baseline. This account identifies opportunities for capabilities
to carry responsibilities. The [library assessments](../libraries/README.md) and
[comparative reviews](../reviews/README.md) retain their earlier source/version contexts. Agreement
between accounts that read the same source is not independent proof, and none authorizes implementation.

## Items that may need a diagnostic follow-up

These source observations can matter independently of a library choice. They were not reproduced,
and this audit neither fixes them nor schedules another engagement.

- **Capture recovery.** CBO-071's current source reading still finds a gap between open/commit capture
  and dead-owner gate recovery. It names the exact custody/fencing conditions and the ADR-0230 reason
  for gating; a different observer route must retain the consistency its real consumers require.
- **Authoring comment preservation.** CBO-113's `assign_ids_with`/whole-document render path still
  merits checking against a comment-preserving user journey. Static source reading is not a reproduced
  data-loss report.
- **Supplier normalization guard.** CBO-106 records a suspected vendor shape and the exponential
  staging barrier. An authored reachable witness and an upstream correction remain unestablished;
  it is not a fresh demonstrated supplier defect.
- **Recorded witness consumption.** CBO-156 retains the bounded earlier no-caller observation, while
  CBO-099 asks for a concrete production composition witness for portable mathematics/child execution.
  Current canonical opening positively checks interpretation/schema and module/incarnation identity;
  equivalence to every directional promise remains unsettled. Broad plan incompleteness is not inferred.
- **Policy/document residue.** CBO-049's unused scheduling declarations, CBO-116's deleted arithmetic
  control script reference and CBO-065's historical operational residue need their current owners and
  current policy reconciled. No old host-unit state is asserted live, and no workstation cap is proposed.

## Input to the next stage

The [library collection](../libraries/README.md) and [review collection](../reviews/README.md)
were not substantively refreshed here. Their source/library versions and earlier judgments remain
attributed to their original baselines. Links to retired occurrences still resolve to explicit
pointers; a retained reusable library type does not require retaining a false current occurrence.
New IDs have no contextual library assessment yet. Completed plans and superseded Plan57 authorize
no new work.

A focused `leverage-library-assessment` can now investigate these questions, choosing depth by value
and uncertainty rather than treating this list as an implementation order:

1. **Prediction and retained accounting.** Which suppliers expose conservative symbolic estimates,
   budget/fallible hooks or accounting leaves? Where could drift checks help without replacing prepay
   with post-allocation observation? Can existing generated/local accounting serve the need first?
2. **Durable execution.** Can any reusable continuation/settlement mechanism use canonical SurrealDB
   authority, caller-owned native effects and no replay of a started scientific effect? What real
   responsibility disappears, compared with improving existing shared owners? Start at CBO-180.
3. **Telemetry inquiry.** After native checkpoint inspection and the backend UI, what recurring operator
   question is still missing? Can existing health/internal-metric/query interfaces compose a bounded
   passive answer with independent stage availability and loss semantics? Start at CBO-181.
4. **Protocol and API projections.** Which CBO-195 mapping is mechanical correspondence and which is
   contextual admission? Can owned/views/reflection or derivation reduce that correspondence? For
   CBO-196, can public recursive schema constraints be published without a JSON intermediate or a
   second manually maintained supplier vocabulary, while preserving exact schema qualification?
5. **Store language and observer reads.** What can the exact locked SurrealQL tooling expose about
   signatures, fields and refusal vocabularies? Can a smaller consistent observer read avoid durable
   capture effects without weakening frozen cohort/result protection? How are physical scan budgets kept?
6. **Columnar production and decoding.** Can generated relation admission accept prebuilt columns?
   Which projection, encoding or compression capabilities preserve bounded IPC admission, metadata and
   original row identities? Use CBO-075–078 and CBO-005, not a universal serialization rewrite.
7. **Incremental and supplier seams.** Can the pinned Salsa model express the specific dependency
   scopes and transient refusals described in CBO-107/108? Which current supplier delta actually removes
   a recorded limitation? Old upstream commit/version comparisons are dated leads to refresh first.
8. **Authoring and scientific workflows.** Which lossless syntax/recovery tools compose with current
   authored rows and spans? Which proposed surrogate, property, branch or uncertainty product has a
   concrete consumer and validity contract before investigating its library support?
9. **Assurance and host preparation.** Can controlled schedule/fault/model exploration complement the
   existing deterministic tests at a small real owner? Can guarded metadata reuse reduce test-loop
   preparation without weakening independent build-unit checks? Which accounting/fixture facts already
   exist and should be reused?

The base and additional pattern set adequately framed this pass; no new pattern was needed.
[Revision 5](../assessment-patterns.md) remains the shared reusable output, with no edition restamp.

## Opportunity index

The Status column separates a *current promise* (an existing contract, with only its realization in question), a
*current mechanism* (an implementation choice with no stronger promise attached), *accepted planned* work, a
*declared target, undelivered* (architecturally in scope, with no accepted plan) and a *proposed* addition. "Weak" marks an opportunity whose counterevidence is strong, retained so that later work can
confirm or retire it.

| ID | Opportunity | Status | Home |
| --- | --- | --- | --- |
| CBO-001 | Prepaid prediction mirrors library internals | Current promise | [Cross-cutting](cross-cutting-mechanisms.md#cbo-001) |
| CBO-002 | Residual bounded writers beside the shared typed sink | Current mechanism | [Cross-cutting](cross-cutting-mechanisms.md#cbo-002) |
| CBO-003 | Owned private files and durable publication have no shared owner | Current mechanism | [Cross-cutting](cross-cutting-mechanisms.md#cbo-003) |
| CBO-004 | Process identity and supervision: name-based scans beside owned handles | Current mechanism | [Cross-cutting](cross-cutting-mechanisms.md#cbo-004) |
| CBO-005 | Local IPC admission around Arrow's decoders needs check-by-check upstream tracking | Current promise | [Result data](result-data-paths.md#cbo-005) |
| CBO-008 | Seeds, progress and receipts as JSON payloads beside typed relations | Current mechanism (weak) | [Result data](result-data-paths.md#cbo-008) |
| CBO-010 | Guarded scripts: logic split between stored functions and inline text, reparsed per call | Current mechanism | [Persistence](persistence-and-transport.md#cbo-010) |
| CBO-011 | Single-flight, I/O gates and load admission beside caching libraries | Current mechanism | [Persistence](persistence-and-transport.md#cbo-011) |
| CBO-013 | One interval spelled per sink; metric declaration beside installation | Current mechanism | [Lifecycle](execution-lifecycle-and-observation.md#cbo-013) |
| CBO-016 | Waiting by polling for state owned elsewhere | Current promise | [Lifecycle](execution-lifecycle-and-observation.md#cbo-016) |
| CBO-019 | Process facts: incomplete adoption of the owner that exists | Current mechanism | [Lifecycle](execution-lifecycle-and-observation.md#cbo-019) |
| CBO-021 | Caller-side seams around the already vendored Symbolica | Current promise | [Mathematics](mathematics-and-compilation.md#cbo-021) |
| CBO-022 | Which prepared-mathematics lifetimes should share one eviction policy | Current promise | [Mathematics](mathematics-and-compilation.md#cbo-022) |
| CBO-027 | Diffsol's infallible traits force panic control flow and local workarounds | Current mechanism | [Native solvers](native-solver-integration.md#cbo-027) |
| CBO-028 | Supplier deltas with numerics inside integration: Uno, and upstream-eligible fixes | Current mechanism | [Native solvers](native-solver-integration.md#cbo-028) |
| CBO-030 | Process-wide library state makes independent solves exclude each other | Current mechanism | [Native solvers](native-solver-integration.md#cbo-030) |
| CBO-034 | Non-local uncertainty: sampled propagation, global sensitivity and cohort statistics | Current mechanism | [Numerical workflows](numerical-workflows.md#cbo-034) |
| CBO-035 | Least-squares route: value, eligibility and dense-algebra split (weak) | Current mechanism (weak) | [Numerical workflows](numerical-workflows.md#cbo-035) |
| CBO-036 | Fit quality, identifiability and the unknown-sigma model | Current mechanism | [Numerical workflows](numerical-workflows.md#cbo-036) |
| CBO-037 | Grammar facts in several hand-written forms | Current promise | [Authoring](authoring-and-identity.md#cbo-037) |
| CBO-038 | Held spans do not reach human diagnostics | Current mechanism | [Authoring](authoring-and-identity.md#cbo-038) |
| CBO-039 | Identity framing written by hand, with silent omission possible | Current mechanism | [Authoring](authoring-and-identity.md#cbo-039) |
| CBO-040 | Registry products baked as Rust, JSONL and IPC | Current mechanism | [Registry](registry-generation-and-python-boundary.md#cbo-040) |
| CBO-042 | Document wire envelopes after the foreign reader retired | Current mechanism | [Registry](registry-generation-and-python-boundary.md#cbo-042) |
| CBO-045 | Two diagnostic-code authorities and macro noise | Current mechanism | [Registry](registry-generation-and-python-boundary.md#cbo-045) |
| CBO-046 | The canonical store program: SurrealQL in Rust strings with no semantic toolchain | Current mechanism | [Registry](registry-generation-and-python-boundary.md#cbo-046) |
| CBO-048 | Native unit accounting exists; peak and OOM interpretation remain separate | Current mechanism | [Tooling](developer-tooling.md#cbo-048) |
| CBO-049 | Native test discovery still repeats admission and policy queries | Current mechanism | [Tooling](developer-tooling.md#cbo-049) |
| CBO-051 | Tooling interchange and documents read through several parsers | Current mechanism | [Tooling](developer-tooling.md#cbo-051) |
| CBO-052 | Named recipes still compose test selections outside the native intent route | Accepted planned | [Tooling](developer-tooling.md#cbo-052) |
| CBO-054 | Immutable artifact generations implemented once per kind | Current mechanism | [Host](host-and-invocation.md#cbo-054) |
| CBO-055 | Arrow force-validation delivered by a Cargo argv and unit-graph adapter | Current mechanism | [Host](host-and-invocation.md#cbo-055) |
| CBO-056 | Invocation ledger has no retention, and queries scan every record | Accepted planned in part | [Host](host-and-invocation.md#cbo-056) |
| CBO-057 | Native provider construction is a hand-maintained build system | Current mechanism (weak) | [Host](host-and-invocation.md#cbo-057) |
| CBO-058 | Tool provisioning has several independent declarations | Current mechanism | [Host](host-and-invocation.md#cbo-058) |
| CBO-059 | System facilities used through parallel spellings | Current mechanism | [Host](host-and-invocation.md#cbo-059) |
| CBO-060 | Executable input completeness rests on a hand-written review catalogue | Current mechanism | [Tooling](developer-tooling.md#cbo-060) |
| CBO-061 | Macro-supplier patches enforce build purity one owner at a time | Current mechanism | [Tooling](developer-tooling.md#cbo-061) |
| CBO-062 | Artifact-to-unit association re-derives what compiler and Cargo could report | Current mechanism | [Tooling](developer-tooling.md#cbo-062) |
| CBO-063 | Generator forms emitted or transformed by hand | Current mechanism (weak) | [Tooling](developer-tooling.md#cbo-063) |
| CBO-065 | Premises and residue that outlived their consumers | Current mechanism | [Tooling](developer-tooling.md#cbo-065) |
| CBO-066 | Query responses pass through five representations behind a hand-kept preflight | Current mechanism | [Persistence](persistence-and-transport.md#cbo-066) |
| CBO-067 | Referential and lifecycle rules rebuilt on string keys beside native references | Proposed | [Persistence](persistence-and-transport.md#cbo-067) |
| CBO-068 | Named guards and FOR UPDATE registrations may overlap what SSI tracks | Current mechanism | [Persistence](persistence-and-transport.md#cbo-068) |
| CBO-069 | Schema rollout has overlapping machineries | Current mechanism | [Persistence](persistence-and-transport.md#cbo-069) |
| CBO-070 | Server patch series carries integration later releases may absorb | Current mechanism | [Persistence](persistence-and-transport.md#cbo-070) |
| CBO-071 | A live status poll is realized as a durable, globally gated capture | Current promise; polling a current mechanism | [Persistence](persistence-and-transport.md#cbo-071) |
| CBO-072 | Read custody is durable lease rows with their own write and release cycle | Current mechanism | [Persistence](persistence-and-transport.md#cbo-072) |
| CBO-073 | A study occurrence takes many round trips and reassembles known facts | Current mechanism | [Persistence](persistence-and-transport.md#cbo-073) |
| CBO-074 | No composed application backup and restore | Current promise, no owner found | [Persistence](persistence-and-transport.md#cbo-074) |
| CBO-075 | Dense native results produced row by row through per-cell dispatch | Current mechanism | [Result data](result-data-paths.md#cbo-075) |
| CBO-076 | Result scans hide column, parallel and statistics intent | Current mechanism | [Result data](result-data-paths.md#cbo-076) |
| CBO-077 | Selected output reads hand-compose what a physical plan expresses | Current mechanism | [Result data](result-data-paths.md#cbo-077) |
| CBO-078 | Per-relation result projection repeats setup and selected report walks | Current mechanism | [Result data](result-data-paths.md#cbo-078) |
| CBO-079 | Operation custody carriers and per-layer re-admission | Current mechanism | [Lifecycle](execution-lifecycle-and-observation.md#cbo-079) |
| CBO-080 | Cancellation has three owners split by a dependency ceiling | Current mechanism | [Lifecycle](execution-lifecycle-and-observation.md#cbo-080) |
| CBO-081 | Completion-owned scheduling loops written per site | Current mechanism (weak) | [Lifecycle](execution-lifecycle-and-observation.md#cbo-081) |
| CBO-082 | Paired resource admission wait recurs | Current mechanism | [Lifecycle](execution-lifecycle-and-observation.md#cbo-082) |
| CBO-083 | Owner-affine worker teams implemented separately | Current mechanism | [Lifecycle](execution-lifecycle-and-observation.md#cbo-083) |
| CBO-084 | Record families and metric units are free strings across crates | Current mechanism | [Lifecycle](execution-lifecycle-and-observation.md#cbo-084) |
| CBO-085 | Span-loss accounting scrapes SDK internal diagnostics | Current mechanism | [Lifecycle](execution-lifecycle-and-observation.md#cbo-085) |
| CBO-086 | Retained-size walkers maintained by hand across the product graph | Current mechanism | [Cross-cutting](cross-cutting-mechanisms.md#cbo-086) |
| CBO-087 | Source selection carries a hand-built query engine beside Salsa | Current mechanism | [Modeling workflows](modeling-and-preparation-workflows.md#cbo-087) |
| CBO-088 | Dynamic preparation interprets the compiler's product in orchestration | Current mechanism | [Modeling workflows](modeling-and-preparation-workflows.md#cbo-088) |
| CBO-089 | Assessment reinterprets outputs at every observed point | Current mechanism | [Modeling workflows](modeling-and-preparation-workflows.md#cbo-089) |
| CBO-090 | Observation crossings and demand discovered in stages | Current mechanism | [Modeling workflows](modeling-and-preparation-workflows.md#cbo-090) |
| CBO-091 | Revision comparison and diff implemented per operation | Current mechanism | [Modeling workflows](modeling-and-preparation-workflows.md#cbo-091) |
| CBO-092 | Project ancestry walked one guarded round trip per revision | Current mechanism | [Modeling workflows](modeling-and-preparation-workflows.md#cbo-092) |
| CBO-093 | Logical keys and identities cross boundaries as formatted strings | Current mechanism | [Modeling workflows](modeling-and-preparation-workflows.md#cbo-093) |
| CBO-094 | One requested operation in several serialized request languages | Current mechanism | [Modeling workflows](modeling-and-preparation-workflows.md#cbo-094) |
| CBO-095 | Nonlinear explanation runs a sequential deletion filter | Proposed | [Modeling workflows](modeling-and-preparation-workflows.md#cbo-095) |
| CBO-096 | Runtime numerics rebuild stable preparation at the wrong lifetime | Current mechanism | [Modeling workflows](modeling-and-preparation-workflows.md#cbo-096) |
| CBO-097 | Identity hashes recomputed from full data at every use | Current mechanism | [Modeling workflows](modeling-and-preparation-workflows.md#cbo-097) |
| CBO-098 | PETSc callbacks evaluate through a map-keyed point | Current mechanism | [Modeling workflows](modeling-and-preparation-workflows.md#cbo-098) |
| CBO-099 | Confirming the consumers of portable mathematics and child execution | Current promise; consumer confirmation open | [Modeling workflows](modeling-and-preparation-workflows.md#cbo-099) |
| CBO-100 | Derivative product construction has several routes | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-100) |
| CBO-101 | C++ compiler driver and code cache beside an in-process JIT | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-101) |
| CBO-102 | Case evaluation runs identical programs one instance at a time | Proposed | [Mathematics](mathematics-and-compilation.md#cbo-102) |
| CBO-103 | Polynomial and affine recognition repeated by consumers | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-103) |
| CBO-104 | Support analysis is a generic sparsity propagator with repeated symbolic work | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-104) |
| CBO-105 | Several interval and constant sweeps over one factorable DAG | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-105) |
| CBO-106 | Exponential staging protects against a suspected normalization defect | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-106) |
| CBO-107 | Salsa memoizes around a driver that feeds results back as inputs | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-107) |
| CBO-108 | Physical dependency tracking on thread-local capture beside Salsa | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-108) |
| CBO-109 | Specialization builds two representations in lockstep | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-109) |
| CBO-110 | Parallel wire hierarchies with hand capture and restore | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-110) |
| CBO-111 | Structural analysis caps set by the matching kernel's recursion | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-111) |
| CBO-112 | Tabular knowledge hydrated to scalars and admitted by a private engine | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-112) |
| CBO-113 | The modeling language has a semantic parser but no language toolchain | Current mechanism; toolchain proposed | [Authoring](authoring-and-identity.md#cbo-113) |
| CBO-114 | Checking reports one failure per cycle | Proposed | [Authoring](authoring-and-identity.md#cbo-114) |
| CBO-115 | Authoring load takes a long route with parallel span and accounting forms | Current mechanism | [Authoring](authoring-and-identity.md#cbo-115) |
| CBO-116 | The faer arithmetic fork: enforced per supplier, with no stack-wide statement of coverage | Current mechanism | [Native solvers](native-solver-integration.md#cbo-116) |
| CBO-117 | Covering proofs consume an unseeded process-lifetime random stream | Current mechanism | [Native solvers](native-solver-integration.md#cbo-117) |
| CBO-118 | Point interval enclosures rebuilt per call under the proof lock | Current mechanism | [Native solvers](native-solver-integration.md#cbo-118) |
| CBO-119 | Rank and spectral diagnostics densify sparse operators | Proposed | [Native solvers](native-solver-integration.md#cbo-119) |
| CBO-120 | NLP presolve bound passes disabled for lack of multiplier recovery | Current mechanism | [Native solvers](native-solver-integration.md#cbo-120) |
| CBO-121 | The pinned POUNCE fork carries supplier deltas awaiting assessment | Current mechanism | [Native solvers](native-solver-integration.md#cbo-121) |
| CBO-122 | Ipopt reuse keeps objects but discards numerical preparation | Current mechanism | [Native solvers](native-solver-integration.md#cbo-122) |
| CBO-123 | Native boundary declarations exist twice with no generated link | Current mechanism | [Native solvers](native-solver-integration.md#cbo-123) |
| CBO-124 | Oracle wrappers forward by hand, and optional facts default silently | Current mechanism | [Native solvers](native-solver-integration.md#cbo-124) |
| CBO-125 | Direct-factor admission charges dense fill | Current mechanism | [Native solvers](native-solver-integration.md#cbo-125) |
| CBO-126 | Three KLU owners with different reuse and observation | Current mechanism | [Native solvers](native-solver-integration.md#cbo-126) |
| CBO-127 | Diffsol rebuilds patterns and symbolic analysis per segment | Current mechanism | [Native solvers](native-solver-integration.md#cbo-127) |
| CBO-128 | IDAS gradients and sensitivities repeat preparation | Current mechanism | [Native solvers](native-solver-integration.md#cbo-128) |
| CBO-129 | A Diffsol supplier move is worth assessing on its contents | Proposed | [Native solvers](native-solver-integration.md#cbo-129) |
| CBO-130 | Hybrid trajectory ordering written separately per backend | Current mechanism | [Native solvers](native-solver-integration.md#cbo-130) |
| CBO-131 | SUNDIALS resource ownership hand-written per adapter | Current mechanism | [Native solvers](native-solver-integration.md#cbo-131) |
| CBO-132 | Design generator family: sequences and classical forms | Current mechanism | [Numerical workflows](numerical-workflows.md#cbo-132) |
| CBO-133 | Repeated-fit statistics over one prepared fit | Current mechanism | [Numerical workflows](numerical-workflows.md#cbo-133) |
| CBO-134 | Fit multistart and local-minimum detection | Proposed | [Numerical workflows](numerical-workflows.md#cbo-134) |
| CBO-135 | Train, validate and embed a surrogate as a product | Declared target, undelivered | [Numerical workflows](numerical-workflows.md#cbo-135) |
| CBO-136 | Measurement-error models beyond independent absolute sigma | Proposed | [Numerical workflows](numerical-workflows.md#cbo-136) |
| CBO-137 | Experiment design and data reconciliation | Proposed | [Numerical workflows](numerical-workflows.md#cbo-137) |
| CBO-138 | Propagation demand: variances, bands and dense kernels | Current mechanism | [Numerical workflows](numerical-workflows.md#cbo-138) |
| CBO-139 | Multiple-shooting parameter estimation | Proposed | [Numerical workflows](numerical-workflows.md#cbo-139) |
| CBO-140 | Retained branch diagrams and steady-state stability | Proposed | [Capability gaps](capability-gaps.md#cbo-140) |
| CBO-141 | Transport, Helmholtz and high-accuracy property backends | Declared target, undelivered | [Capability gaps](capability-gaps.md#cbo-141) |
| CBO-142 | Provider envelopes and global solving through providers | Current promise | [Capability gaps](capability-gaps.md#cbo-142) |
| CBO-143 | Engineer-readable outputs: stream tables, diagrams, equations | Proposed | [Capability gaps](capability-gaps.md#cbo-143) |
| CBO-144 | Solver-neutral problem export for reports and comparison | Proposed | [Capability gaps](capability-gaps.md#cbo-144) |
| CBO-145 | Transitive impact query over retained lineage | Proposed | [Capability gaps](capability-gaps.md#cbo-145) |
| CBO-146 | Jacobian-derived scaling proposals | Proposed | [Capability gaps](capability-gaps.md#cbo-146) |
| CBO-147 | Bounded nonlinear roots have no native bounded-root method | Proposed | [Capability gaps](capability-gaps.md#cbo-147) |
| CBO-148 | Adjoints stop at events and resets | Current mechanism | [Capability gaps](capability-gaps.md#cbo-148) |
| CBO-149 | Integral objectives become extra states rather than native quadrature adjoints | Current mechanism | [Capability gaps](capability-gaps.md#cbo-149) |
| CBO-150 | Sparse preconditioning is thin | Proposed | [Capability gaps](capability-gaps.md#cbo-150) |
| CBO-151 | IDAS controls and native diagnostics are a narrow subset | Proposed | [Capability gaps](capability-gaps.md#cbo-151) |
| CBO-152 | Structural DAE index analysis is absent | Proposed | [Capability gaps](capability-gaps.md#cbo-152) |
| CBO-153 | Arrow projection generated for every relation regardless of consumers | Current mechanism | [Registry](registry-generation-and-python-boundary.md#cbo-153) |
| CBO-154 | Engine SQL, DML and export surfaces have no production door | Current mechanism | [Registry](registry-generation-and-python-boundary.md#cbo-154) |
| CBO-155 | Field metadata re-established around DataFusion by a family of adapters | Current mechanism | [Registry](registry-generation-and-python-boundary.md#cbo-155) |
| CBO-156 | Recorded contracts, obligations and invariants: overlapping and unattached routes | Current promise | [Registry](registry-generation-and-python-boundary.md#cbo-156) |
| CBO-157 | Durable-effect fault injection is per-site flags in production modules | Current mechanism | [Test and assurance](test-and-assurance.md#cbo-157) |
| CBO-158 | Ownership and settlement concurrency has no schedule exploration | Proposed | [Test and assurance](test-and-assurance.md#cbo-158) |
| CBO-159 | Fuzz, mutation and coverage tools are pinned with no standing route | Current mechanism | [Test and assurance](test-and-assurance.md#cbo-159) |
| CBO-160 | Property coverage exists beyond leaves; model and invariance coverage remains selective | Proposed | [Test and assurance](test-and-assurance.md#cbo-160) |
| CBO-161 | Corpora and tables run inside one test identity; test mass and burden | Current mechanism | [Test and assurance](test-and-assurance.md#cbo-161) |
| CBO-162 | Benchmarks use Criterion for scheduling and keep the measurement beside it | Current mechanism | [Test and assurance](test-and-assurance.md#cbo-162) |
| CBO-163 | Shared test support is included by path, with repeated fixture assembly | Current mechanism | [Test and assurance](test-and-assurance.md#cbo-163) |
| CBO-164 | CPU team ownership spans Rayon, OpenMP, MKL and FERAL with no single owner | Current mechanism | [Lifecycle](execution-lifecycle-and-observation.md#cbo-164) |
| CBO-180 | Durable execution owns a complete claim-to-settlement engine | Current mechanism | [Lifecycle](execution-lifecycle-and-observation.md#cbo-180) |
| CBO-181 | Telemetry delivery has components but no composed passive health/query operation | Proposed | [Lifecycle](execution-lifecycle-and-observation.md#cbo-181) |
| CBO-195 | Native evidence is mapped field by field across the local protocol | Current mechanism | [Tooling](developer-tooling.md#cbo-195) |
| CBO-196 | Public API contracts mirror rustdoc shapes through a JSON intermediate | Current mechanism | [Tooling](developer-tooling.md#cbo-196) |

## Coverage and limits

All 142 formerly live entries had an owning reconsideration. The 14 topic pages record which
representative operations, callers and test bodies were freshly read and which detailed evidence
remains inherited from unchanged source at the October 30 baseline. That is broad responsibility
coverage, not a line-by-line source census or whole-program reachability proof.

- **Changed engineering/host surfaces.** Native discovery and completed-report reconciliation,
  same-build binary capture, fixture-input preparation, native resource facts, generation retention,
  source/document associations, host-local protocol mappings and API-contract projection were inspected.
  Completed Plan59–63 evidence was used for context without rerunning its qualification.
- **Previously shallow complete operations.** Durable readiness/claim/start, publication handoff,
  finish/termination, writer fencing and recovery outcome projection were read as a complete operation.
  Telemetry settings, host/providers, collector configuration and passive service/checkpoint readers
  were traced as a composed pipeline; no live ingestion/query journey was attempted.
- **Unchanged science and data.** Specialization fragments, selected-function composition, compact
  native assembly, library mathematics, solver adapters, normalization/diagnostics, provider boundaries,
  canonical store/result paths and capability gaps received representative rechecks. Deeper restore,
  full rewrite, full trajectory equivalence, every transaction and every native adapter remain outside
  this pass. Scientific scope limits are retained rather than converted into implementation commitments.
- **Assurance.** Representative fault seams, direct-poll stop tests, transport custody state,
  property/framing tests, shared fixtures and benchmark accounting were read. No test suite, fuzzing,
  schedule exploration, scientific reference or benchmark ran. A source test name is not a passing result.
- **Historical evidence.** Counts, host snapshots, upstream release/commit comparisons and previous
  probes retain their original date and conditions. No current host-unit, filesystem capability or
  upstream-currency claim follows from them. Generated outputs, external reference implementations
  and vendor internals were not exhaustively audited; authored generators and selected existing deltas
  supplied representative context.
- **Uncertainty and demand.** Negative source/name/import searches do not cover all trait dispatch,
  macro expansion or external embedders. Public-but-unconsumed claims are bounded accordingly.
  Proposed capabilities often name a plausible user archetype without recorded recurring demand.
  No performance, memory, contention or benefit measurement was made.
- **Library stage.** Technical library names and older capability notes are leads. Current library
  applicability, exact-version feature support and complete alternatives were not assessed here.
  No library absence or unsuitability is inferred from an empty lookup.

The full CLI was considered as a toolkit, not a coverage checklist. Passive source evidence was
sufficient for the retained questions; no audit-owned semantic session, acquired API world, compiler
probe or managed service was created, so no service teardown or selector restoration was required.

## Independent challenge

A fresh design reviewer independently checked the two proposed retirements (CBO-012/064), four new
opportunities (180/181/195/196), and selected consequential premises (002/048/049/054/071/156).
Scientific sampling covered 140/146/152 and 095/106; 099/121 qualification wording was read without
renewing the complete consumer census or upstream comparison. This is independent sampling, not
endorsement of the other entries or an exhaustive durable-state/failure-window audit.

Four corrections were integrated after coordinator verification: historical retirement links now use
publication `4b48509b5` rather than inspected source `c8244cff5`; removed `GenerationUse` no longer
appears as a current CBO-054 mechanism; CBO-156 acknowledges canonical interpretation/schema and
module/incarnation admission instead of claiming a general semantic-checking gap; and CBO-071 now
traces the native Study query through retained capture custody and connection drain. The latter
keeps public investigation capture distinct from internal canonical freeze/protection writes.

The reviewer supported the bounded retirements and sampled opportunity mechanisms. It did not
reproduce the capture recovery suspicion or a scientific defect, prove every dynamic recovery route
absent, establish directional/exact compatibility equivalence, select a replacement durable engine,
or establish recurring demand for telemetry queries. These remain explicit research limits.

## Maintenance

Refresh this account in place:

- Use changes in contracts, consumers, dependencies and intended work to direct attention.
- Treat this pass's opportunities as leads, not as a boundary on what to inspect.
- Retire an opportunity only on evidence that it is resolved, superseded, has lost relevance, or rested on a corrected
  premise. Keep a one-line anchored pointer, and never reuse its ID.
- A chosen or rejected library does not resolve an opportunity.
- Accepted plans own scheduling and closure evidence. This account links to them rather than copying their status.
