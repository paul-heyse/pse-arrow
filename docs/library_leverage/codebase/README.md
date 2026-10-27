# Codebase opportunities for library leverage

**Account state: first broad audit, 2026-10-27.** This page is the entry point to the living
account of where library capabilities could improve how pse-arrow fulfils its responsibilities.
The [codebase audit skill](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/leverage-codebase-audit/SKILL.md)
maintains it. Each opportunity has a stable `CBO` ID and one primary home on a topic page.

An opportunity says that a responsibility deserves investigation. It is not a defect finding,
it does not show that a library fits, and it is not a recommendation. Applicability research
belongs to [library assessments](../libraries/README.md). Choosing between options belongs to
[comparative reviews](../reviews/README.md).

**Library assessment (2026-10-27).** The first [library applicability assessment](../libraries/README.md)
covers every opportunity. Its premise corrections are recorded in place on the topic pages as dated
"Library assessment correction" bullets.

Read the [framework](../library-leverage-framework.md), the
[base patterns](../patterns.md), the [shared vocabulary](../references/shared-language.md) and the
[shared additional patterns](../assessment-patterns.md) alongside this account.

## Inspected baseline and method

| Item | Value |
| --- | --- |
| Inspection date | 2026-10-27 |
| Source baseline | `main` at `dbc4c94369425fd2a26355585ff98888a2da555d` |
| Included dirty changes | None in product, tooling or test source. The working tree had uncommitted edits to three `docs/plans/` files and `history.txt`, plus untracked `docs/libraries/*.md` overviews. None of these were used as evidence. |
| Criteria | Framework edition 0.2, base patterns edition 0.2 (`LOP-01`–`LOP-12`), [additional patterns](../assessment-patterns.md) revision 1 (`LOP-X001`–`LOP-X003`, created in this pass; revision 2 added `LOP-X004`–`LOP-X010` during the library assessment) |
| Prior accounts | None. This is the first audit. By operator instruction, prior reviews and plans were consulted only to confirm that `CBO` and `LOP-X` numbering does not collide. |
| Method | Nine read-only subsystem passes, using static reading, `rg` and `git`. Builds, tests and probes were not run. The coordinator then spot-checked the decisive claims against source. |
| Evidence level | Mechanisms are *Interface-checked* by static inspection at the baseline. Every value claim is *Proposed*, and nothing is *Measured*. |

The spot checks corrected one premise before publication. The math pass had nominated a missing
weighted tear selection, but `pse-backend-native/src/tears.rs` already solves tear selection as a
HiGHS MILP, so that candidate was dropped. The other candidates the subsystem passes nominated
but this account did not retain are listed under "Examined and not retained" on their topic
pages.

## System picture

pse-arrow is a clean-room process-systems core of about 0.9M lines of non-generated Rust under
`crates/`, a count that includes in-crate tests. It also has about 7k lines in the hand-written Python package and
about 100k lines of Python tooling. The
[architecture overview](../../authoritative_design/sections/architecture-overview.md) assigns
operations to libraries by fit:
- Symbolica and Numerica own algebra, derivatives and evaluators.
- Class-specific native solvers own iteration and factorization.
- Salsa owns pure semantic reuse.
- Arrow and DataFusion own columnar and relational boundaries.
- SurrealDB owns canonical persistence.
- PyO3 owns the Python boundary.

That choice is visible in the code. The hardest domain-critical algorithms are already
library-led:
- derivatives, jets, root isolation and collocation through Symbolica;
- matching and DM/BTF through pounce;
- tear MILPs and lexicographic objectives through HiGHS;
- covariance through KKT sensitivity and faer SVD;
- result selection through DataFusion pruning.

The local code mostly carries qualification, identity, admission and lifecycle.

That local code is therefore where most leverage opportunities sit. They cluster into five
shapes.

