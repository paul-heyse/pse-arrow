---
title: "Agent effectiveness: reasoning, changes, experiments and evidence"
date: 2026-10-23
doc_role: review
doc_owner: docs/functional_reviews/README.md
doc_retention: while-dependent
doc_topics: [environment, validation, design, diagnostics, libraries]
---

# Agent effectiveness: reasoning, changes, experiments and evidence

## Judgment and scope

**Proposed:** extend the substantial tooling already delivered by Plans50–51 into a coherent
working environment for understanding the system, designing changes, making reliable edits,
challenging correctness, running experiments and reconstructing decisions. The strongest next
directions are semantic refactoring through the existing language bridge, source-linked task
context, stronger behavioral test techniques, resumable qualification, and better composition of
existing execution and evidence interfaces. SurrealDB can provide useful derived graph, search
and experiment views; it should not become a second authority for source, plans or test outcomes.

The consumer is Codex, including its subagents, working for Paul as sole operator on this
workstation. This review covers ordinary editing and implementation as well as research,
architecture, scientific validation, performance work, maintenance, recovery and handoff. It is
a functional exploration, not a diagnostic audit or a claim that every proposed capability fixes
a present defect. New activities can justify an addition without a recorded incident.

The operator selected **free-to-run additions**, preferably downloaded and local. Existing Codex
access is the baseline. No recommendation depends on another paid subscription, metered model
API or hosted database. Local model weights and services are candidates, not prerequisites.
Machine-specific optimization is welcome. Ordinary build defaults remain sixteen compiler jobs
and the selected unbounded build mode; this review proposes no new global memory budget or
concurrency governor. Generated data can be regenerated. Historical artifacts are retained only
when a concrete investigation or current reference needs them.

All O01–O30 below are **Proposed** functional directions. Their mechanisms have differing levels
of interface evidence; none is newly implemented, tested or measured by this review. Favorable
judgment does not select implementation scope, modify rules, reopen completed plans or authorize
installation. The rule questions at the end remain proposals for subsequent planning.

## Baseline, method and evidence limits

**Interface-checked:** product and tooling inspection began at
`0f0f5a67abf847bdbb14bb691a211a375ca7a1cd`. Concurrent process work landed as
`e413aa77ef3774dbd100e71d135846a1d8a8af65` during the inquiry; its test-assessment guidance and
test-audit routing were reconciled before publication. ADR-0221 remains proposed. This review
neither conducts that audit nor treats a proposed ADR as accepted architecture.

The selected standard is Core3.4, Efficient Architecture Heuristics1.0, Process Simulator1.5,
the pse-arrow binding and the newly included Test Assessment1.0 companion. Scientific truth,
independent expectations, explicit force-validation, resource ownership and truthful completion
shape the proposals. No broad diagnostic or test audit was needed to explore functional value.

The inquiry combined source inspection, existing operator guides and plan Outcomes, sanitized
recent session evidence, three independent bounded reviewer contributions, and primary tooling
documentation. The supplied [SurrealDB tooling reference](../libraries/surrealdb_tooling.md) was
read **end to end**, including its capability inventory, compatibility cautions and source
directory. Supplied upstream links were followed selectively for consequential details; its
inventory is orientation rather than proof that every advertised feature works in our custom
server or pinned client.

