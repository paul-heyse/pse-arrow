# Boundaries and composition

These findings concern three kinds of structural cause:
- a responsibility placed in the wrong owner;
- a protocol or host surface re-authored per instance rather than composed;
- an integration boundary that carries decisions it should not.

The [collection README](README.md) gives the baseline and limits. Citations are `path::symbol`
at `cbf45ccc9`.

## <a id="da-033"></a>DA-033 — The durable lifecycle protocol is shared at the transport layer but re-authored per owner at its semantic envelope

**Principles:** AP-03, AP-07, DP-01, DP-16, DP-19; H25, H27; G1, G9 (PSE-S04, PSE-S06, and
"add a durable owner").

**Refresh status:** new. It inherits the surviving structural part of DA-025 (machinery scale)
and overlaps DA-007 and DA-019.

**Diagnosis.** §20.2 states one contract: immutable requests and acknowledgements settle
uncertain outcomes by exact identity and request comparison.

The realization shares the transport layer:
- the original-clock deadline (`canonical_transport.rs::within_clock`);
- conflict retry (`canonical.rs::protected_query`, `GUARDED_ATTEMPTS`);
- activity admission;
- server guard primitives (`fn::pse_execution_v1::{deadline, touch, register, lease}`);
- the roots table;
- one receipt table that execution, studies and history share.

It re-authors the *semantic envelope* per owner, across 14 server-function namespaces (171
functions, up from 116 at the previous baseline):
1. **About ten acknowledgement and receipt schemes.**
   - Separate tables for execution, project, maintenance, selection and version receipts.
   - Research and context rows that act as their own acks.
   - Artifact and analysis headers that act as both intent and ack.
   - File-instance intents.
   - Import rows.
2. **Intent identity formats and creation floors.** Each owner defines its own on the shared
   retention guard: `pse.artifact.v1:` with `artifact_creation_closed_through`,
   `pse.analysis.v2:`, `pse.protection.v2`, and others.
3. **Settlement classification is coded per owner, and the owners already disagree.**
   - `canonical_execution.rs::settle_operation_for_run` returns the original error on absence.
   - `canonical_projects.rs::settle_project_effect` and the context append mint
     `ProjectUnsettled`.
   - `canonical_research.rs::create_research_dataset`/`selection` return the original error
     even when an existing row holds a *different* request, where projects return
     `OperationReused`. A readback failure can also replace the original error.
   - `CanonicalError` lists owners in its variants: `AnalysisUnsettled`, `ArtifactUnsettled`,
     `SourceEditUnsettled`, `ProjectUnsettled`.
4. **The physical file protocol is implemented twice.**
   - Layout, alias settlement and owner identity live in Rust
     (`pse-operations/src/canonical_files.rs`).
   - They are implemented again in Python (`scripts/payload_files.py`) for backup, restore and
     owner recovery.
   - The Python owner check omits the invocation and cgroup fields that Rust registers.

**Consequence.**
- *Change to the shared contract.* For example, how a failed acknowledgement readback is
  classified must be found and re-made in about ten places across SurrealQL templates and Rust.
- *Change to payload layout or crash protocol (PSE-S04).* This crosses Rust and Python.
- *Add a durable owner.* Plan 51 is the demonstrated case.
  - Its new namespaces (`project`, `project_import`, `research`, `context`, `maintenance`,
    `lifecycle`, `workbench`) account for 39 of the 55 server functions added since the previous
    baseline. The tables grew by a net 17.
  - Each new owner brought its own ack, authority, root and settlement code.
  - Nothing in the current structure stops the next owner from doing the same.
- *Callers decide whether to replay on "is this effect unknown?"* That rule already differs by
  owner (item 3).

**Counterevidence.**
- Owners have genuinely different atomicity shapes:
  - single-transaction project creation;
  - multi-page staged activation;
  - long-lived fenced attempts;
  - creation windows with floors.
- ADR-0177 and ADR-0182 deliberately give each owner its own admission "while sharing immutable
  payload, protection and conflict-guard mechanisms".