1. **Guarantees a library does not offer, enforced by mirroring the library.** pse-arrow prepays
   memory and work, cancels promptly and keeps outcomes deterministic. Where a library exposes no
   size estimate, budget or abort hook, the code reproduces its internals, or patches or forks it.
   - Mirrors: [CBO-001](cross-cutting-mechanisms.md#cbo-001), [CBO-021](mathematics-and-compilation.md#cbo-021).
   - Patches and forks: [CBO-009](persistence-and-transport.md#cbo-009),
     [CBO-027](native-solver-integration.md#cbo-027), [CBO-028](native-solver-integration.md#cbo-028).

   The decisive research question here is usually whether there is a newer upstream hook or
   whether one could be contributed, rather than which library to swap in. New pattern:
   [LOP-X001](../assessment-patterns.md#lop-x001).
2. **One lifecycle obligation re-implemented per site.**
   - Child-process drain: [CBO-004](cross-cutting-mechanisms.md#cbo-004).
   - Execution-context propagation: [CBO-014](execution-lifecycle-and-observation.md#cbo-014).
   - Cancellation and lease renewal: [CBO-015](execution-lifecycle-and-observation.md#cbo-015).
   - Signal-aware waits: [CBO-017](execution-lifecycle-and-observation.md#cbo-017).
   - Native session ownership: [CBO-030](native-solver-integration.md#cbo-030).
   - Single flights: [CBO-011](persistence-and-transport.md#cbo-011).
   - Retry: [CBO-010](persistence-and-transport.md#cbo-010).
   - Durable files: [CBO-003](cross-cutting-mechanisms.md#cbo-003), [CBO-047](developer-tooling.md#cbo-047).
   - Host admission above systemd: [CBO-048](developer-tooling.md#cbo-048).

   Each implementation is careful. The cost is that stricter rules added at one site do not
   reach the others. Several are already half-consolidated, so a local shared owner composed
   from tokio-util, systemd or tempfile capabilities may be the remedy rather than a new
   framework.
3. **One meaning in several hand-maintained or generated forms.**
   - Output budgets: [CBO-002](cross-cutting-mechanisms.md#cbo-002).
   - Output predicates: [CBO-007](result-data-paths.md#cbo-007).
   - Grammar forms: [CBO-037](authoring-and-identity.md#cbo-037).
   - Identity framing: [CBO-039](authoring-and-identity.md#cbo-039).
   - IR operator vocabularies: [CBO-024](mathematics-and-compilation.md#cbo-024).
   - Conic encodings: [CBO-031](native-solver-integration.md#cbo-031).
   - NLP callback translators: [CBO-029](native-solver-integration.md#cbo-029).
   - Observation sinks: [CBO-013](execution-lifecycle-and-observation.md#cbo-013).
   - Diagnostics macros: [CBO-045](registry-generation-and-python-boundary.md#cbo-045).
   - Test-runner configuration: [CBO-049](developer-tooling.md#cbo-049).
   - Command surfaces: [CBO-052](developer-tooling.md#cbo-052).

   The registry and the Python boundary add a specific variant: generation done well at the
   authority, with a costly tower of emitters and transcodes downstream. That variant covers
   [CBO-040](registry-generation-and-python-boundary.md#cbo-040)–[CBO-044](registry-generation-and-python-boundary.md#cbo-044)
   and the new pattern [LOP-X003](../assessment-patterns.md#lop-x003).
4. **Adopted libraries used below their capability.**
   - Arrow compute and IPC validation on the largest result tables:
     [CBO-005](result-data-paths.md#cbo-005), [CBO-006](result-data-paths.md#cbo-006).
   - miette source labels: [CBO-038](authoring-and-identity.md#cbo-038).
   - schemars contract modes: [CBO-042](registry-generation-and-python-boundary.md#cbo-042).
   - The pinned `egobox-doe`: [CBO-033](numerical-workflows.md#cbo-033).
   - `statrs` distributions: [CBO-036](numerical-workflows.md#cbo-036).
   - `tempfile` and `object_store`: [CBO-003](cross-cutting-mechanisms.md#cbo-003).
   - `quick_cache`: [CBO-011](persistence-and-transport.md#cbo-011).
5. **Capability gaps with plausible consumers.**
   - No producer of sampling or experimental designs, although IDAES-style sweeps are listed as
     in scope: [CBO-033](numerical-workflows.md#cbo-033).
   - Only first-order uncertainty propagation: [CBO-034](numerical-workflows.md#cbo-034).
   - No estimated-variance fit statistics: [CBO-036](numerical-workflows.md#cbo-036).
   - Waits poll for state that could notify: [CBO-016](execution-lifecycle-and-observation.md#cbo-016).

**Existing strengths are counterevidence.** These areas were examined and found to be library-led
or legitimately custom, and should not be reopened without new evidence:
- DataFusion use in `pse-engine`.
- SurrealQL depth.
- Symbolica/Numerica use.
- HiGHS and SCIP depth.
- Salsa durability and LRU.
- PyO3 introspected stubs.
- nextest groups.
- `systemd-run` scopes.
- FMA-free deterministic summation.
- Original-coordinate qualification.
- Identity hashing.

Where a topic page records a declined candidate, it gives the reason.

## Where to start research

Rank by consequence, not by implementation order. The following are the questions most likely to
change a later decision:

1. **Upstream hooks for prepaid accounting and cancellation.** Do Symbolica (after 3.0.1), faer,
   FERAL, KLU and Arrow expose size estimates, budgets or abort hooks that would retire the
   mirrors in [CBO-001](cross-cutting-mechanisms.md#cbo-001) and
   [CBO-021](mathematics-and-compilation.md#cbo-021)?
2. **SurrealDB SDK deltas.** Which items of the 35k-line SDK delta now have upstream equivalents
   ([CBO-009](persistence-and-transport.md#cbo-009))?
3. **systemd as the authority.** Which host-admission and supervision facts can systemd (over
   D-Bus) own authoritatively, and what ledger must remain
   ([CBO-048](developer-tooling.md#cbo-048), [CBO-020](execution-lifecycle-and-observation.md#cbo-020),
   [CBO-004](cross-cutting-mechanisms.md#cbo-004))?
4. **Arrow kernels and validation.** Which Arrow 59 kernels and IPC decoder validation discharge
   the hand-written dense-trajectory and IPC preflight work
   ([CBO-005](result-data-paths.md#cbo-005), [CBO-006](result-data-paths.md#cbo-006))?
5. **The generation tower.** Can registry products and boundary documents move through fewer
   encodings while keeping determinism and fingerprints? See
   [CBO-040](registry-generation-and-python-boundary.md#cbo-040)–[CBO-043](registry-generation-and-python-boundary.md#cbo-043).
6. **Reproducible design generation.** Is `egobox-doe`, or a Sobol crate, bit-stable across
   CPUs for seed-determined designs ([CBO-033](numerical-workflows.md#cbo-033))?
7. **Observation fan-out.** Can a single tracing layer fan one lifecycle event out to the hub,
   recorder and metrics, and can span extensions carry the execution context
   ([CBO-013](execution-lifecycle-and-observation.md#cbo-013),
   [CBO-014](execution-lifecycle-and-observation.md#cbo-014))?

## Opportunity index

| ID | Opportunity | Primary patterns | Status | Home |
| --- | --- | --- | --- | --- |
| CBO-001 | Prepaid memory and work accounting mirrors library internals | LOP-X001, LOP-02 | Current promise | [Cross-cutting](cross-cutting-mechanisms.md#cbo-001) |
| CBO-002 | Bounded and counting output sinks re-implemented per site | LOP-09, LOP-02 | Proposed | [Cross-cutting](cross-cutting-mechanisms.md#cbo-002) |
| CBO-003 | Durable file publication and the sealed-file byte layer on `std::fs` | LOP-09, LOP-01 | Current promise | [Cross-cutting](cross-cutting-mechanisms.md#cbo-003) |
| CBO-004 | Child-process and process-group supervision per consumer | LOP-09, LOP-08 | Current promise | [Cross-cutting](cross-cutting-mechanisms.md#cbo-004) |
| CBO-005 | Arrow IPC untrusted-input verification written by hand | LOP-03, LOP-11 | Current mechanism | [Result data](result-data-paths.md#cbo-005) |
| CBO-006 | Dense trajectory indexing and selection done row by row | LOP-04, LOP-06 | Proposed | [Result data](result-data-paths.md#cbo-006) |
| CBO-007 | Output-predicate semantics in several forms | LOP-02, LOP-05 | Proposed | [Result data](result-data-paths.md#cbo-007) |
| CBO-008 | Progress and receipts as JSON payloads beside typed relations | LOP-06 | Proposed (weak) | [Result data](result-data-paths.md#cbo-008) |
| CBO-009 | Large local SurrealDB SDK delta with a mirrored lifetime layer | LOP-10, LOP-11 | Current promise | [Persistence](persistence-and-transport.md#cbo-009) |
| CBO-010 | Conflict retry and transaction preludes assembled per site | LOP-02, LOP-09 | Current mechanism | [Persistence](persistence-and-transport.md#cbo-010) |
| CBO-011 | Single-flight, I/O gates and load admission beside caching libraries | LOP-08, LOP-03 | Current mechanism | [Persistence](persistence-and-transport.md#cbo-011) |
| CBO-012 | Imperative nested-occurrence traversal beside the plan route | LOP-09, LOP-05 | Uncertain | [Persistence](persistence-and-transport.md#cbo-012) |
| CBO-013 | One lifecycle event fanned out to hand-fed observation sinks | LOP-09, LOP-02 | Current promise | [Lifecycle](execution-lifecycle-and-observation.md#cbo-013) |
| CBO-014 | Execution context re-established by hand at each boundary | LOP-X002 | Proposed | [Lifecycle](execution-lifecycle-and-observation.md#cbo-014) |
| CBO-015 | Cancellation, deadline and lease renewal composed per site | LOP-08 | Current promise | [Lifecycle](execution-lifecycle-and-observation.md#cbo-015) |
| CBO-016 | Waiting by polling for state owned elsewhere | LOP-08, LOP-12 | Proposed | [Lifecycle](execution-lifecycle-and-observation.md#cbo-016) |
| CBO-017 | Signal-aware blocking waits at the Python boundary per call | LOP-09, LOP-08 | Current promise | [Lifecycle](execution-lifecycle-and-observation.md#cbo-017) |
| CBO-018 | Two executor threads per native job to wait for a join | LOP-08 | Current promise (weak) | [Lifecycle](execution-lifecycle-and-observation.md#cbo-018) |
| CBO-019 | Linux process, cgroup and pressure facts parsed by hand | LOP-01, LOP-09 | Current promise | [Lifecycle](execution-lifecycle-and-observation.md#cbo-019) |
| CBO-020 | Primary-receiver supervision split between Rust and a Python script | LOP-09, LOP-08 | Current promise | [Lifecycle](execution-lifecycle-and-observation.md#cbo-020) |
| CBO-021 | Bounded and cancellable Symbolica operations through version-coupled mirrors | LOP-X001, LOP-10 | Current promise | [Mathematics](mathematics-and-compilation.md#cbo-021) |
| CBO-022 | Prepared-mathematics reuse in four layers beside Salsa | LOP-07, LOP-10 | Current promise | [Mathematics](mathematics-and-compilation.md#cbo-022) |
| CBO-023 | Structural decomposition and closure owned in several places | LOP-09, LOP-03 | Current promise | [Mathematics](mathematics-and-compilation.md#cbo-023) |
| CBO-024 | One expression through several local IRs | LOP-06, LOP-02 | Current promise | [Mathematics](mathematics-and-compilation.md#cbo-024) |
| CBO-025 | Exact rational LDLᵀ and cubic-refinement ladder carried locally | LOP-01 | Current promise (weak) | [Mathematics](mathematics-and-compilation.md#cbo-025) |
| CBO-026 | Sparse factorization wrappers split across faer, FERAL and KLU | LOP-09, LOP-07 | Proposed | [Native solvers](native-solver-integration.md#cbo-026) |
| CBO-027 | Diffsol linear solvers and adjoint checkpoints re-implemented | LOP-11, LOP-10 | Current mechanism | [Native solvers](native-solver-integration.md#cbo-027) |
| CBO-028 | Vendored supplier numerics whose premises may have aged | LOP-11 | Current mechanism | [Native solvers](native-solver-integration.md#cbo-028) |
| CBO-029 | Two NLP-oracle callback translators | LOP-09, LOP-02 | Proposed | [Native solvers](native-solver-integration.md#cbo-029) |
| CBO-030 | Native session lifecycles re-expressed per adapter | LOP-08, LOP-09 | Proposed | [Native solvers](native-solver-integration.md#cbo-030) |
| CBO-031 | Convex and conic encodings hand-synchronized per solver | LOP-02, LOP-09 | Current promise | [Native solvers](native-solver-integration.md#cbo-031) |
| CBO-032 | KKT-point sensitivity beside fuller library sensitivity surfaces | LOP-03 | Premise false (confirmed by the [comparative review](../reviews/native-solver-integration.md#cbo-032)); retire at the next audit refresh | [Native solvers](native-solver-integration.md#cbo-032) |
| CBO-033 | No producer of sampling or experimental designs | LOP-12, LOP-03 | Proposed | [Numerical workflows](numerical-workflows.md#cbo-033) |
| CBO-034 | Only first-order uncertainty propagation | LOP-12 | Proposed | [Numerical workflows](numerical-workflows.md#cbo-034) |
| CBO-035 | Separate dense-algebra universes for LM and surrogate routes | LOP-09, LOP-11 | Current promise | [Numerical workflows](numerical-workflows.md#cbo-035) |
| CBO-036 | No estimated-variance fit statistics | LOP-12 | Proposed (weak) | [Numerical workflows](numerical-workflows.md#cbo-036) |
| CBO-037 | Grammar facts maintained in several hand-written forms | LOP-02, LOP-09 | Proposed | [Authoring](authoring-and-identity.md#cbo-037) |
| CBO-038 | Author-facing diagnostics do not render source context | LOP-03, LOP-12 | Proposed | [Authoring](authoring-and-identity.md#cbo-038) |
| CBO-039 | Hand-written identity framing beside the serde-derived framer | LOP-02, LOP-03 | Proposed | [Authoring](authoring-and-identity.md#cbo-039) |
| CBO-040 | Registry products baked as Rust plus JSONL plus IPC | LOP-X003, LOP-06 | Current mechanism | [Registry and Python](registry-generation-and-python-boundary.md#cbo-040) |
| CBO-041 | Boundary documents transcoded through JSON, checks in both languages | LOP-06, LOP-03 | Current promise | [Registry and Python](registry-generation-and-python-boundary.md#cbo-041) |
| CBO-042 | Document wire envelopes and directional schema surgery | LOP-10, LOP-02 | Current mechanism | [Registry and Python](registry-generation-and-python-boundary.md#cbo-042) |
| CBO-043 | Two Python model systems and a custom schema-to-msgspec emitter | LOP-X003, LOP-09 | Current mechanism | [Registry and Python](registry-generation-and-python-boundary.md#cbo-043) |
| CBO-044 | Python wrapper layer forwarding native handles | LOP-02, LOP-04 | Current mechanism | [Registry and Python](registry-generation-and-python-boundary.md#cbo-044) |
| CBO-045 | Diagnostics through a forwarding macro beside the miette derive | LOP-02, LOP-03 | Current mechanism | [Registry and Python](registry-generation-and-python-boundary.md#cbo-045) |
| CBO-046 | SurrealQL and Python emitted through string templates | LOP-01 | Current mechanism (weak) | [Registry and Python](registry-generation-and-python-boundary.md#cbo-046) |
| CBO-047 | File-ledger primitives re-derived per script | LOP-09, LOP-01 | Proposed | [Tooling](developer-tooling.md#cbo-047) |
| CBO-048 | Host workload admission and service supervision rebuilt above systemd | LOP-08, LOP-03 | Current promise | [Tooling](developer-tooling.md#cbo-048) |
| CBO-049 | Test selection, requirements and runner configuration beside nextest | LOP-02, LOP-03 | Current promise | [Tooling](developer-tooling.md#cbo-049) |
| CBO-050 | Development evidence and knowledge pipeline in two languages | LOP-09, LOP-06 | Current promise | [Tooling](developer-tooling.md#cbo-050) |
| CBO-051 | Hand-written wire and front-matter parsers in tooling | LOP-01 | Proposed | [Tooling](developer-tooling.md#cbo-051) |
| CBO-052 | Several command surfaces without a shared declaration | LOP-02, LOP-09 | Proposed (weak) | [Tooling](developer-tooling.md#cbo-052) |

The Status column distinguishes three cases:
- *Current promise*: the responsibility is an existing contract, and only its realization is
  in question.
- *Current mechanism*: an implementation choice with no stronger promise attached.
- *Proposed*: an addition or consolidation this audit suggests.

"Weak" marks an opportunity whose counterevidence is strong. It is retained so that later work
can confirm or retire it.

## Coverage and limits

- **Depth.**
  - Each subsystem pass read representative paths, callers and module heads, and did not take a
    census.
  - Each topic page states its inspected versus suspected footprint.
  - Several opportunities rest on module structure rather than full bodies: CBO-020, CBO-048,
    CBO-050, CBO-028 (patch diffs), CBO-029 (translators not diffed) and CBO-031 (permutation
    tables not compared).
- **Not inspected or thinly inspected.**
  - `pse-kernels` internals, `pse-rules`, `pse-study-module` and `pse-testkit`.
  - The C++ solver bridges.
  - Large parts of `pse-runtime/src/workflow/modeling/*` (including `dynamics.rs` and `cases.rs`)
    and `math/staged.rs`.
  - The IDAS body.
  - The `pse-admission` loaders.
  - Python `authoring`, `adapters`, `parity` and `_transfer`.
  - The bodies of `scripts/surreal_server.py` and the capability server.
  - Test code, except as evidence of dependencies.
  - Docker images and the solver build.
- **Architecture context.** Architecture sections were consulted for subsystem contracts. ADR
  rationale was read only where a pass cited it. Some "deliberate design" counterevidence is
  therefore inferred from code comments.
- **Measurement.** No performance or memory measurement was made. Performance value is
  qualitative throughout.
- **Library research.** Library names are research leads only. Absent or unknown leads do not
  weaken an opportunity.

## Maintenance

Refresh this account in place:
- Use changes in contracts, consumers, dependencies and intended work to direct attention.
- Treat this pass's opportunities as leads, not as a boundary on what to inspect.
- Inherited observations keep the 2026-10-27 date and the limits stated above.
- Retire an opportunity only on evidence that it is resolved, superseded, has lost relevance, or
  rested on a corrected premise. Record the reason in Git, and never reuse its ID.
- A chosen or rejected library does not resolve an opportunity.
- Accepted plans own scheduling and closure evidence. This account links to them rather than
  copying their status.