Recent-history inspection covered the main long-lived session from **2026-10-22 18:45:30.753 UTC**
(the earlier review's cutoff) through **2026-10-23 23:08:00.842 UTC** (this review request).
Streaming that file and deduplicating record type plus call ID produced 4,526 tool-call/output
records, representing 2,263 calls. These are record counts, not independent incidents or measured
labor. Repeated polls, quoted output and pattern matches were investigation leads. The existing
bounded retrospective helper was also inspected and exercised read-only: its recent-directory
selection omitted this older, still-active session, and its result was truncated. Consequently
it cannot support a complete population claim. O28 addresses that specific coverage limitation.
Raw conversation text, credentials and full tool arguments are not republished here.

The observations and primary-source checks establish plausible integration routes, not runtime
compatibility or speedups. No new service, model, database, build, scientific campaign or product
test was started for this review. Documentation publication checks establish documentation only.

## What already exists

The [Plan50 Outcome](../plans/50-codex-effectiveness-and-diagnostics.md#outcome) owns delivery of
all 32 packages and selected O01–O20 routes from the
[earlier review](codex-effectiveness-and-diagnostics-2026-10-22.md). The
[Plan51 coordinator](../plans/51-engineering-workspaces-and-evidence.md) owns the completed
project, scientific-data, research/evidence, lifecycle, recovery and portable-exchange scope.
Plan49's selected scope is complete and retired; [current work](../plans/README.md) routes to
the actual qualification owners and independent remaining Plans28/32/40. These are foundations,
not a backlog to implement again.

Source-inspected foundations include:

| Foundation | Existing capability | Opportunity beyond it |
|---|---|---|
| Environment and attachment | `pse-env`, selected-store Codex sessions, pinned tools, native routes | Explain and compose context without repeatedly reconstructing it |
| Language intelligence | mcpls0.8.0, Rust analyzer and Pyrefly definitions, references and call hierarchies | Refactoring, code actions and cross-language declared boundary paths |
| Source scoping | `workspace_context`, paged native owners and recipes, bounded text boundary leads, `affected` reverse dependencies | Semantic ownership paths, design alternatives and stronger behavioral selection |
| Builds and processes | Activity, admission, build-stage/cache observations, stable product-test readers | Event-driven observation, topology/pressure interpretation and resumable combinations |
| Database development | Actual custom-server parser/schema/query/plan feedback, SurrealKit parity and actor tests | Derived research/search views and cheaper representative disposable fixtures |
| Diagnosis | Finite OTel captures, GDB, perf, py-spy, task dump, Heaptrack, Memray and rr | Application-to-server correlation and smaller reproducible scientific failures |
| Correctness | Proptest, independent scientific references, finite-difference checks, coverage and focused Miri | Metamorphic/stateful scenarios, mutation, fuzzing and reference workbenches |
| Evidence | Supplied-source Arrow/DataFusion analysis, disposable Surreal projection, exact cuts and incomplete facts | Paired comparisons, experiment navigation and agent-readable compositions |
| Documentation | Native owners, lifecycle metadata, published Pagefind search, Git history | Headless authority-aware retrieval and concise task continuity |

These claims are **Implemented, source-inspected** here. Their actual prior **Tested** boundaries
remain in the owning Outcomes and [qualification guide](../dev/validation-assessment.md); this
review does not repeat or broaden them.

### Recent work as opportunity evidence

| Observed pattern or resolved incident | What it supports; what it does not establish |
|---|---|
| Repeated shell inspection, result reading and polling | O01/O15/O26 composition; command frequency alone does not prove wasted time |
| A comment-only Cargo configuration change changed the custom-server installation identity | O17 semantic identity investigation; conservative raw hashing currently has an understandable safety purpose |
| A sixteen-CPU observer was refused inside an outer four-CPU functional allocation | O20/O26 context explanation; the correct build-class route already works |
| A root-owned pytest temporary directory was unusable; an owned build directory was used | O22 private experiment environments and O26 setup context; no blanket host-permission change is needed |
| Feature combinations, composed qualification and source repair required manual resumption/accounting | O16; gate-level reuse already exists and overlapping Cargo drivers remain unsafe |
| Objective-free fitting, role selection, lazy Arrow consumption and stale-parent negative fixtures needed correction | O09/O10/O12 challenges to plausible wrong behavior; these repaired cases are not new outstanding failures |
| Graph query bounds, literal planning and unique source operation IDs needed exact-context diagnosis | O06/O24/O25 source-linked query experiments; existing graph bounds and transport corrections are delivered |
| Cherry-pick conflicts and retirement/reference updates required contextual reconstruction | O01/O29 integration help; no evidence that automatic conflict resolution is safe |
| Recent-directory history scanning missed the main long-lived session | O28 discovery by activity and logical identity, with explicit incomplete coverage |

## Reasoning, context and reliable edits

<a id="O01"></a>
### O01 — Reconstruct a useful task context and its rationale

**Recommend early.** A new turn or subagent should be able to ask “what owns this change, what
has already been decided, what is uncommitted, and what remains uncertain?” and receive a small
source-linked packet. Compose native plan/checkpoint status, current architecture owners, Git
commits and dirty changes, selected result references, active owned processes and open questions.
Use the existing `workspace_context.py`, native scope relationships and documentation lifecycle
routes rather than introducing a second mutable handoff ledger.

Git's [`log -S`, `-G`, `-L` and history traversal](https://git-scm.com/docs/git-log) can recover
why a symbol or contract changed. Explicit conversation-history retrieval can supplement this,
but an old instruction or model summary is historical evidence, never current authorization.
The packet should separate accepted decisions, current observations and unresolved hypotheses.
Its source cut must expose dirty overlays and stale evidence. A representative demonstration is
resuming a partially implemented boundary change with the correct generator, consumers, test
scope and next decision, without rereading a whole prior conversation. Plain source-linked text
is the first realization; persistent retrieval depends on O05/O28 only when that adds value.

<a id="O02"></a>
### O02 — Trace declared meaning across Rust, generated contracts and Python

**Recommend early, initially derived in memory.** Existing LSP navigation works within languages;
`boundary_leads` supplies bounded substring leads. Add a navigable path from an authored boundary
declaration through its generator and catalogue to generated Rust/Python contracts, PyO3 entry
points, facade methods and tests. A useful question is “what changes if this research request
field changes?” The answer should show declared links, semantic references, text leads and
unknown dynamic edges separately.

Inputs already include `for_each_boundary_document!`, boundary/schema registries, xtask schema
generation and `CatalogueDocument`, plus language references. Extend those producers to expose
their real relationships rather than infer all links from names. `affected.py` remains a
conservative crate-level reach estimate. A SurrealDB projection can later support graph queries
and O07, but a persistent universal code graph is unnecessary for a first useful route. Generated
outputs remain disposable; the graph must point edits back to their declarations. Demonstrate a
Rust→Python contract change and a deliberately unresolved string/dynamic reference without
claiming the graph proves complete impact.

<a id="O03"></a>
### O03 — Use semantic rename and code actions through the installed bridge

**Recommend early.** The pinned mcpls bridge exposes more than the already exercised navigation:
[`get_tool_support`, `prepare_rename`, `rename_symbol`, `get_code_actions`](https://raw.githubusercontent.com/bug-ops/mcpls/v0.8.0/book/src/tools/refactoring.md)
can return edits from the language server. Use capability discovery first, preview a coherent
edit set, then apply it through the ordinary owned-change workflow. This could remove much
manual reference enumeration during API cleanup while keeping the actual compiler and targeted
tests responsible for verification.

Runtime support from the selected Rust analyzer and Pyrefly must be checked independently.
Indexing state, dropped edits, truncation, stale documents and invalid source positions must be
visible; a partial edit set cannot be called a completed rename. PyO3 names, serialized fields,
generated outputs and string-based entry points need O02 or explicit treatment. A representative
trial renames one internal Rust API and one supported Python symbol, includes cross-file uses,
and shows refusal or incomplete status for an unsupported case. Serena's richer
[symbol editing](https://github.com/oraios/serena/blob/main/src/serena/tools/symbol_tools.py) is a
candidate if it enables useful body-level work the existing bridge cannot; its second language
service lifecycle is a real cost. Prefer extending the current bridge first.

<a id="O04"></a>
### O04 — Make repeated structural changes with tested codemods

**Recommend for recurring transformations.** Consider ast-grep, already included in bootstrap
tooling, for syntax-aware shape matching, contextual `inside`/`has` constraints, metavariables and rewrites. Its
[rule configuration](https://ast-grep.github.io/guide/rule-config.html),
[rewrite mechanisms](https://ast-grep.github.io/guide/rewrite-code.html) and
[positive/negative tests](https://ast-grep.github.io/guide/test-rule.html) support a much more
precise change than repeated textual substitutions. Examples include moving an error conversion
idiom, correcting a callback construction pattern or updating authored registry syntax.

For Python changes needing import/name resolution and comment preservation, consider LibCST's
[`FullRepoManager`, qualified-name/scope metadata](https://libcst.readthedocs.io/en/latest/metadata.html)
and [codemod/import visitors](https://libcst.readthedocs.io/en/latest/codemods.html). Exact Python3.14
syntax support is a prerequisite, not assumed. Run transformations against generator inputs,
never hand-edit generated paths. Show matched and rejected examples and a preview of all files;
compile and test affected behavior afterward. Keep reusable rules only for transformations or
invariants likely to recur. A one-off bounded script or LSP rename is preferable when a retained
codemod framework would add more maintenance than value.

<a id="O05"></a>
### O05 — Retrieve local knowledge by meaning, authority and freshness

**Recommend lexical/source retrieval first; investigate local semantic enrichment.** Enable
questions such as “where is stale result acceptance prevented?” or “why is this solver class
selected?” across source, architecture, current plans, diagnostic guides and selected history.
Return exact excerpts, source paths/revisions, document roles and current-versus-historical
status. The existing published Pagefind index is a foundation, not a missing capability:
its [search API/configuration](https://pagefind.app/docs/search-config/) is a candidate access
route, with actual headless runtime compatibility still to establish. Its
[Node API](https://pagefind.app/docs/node-api/) supports indexing, not an established server-side
query interface. Compare a fresh source index using [SQLite FTS5](https://www.sqlite.org/fts5.html)
with native SurrealDB BM25 and graph-filtered retrieval; avoid maintaining two equivalent lexical
indexes by default.

SurrealDB [hybrid search](https://surrealdb.com/docs/learn/data-models/vector-search/hybrid-search)
offers BM25, vector retrieval and rank fusion. Downloadable
[Qwen3-Embedding-0.6B](https://huggingface.co/Qwen/Qwen3-Embedding-0.6B) and
[Qwen3-Reranker-0.6B](https://huggingface.co/Qwen/Qwen3-Reranker-0.6B) are credible optional local
text/code mechanisms. The RTX5090 offers room to investigate them; that is not a measured quality
or throughput result. Task-local batch inference is a simple first candidate; one optional shared
runtime, for example [llama.cpp server](https://github.com/ggml-org/llama.cpp/tree/master/tools/server),
can serve repeated consumers if that merits a service. Avoid loading duplicate model copies
where shared execution benefits actual consumers. Exact model/runtime/Blackwell compatibility
remains to qualify. Test real concept questions against lexical retrieval; preserve dirty-source
overlays, model identity, deletion and reindexing. Retrieved similarity never establishes truth.

<a id="O06"></a>
### O06 — Assemble exact-version research and API-evolution packets

**Recommend.** For a dependency or advanced feature, assemble a task-local dossier containing
the actual pin, enabled features, relevant installed/patched source, primary API definitions,
upstream tests, migration differences and unresolved semantic questions. Existing overviews and
research routes remain orientation. This helps turn “the library advertises this” into “this
exact integration has these options and lifetime constraints,” and reduces repeated source
discovery across turns.

[`cargo_metadata`](https://docs.rs/cargo_metadata/latest/cargo_metadata/) and
[rustdoc JSON](https://docs.rs/about/rustdoc-json) can supply machine-readable package/API context;
JSON `format_version`, features, target and toolchain matter. Registry documentation cannot stand
in for the patched SurrealDB adapter or custom server. Preserve links and concise excerpts rather
than create a permanent copy of every library. Compare concrete callers against an upgrade's
changed signatures and behavior, with a small probe when static evidence cannot settle it. A
useful demonstration reconstructs one SurrealDB or Symbolica feature decision from exact source
and identifies a real unknown without automatically changing pins. A manually assembled packet
is an acceptable first route; automation should target repeated work.

<a id="O07"></a>
### O07 — Query the consequences of design alternatives

**Recommend after O01/O02; useful even before persistence.** Support questions such as “if this
coordination moves into SurrealDB, which owners, generators, runtime boundaries, tests and
decisions change?” or “what is required to add a new solver-backed operation?” Compose declared
scope relationships, O02 paths, dependency metadata, lifecycle roles and O06 research into a
bounded comparison of alternatives. Show the proposal, affected current contracts, likely
removals, unresolved edges and evidence needed to decide.

This is assistance for design judgment, not automated architecture approval or a completeness
certificate. Native plans own scheduled scope; architecture owns contracts; an index is derived.
Begin with a CLI returning typed JSON and readable text for a concrete question. A graph query
can improve multi-hop analysis if it proves useful; a general design ontology is not a
prerequisite. Demonstrate two plausible realizations of one change, including a simpler option
and an honestly unresolved dynamic dependency. The value is better questions and earlier
discovery of consequences, not a numerical architecture score.

## Behavioral and scientific verification

<a id="O08"></a>
### O08 — Connect changed behavior to exercised and missing test paths

**Recommend selectively.** Existing `affected` previews reverse dependency reach and the coverage
recipe produces cargo-llvm-cov/nextest LCOV. Add source-region, branch and missing-line inspection
for a scoped edit, using [cargo-llvm-cov's JSON and reporting routes](https://github.com/taiki-e/cargo-llvm-cov).
The agent should be able to ask which changed branches were exercised and which tests are worth
running next. Merged coverage cannot identify the individual test responsible; selected per-test
captures or another explicit mapping are necessary before making that stronger claim.

Tie observations to source, feature set, generated inputs, toolchain and instrumentation. Rust
coverage does not establish native C/C++ coverage. Use the result to expose untested behavior,
not merely optimize away tests or impose a percentage gate. A representative change with a
new error branch should show both the missing path and the targeted test that exercises it.
Keep dependency-based selection as a conservative fallback when coverage is stale or absent.
Behavioral assertions and independent expectations remain necessary even at complete line coverage.

<a id="O09"></a>
### O09 — Generate stateful and metamorphic behavioral challenges

**Strongly recommend.** Broaden existing Proptest use beyond local round trips to scientific
relations and operation sequences. Examples include coherent unit changes with independently
expected transformations, covariance permutation/scaling (`C′ = D C Dᵀ`), keyed coordinate
permutations, and edit→prepare→bind→evaluate sequences checked against small analytic cases.
This exercises combinations an agent may overlook when writing examples around its own change.

Use existing Proptest1.11 and its
[failure persistence](https://proptest-rs.github.io/proptest/proptest/failure-persistence.html).
The separate [`proptest-state-machine` crate](https://github.com/proptest-rs/proptest/blob/main/book/src/proptest/state-machine.md)
is a candidate addition for shrinking state/transition sequences; that capability is not supplied
merely by the existing Proptest pin. Ordinary operation-sequence generation through Proptest is
an alternative.
For Python boundary workflows, Hypothesis's
[`RuleBasedStateMachine`, bundles, preconditions and invariants](https://hypothesis.readthedocs.io/en/latest/stateful.html)
are a candidate new dependency when they add useful boundary coverage. Preserve a concrete minimal
case as well as a seed. Conditions matter: nonlinear branches, domain restrictions and solver
tolerances can invalidate naive equivalence. An expected result obtained from the same faulty
transformation is not an independent oracle. Demonstrate a generated sequence that catches a
plausible stale-state or coordinate-association error and shrinks to an interpretable case.

<a id="O10"></a>
### O10 — Test whether assertions reject plausible wrong implementations

**Strongly recommend for consequential predicates.** Use targeted mutation to challenge result
acceptance, status mapping, comparator direction, tolerance scaling and stale-parent protection.
The question is whether tests would reject realistic wrong behavior, not whether they reproduce
the current implementation. cargo-mutants is listed in bootstrap; installed availability and the
actual validated native environment still need checking before use.

Its [diff targeting](https://mutants.rs/in-diff.html),
[nextest integration](https://mutants.rs/nextest.html),
[Cargo arguments](https://mutants.rs/cargo-args.html) and
[result classification](https://mutants.rs/using-results.html) provide a bounded route. A test-only
diff needs an explicitly selected production mutation target. Caught, missed, unviable and timed-out
mutants have different meanings; do not fold them into a pass count. Preserve force-validation,
native shared libraries and license configuration in any copied workspace. Demonstrate selected
meaningful mutations for one acceptance predicate, then improve missing behavioral assertions.
No repository-wide mandatory mutation score or campaign is proposed. A deliberately constructed
wrong implementation can be simpler for a tightly scoped case.

<a id="O11"></a>
### O11 — Fuzz hostile and structurally varied boundaries

**Recommend for exposed parsers and codecs.** Use coverage-guided fuzzing for scientific input
parsers, frozen transport, IDs/quantities and boundary decoders. The value is discovery of unusual
combinations, denial-of-progress cases and malformed input behavior beyond hand-picked examples.
[cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz),
[LibFuzzer](https://llvm.org/docs/LibFuzzer.html) and
[structure-aware fuzzing](https://github.com/google/fuzzing/blob/master/docs/structure-aware-fuzzing.md)
support finite campaigns, value profiling, corpus minimization and failure reduction.

Choose authentic seed corpora and meaningful properties: no unexpected panic, bounded handling,
appropriate rejection, and preserved independently defined semantics. A codec round trip alone
can agree with its own bug. Proptest's `Arbitrary` and fuzzing's similarly named traits are not
interchangeable. Keep generated campaigns separate from ordinary unit tests, promote reduced
regressions into the appropriate test owner and retain exact instrumentation. Start with a pure
boundary rather than a full solver/database session. This complements O09/O10; it does not replace
representative normal operation or scientific oracle tests.

<a id="O12"></a>
### O12 — Provide a numerical reference and derivative investigation workbench

**Strongly recommend.** Compose existing independent references, finite-difference diagnostics
and physical-domain information into an experiment that compares values, derivatives, scales
and outcomes. Show error versus legal step size, central versus inward-bound formulas, domain
refusals and conditioning clues. This is more useful than a single discrepancy number when an
agent must distinguish a formula error, coordinate error, tolerance issue or unsuitable oracle.

Existing `plan14_reference.py`, native derivative diagnostics and implicit-property tests are
starting owners. [SciPy's adaptive Jacobian API](https://docs.scipy.org/doc/scipy/reference/generated/scipy.differentiate.jacobian.html)
is a candidate mechanism subject to the actual pin. Optional
[mpmath contexts](https://mpmath.org/doc/current/contexts.html) can support precision experiments;
its experimental interval capabilities do not establish certified enclosures. Finite differences
of the production evaluator test derivatives of that evaluator, not the physical formula itself.
Higher precision does not make a shared formula independent. Demonstrate an analytic case and a
domain-limited nonlinear case, with units, conditioning, coordinate selection and oracle provenance
visible. Export standalone scientific plots and machine-readable data for later comparison.

<a id="O13"></a>
### O13 — Reduce scientific failures while preserving their meaning

**Recommend after a useful predicate is defined.** Existing strict authored/frozen reproducers
can replay a failure. Add reduction of equations, operations, records or authored AST structure
to produce a smaller explanation of that same issue. General
[delta debugging](https://www.debuggingbook.org/html/DeltaDebugger.html) supplies the search
mechanism; the scientific reproduction predicate supplies meaning.

Current `ReproductionPredicate.matches` preserves code, stage, rule, source population, validity
and conditions. Reduction can intentionally change source identities, so define an explicit
reduction predicate and source correspondence rather than weakening
`OriginalFailureReproduced`. A smaller parser refusal, unrelated singularity or different domain
failure is not a reduced version of the original numerical problem. Candidate evaluations need
finite owned execution and cancellation; timeout, schedule and resource failures may require a
different reducer or remain unsupported. Demonstrate reduction of a genuine multi-operation
case to a minimal relevant submodel, retaining the original reproducer and a clear explanation
of what is preserved. Manual subtraction remains preferable when a case is already small.

<a id="O14"></a>
### O14 — Inspect lifetimes and controlled concurrency with suitable instrumentation

**Recommend narrowly at risk-bearing boundaries.** Extend focused safety diagnosis around solver
callbacks, cancellation/drain, borrowed buffers and teardown. Existing Miri work covers a pure
ID route; [Miri](https://github.com/rust-lang/miri/) generally cannot execute arbitrary native
FFI. Rust [sanitizer support](https://doc.rust-lang.org/stable/unstable-book/compiler-flags/sanitizer.html)
can instead motivate a small consistently instrumented native callback/lifecycle target.

Start with an actual owner such as native HiGHS callbacks or PETSc flow, selected because of
unsafe lifetime/concurrency exposure. ASan and TSan have different supported configurations;
instrumenting Rust while leaving the relevant native library opaque limits the conclusion.
Record exact compiler/native build and activation, and exercise both successful and interrupted
teardown. A useful delivery is a dependable focused diagnostic route with one representative
consumer, not a claim that a whole solver fleet is sanitizer-qualified. Existing GDB/rr/profilers
remain the first tools when they already answer the question. Keep specialized instrumentation
out of the ordinary production build and do not create a second product implementation.

Separately, investigate controlled interleavings for a small cancellation/drain or publication
mechanism. [Loom0.7.2](https://docs.rs/loom/0.7.2/loom/) offers `loom::model`, replacement synchronization
types, bounded preemption and failure-path checkpoints. It requires deterministic modeled inputs;
ordinary synchronization, system calls and native FFI remain outside its observation. Small
models need explicit correspondence to production transitions, or they merely test a duplicate
story. Preemption bounds and incomplete weak-memory modeling limit conclusions even on success.
Injected clocks/schedules in an existing lifecycle test may be simpler than introducing Loom.
Demonstrate cancellation on either side of a selected publication transition, including drain
and retained ownership. This is a candidate feasibility investigation, not a full native-system
model check. Context7 resolution did not find the relevant Rust library; its primary documentation
supplies this lead.

## Execution, experiments and workstation use

<a id="O15"></a>
### O15 — Watch owned work through transitions rather than repeated polling

**Recommend early.** Build on existing exact activity/build/test handles to provide a bounded
watch that reports meaningful transitions, terminal results and loss of observation. A consumer
could await “compilation ended,” “tests started,” “this owned scope exited” or “new failure
available,” then read the existing result owner. This would make concurrent work easier to
understand without inventing another scheduler.

Long-polling the current observation owner is the simplest first mechanism.
[systemd D-Bus subscriptions, job signals and property changes](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1.html)
can improve process observation where available; result files and application owners supply
their own transitions. Include a resumable cursor and reconcile after event loss or observer
restart. A disappeared process does not prove success, and silence does not prove inactivity.
Show one build→test→terminal sequence and an interrupted observer that recovers the authoritative
outcome. Optional local desktop notifications can be a subscriber, without an external messaging
service. Keep `just activity` usable even when workload admission is unavailable.

<a id="O16"></a>
### O16 — Resume complete feature campaigns at combination boundaries

**Recommend early.** Existing validation supports gate-level reuse and transfer. Extend the
feature campaign so an interrupted powerset can preserve its exact original enumeration and
the result of each completed combination, resume unfinished work, and explain invalidation
after a repair. This addresses expensive repeated enumeration/build work without pretending
reused historical results were freshly executed.

Pinned [cargo-hack0.6.45](https://github.com/taiki-e/cargo-hack/blob/v0.6.45/README.md) offers
`--print-command-list`; its unstable enumeration interface must be pinned and checked. Preserve
the actual command set, source/features/toolchain/environment identity and completion state per
combination. Changes must invalidate every affected combination, conservatively if uncertain.
The existing Cargo dirty-graph overlap problem remains binding: partitions do not isolate shared
units, and multiple drivers in one target directory can deadlock. This proposal is resumability,
not naive parallel Cargo invocations or sharing target directories between checkouts. Show
interrupt→resume and repair→correct invalidation while retaining original failures. A smaller
representative check remains appropriate during implementation; full qualification retains its
own required scope.

<a id="O17"></a>
### O17 — Separate semantic build identity from raw configuration provenance

**Recommend investigation before changing reuse.** `surreal_source.py` includes raw
`.cargo/config.toml` and `rust-toolchain.toml` hashes in installation identity. Consequently a
comment-only edit can create a different custom-server install/target namespace. This is a
clear opportunity to reduce unnecessary cold work if equivalence can be established safely.

Compare canonical parsed TOML identity with conservative raw hashing. A semantic fingerprint
must retain all effective keys, ordered arrays, relative-path context, toolchain, upstream
revision, patches, features, profile, platform and relevant native configuration. Raw file hashes
remain provenance even if no longer build-equivalence keys. Unknown or environment-dependent
semantics should invalidate reuse. This affects an intentional build identity contract and needs
an explicit decision; it is not a casual cache tweak. Demonstrate comment/key-format changes
reusing the same artifact and a real flag/path/toolchain change refusing reuse. If complete
equivalence is not practical, retain conservative identity and expose why a rebuild occurred,
with an explicitly reviewed candidate-reuse route rather than silent aliasing.

<a id="O18"></a>
### O18 — Replay a selected historical test binary without rebuilding everything

**Recommend when a concrete comparison needs it.** Nextest
[archives](https://nexte.st/docs/ci-features/archiving/) can preserve selected test binaries and
metadata for `run --archive-file`, filtering and workspace remapping. This is useful for comparing
a failing historical implementation with a candidate, or replaying a slow-to-build test after
the checkout changes. Existing within-invocation frozen metadata does not itself provide this
cross-run capability.

Archive only selected investigations. Native shared libraries, solver data, Python extension
context and fixture files may need explicit association; an archive is not automatically
self-contained. Identify source cut, force-validation, native ABI and environment, and distinguish
historical replay from qualification of current source. Demonstrate one pure Rust case first,
then one native case only if its dependency closure can be represented reliably. Regeneration
remains the default when cheaper and clearer; there is no general duty to preserve every binary
or old database.

<a id="O19"></a>
### O19 — Create representative disposable database experiment baselines

**Recommend.** Existing private SurrealKit namespaces, credentials and actor fixtures supply
ownership. Add a convenient way to seed a small, representative project/evidence/query dataset,
run an experiment, compare results and dispose of it. This supports query-plan diagnosis,
permission checks, schema changes and realistic result analysis without coupling tests to a
production-scale configuration.

Prefer deterministic generation or logical
[export/import](https://surrealdb.com/docs/reference/rust/methods/export) first. Physical snapshots
or copies require a stopped/drained dedicated engine, all required files, exact engine compatibility
and the [backup/recovery rules](https://surrealdb.com/docs/manage/self-hosted/backups-and-recovery).
The workstation uses ext4; Btrfs/ZFS instant snapshots cannot be assumed. Live-copying an AOF or
reusing another scope's credentials is not a sound shortcut. Demonstrate independent fixtures
with fresh credentials, reproducible logical content and cleanup under cancellation. Larger
performance fixtures remain explicit campaigns. Existing Plan51 project cloning/portable exchange
should be reused where it already supplies the needed semantics.

<a id="O20"></a>
### O20 — Explain host pressure and offer explicit topology experiments

**Recommend observation; investigate placement changes only with measurements.** Pair current
build-stage and scope observations with Linux
[pressure-stall information](https://www.kernel.org/doc/html/latest/accounting/psi.html), cgroup
`cpu.stat`, `io.stat`, `memory.events`, CPU placement and device activity. An agent should be able
to distinguish compilation, linking, I/O, throttling, memory pressure and an idle wait instead of
guessing from aggregate CPU use.

This machine has a Ryzen9950X3D with sixteen physical cores, two unequal L3 groups, 192GB memory,
an ext4 NVMe workspace and an RTX5090. Existing timing and functional CPU classes already reflect
placement choices. Expose actual cpusets and ancestor restrictions before suggesting changes;
an inner request cannot silently exceed its outer scope. Offer explicit build/latency/cache
placement experiments only for a question that benefits from them. No claim that one group is
universally faster, and no new memory governor, follows from the topology. Demonstrate a pressure
case and an idle wait with truthful unavailable fields; performance benefit needs a controlled
measurement, not a hardware specification.

<a id="O21"></a>
### O21 — Run reproducible regression searches and performance experiments

**Recommend after predicates and ownership are explicit.** Compose existing strict reproducers,
build measurements, case measurements, rr and nextest
[stress tests](https://nexte.st/docs/features/stress-tests) into a bounded good/bad experiment.
[Git bisect run](https://git-scm.com/docs/git-bisect) supplies historical search and exit125 for
untestable revisions; a tiny script can be better than a new orchestration service.

Use an owned isolated checkout when history traversal would disturb current work. Native pins,
schema generations and unsupported old revisions need explicit handling. A flaky single trial
is not a deterministic bisect predicate; define repetition and mixed/unknown outcomes. Performance
comparisons need matched conditions, cold/warm state, source identities and variability, with
O20 observations where relevant. Show one deterministic semantic regression and one deliberately
inconclusive noisy comparison. Preserve native result owners; the experiment view records their
relationship and selected question rather than overwriting outcomes or choosing arbitrary
thresholds. No full historical campaign is a prerequisite to ordinary work.

<a id="O22"></a>
### O22 — Isolate disposable tooling experiments without disturbing the checkout

**Recommend selectively.** Free local experiments sometimes need another tool version, parser,
model runtime or upstream build. Rootless
[Podman](https://docs.podman.io/en/latest/markdown/podman-run.1.html), with user namespace handling
such as `--userns=keep-id`, can provide read-only source, private temporary directories and owned
writable outputs. Bubblewrap is a lighter candidate when namespace isolation is sufficient.
This can prevent root-owned artifacts and make a failed experiment cheap to discard.

Do not make containers mandatory for the ordinary Rust/Python loop or recursively change checkout
ownership. Start with an upstream tool or pure-language probe. Native solvers, systemd/cgroup
integration, selected-store transport, devices and licensing have additional requirements; they
are not automatically container-qualified. Image/package sources must be pinned if retained,
and results must expose the environment difference. Demonstrate cancellation and deletion of
only owned experiment state. A private directory and existing `pse-env` are the preferred simpler
alternative when no meaningful dependency or namespace separation is required.

## Evidence, database composition and agent interfaces

<a id="O23"></a>
### O23 — Compare outcomes across runs without losing denominators or unknowns

**Recommend early.** Plan51's `EvidenceAnalysis` already consumes explicitly supplied sources
through Arrow/DataFusion and has typed Subjects, Failures, Missing, Distribution, Diagnostics
and Measurements operations. Extend that surface with paired baseline/candidate outcomes,
new/resolved/persistent failures, missing execution and repeated-condition measurement views.
Questions include “which affected crates failed in both runs?” and “did the candidate improve
this case under matched conditions?”

Use `evidence_query.rs`, `evidence_sources.py`, the existing projection and case-comparison owners;
do not introduce a public arbitrary-SQL interface merely to gain another report. Preserve source
cuts, selection denominators, coverage gaps, mixed outcomes and unavailable measurements. Binary
test IDs across rebuilds require explicit correspondence; renamed/ambiguous subjects remain
unmatched. Failure distributions do not establish causes or flakiness by themselves. Demonstrate
a comparison containing a renamed test, an unexecuted test and a real failure→pass transition.
The offline supplied-source route remains useful when the selected database is unavailable;
persistent discovery should be a rebuildable convenience over the same native facts.

<a id="O24"></a>
### O24 — Use SurrealDB as a derived research and experiment workbench

**Recommend derived development-knowledge views.** Join design/research questions,
source revisions, API evidence, experiment invocations, diagnostics and measurements through
typed relations. This allows “show experiments supporting this design choice” or “find failed
query cases sharing a feature/owner” with source-linked retrieval. Plan51 supplies development-evidence
readers, generation/source-cut mechanisms and a disposable projection that can be reused. Its
canonical scientific research tables describe datasets, selections and physical ancestry; they
are not generic containers for design notes, library research or agent knowledge. Give
development-knowledge facts a separate derived schema and typed source membership. Do not widen
canonical scientific research merely to make a convenient knowledge graph.

Native graph traversal, full-text indexes, bounded vector search and
[derived table views](https://surrealdb.com/docs/reference/query-language/statements/define/table)
can provide readable local views. Use SurrealQL functions for shared derived calculations or
queries; use the selected Surrealism integration for genuinely useful Rust/WASM execution, not
to duplicate the main application's scientific engine. Events/changefeeds can invalidate a
projection or notify a consumer where actual transaction semantics fit. They are not automatically
a durable external workflow scheduler. Keep arbitrary external actions and solver lifetimes in
their current owners. Demonstrate research→experiment→diagnostic traversal and complete rebuild
from supplied sources. Compare direct typed queries and offline DataFusion before adding a new
database module. Query plans and maintenance must be checked on the exact custom server.

<a id="O25"></a>
### O25 — Correlate application, transport and database execution

**Recommend a focused extension; reconcile independent Plan40 scope.** Existing finite OTel
capture demonstrates server executor→Surrealism parent/child context. The operator guide records
that application-originating Rust context is not established. Connect selected operation/invocation
IDs and trace context through application→SDK/transport→server→extension so an agent can distinguish queueing,
network, database and module work. Existing query feedback, plan inspection and selected metrics
remain useful without this extension.

Use the existing finite capture and metric owners in `surreal_server.py`, `storage_health.py`
and the [substrate guide](../dev/surreal-substrate.md). Confirm exact SDK/transport propagation
instead of assuming a server feature automatically instruments the application. Bound spans
and payloads; keep credentials and raw scientific data out of default telemetry. Metrics must
retain server/engine identity and completeness, not silently combine incompatible scopes.
Demonstrate one operation with trace continuity and one unsupported link reported explicitly.
No permanent observability stack is required. Loki/Tempo/Grafana or Tokio console remain optional
free local tools for a specific unanswered question; they are not justified merely by appearing
in the ecosystem reference. Diagnostic-build effects remain visible in measurements.

<a id="O26"></a>
### O26 — Give agents typed actions and structured results at useful boundaries

**Recommend composition-first.** Existing CLIs expose many capabilities, but an agent still
reconstructs options, scope handles, result locations and completion meaning. Offer small typed
actions such as describe-context, inspect-change, watch-owned-run, read-result, compare-evidence
and explain-query. Return bounded structured results with source basis, continuation, unknowns,
owner handle and the next meaningful read/action. Reuse existing command/result owners.

CLI JSON plus readable text is the first transport. A thin MCP facade is worthwhile only where
tool discovery or typed interaction materially improves use; it should not duplicate every CLI,
invent another dispatcher or bypass `pse-env` ownership. The native database MCP is a query
transport, not a substitute for project-specific action semantics. Validate a real consumer
sequence from fresh Codex session to targeted test result, including an environment-shaped
failure. Capability inspection should distinguish available/configured/attached/exercised so
permission alone does not imply functionality. An optional Codex plugin can package the existing
skill/tool wiring, subject to actual client support; it is not a reason to add paid services.

<a id="O27"></a>
### O27 — Improve instruction discovery and evaluate representative agent tasks

**Recommend focused improvements.** Keep common instructions concise and route to task-specific
owners, examples and tools through progressive disclosure. The existing scope rules and process
skills already do this. Improve their discoverability when recurring questions arise, such as
which environment a recipe selects, which source owns generated output, or whether a review
authorizes implementation. Do not grow a universal checklist that displaces source investigation.

Official guidance on [skills and prompts](https://developers.openai.com/blog/rethinking-skills-and-prompts-for-gpt-6-astra)
and [skill evaluations](https://developers.openai.com/blog/eval-skills) suggests practical probes:
small representative tasks assessed for outcome, workflow compliance and unnecessary work.
Use the existing Codex access, local fixtures and explicit human judgment rather than another
metered evaluation service. Examples can cover a generated-contract edit, stale evidence refusal,
partial test outcome and a planning-only request. Static instruction lint cannot prove correct
agent behavior; a successful task does not establish all future reliability. Retain only scenarios
that reveal a meaningful failure mode, and revise the native instruction/skill owner rather than
create a second policy document. Model migration is not proposed.

<a id="O28"></a>
### O28 — Retrieve session activity and decisions from the complete relevant history

**Recommend early.** Repair the retrospective discovery boundary: sessions should be selected
by relevant activity, not only the directory/date when they began. Include explicitly associated
worktrees/subagents, track stable session identity and deduplicate logical call IDs rather than
only serialized-record hashes. Preserve bounded reads, private data handling and clear
incomplete/truncated results. Distinguish a command attempt, returned failure, repeated poll,
quoted failure and confirmed incident.

Current Codex0.163.0 exposes experimental `app-server` commands. The official
[App Server protocol](https://learn.chatgpt.com/docs/app-server) documents cursor-based thread
listing with activity ordering/cwd filters and thread/turn reads; `useStateDbOnly` can avoid
default scan/repair behavior. Those methods are candidates subject to exact installed-client
schema compatibility, not a runtime-qualified integration. A local streaming/index adapter is
a viable fallback and may be simpler. Do not start/resume threads to inspect them or silently
write global memories. Demonstrate discovery of a pre-window session active in-window and a
subagent worktree, then answer a bounded rationale query with historical attribution. O01/O05
can consume this adapter; raw conversation ingestion is optional, not mandatory context.

<a id="O29"></a>
### O29 — Assist integration and closeout with ownership-aware change reconstruction

**Recommend.** Compose Git state, worker ownership, native plans, generated-output ownership and
selected qualification outcomes into a reviewable closeout view. It should identify which edits
belong to which task, unresolved conflicts, required regeneration, current references to retired
documents and the concrete remaining work. This supports discrete commits and reliable handoff
without forcing a new worktree or ceremony for every small edit.

Reuse existing lifecycle/retirement routes and native scope owners. Conflict assistance can
explain both changes and affected consumers; it cannot infer that a textually clean merge is
semantically correct. An unrelated dirty file remains excluded even when a broad formatter or
commit command would include it. Show a representative overlapping change with one generated
consumer and one unrelated dirty file, producing an explicit commit selection and targeted
verification scope. Do not automatically certify plan completion, merge arbitrary worktrees or
replace requested independent completion audits. A simple read-only command may deliver much
of the benefit before any editor or MCP integration.

<a id="O30"></a>
### O30 — Give the operator an optional local investigation workspace

**Recommend only around concrete workflows.** An interactive local view could place source-linked
diagnostics, experiment comparisons, graph paths and scientific plots beside a question and its
evidence. Paul could inspect what Codex is doing, compare two results and choose a follow-up
without reading raw logs. Start with existing headless reports and standalone plots; use a small
local page or notebook only where interaction adds value.

SurrealDB Studio is a useful database/schema/query consumer, not a second schema author or an
all-purpose simulator frontend. The [tooling reference](../libraries/surrealdb_tooling.md) supplies
Studio, query-plan and graph-visualization leads. Typed local APIs, including
[`DEFINE API`](https://surrealdb.com/docs/reference/query-language/statements/define/api) where
appropriate, can serve selected derived views; exposing a generic database writer is unnecessary.
Demonstrate one baseline/candidate investigation with missing data visible and links back to
native results. Do not duplicate durable scientific data in a dashboard or make a service a
prerequisite to CLI work. A full product frontend remains outside this review's target.

## How the SurrealDB tooling reference changes the recommendations

The reference broadens what is feasible, but selected custom-server compatibility and actual
consumer value decide adoption. All its sections were considered; related families are grouped
below to show their design role rather than restate the catalogue.

| Reference family | Disposition in this review |
|---|---|
| §§1–2 orientation/inventory | Capability leads, not an installation list or compatibility guarantee |
| §3 MX/KV/RocksDB/TiKV diagnostics | Retain selected MX architecture; exact engine statistics can enrich O20/O25. No engine switch or distributed deployment is justified by agent effectiveness alone |
| §4 CLI/lifecycle/backup | Use the selected custom binary and existing lifecycle owner. O19 explores representative regeneration and supported snapshots; no second version launcher or Cloud account is needed |
| §5 SurrealKit | Existing schema parity/actor tests are delivered. Extend representative fixtures and exact migration experiments through those owners, not a competing migration/schema authority |
| §6 SDK/types/errors/streams | Concrete typed result and trace integration for O24–O26. Transactions, streams and live queries need explicit lifetime/reconnection semantics; availability is not durable workflow ownership |
| §7 authoring/LSP/tree-sitter | Exact parser feedback already works. Advertised SurrealQL LSP0.6.3 was incompatible in prior inquiry; no fresh compatibility evidence reverses that. Tree-sitter/function metadata remain useful syntax/index leads, not semantic proof |
| §8 MCP/agent docs | Thin transport for useful actions, exact documentation discovery and optional packaging. Native database MCP, hosted Cloud MCP and Agent Memory are different products |
| §9 graph/inline/GQL | O02/O07/O24 can use bounded native graph paths. Existing production graph bounds are delivered. Inline caches or GQL need a query consumer and exact-plan evidence; no extra graph engine is selected |
| §10 full-text/vector/hybrid | Strong lead for O05 source-aware retrieval; lexical baseline, authority filtering, exact model/index identity and rebuildability come first |
| §11 metrics/OTel/console/health | Existing finite diagnostics plus O15/O20/O25. Optional local deep tools answer selected questions; no always-on telemetry platform is assumed |
| §12 Surrealism/functions/events/views/APIs | Useful derived research/experiment queries and constrained computation in O24/O30. External solver execution, durable side effects and scientific authority retain their owners |
| §12 buckets/object storage/SurrealML | Candidate artifact/reference mechanisms if a new consumer needs them. Bucket files are not assumed transactionally identical to KV records; no opaque model execution replaces authored mathematics |
| §13 protocol adapters | PostgreSQL wire compatibility does not make SurrealQL PostgreSQL SQL or establish tool compatibility. gRPC/alternative clients wait for an actual transport need and exact-build verification |
| §14 Sync/migration/CDC | No external source consumer currently justifies a new ingestion service. Projection invalidation can use existing source cuts first; three distinct migration problems should not be conflated |
| §15 benchmark/upstream tests | Reuse focused upstream language/engine cases for O06/O19 or a diagnosed query experiment. A general CRUD benchmark is not a representative simulator correctness test |
| §16 Studio/deployment/agent docs | O30 and O06 consume local visualization and machine-readable docs. Containers are optional experiments; Kubernetes, Cloud administration and adjacent commercial services add no selected value here |
| §§17–19 routing/maturity/sources | Useful discovery map and compatibility questions throughout; primary and pinned source settle consequential claims |

### Agent Memory is a useful design reference, not a selected free local dependency

The current [embedded quickstart](https://surrealdb.com/docs/agent-memory/quickstarts/embedded)
does not establish a supported fully in-process extraction/recall path; its setup still involves
a deployed service. The current
[model/security surface](https://surrealdb.com/docs/agent-memory/architecture/surface-security-and-models)
does not establish our required downloadable local model backend. Therefore it is **not yet
demonstrated to satisfy this engagement's free-local constraint**. This is not proof that no
such route exists or will exist.

Retain its useful ideas—provenance, attributable recall, explicit supersession and traceable
retrieval—but prefer O05/O24/O28 using the existing database and optional downloadable models.
Revisit a packaged Agent Memory dependency if a supported exact version demonstrates that route
with lower total machinery. No bespoke general memory product is required in the meantime.

## Recommended composition and order

The highest-value initial work combines O01–O04, O15–O17, O23 and O28: understand the task, make
reliable changes, inspect execution, resume expensive work and compare evidence. These mostly
extend existing owners and directly address demonstrated repeated work. O17 must first settle
safe equivalence; it is not permission to weaken build identity. O03 must qualify real language
server support before promising semantic editing.

Pursue O09/O10/O12 alongside those foundations for consequential scientific/result behavior.
Their benefit is stronger challenge to incorrect implementations, not just faster execution.
O08 makes missing coverage visible; O13 turns complex failures into smaller explanatory cases.
O11/O14 are focused tools selected by boundary exposure and unsafe/native risk rather than
mandatory repository-wide campaigns.

O05/O06/O07/O24/O26 form the next coherent reasoning and research layer. First expose existing
source/knowledge/evidence through useful questions and typed results. Add persistent graph/search
projections only when they improve those consumers. Local embeddings/reranking are promising
investigations, not blockers for useful lexical retrieval. This avoids three separate general
indexes for code, evidence and agent memory with overlapping ownership.

O18–O22 support selected historical/native/performance experiments. O25 should reconcile its
trace boundary with independent Plan40 work before implementation. O27/O29 improve process use
without multiplying ledgers or completion gates. O30 is an optional operator consumer of the
same results, after at least one investigation benefits from interaction.

Shared integration rules follow from the current architecture: native owners retain truth;
projections carry source cuts and are disposable; static/source/observed evidence remain
distinct; edits target declarations; active commands retain scope ownership and cancellation;
existing recipes/CLIs remain usable without a new service. These shape the realization rather
than impose an adoption quota or numerical benefit model.

## Decisions and investigations to carry into planning

These are proposed decisions, not accepted rule changes. Ordinary use of existing tools within
their contracts needs no new approval ritual. Apply the repository's decision route only when
selected scope actually changes a contract.

| Reference | Proposed question/change | Owning opportunities and next evidence |
|---|---|---|
| RC01 | Define derived context/index authority, dirty overlays and historical attribution without a second policy or plan owner | O01/O02/O05/O07/O24/O28; demonstrate source-linked questions and complete rebuild |
| RC02 | Define completeness/refusal semantics for semantic edits and structural transformations | O03/O04; qualify pinned language servers and generated/dynamic boundary cases |
| RC03 | Permit combination-level qualification reuse only with exact enumeration, context, invalidation and explicit reused-result labels | O16; interrupted/repaired campaign demonstration, preserving unsafe Cargo-overlap restriction |
| RC04 | Consider effective semantic configuration identity separately from raw provenance | O17; equivalent/non-equivalent fixtures and conservative unknown handling before changing cache reuse |
| RC05 | Define selected historical replay and scientific reduction identities without weakening original reproduction truth | O13/O18/O21; native closure and meaning-preserving examples |
| RC06 | Extend typed evidence comparisons and derived research views while retaining supplied-source limits and native result authority | O23/O24; unmatched/missing/mixed cases and rebuild demonstration |
| RC07 | Define thin agent action/result contracts, with exact ownership, continuation and observation failure semantics | O15/O26/O29; actual fresh-session consumer sequence, no duplicate scheduler |
| RC08 | Select optional local model/service or sandbox activation only when a concrete consumer justifies it | O05/O22/O30; free local compatibility and operational lifecycle, lexical/private-directory alternatives |
| RC09 | Extend finite trace correlation across the application boundary, reconciling existing Plan40 scope | O25; exact SDK propagation and finite capture with explicit unsupported links |

Further investigations must remain attached to their value question: whether mcpls edits are
complete for our languages; whether structural codemods support the actual Python grammar;
whether semantic retrieval improves real concept questions; whether effective configuration
identity can be safe; whether per-combination reuse survives a repair; whether native archives
are genuinely replayable; whether the reducer preserves the chosen scientific issue; and whether
trace context crosses our exact transport. Negative or inconclusive results should select the
simpler alternative, not automatically trigger a larger platform.

## Coverage and limits

| Agent activity | Review routes |
|---|---|
| Understand a task and recover decisions | O01/O05/O06/O28 |
| Navigate and scope changes | O02/O07/O08/O26 |
| Edit/refactor/generated-contract work | O03/O04/O29 |
| Design and plan functional changes | O06/O07/O24/O27 |
| Challenge behavior and scientific correctness | O09–O14 |
| Compile, test and resume qualification | O08/O15–O18/O26 |
| Diagnose performance and resource use | O20/O21/O25 |
| Investigate complex failures | O12/O13/O18/O21/O23 |
| Research library capabilities and upgrades | O06/O19/O22/O24 |
| Integrate, document and hand off | O01/O27/O29 |
| Collaborate with the sole operator | O15/O23/O26/O30 |
| Learn from local execution history | O23/O27/O28 |

This is broad coverage of activity types, not a proof that every possible enhancement has been
enumerated. It intentionally leaves product frontend design, new solver functionality, commercial
deployment and unrelated remaining plan scope with their owners. No quantitative productivity,
performance, reliability or retrieval-quality improvement is claimed. Proposed demonstrations
are representative acceptance leads for planning, not already executed tests.

### Publication evidence

**Interface-checked:** current source, pinned interfaces and linked primary documentation support
the proposed routes with the limits above. **Proposed:** every opportunity and rule question.
**Tested, documentation only:** `scripts/pse-env --docs -- python3 -m scripts.document_lifecycle validate`
returned zero errors against the zero target, with 1,014 scope items validated. `just docs` built
and published 592 chapters with scoped Pagefind search, exiting successfully. `scripts/pse-env -- just turn-end`
passed ADR indexing and formatting with zero failed stages. Its resource cleanup separately
reported four deferred cleanups for retained unsupported server profiles; no cleanup or profile
retirement was performed by this review. These checks do not establish product, scientific,
native-tool or model qualification. No quality baseline is introduced.
