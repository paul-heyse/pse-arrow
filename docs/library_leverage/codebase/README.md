# Codebase opportunities for library leverage

**Account state: whole-codebase audit, 2026-10-30.** This page is the entry point to the living
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
| Inspection date | 2026-10-30 |
| Source baseline | `main` at `c8244cff5`. The host CLI and test scope was under active parallel change during the audit: entries in that scope were read at the baseline, and `9e696e29d` is cited where it already moved one. |
| Included dirty changes | None assessed as evidence. Untracked overviews under `docs/libraries/` (`gix.md`, `rust_analyzers/*`, `rust_testing_library_compendium.md`) were awareness only. |
| Host state | Inspected read-only where a claim depended on it: user systemd units, `~/.config/nextest`, `~/.local/state/pse-arrow`. Dated with this audit and not part of the source baseline. |
| Criteria | Framework edition 0.2, base patterns edition 0.2 (`LOP-01`–`LOP-12`), [additional patterns](../assessment-patterns.md) revision 4 consumed, refined to revision 5 (`LOP-X015`–`LOP-X022` added) |
| Prior account | Inspected at `dbc4c9436`, refreshed at `4f52fd239`, published at `93946f572`. Used only after blind discovery, as a set of claims to test. |
| Evidence level | Mechanisms are *Interface-checked* by static reading at the baseline. Every value claim is *Proposed*. Nothing is *Measured*. Nothing was built, tested or run for this audit. |