- Backup must run with the Rust service stopped, which is why a non-Rust reader exists.
- This justifies owner-specific *predicates*. It does not justify re-authoring the generic
  envelope. The shared execution receipt table already shows that one envelope can serve several
  owners.

**Evidence:** *Implemented* for the inventory and the divergent settlement sites. The change
cost is reasoned. Tracing one settlement-rule change across all owners would demonstrate it.

**Improvement direction.**
- One idempotent-operation envelope declared once: operation key, request digest, ack record,
  settlement classification and creation window. Every owner consumes it, and owners differ only
  in domain predicates.
- One declared physical-file contract consumed by both the service and offline tooling.
- Owner-agnostic `Unsettled{owner, intent}` semantics.

**Related:**
- [DA-007](structure-and-invariants.md#da-007): the same templates spell the vocabularies.
- [DA-034](execution-fit.md#da-034) and [DA-025](execution-fit.md#da-025): the publication unit and
  collection that the envelope governs.
- The per-problem creator guard, which every owner's creators share. It is kept by decision I01;
  see [execution fit](execution-fit.md#considered-and-not-raised).

## <a id="da-019"></a>DA-019 — Durable execution's deployment layer lives in development tooling, joined to the product by an unregistered cross-language contract

**Principles:** AP-01, AP-02, AP-05, DP-01, DP-17, DP-24; H13, H25, H27; G1.

**Refresh status:** reframed and narrowed. The second canonical writer is gone: supervisor
mutations go through generated guarded functions. The Python diagnostics launch fell from
about 335 to 23 lines. What persists, and has grown, is the deployment contract.

**Diagnosis.** "Run a durable study on the managed primary" is product behavior. Its pieces sit
in four places joined by hand-mirrored formats:
- **The Python supervisor writes the deployment.**
  - `scripts/surreal_server.py` (10.7k lines) writes `config.json` (`profile_version` 3),
    `.contexts/<db>.json` and receiver configurations.
  - It validates them in Python (`config_for`, `resources`, `validate_execution`).
- **The Rust runtime re-reads and re-validates them.**
  - `workflow/native_deployment.rs` declares mirror structs with `deny_unknown_fields`.
  - It re-implements the envelope rules in `NativeAllocation::validate` and hard-codes the
    profile version and `.contexts` layout.
- **The store client reads them a third time.** Its own partial structs live in
  `canonical.rs::from_state_for_database`, `canonical_files.rs`, `canonical_maintenance.rs` and
  `canonical_study_module.rs`.
- **Control runs through untyped stdout JSON.**
  - The runtime spawns `scripts/surreal_server.py` with `ensure-primary` and `observe-primary`
    (`workflow/worker.rs::{ensure_managed_primary, ManagedPrimary::observe}`) and parses
    `Ready`, `Observation` and phase strings.
  - The supervisor launches the production worker, a binary in the tooling package
    (`xtask/src/bin/pse-worker.rs`, gated on feature `package-fixtures`).
  - xtask also hosts production schema activation, the Plan 51 `pse-project` CLI and
    `pse-native-lane`.
- **Frozen whole-tree generations.** To keep the long-running service stable, the supervisor
  freezes a content-addressed copy of *all* `scripts/*.py` plus configs, Cargo and toolchain
  files into `.generations/<sha>` (`publish_generation`, `verify_generation`), and runs from it.
- **A hand-listed logical restore comparison.**
  - Backup itself copies the whole database directory and payload inventory, and checks a SHA-256
    for every file on restore. That part is sound.
  - Backup format 4, new in Plan 51, adds a logical inventory of "original catalogue owners"
    (`APPLICATION_BACKUP_TABLES`, plus 26 lifecycle field names), compared after restore and
    before authority rotation.
  - That owner set lists 36 of 79 canonical tables. It is checked against itself, not derived
    from registry table classification.
  - Plan 51 owner tables are absent from the comparison, though backed up and byte-verified:
    - `canonical_context_journals` and `canonical_context_ancestry`;
    - `canonical_draft_diagnostics`;
    - `canonical_maintenance_operations`.

    The coordinator confirmed these absences. Whether they are intended is undocumented.

**Consequence.**
- **Adding an execution-profile field.** This needs edits to:
  - the Python writer and validators;
  - `.config/agent-capacity.toml`;
  - the Rust mirror and `validate`;
  - the worker.

  A Python-only addition surfaces as a refusal at durable startup, not at compile time.
- **Generation churn.** Any edit to any script or to `Cargo.toml` changes the generation
  identity, so the next setup or reconfiguration publishes another full copy (H13).
- **Cross-version state.** Frozen generations and the live checkout run different versions of
  the admission and test-resource code against one host ledger. This is a hypothesis.
- **New tables.** A new owner table passes codegen and compilation. Restore then copies and
  byte-verifies it but leaves it out of the logical owner comparison until someone edits Python.
  That is a second, hand-maintained table classification (DP-01).

**Counterevidence.**
- §3.2 assigns "worker and recovery executable composition roots" to xtask.
- ADR-0164 requires immutable receiver generations; durable work genuinely needs an executable
  that cannot change underneath it.
- Python is a pragmatic systemd language.
- These justify immutability and a separate composition root. They do not justify:
  - an undeclared contract mirrored across three Rust modules and Python;
  - a frozen closure that is the whole development tree;
  - production roots sharing a package and feature names with fixtures and generators.

**Evidence:** *Implemented* for the spawn chain, the mirrors and the table omissions. Churn and
cross-version effects are *Interface-checked* hypotheses.

**Improvement direction.**
- A product-owned deployment contract (profile, context, receiver association, readiness and
  observation messages) declared once and consumed by every reader and writer.
- A supervisor closure bounded to the receiver's responsibility.
- The logical restore comparison derived from registry table classification.
- Product operator roots in an identifiable product location.

**Related:**
- [DA-036](development-tooling.md#da-036): the execution-profile authority.
- [DA-037](development-tooling.md#da-037): the supervisor imports the admission and
  test-resource web.
- [DA-033](#da-033): the file protocol is shared with Python.

## <a id="da-015"></a>DA-015 — Hosts hand-project runtime operations, request envelopes, effects and defaults

**Principles:** AP-02, AP-03, DP-14, DP-17, DP-18; §E.

**Refresh status:** narrowed for the modeling chain and widened by Plan 51. The 13 hard-coded
`declared_execution` chains are gone, and request-driven host routes reach the runtime's
preparation owner.

**Diagnosis.**
- **About 50 Plan 51 operations are projected by hand into four hosts:**
  - pse-py, with about 52 methods in `crates/pse-py/src/workflow.rs` and `workflow/*.rs`;
  - the Python facade, with about 54 functions in `python/pse/_workflow.py`;
  - xtask `pse-project`, whose `xtask/src/project_runtime.rs::Action` has 49 variants under a
    versioned JSON envelope with no generated schema;
  - FastMCP (`scripts/capabilities/native.py::BINDINGS`), whose receivers are strings, whose
    effects are prose, and whose request models carry their own bounds.
- **Defaults such as the 16 MiB document bound and branch `"main"` are re-spelled per host.**
- **Hosts also compose operations.** `Action::StudyAnalyze` freezes a cohort, aggregates,
  registers its own memory consumer, encodes IPC and writes a file.
- **pse-py still composes document admission itself and calls
  `CanonicalStore::initialize_database` directly.**
- **Zero `ModelingLimits` are refused only in the binding.** `pse_modeling::Limits` has public
  fields and no validated constructor.

**Consequence.** A new Plan 51 capability (change scenario "new project or exchange operation")
needs three or four hand-written host additions plus defaults. The hosts can diverge in bounds
and effect descriptions, and the CLI envelope can evolve without schema generation.

**Counterevidence.**
- Several hosts are legitimate: authoring, a one-shot CLI, an agent interface.
- ADR-0164 has each host assert its own deployment.
- Payload documents are generated and not duplicated.
- No current divergence beyond the duplication itself was found.

**Evidence:** *Implemented* for the enumerations and duplicated defaults.

**Improvement direction.**
- A runtime-owned operation catalogue declaring each operation's name, request and result
  documents, defaults, bounds and effects.
- Hosts project the catalogue mechanically.
- The CLI envelope is schematized.

**Related:** [DA-009](structure-and-invariants.md#da-009) (kind-internal projection) and
[DA-019](#da-019) (product roots in xtask).

## <a id="da-017"></a>DA-017 — Solver capability truth is split between static records, settings-dependent capability and runtime name checks

**Principles:** AP-02, DP-15, DP-22, PS-09; G7, G2 (PSE-S02).

**Refresh status:** retained. The residue of DA-016 merges here: solver identity outside the
adapter seam. Uno's record is fixed.

**Diagnosis.**
- **Published capability is the static record.**
  - `pse-backend-native/src/execution.rs::Table::published` maps `a.capability()`.
  - KINSOL's static record still lists `DeclaredFixedPoint` among its automatic classes.
  - The newer settings-dependent `execution/kinsol.rs::capability_for` gives fixed-point no
    automatic class.
  - `routing::problem_classes` never derives `DeclaredFixedPoint`.
  - Plan 49 added the authoritative settings-dependent capability without deriving the
    published row from it, so the two now disagree inside the adapter.
- **PETSc methods are offered to Python but always refused.** The generated Python settings
  offer `nonlinear_additive_schwarz` and `pseudo_transient`, which `execution/petsc.rs` refuses
  before work. No Python route reaches the runtime composition owner that implements them.
- **The runtime still names backends:**
  - `workflow/completion.rs` maps any dynamics method other than IDAS to `Diffsol` with a
    wildcard;
  - the POUNCE recovery ladder is gated by `Backend::Pounce` in `math/solves.rs` and
    `math/strategy/target.rs`;
  - KINSOL is forced for declared roots in `math/initialization.rs` and
    `workflow/strategies/conditional.rs`.
- **The architecture overview omits Uno, PETSc and their sys crates.**

**Consequence.**
- *PSE-S02:*
  - A third dynamics library compiles, and its runs are recorded as Diffsol in canonical
    `computation_runs`, a silent G2 misattribution.
  - Another NLP backend offering recovery profiles needs runtime edits.
- *G7:* the published inventory claims an automatic route that routing never takes.

**Counterevidence.**
- Every refusal is typed and happens before effects, so the silent-fallback clause of G7 holds.
- §0.5 assigns fixed-point maps to KINSOL, and only POUNCE currently exposes a ladder.
- The exhaustive `adapter(Backend)` match is a strength.

**Evidence:** *Implemented*. The coordinator re-read the KINSOL record, `published` and the
dynamics wildcard.

**Improvement direction.**
- Published capability is derived from the same settings-dependent capability that admission
  uses.
- Runtime reads backend identity and ladder eligibility from the adapter's resolved method and
  declared capability.
- Settings offer only routes the boundary can reach.

## <a id="da-020"></a>DA-020 — Development-tooling families and tools are compiled into product crates and contracts

**Principles:** AP-01, AP-04, DP-02, DP-17; H26, H27.

**Refresh status:** retained and widened.
- Plan 49 decision I21 kept `pse-runtime::tooling` in the runtime and the observation hub in
  `pse-diagnostics`. It found that a separate tooling crate would need a runtime/catalogue seam,
  with no independent build benefit established.
- That decision stands for the runtime module. The widening below happened after it. The dependency-direction half
is corrected: product result-selection contracts now live in `workflow/result_selection.rs` and
tooling consumes them.

**Diagnosis.**
- `pse-runtime::tooling` (23k lines: capture, review packs, inspection comparison, taskdump) is
  compiled unconditionally into the product orchestrator.
- `workflow/reduction.rs` (a source reducer that re-parses equation text) adds a further
  diagnostic tool there.
- Plans 50 and 52 then placed development stores in the product registry and store client:
  - the tooling evidence index and the development-knowledge corpus are declared in
    `pse-schema/src/catalog/{evidence,knowledge}.rs` under `Namespace::Runtime`, a namespace the
    vocabulary defines as "execution records and their results";
  - they are generated into the foundation crate (`pse-model/src/generated/runtime/{evidence_*,knowledge_*}.rs`),
    product Python contracts and generated relation docs;
  - they are compiled unconditionally into `pse-operations` (`pub mod evidence; pub mod knowledge;`);
  - the only consumer is `xtask/src/evidence_tools.rs`.
- The product depends on tooling. `workflow/project_portability.rs` validates scientific IPC
  through `tooling::review_pack` and uses `tooling::contained_copy`.
- A product crate compiles files from the tooling tree:
  `tooling/review_query.rs` `include_str!`s `scripts/diagnostic_sql/*.sql`.

**Consequence.**
- **Rebuild amplification.** A development-knowledge format change regenerates `pse-model`,
  which sits beneath nearly every crate, plus product Python contracts and docs. It rebuilds
  `pse-operations`, then `pse-runtime`, then `pse-py`.
- **Tooling in production artifacts.** The product extension and the worker carry tooling
  operators.
- **Disposable data inherits canonical machinery.** The derived corpus gets staged generations,
  compare-and-swap activation, replay reconciliation and issuer-bound cursors (H24, H26). It does
  not inherit the canonical refusal discipline: its hand SurrealQL throws free text.
- **Product depends on tooling.** Product import depends on review-pack tooling, which is why
  tooling extraction remains blocked.

**Counterevidence.**
- ADR-0222 deliberately reuses the existing generator and service rather than adding a store.
- The registry gives one declaration and generated codecs.
- A separate database and interpretation string exist.
- §23.1's shared-interpreter rationale applies to frozen inspection.
- These justify one generator and one service. They do not justify tooling types in the
  foundation semantic model, or tooling operators in every product build.

**Evidence:** *Implemented* for placement, the missing gating and the sole consumer. Build cost
was not measured.

**Improvement direction.**
- A scoped tooling partition of registry output: namespace, generated target and crate feature.
- Contracts shared by product and tooling (IPC lineage validation, contained copy, review
  documents) live in an owner beneath both.
- Lifecycle machinery for derived tooling corpora proportionate to their rebuildability.

## Resolved, merged and moved identities

| ID | Refresh outcome | Where its meaning now lives |
|---|---|---|
| <a id="da-012"></a>DA-012 | Resolved for the run lifecycle | `run.rs::RunSupervisor::start_native` serves every kind, and the horizon forwards the caller's cancellation (`horizon/driver.rs` ≈124). Declared-procedure dispatch merges into [DA-009](structure-and-invariants.md#da-009). Strategies stay nondurable by maintainer decision. |
| <a id="da-013"></a>DA-013 | Resolved | One pure `pse-model::study_policy::{prediction_entry, prediction_choice}` is consumed by both executors. The copied sequencing flow is noted under [DA-009](structure-and-invariants.md#da-009). |
| <a id="da-014"></a>DA-014 | Closed by decision | Fit and Simulation points bind parameter values (`study_operations.rs`). Horizon parameter overlays, Initialize, Shooting and Check are not study operations by maintainer decisions I22 and RC12, and they refuse explicitly. |
| <a id="da-016"></a>DA-016 | Narrowed and merged | KINSOL fixed-point and arclength suitability moved into the adapter seam. The remaining name checks merge into [DA-017](#da-017). The PETSc NASM and pseudo-time composition is a legitimate PS-09 composition owner. |
| <a id="da-018"></a>DA-018 | Reframed and closed by decision | Most of the Uno patch is integration (accounting, cancellation, admission). Four numerical hunks and the SQP-LBFGS dense-Hessian route are retained and declared by decision I23. Status literals are named constants. The supported problem size of that route is undeclared outside proposed ADR-0157, a contract-text gap under [DA-023](local-reasoning-and-assurance.md#da-023). |

The removal commit of this refresh preserves their 2026-10-21 text.