**Method.** Thirteen responsibility passes and three lenses ran independently and read-only, without
access to the prior account, the library and review collections, Plan 57 or the diagnostic audit
accounts. The lenses were whole-operation traces across crates, dependency-first inversion from the
manifests, and capability gaps against scope and consumers. Each pass then closed its largest coverage
gaps while still blind, and only then judged the prior entries in its area against current code. The
coordinator read every report, settled conflicts between passes, verified decisive and surprising
claims directly against source, and read the diagnostic accounts for interaction and counterevidence
only. A fresh design reviewer then challenged the draft from source without the coordinator's verdicts;
the coordinator verified its corrections against source and applied them. See
[Independent challenge](#independent-challenge). Topic pages were drafted from the pass reports by separate
writers under the coordinator's disposition, and the coordinator wrote this page.

Blindness was imperfect, and three leaks are known: one pass saw a Plan 57 filename in a directory
listing, and two passes read prior text before finishing their gap-closing edits. Those reports mark it.

## Disposition of the prior account

The prior account named 52 current opportunities at its baseline. Against this baseline:

| Outcome | Count | IDs |
| --- | --- | --- |
| Refreshed in place | 31 | Every ID in the [index](#opportunity-index) below `CBO-054` |
| Retired on evidence | 21 | `CBO-006`, `007`, `009`, `014`, `015`, `017`, `018`, `020`, `023`, `024`, `025`, `026`, `029`, `031`, `033`, `041`, `043`, `044`, `047`, `050`, `053` |
| New | 111 | `CBO-054`–`CBO-164` |

A retired ID keeps a one-line anchored pointer on its home page, with its reason, evidence, successor
and a permalink to its last description, so that existing references in other collections still
resolve. No ID was reused. `CBO-032` retired earlier, on its corrected premise.

Most of this movement has one cause. In the 287 commits after the prior publication, `600d4df30`
retired the Python product and most Python tooling, `bee1a2271` replaced the vendored SurrealDB SDK with
an owned client over the protocol crate, `443ca3ecb` unified execution context and native-thread custody,
and `pse-host` arrived as about 68k lines of new Rust. Several earlier opportunities were resolved by that
work, which is the framework's intended outcome and also a sign that this account should be read for what
it says now.

## System picture

pse-arrow is a clean-room process-systems core of about 1.04M lines of non-generated Rust under `crates/`
and `xtask/`, including in-crate tests, plus about 405k lines of generated Rust, 100 authored `.pse`
packages and about 11k lines of independent Python references and tests. The
[architecture overview](../../authoritative_design/sections/architecture-overview.md) assigns operations
to libraries by fit. Symbolica and Numerica own algebra, derivatives and evaluators. Class-specific native
solvers own iteration and factorization. Salsa owns pure semantic reuse. Arrow and DataFusion own columnar
and relational boundaries. SurrealDB owns canonical persistence. The native host owns process and service
control.

**Read as strong at this baseline, with limits.** The claim here concerns reimplementation, not depth: solver and library work is delegated rather than rebuilt, and depth stays open in the entries named. Solver adapters drive libraries through their own APIs, such as HiGHS IIS, ranging and presolve, SCIP expressions and exact mode, and POUNCE retained symbolics. The HiGHS and IBEX statements are adapter-level, since the HiGHS solve body was read representatively and IBEX sources were not available. Depth questions are in [CBO-120](native-solver-integration.md#cbo-120), [CBO-122](native-solver-integration.md#cbo-122) and [CBO-150](capability-gaps.md#cbo-150). Derivatives are library-owned, with `SymbolicComposition` as a good example. Pure and effectful code are separated, with `pse-modeling` free of stores and solvers. The host's cgroup, `AddRef`, `BindsTo` and pidfd use is the mechanism rather than a supervisor rebuilt beside it, with open entries on the same subject at [CBO-004](cross-cutting-mechanisms.md#cbo-004) and [CBO-048](developer-tooling.md#cbo-048). Honest withholding, typed availability and original-space qualification are contracts worth keeping. Many generic-algorithm suspicions dissolved once bodies were read, in the code that was read closely. A strength recorded here was read, not tested, and remains open to later comparison.

## Where opportunities cluster

Eleven shapes account for most of the list. They are clusters for reading, not recommendations, and they interact.

1. **A real guarantee, prepaid by hand against many suppliers' internals.** Admission before allocation is a genuine contract. Prediction mirrors of Symbolica, Arrow, DataFusion, Salsa, petgraph, pounce-presolve, quick_cache and SUNDIALS sit beside hand-walked retained sizes. Some suppliers' own estimates are already consumed (faer scratch, FERAL, IBEX workspace bytes). A bump is ordinary agent judgment by policy, yet there is no general drift detector: some Symbolica paths reconcile after the fact, and none was found on the Arrow and DataFusion formulas. Whether suppliers can publish predictions is an open research question. [CBO-001](cross-cutting-mechanisms.md#cbo-001), [CBO-086](cross-cutting-mechanisms.md#cbo-086), [CBO-021](mathematics-and-compilation.md#cbo-021), [CBO-125](native-solver-integration.md#cbo-125).

2. **Reads that pay write protocols.** A status poll is a durable, globally gated capture that the gate's design reasons do not obviously require at its size, and it has a suspected crash window. Opening a result writes a lease. A study occurrence takes many round trips. [CBO-071](persistence-and-transport.md#cbo-071), [CBO-072](persistence-and-transport.md#cbo-072), [CBO-073](persistence-and-transport.md#cbo-073), [CBO-016](execution-lifecycle-and-observation.md#cbo-016). Pattern [LOP-X016](../assessment-patterns.md#lop-x016).

3. **A large program in the store's language without its toolchain.** The canonical state machine is about 423 KB of generated SurrealQL with 205 functions, parse-only tooling and hand correspondence at the Rust boundary, with guarded logic split across stored and inline text. [CBO-046](registry-generation-and-python-boundary.md#cbo-046), [CBO-010](persistence-and-transport.md#cbo-010), [CBO-066](persistence-and-transport.md#cbo-066), [CBO-067](persistence-and-transport.md#cbo-067), [CBO-068](persistence-and-transport.md#cbo-068). Pattern [LOP-X015](../assessment-patterns.md#lop-x015).

4. **Dense scientific data forced through row forms.** A solver's sample-by-output matrix becomes typed rows with about seven dynamic downcasts per sample row, then uncompressed IPC (a documented choice), then a full-column decode of a bounded block to read one scalar. The prior account's dense concerns were resolved downstream of this; the production side was not examined then. [CBO-075](result-data-paths.md#cbo-075), [CBO-076](result-data-paths.md#cbo-076), [CBO-078](result-data-paths.md#cbo-078), [CBO-077](result-data-paths.md#cbo-077), [CBO-008](result-data-paths.md#cbo-008).

5. **Forms with no reader.** The retirement of Python left uniform emission and guards in place: 7 of 90 canonical Arrow modules have a production consumer by import walk, engine SQL and DML doors have none, and residue remains in manifests and on the host. The question for each is whether to retire it or give it a consumer. [CBO-153](registry-generation-and-python-boundary.md#cbo-153), [CBO-154](registry-generation-and-python-boundary.md#cbo-154), [CBO-065](developer-tooling.md#cbo-065), [CBO-069](persistence-and-transport.md#cbo-069). Pattern [LOP-X020](../assessment-patterns.md#lop-x020).

6. **Stated guarantees with no executor.** This is a different finding from an unused form, and deleting is not the default. The architecture and Plan 28b describe the durable body tier as implemented, while `pse-compiler` captures payloads for it and nothing in production consumes it. §4.3 says durable opening verifies a recorded witness, while `verify_recorded` has no production caller. Each is a conflict between owners and code, and the plan-completion or decision routes own part of it. [CBO-099](modeling-and-preparation-workflows.md#cbo-099), [CBO-156](registry-generation-and-python-boundary.md#cbo-156).

7. **The supplier relationship is itself a decision surface.** The vendored and forked set ranges from build freshness and admission hooks to a true arithmetic fork, and no single account covers it. faer alone gave up SIMD lanes for a no-FMA policy that is enforced per supplier without a stated coverage. The POUNCE fork is 441 commits behind upstream, unassessed. Diffsol has a small delta and workarounds written against its behaviour. Fourteen patches bind the server. Macro suppliers are patched one at a time for build purity. [CBO-116](native-solver-integration.md#cbo-116), [CBO-121](native-solver-integration.md#cbo-121), [CBO-129](native-solver-integration.md#cbo-129), [CBO-027](native-solver-integration.md#cbo-027), [CBO-028](native-solver-integration.md#cbo-028), [CBO-070](persistence-and-transport.md#cbo-070), [CBO-061](developer-tooling.md#cbo-061), [CBO-106](mathematics-and-compilation.md#cbo-106). Pattern [LOP-X017](../assessment-patterns.md#lop-x017).

8. **Stable work prepared at the wrong lifetime, or tracked by a second engine.** Salsa sits under a driver that feeds results back as inputs, source selection and physical premises each carry their own dependency tracking, and native preparation is repeated per segment, per correction and per point even where a fixed pattern is declared. Library process-global state and per-supplier team control turn independent lanes into exclusive or separately governed ones. Reuse exists and is good where it applies; the opportunity is unevenness. [CBO-107](mathematics-and-compilation.md#cbo-107), [CBO-087](modeling-and-preparation-workflows.md#cbo-087), [CBO-108](mathematics-and-compilation.md#cbo-108), [CBO-096](modeling-and-preparation-workflows.md#cbo-096), [CBO-127](native-solver-integration.md#cbo-127), [CBO-126](native-solver-integration.md#cbo-126), [CBO-030](native-solver-integration.md#cbo-030), [CBO-118](native-solver-integration.md#cbo-118), [CBO-164](execution-lifecycle-and-observation.md#cbo-164). Patterns [LOP-X018](../assessment-patterns.md#lop-x018), [LOP-X019](../assessment-patterns.md#lop-x019).

9. **Mechanisms that stop short of a product.** The hard numerics are often done, and the output is confined to an internal role. Egobox trains models and discards them. Arclength continuation yields starts, not branches. A covariance withheld at a bound has no continuation. The sampled-study middle composes well and its ends are missing. These are capabilities, so demand is the open question, and none is accepted scope. [CBO-135](numerical-workflows.md#cbo-135), [CBO-034](numerical-workflows.md#cbo-034), [CBO-036](numerical-workflows.md#cbo-036), [CBO-133](numerical-workflows.md#cbo-133), [CBO-140](capability-gaps.md#cbo-140), [CBO-141](capability-gaps.md#cbo-141), [CBO-119](native-solver-integration.md#cbo-119). Pattern [LOP-X022](../assessment-patterns.md#lop-x022).

10. **The authoring language has a parser but not a toolchain.** Authoring is the product's main write path. It has no lossless tree, no recovery and one failure per cycle; the `assign_ids` rewrite drops `.pse` comments; spans are held but do not reach human output; and syntax is hand-written in several forms beside a registry that already declares payload shape. [CBO-113](authoring-and-identity.md#cbo-113), [CBO-114](authoring-and-identity.md#cbo-114), [CBO-038](authoring-and-identity.md#cbo-038), [CBO-037](authoring-and-identity.md#cbo-037), [CBO-115](authoring-and-identity.md#cbo-115).

11. **Assurance held by enumeration.** A guarantee about a population is discharged by listing its members by hand: 158 source-hash-bound I/O reviews, one patch per macro supplier, one task-local flag per uncertainty window, one fuzz target for many hostile decoders. The list covers what someone remembered. The platform may in some cases observe, confine or sweep the population instead, though observation covers only the executed branch. [CBO-060](developer-tooling.md#cbo-060), [CBO-061](developer-tooling.md#cbo-061), [CBO-157](test-and-assurance.md#cbo-157), [CBO-158](test-and-assurance.md#cbo-158), [CBO-161](test-and-assurance.md#cbo-161). Pattern [LOP-X021](../assessment-patterns.md#lop-x021). The unrouted fuzz, mutation and coverage tools are a related gap at [CBO-159](test-and-assurance.md#cbo-159).

**How they interact.** Clusters 1 and 7 compound: every supplier move carries the prediction mirrors, so the cost of holding a pin and the cost of moving one are both partly the accounting. Cluster 5 precedes format work on the registry products in [CBO-040](registry-generation-and-python-boundary.md#cbo-040), because dropping unconsumed products shrinks what a representation change must carry; it does not shrink cluster 4, whose relations have real consumers. Clusters 2 and 8 both limit lane throughput, through independent causes, and neither is measured. Cluster 9's sampled workflows may be held back by the study protocol in [CBO-073](persistence-and-transport.md#cbo-073) and by dense production in cluster 4.

**Relation to the diagnostic account.** The [design-principle audit](../../design_review/audit/README.md) at `f83bac039` reports six integrated causes, and several overlap in subject: the fine-grained protocol per study point, machinery outliving consumers, and the orchestration crate interpreting lower owners. The two accounts were kept independent. This one asks where a capability could carry a responsibility; that one judges design alignment. Neither substitutes for the other, and agreement between them is not independent proof, since both read the same code.

## Items that may be defects rather than leverage

These came up in the audit. They are recorded in their entries, and they are stated here, ordered by consequence, because they should not wait for a leverage decision. **All are from static reading or host inspection; none was run or reproduced.** Their owners and severity are for the maintainer to decide.

- **Capture gate crash window.** A process dying between `open_capture` and `commit_capture` appears to leave the global gate set, refusing all 64 publication sites and further captures. No complete recovery route was found: nothing reads the stored key, and a different custodian needs the dead owner drained, which only that live owner can do. Each status poll opens this window ([CBO-071](persistence-and-transport.md#cbo-071)).
- **Comment loss in `assign_ids`.** On `.pse` documents, `assign_ids_with` ends in `render(&rows)` and `edit.rs` takes that whole string as `DocumentEdit.after`, so authored comments are lost on the main write path ([CBO-113](authoring-and-identity.md#cbo-113)). Confirmed at both ends by reading.
- **Supplier normalization defect.** `simplify_exp_log` in the vendored Symbolica can overwrite its base for a narrow two-sum shape. The `Exp` barrier protects staged evaluation, and flattening routes may reach the defect. Whether any authored body produces the shape is not established ([CBO-106](mathematics-and-compilation.md#cbo-106)).
- **Inert test scheduling.** `.config/nextest.toml` declares `scientific` and `store` groups, with comments promising shared budgets and exclusion, and nothing assigns them since `600d4df30` removed the projection ([CBO-049](developer-tooling.md#cbo-049)). Plan completion rows still record this work as done.
- **Stated witness with no caller.** §4.3 says durable opening first verifies a recorded witness, and `verify_recorded` has no production caller ([CBO-156](registry-generation-and-python-boundary.md#cbo-156)). The canonical interpretation check may discharge the meaning by another route; that was not established.
- **Deleted arithmetic control.** `vendor/faer-arithmetic.md` instructs rerunning its controls on a supplier update and names `scripts/faer_arithmetic.py`, which `600d4df30` deleted. Probes and unit tests remain, and the documented LLVM inspection route does not ([CBO-116](native-solver-integration.md#cbo-116)).
- **Differing window admission.** `WindowCoverage::admit_storage` returns early on the linear scope, while `foreign.rs::Coverage` keeps a parent call "to preserve strict unknown-bound refusal". Whether this is a gap or justified was not determined ([CBO-083](execution-lifecycle-and-observation.md#cbo-083)).
- **Live retired service (operational).** A user unit still runs a retained copy of the deleted `surreal_server.py` from a fixture directory, among 83 enabled `pse-surreal-*` units. It may be deliberate, and it costs a resident process ([CBO-065](developer-tooling.md#cbo-065)).

## Input to the next stage

**Library assessments and comparative reviews.** The [library collection](../libraries/README.md) and
[review collection](../reviews/README.md) consumed the prior account at earlier baselines, and they were not
edited here. Their references to retired IDs resolve to the pointers above, and their content for those IDs
should be read as historical. Every new ID has no library assessment yet. Plan 57 is superseded and is not a
backlog.

The next `leverage-library-assessment` refresh should start from these questions, ordered by consequence and not
by implementation order:

1. **Prediction seams.** For each supplier in cluster 1: is there a symbolic estimate, budget, fallible allocation
   or allocator hook that could replace a mirror? Could the pinned `allocation-counter` act as a drift detector even
   where it cannot own admission?
2. **SurrealQL tooling.** What can `surrealdb-sql` and `surrealdb-syn` expose at 3.3.4 for signatures, `THROW`
   inventory and typed call stubs? What does the server's `Run` RPC do about transactions?
3. **Read paths on the store.** Can a single-statement read deliver the consistency an observer consumes? Does
   SurrealDB `LIVE` work as a wake hint on the selected transport with bounded memory? What is the semantics and cost
   of `REFERENCE ON DELETE REJECT` under SSI in MX 1.0?
4. **Dense columnar production.** Can generated relations accept prebuilt columns under the same field checks? Which
   of `FixedSizeList`, dictionary, run-end and IPC compression survive bounded IPC admission and DataFusion pruning?
   Can `FileDecoder::with_projection` compose with field-level admission?
5. **Supplier moves.** Which of POUNCE's 441 commits resolve a recorded limitation, especially presolve multiplier
   return? What do Diffsol 0.17 and 0.18 retire? What promise does faer's nonfused delta serve, which consumers rely on
   it, and can `pulp` select it without a source delta?
6. **Incremental engines.** Can Salsa express source-order scoping and transient refusal in its pull model, or export
   dependency edges? Which frameworks offer branchable immutable state?
7. **Language toolchain.** Can a lossless tree (`rowan`, `cstree`) sit beneath the current cursor without changing
   rows? What LSP base composes with the existing Salsa workspace?
8. **Sampling and surrogate products.** Which low-discrepancy crates meet the recorded-point contract? Does
   `egobox-moe` expose reloadable models with derivatives usable in an NLP? Can symbolic surrogates lower directly to
   Symbolica mathematics?
9. **Assurance routes.** Can a process tree's file accesses be observed or confined deterministically for build
   scripts? Which of `fail`, `shuttle`, `loom`, `cargo-mutants --in-diff` and structured fuzzing fit real owners?

## Opportunity index

The Status column separates a *current promise* (an existing contract, with only its realization in question), a
*current mechanism* (an implementation choice with no stronger promise attached), *accepted planned* work, a
*declared target, undelivered* (architecturally in scope, with no accepted plan) and a *proposed* addition. "Weak" marks an opportunity whose counterevidence is strong, retained so that later work can
confirm or retire it.

| ID | Opportunity | Status | Home |
| --- | --- | --- | --- |
| CBO-001 | Prepaid prediction mirrors library internals | Current promise | [Cross-cutting](cross-cutting-mechanisms.md#cbo-001) |
| CBO-002 | Residual bounded and counting sinks (weak) | Current mechanism (weak) | [Cross-cutting](cross-cutting-mechanisms.md#cbo-002) |
| CBO-003 | Owned private files and durable publication have no shared owner | Current mechanism | [Cross-cutting](cross-cutting-mechanisms.md#cbo-003) |
| CBO-004 | Process identity and supervision: name-based scans beside owned handles | Current mechanism | [Cross-cutting](cross-cutting-mechanisms.md#cbo-004) |
| CBO-005 | Local IPC admission around Arrow's decoders needs check-by-check upstream tracking | Current promise | [Result data](result-data-paths.md#cbo-005) |
| CBO-008 | Seeds, progress and receipts as JSON payloads beside typed relations | Current mechanism (weak) | [Result data](result-data-paths.md#cbo-008) |
| CBO-010 | Guarded scripts: logic split between stored functions and inline text, reparsed per call | Current mechanism | [Persistence](persistence-and-transport.md#cbo-010) |
| CBO-011 | Single-flight, I/O gates and load admission beside caching libraries | Current mechanism | [Persistence](persistence-and-transport.md#cbo-011) |
| CBO-012 | Imperative nested-occurrence traversal beside the plan route (weak) | Current mechanism (weak) | [Persistence](persistence-and-transport.md#cbo-012) |
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
| CBO-048 | The host uses the manager but not its accounting and snapshot facts | Current mechanism | [Tooling](developer-tooling.md#cbo-048) |
| CBO-049 | Test execution beside nextest: inert groups, binary capture, repeated resolution | Current mechanism | [Tooling](developer-tooling.md#cbo-049) |
| CBO-051 | Tooling interchange and documents read through several parsers | Current mechanism | [Tooling](developer-tooling.md#cbo-051) |
| CBO-052 | Two test and check front doors with different durability | Accepted planned | [Tooling](developer-tooling.md#cbo-052) |
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
| CBO-064 | Knowledge and evidence stores: narrow query capability, and whether they earn their keep | Proposed | [Tooling](developer-tooling.md#cbo-064) |
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
| CBO-078 | Selected result tables re-walk the whole report once per relation | Current mechanism | [Result data](result-data-paths.md#cbo-078) |
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
| CBO-099 | The durable body tier and child route have no production consumer | Current promise, unconsumed | [Modeling workflows](modeling-and-preparation-workflows.md#cbo-099) |
| CBO-100 | Derivative product construction has several routes | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-100) |
| CBO-101 | C++ compiler driver and code cache beside an in-process JIT | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-101) |
| CBO-102 | Case evaluation runs identical programs one instance at a time | Proposed | [Mathematics](mathematics-and-compilation.md#cbo-102) |
| CBO-103 | Polynomial and affine recognition repeated by consumers | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-103) |
| CBO-104 | Support analysis is a generic sparsity propagator with repeated symbolic work | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-104) |
| CBO-105 | Several interval and constant sweeps over one factorable DAG | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-105) |
| CBO-106 | The exponential barrier compensates for a live supplier defect | Current mechanism | [Mathematics](mathematics-and-compilation.md#cbo-106) |
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
| CBO-121 | The POUNCE fork is far behind upstream | Current mechanism | [Native solvers](native-solver-integration.md#cbo-121) |
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
| CBO-160 | Property testing stops at leaf crates | Proposed | [Test and assurance](test-and-assurance.md#cbo-160) |
| CBO-161 | Corpora and tables run inside one test identity; test mass and burden | Current mechanism | [Test and assurance](test-and-assurance.md#cbo-161) |
| CBO-162 | Benchmarks use Criterion for scheduling and keep the measurement beside it | Current mechanism | [Test and assurance](test-and-assurance.md#cbo-162) |
| CBO-163 | Shared test support is included by path, with repeated fixture assembly | Current mechanism | [Test and assurance](test-and-assurance.md#cbo-163) |
| CBO-164 | CPU team ownership spans Rayon, OpenMP, MKL and FERAL with no single owner | Current mechanism | [Lifecycle](execution-lifecycle-and-observation.md#cbo-164) |

## Coverage and limits

Coverage is by responsibility, not by line. Every tracked source group had an owning pass, and each topic page
states what was read closely and what was only skimmed.

- **Read closely.** Host launch, ownership and test workflow; generation, `producer_identity` selection and
  documentation tooling; the canonical client, script envelope and capture paths; result reading and publication;
  fit covariance and profile logic; the Symbolica boundary; the Diffsol route and the main `idas.rs` paths; solver
  gates; `pse-structural` and `pse-study-module`.
- **Read representatively, with real gaps remaining.**
  - `pse-modeling` `instantiate_body`, `specialize/{functions,process,operations,transformations}`,
    `resolved/{selection,region_composition,fragment}`: about 15k lines by signature only.
  - `workflow/strategies/` recycle bodies, the `horizon/` loop, `replay` reconstruction, `study_diagnostics`,
    `reproducer`, `review_context`, `native_deployment*`.
  - `canonical_selection`, `canonical_studies` type and progression regions, and generated retention SurrealQL.
  - `pse-engine` provider and resolution modules, `solve.rs` and the HiGHS solve body.
  - The implicit-selection family in `pse-math`: `implicit_products` and `implicit_regimes` (`prepare_selection` unread) representatively, `assembly_entries` skimmed.
  - Test code as a subject: about 224k lines were read for shape by search, not as a census. Counts are heuristics.
- **Not framed as a whole.** No pass assessed the durable execution engine (`workflow/durable.rs`, `worker.rs`, `study.rs`: claim, lease, heartbeat, journal, retry, recovery) against durable-execution or job-queue libraries; pieces appear in [CBO-016](execution-lifecycle-and-observation.md#cbo-016), [CBO-073](persistence-and-transport.md#cbo-073) and [CBO-079](execution-lifecycle-and-observation.md#cbo-079). The telemetry pipeline (`telemetry_service`, OTLP, `tokio-metrics`) appears only in fragments ([CBO-013](execution-lifecycle-and-observation.md#cbo-013), [CBO-084](execution-lifecycle-and-observation.md#cbo-084), [CBO-085](execution-lifecycle-and-observation.md#cbo-085)). CPU team ownership was found only by the challenge ([CBO-164](execution-lifecycle-and-observation.md#cbo-164)), which suggests other cross-cutting responsibilities could still be missing.
- **Not inspected.** Upstream bodies of vendored suppliers beyond their deltas; IBEX sources, which are absent from
  the solver prefix; POUNCE fork commit bodies, classified by subject only; `.config/*-reviews.json` entry by entry.
  `external/`, `build/`, `target/` and generated outputs were excluded by design, though generators were in scope.
- **Absence claims.** "No caller" and "no production consumer" come from name and import searches over stated scopes.
  Trait dispatch, macros and external embedders of a `pub` API can escape them.
- **Measurement.** No performance, memory or contention measurement was made. Every statement about cost is
  structural. Some opportunities may prove immaterial once measured, and several entries say so.
- **Library research.** Library names are leads only. Where a pass checked installed source, it says so; that
  establishes presence, not fit.
- **Demand.** Capability entries name a plausible consumer. Demand from similar systems is a prompt, not proof, and
  none of these is accepted scope.

## Independent challenge

A fresh design reviewer read the draft and tested about 30 entries in depth from source, with the rest sampled. It reported 15 corrections and 12 judgment calls. The coordinator verified the decisive corrections against source and applied them: refreshed status labels, a corrected example for [CBO-106](mathematics-and-compilation.md#cbo-106), a deeper recovery account and the ADR-0230 basis for [CBO-071](persistence-and-transport.md#cbo-071), softened claims in this page, three corrected interactions, a new [CBO-164](execution-lifecycle-and-observation.md#cbo-164), and splitting two clusters that joined different obligations.

Its counts and most retirements held. Its independent import walk found the same 7 of 90 modules. It could neither confirm nor refute whether IBEX's random draws change proof outcomes ([CBO-117](native-solver-integration.md#cbo-117), which was left out of the defects list for that reason), or whether any authored body produces the [CBO-106](mathematics-and-compilation.md#cbo-106) shape. The other 110 or so entries rest on their pass and the coordinator's reconciliation, without this second reading.

## Maintenance

Refresh this account in place:
- Use changes in contracts, consumers, dependencies and intended work to direct attention.
- Treat this pass's opportunities as leads, not as a boundary on what to inspect.
- Retire an opportunity only on evidence that it is resolved, superseded, has lost relevance, or rested on a corrected
  premise. Keep a one-line anchored pointer, and never reuse its ID.
- A chosen or rejected library does not resolve an opportunity.
- Accepted plans own scheduling and closure evidence. This account links to them rather than copying their status.
