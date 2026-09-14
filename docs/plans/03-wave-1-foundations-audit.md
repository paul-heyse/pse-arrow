---
title: Wave 1 foundations implementation audit
status: in-progress
date: 2026-09-14
plan: docs/plans/03-wave-1-foundations.md
blueprint-revision: 31
evidence: Interface-checked source audit with scoped Tested receipts
---

# Wave 1 foundations implementation audit

The original checkpoint below is retained as historical evidence. The
[bounded correction addendum](#bounded-correction-addendum--2026-09-14-1156-utc)
and [bounded evidence refresh](#bounded-evidence-refresh--2026-09-14-1225-utc)
supersede the identified historical findings only. The latest refresh closes A-03 at
its bounded scope and records revision 31 semantic evidence. Overall completion remains
incomplete. The [Appendix B scope correction](#appendix-b-scope-correction--2026-09-14-1306-utc)
records additional registry omissions exposed by terminal governance.
The [terminal evidence refresh](#terminal-evidence-refresh--2026-09-14-1447-utc)
is the latest status: A-01–A-08 have bounded closure; A-09 and final acceptance remain open.

## Verdict and audit boundary

**Wave 1 is not complete at this audit checkpoint.** Most foundations have substantive
implementations and focused behavioral tests. The required Q-material helpers remain
empty scaffold modules. Several specified acceptance cases are absent, the latest
integrated engine command has one timeout, and the complete exit commands have not been
qualified against the final source tree. No numerical IDAES equivalence is claimed.

This is a read-only implementation audit of the approved combined sequence and packet
inventory in [plan 03](03-wave-1-foundations.md). The inspected checkout is branch
`wave1/foundations-completion`, based on commit
`115c08fbc657f6603a6a673a23133ff9f14eb03e`, with extensive concurrent uncommitted changes.
Blueprint revision 30 and proposed ADR-0052–ADR-0059 define the authorized correction
scope. A commit identifier alone does not reproduce this dirty checkpoint. Findings
below describe the inspected source; later fixes need their own source and execution
review before closing a row.

No Cargo, native build, golden regeneration, environment repair or production edit was
run by this auditor. `just doctor` and `just --list` were run; source was traced with
`rg`, bounded reads and the installed `ast-grep`. The parent owns the single Cargo lane.
Reported Rust receipts explicitly enable `pse-relations/force-validate` through the
repository recipes. The failure baseline for every command is zero.

**Inspection limits:** every plan packet is mapped below, but this was not an exhaustive
manual comparison of every registry column or all invariant fixture bodies against the
blueprint. Assembly, regeneration and invariant iteration were traced to their actual
consumers, with targeted negative cases inspected. Numerical algorithms, later-wave
placeholders, remote PR status, hardware power-loss behavior and new changes made after
the cutoff were not qualified. The separate implementation design review owns the
broader DM/G semantic assessment. Later worker assignments are remediation plans, not
evidence that these findings are closed.

Status meanings used here:

- **Complete at source boundary:** implementation and its consumers were found; this
  does not imply final executable acceptance.
- **Partial:** meaningful implementation exists, but a required behavior or acceptance
  case is still missing.
- **Missing:** the required implementation or test is absent.
- **Unverified:** source exists, but no applicable successful final receipt establishes
  the claim. Older receipts are scoped evidence, not final certification.

## Required corrections found

| ID | Priority and state | Evidence and consequence | Closure required |
|---|---|---|---|
| A-01 | High — **Missing** | `crates/pse-material/src/element.rs`, `phase_validity.rs`, `stoichiometry.rs` and `testdata.rs` contain only comments. `pse-material` exports their modules, but there is no `ElementTable`, `molecular_weight`, `species_valid_in_phase`, `element_balance` or phase-validity test. Q-material is required; its explicitly deferred neighboring features do not defer these helpers. | Implement the bounded material helpers and their positive/negative actual-value tests; run the packet command and final gates. |
| A-02 | High — **Partial** | `tests/engine/tests/commit_p0_p2.rs::actual_p3_outputs_match_in_process_durable_and_uncached_execution` commits only `package.toml` and exercises unchanged inputs. `memo_dependencies.rs` deliberately collides lookup buckets and changes descriptors, but does not commit a species addition and trace the actual P0/P1/P2/P3 consumers. C-4 explicitly requires changed source content to invalidate the affected computation and equal the clean result. | Add an actual changed-source commit and Driver run, compare complete rows with a fresh uncached execution, and inspect the relevant statuses/keys/attempt records. Preserve an unchanged-input positive control. |
| A-03 | Medium — **Missing acceptance** | `pse-rules/src/plan/head.rs` calls the exact scalar conversion helpers and checks full destination meaning. No test in `pse-rules/tests` exercises head conversion boundaries directly. `pse-quantity/src/numeric.rs` tests exactness helpers, which establishes only the quantity half of the required `head_conversion_exactness` fixture. | Exercise actual rule compilation/execution for exact and inexact integer/float literals, nullable and required destinations, incompatible quantity contracts and enum meaning; assert typed refusal at the head boundary. |
| A-04 | Medium — **Partial acceptance** | `publication_fault_matrix.rs` injects only `call: 1` for each `relations/`, `manifests/` and `refs/` prefix. The fixture store can select later calls, but no test enumerates every object write in a multi-object publication. Existing final-footer failure, corrupt-object restore/reuse, CAS conflict and cancellation controls are substantive. | Cover interruption before every object and manifest/ref boundary, including a later immutable object after an earlier object succeeded; preserve the exact old visible head and observed live-owner baseline. |
| A-05 | Medium — **Partial acceptance** | B-canon requires crate-local property tests. `pse-ids/tests/canonicalization.rs` and the conformance transform matrix use fixed fixtures and fixed permutations; no canonicalization `proptest!` is present despite the dev dependency. The fixed matrix does compare actual canonical preimages and decoded values. | Add the required generated-layout/property coverage, retaining actual-value or preimage oracles and minimized reproducible seeds. Map the E4 regression explicitly instead of treating the old probe as v2 proof. |
| A-06 | Medium — **Partial** | `benches/benches/canonicalization.rs` replaces the placeholder with genuine nullable, unsorted 1,000/10,000-row canonicalization. Its only benchmark operation is the complete `canonicalize` call. B-fixtures asks for a Criterion group per canonicalization stage. | Supply per-stage observations or record an explicitly approved scope correction; `just bench-smoke` must run the final benchmark inventory. No performance result is currently established. |
| A-07 | Medium — **Unverified** | ADR-0050 explicitly leaves independent `b3sum` confirmation of the frozen identity vectors as a Wave-exit item. The plan repeats that limitation; no independent confirmation receipt is present in the inspected source or cited ledger. | Record a reproducible independent vector confirmation, including exact framing inputs and command/tool identity. This checks the encoding contract, not semantic validity. |
| A-08 | High — **Unverified / failing current gate** | Latest integrated engine receipt E-01 has 9 passed, 1 timed out, 0 skipped. The P3 reuse test reaches the default 120-second termination limit. Previous green or longer-profile receipts precede current producer revalidation and do not clear this failure. | Qualify the final test under the required mode; a diagnostic CI-profile run is useful evidence but does not silently replace the default gate. |
| A-09 | High — **Unverified final acceptance** | Current generated files are partly untracked, source changed after the last golden/codegen checks, and the native environment is stale. Full `ci-pr`, feature powerset, release and pinned parity remain open. Outcome headings are empty and plan status remains `in-progress`. | Finish the terminal matrix below, reconcile final review findings and record a candid plan outcome. |

The independent design review also examines semantic/resource contracts and failure
recording; its findings are additional to this plan-coverage audit. Avoid treating either
review as a substitute for the other or for executable gates.

## Approved combined sequence

| Step | Current source/consumer evidence | Completion assessment |
|---|---|---|
| 1. Environment and checkout governance | `scripts/workspace.rs`, `tests/governance/tests/workspace_location.rs`, recipe argument and setup tests remove checkout assumptions. Fresh `just doctor` finds one stale environment blocker and stale extension warning. | Checkout correction **Implemented**, older focused **Tested** evidence; environment refresh **Unverified**, currently not ready. |
| 2. Correct K-1–K-5 | Registry keys, typed manifest wire fields, fallible quantity indices/rationals and diagnostic membership are checked by current builders and negative tests. | **Implemented** with scoped older tests; final workspace gate open. |
| 3. Registry A-1–A-5 | `catalog::declare` wires every required section into `RegistryBuilder::build`; exact rule expressions, joins/recursion, document identities and migrations project into self-description rows. Closed executable pass declarations are P0–P3; fixture P10 is separate. | **Implemented**; source coverage and selected execution are real, latest full registry/invariant matrix remains to run. |
| 4. Admission and generation | Generated builders/views call recursive relation validators; codegen produces Rust, Python and docs; xtask regenerates into scratch and compares both directions plus tracking hygiene. | **Implemented**, regenerated earlier; current final generation equivalence **Unverified**. |
| 5. Reservations and canonicalization | Canonical preflight reserves before the stages; retained buffers and detached children hold shared leases. Transform tests inspect canonical streams and preserve original sorted values. | **Implemented / partially Tested**; A-05/A-06 acceptance gaps remain. |
| 6. Storage, providers, sessions and budgets | Publication/read paths admit whole rows, invoke semantic validation, then compute identities. Sealed providers enforce actual filter expressions; sessions use one finite pool and active plan admission. Local refs compare actual control bytes under a lock. | **Implemented / scoped Tested**; A-04 and final acceptance open. |
| 7. Quantity, ordered math and material | Quantity inference uses full kind/unit/basis/reference components and registered operations. Math canonicalization types occurrences before sharing and preserves ordered payloads. Material helpers are stubs. | Quantity/math **Implemented**, final tests open; Q-material **Missing** (A-01). |
| 8. DSL, authoring and candidate P2 | Parsed source spans and generated contracts flow through staging, target resolution and exact change preimages into a constraint-free candidate session. P2 runs actual relational invariants before publication. | **Implemented / scoped Tested**; rule-head acceptance gap A-03 remains. |
| 9. Commit/Driver/P10/rename/memo/goldens | `Driver::commit` runs P0/P1/P2 and moves one ref after all immutable artifacts; P3 Driver executes/reuses admitted outputs; P10 runs only the complete fixture producer graph. Rename source edits are reparsed and bindings preserved. | **Implemented / partial acceptance**: full rename passes E-01; changed-source incremental test A-02 and timeout A-08 remain; goldens need final refresh. |
| 10. W-exit and outcome | Independent reviews are underway; plan Outcome is empty. | **Unverified / incomplete**. No `done` verdict is justified. |

## Packet-by-packet evidence

These rows trace the actual consuming paths and map renamed/consolidated test files to
their obligations. A missing historical filename is not itself a missing behavior.

| Packet | Source and consumer wiring | Acceptance evidence / residual state |
|---|---|---|
| K-1 | `pse-ids::{derive,frame,float,encoding,resource,snapshot}` feed registry fingerprints, canonical frames and catalog snapshots. `golden_vectors.rs` freezes each context and frame. | **Implemented**; older packet tests recorded in plan; independent check A-07 open. |
| K-2 | Workspace manifests, ADR-0049–0051 and generated targets exist. New ADR-0052–0059 remain proposed rather than being falsely accepted. | **Implemented** within authorized amendments; formal PR/decision lifecycle remains open. |
| K-3 | `RegistryBuilder::build` admits declarations before materializing/fingerprinting; `catalog::assemble` invokes the concrete declaration list. | **Implemented**; `registry_assembles`, `registry_admission`, `fingerprint_stable` provide scoped tests. Stale scaffold prose remains in module docs. |
| K-4 | Quantity dimensions/indices, graph insertion payload checks and material identity/enums are concrete. | Keel **Implemented**; does not imply Q-material helper completion. |
| K-5 | Catalog manifest structs, strict decoding, failure classification, runtime budgets and relation contracts are consumed by store/session code. | **Implemented / scoped Tested**, including malformed metadata/version/calendar tests in the catalog suite. |
| A-1 | `s6_2_physical`, `s6_3_domains`, `s6_4_material`, `s6_5_property` are called by catalog assembly; quantity adapter consumes actual physical rows. | **Implemented**; domain/quantity adapter and invariant tests inspect changed actual values. Final matrix pending. |
| A-2 | Template/instance declarations and §6.14 dictionary mappings are generated into Rust/Python; normalization and target resolution consume them. | **Implemented**; pinned enum/class/discretization parity test exists but current container execution is **Unverified**. |
| A-3 | Symbol/math relation families, operator specs, normalized copies and expression source/payload relations feed P3, relation adapters and P10. | **Implemented**; current smoothing/enum additions postdate older generator and test receipts. |
| A-4 | Cases, targets, numerical/runtime/provenance declarations are assembled. `s14_passes` projects actual closed input/output contracts. | **Implemented** under ADR-0056 correction: 17 blueprint passes are not all executable. P4–P9/P11–P16 are intentionally deferred, not a missing Wave 1 implementation. |
| A-5 | Auto PK/FK and explicit closure/domain rules are compiled and executed by `pse-rules`; declaration checks reject invalid graphs and float keys. | **Implemented**; all 588 fixture pairs passed in E-02's matrix, while that aggregate command still had one other failure. Re-run after latest schema changes. |
| A-6 and final regeneration | Rust generators emit relation constants, typed rows/builders/views, enums and document structs. Views check actual schema and recursive values; `p1` and adapters consume generated rows. | **Implemented**; generation determinism and forged-field tests exist. Final tree regeneration and `codegen-check` **Unverified**. |
| A-7 | `pse-relations::{ext,validate,cells,migrate,registry_relations}` recursively validate nested metadata/value layouts; catalog and generated interfaces call them. Migrations admit both endpoints. | **Implemented / scoped Tested** in `admission.rs`, `generated_contracts.rs`, store/encoding tests; matching-fingerprint invalid controls are real. |
| A-8 | Python generator emits frozen nested attrs rows, enums, msgspec manifest and extension types. `pse.codec` structures actual rows with strict hooks; generated golden reader consumes IPC. | **Implemented / scoped Tested**; final `py-sync`, full Python and enum parity open. |
| A-9 | Markdown/JSON Schema generators consume admitted registry; generated SUMMARY links enter the book. | **Implemented**; E-07 book build passed before final additions; current docs gate open. |
| R-1 | `xtask::codegen` validates roots, rejects symlinks, prunes stale files, generates Python manifest, compares fresh trees and rejects untracked output. | **Implemented**; explicit malformed/missing/extra/untracked filesystem tests exist. E-06 proves generation only, not current check. |
| E-1 | `SharedRuntime::build` constructs FairSpill/TrackConsumers/Peak pools and checks limits; `PoolReserver` and owned buffers retain claims. | **Implemented / scoped Tested**; pool/process peak are distinct observations, not memory equivalence claims. |
| B-canon | `canon::api` preflights, reserves, orders/adopts complete schemas, normalizes nested/null/dictionary values and writes finished streams before identity. | **Implemented / partial acceptance**; fixed storage matrix is real, required property tests A-05 missing. |
| B-providers | Sealed catalog/schema lists own admitted snapshots; `RelationTable::scan_with_args` applies supported physical filters and reports exactness conservatively. | **Implemented / Tested** by current full-result oracle E-01; concrete TableProvider method inventory exists in governance. |
| B-store | `publish_bundle` calls port and stage admission before canonicalization; `read_manifest` verifies encodings and semantic admission before returning a Snapshot. Ref update is separate CAS; local exact-byte protocol implemented. | **Implemented / scoped Tested** by E-04/E-05 and catalog protocol tests. No power-loss hardware experiment claimed. |
| B-session | `session::{config,registry,admission,profile}` and candidate sessions enforce active field/plan/function/source admission and explicit settings; SessionFactory retains the same runtime/reserver. | **Implemented / scoped Tested**; wrong candidate/foreign plan and owned result controls exist in catalog tests rather than historical engine filenames. |
| B-fixtures | Canonical transform matrix compares complete preimages and decoded streams; IPC/Parquet tests compare actual admitted contents before identities. | **Implemented / partial acceptance**; whole-operation benchmark exists, per-stage requirement A-06 missing. |
| Q-1 | Quantity registry admission validates unit numbers, reference/basis relationships, canonical coordinates and full semantic keys; P3/P10 adapters reconstruct these from actual rows. | **Implemented / scoped Tested** in `registry_admission.rs`, `quantity_relations.rs`; newest domain and smoothing cases need final execution. |
| Q-3 | `infer`/`infer_with_evidence` use built-in and registered operation rules; selected permutations/conversions are carried to math inference. | **Implemented / scoped Tested** for point/difference, ambiguity, scoped weight proof, registered integral and invalid full contracts. |
| Q-4 | `standard_registry` is feature-gated fixture data, with explicit unit/kind/basis/reference declarations; numeric exactness helpers feed rule heads. | **Implemented / scoped Tested** in `standard_package`, `dimension_props`, numeric tests. YAML platform authority remains a stated later-wave item. |
| Q-material | Only IDs, enums and errors exist; exported helper modules have no items. | **Missing** A-01. |
| M-1 | Operator table, structural/payload hash frames, topological admission and typed numbering are consumed by canonicalization and generated operator rows. | **Implemented / scoped Tested** in `ordered_foundations` and `hash_consing_props`; actual payload/order equality precedes hash comparison. |
| C-dsl | Winnow LocatingSlice tokenization feeds a deterministic parser/renderer, explicit spans/budgets, expression/equation/predicate ASTs and binding-aware traversal. | **Implemented / scoped Tested**: fixed 1,024-case text property, feature-gated 1,024-case AST property, committed persistence file, reviewed example snapshots. Parser uses committed token control flow rather than literal `cut_err` combinators; behavior controls exist. |
| C-1 | Package reader consumes generated document declarations, exact original text, YAML budgets, TOML spans and explicit/named IDs; ID assignment edits actual parser ranges. | **Implemented / scoped Tested**. Four hostile YAML controls live in `document/value.rs` tests, not a separate `hostile_yaml.rs`. |
| C-2 | Typed row-key/staged-role change sets validate exact preimages. P0 checks versions/dependencies; P1 consumes generated rows and resolves wildcard targets from actual domain facts. Rename reparses complete sources and preserves bindings. | **Implemented / scoped Tested**, including full atomic rename in E-01. No named-policy rename shortcut. |
| M-3 | Guarded literal folding checks ordered direct literals, finite results and excluded regions; canonicalization records deferred checks. | **Implemented / scoped Tested** in `guarded_folding`; independent review is checking all guarded operator families. |
| M-4 | Canonicalization checks all graph cycles, resolves indices/domains, infers each occurrence, folds, then exact typed-shares/numbers nodes. Residual and selection rows persist. | **Implemented / scoped Tested** in `indexed_driver`, `physical_edges`, `ordered_foundations`; latest contracts require final execution. |
| C-3 | Relational compiler lowers scans/filters/projections/joins/unions/distinct/count/unnest/bounded recursion into candidate DataFusion plans; exact head admission precedes output metadata. P2 records actual violating and undecided keys. | **Implemented / partial acceptance**: independent P2 oracle and invariant matrix exist; A-03 head fixture missing. Explicit non-count/conflict/strata/recursive per-row provenance deferrals remain. |
| B-faults | Actual store wrapper injects failures/corruption/CAS/cancellation; local protocol tests race independent catalogs and reject stale bytes with equal ETags. Final encoder footer failure is tested directly. | **Implemented / partial acceptance**: E-04 passes scoped tests; full per-object interruption matrix A-04 missing. |
| B-oracle | Actual RelationTable versus unpruned MemTable queries compare complete RowConverter multisets; negative exactness lies cause missing/empty/duplicate result differences. | **Implemented / Tested** by E-01, including all specified query shapes. |
| E-2 | Runtime SessionFactory delegates one pool to catalog sessions; 200k-row query fixture leaves 64 KiB after live snapshot ownership; failed/cancelled claims return to measured baseline. Detached result buffers retain leases. | **Implemented / scoped Tested** in prior lifecycle receipt; terminal lifecycle run open. |
| B-evidence | Plan codec/session round trips have partial concrete support; `evidence.rs` remains deferred. | **Explicitly deferred**, R-28. Not a Wave 1 completion blocker. |
| R-3 | Bindgen arm exits clearly when unavailable; current binding check remains hygiene-only. | **Explicitly deferred**, R-29. Do not call this regeneration equivalence. |
| Q-5 | `quantity_composition.rs` sends real heater/composition expressions through strict canonicalization and inspects result kinds, reference states and bases; incompatible datums/bases/missing rules fail. | **Implemented / Tested** in E-02, subject to later-source requalification. |
| M-5 | MathRelationSink/Source and compiler adapters emit/load typed and untyped graphs; loader rejects malformed actual references even with claimed hashes. Ordered payload/property tests and guarded-fold tests cover the P10 policy subset. | **Implemented / scoped Tested**; named historical `numerical_policy_conformance.rs` is covered across concrete math tests, not a missing behavior merely because of its filename. Final complete P10 fixture gate remains open. |
| C-4 | Driver connects real commit/P3 execution, exact dependencies, stage hints, full producer revalidation, output admission/publication and pass records. Complete fixture registry/importer provides P10 inputs without synthetic later passes. | **Implemented / partial acceptance** A-02/A-08; independent design review additionally checks terminal failed/cancelled attempt records. |
| R-2 | Golden command commits sources, publishes/reopens/query-checks Model/Case/P3 and separate P10 fixture. Observation compares complete literal rows/source inventory/SQL, then identities; Python decodes actual registry IPC. | **Implemented / previously Tested** E-08; stale after revisions 29–30 and requires regeneration/check. |
| W-exit | Required review and audit are underway; exit matrix and plan outcome are not complete. | **Unverified / incomplete**. |

## Executable evidence inspected

Log files below are local ephemeral receipts, not committed evidence bundles. They contain
test results but generally do not capture the entire dirty source state. Commands are
the repository commands recorded in the execution ledger and parent handoff; these
receipts were inspected, not rerun by the auditor.

| Receipt | Exact command and mode | Result against zero baseline | What it establishes / freshness |
|---|---|---|---|
| E-00 | `just doctor`; host checkout, pinned-tool inventory | **1 blocker**, exit 1; stale native-extension warning | Current environment needs `just py-sync`. `just --list` succeeds and confirms the command surface. |
| E-01 | `just test-package pse-tests-engine -p pse-relations --test commit_p0_p2 --test memo_dependencies --test pushdown_vs_unpruned --test stage_bundle_graph --no-fail-fast`; default nextest, force-validate | **9 passed / 1 timed out / 0 skipped**, run `8e04a9b2-2f6e-4ac7-9a0f-0b40ff1057b9`; `/tmp/pse-wave1-engine-integrated11.log` | Current commit 106.389 s, case binding 72.692 s and full rename 88.144 s pass. P3 reuse terminated at 120.007 s. This is a failing command. |
| E-02 | `just test-package pse-tests-conformance -p pse-compiler -p pse-mathir -p pse-rules -p pse-relations --test invariant_fixtures --test quantity_composition --test quantity_relations --test relation_roundtrip --test invariant_execution`; default, force-validate | **21 passed / 1 failed / 0 skipped**; `/tmp/pse-wave1-leaf-gate29.log` | All 588 invariant fixture pairs passed in the matrix test; the aggregate command failed in full P10 fixture Model construction. Receipt predates latest schema additions. |
| E-03 | `just test-package pse-compiler -p pse-tests-engine -p pse-relations --test quantity_relations --test relational_expansion_p2`; default, force-validate | **6 passed / 0 failed / 0 skipped**, run `b2e3367f-554d-4dae-88bc-4bb78f9cbe32`; `/tmp/pse-wave1-leaf-gate30.log` | Retests corrected P10 fixture and independent P2 keys; does not rerun the entire E-02 matrix. |
| E-04 | `just test-package pse-tests-lifecycle -p pse-catalog -p pse-relations --test local_refs --test publication_fault_matrix --test encoding_admission`; default, force-validate, Unix local filesystem | **13 passed / 0 failed / 0 skipped**; `/tmp/pse-local-publication-gate.log` | Scoped local CAS, cancellation, corruption and encoding controls. Does not prove power-loss behavior or all later source changes. |
| E-05 | `just test-package pse-catalog -p pse-relations --test encoding_admission`; default, force-validate, 512 MiB budget | **7 passed / 0 failed / 0 skipped**; `/tmp/pse-ipc-registry-regression.log` | Full self-description IPC layout and ownership regression; does not certify simultaneous complete Driver execution. |
| E-06 | `just codegen`; schema targets | **0 failures reported**, three targets; `/tmp/pse-wave1-codegen-final-tooling.log` | Generator ran before latest revisions; not a final `codegen-check` receipt. Bindgen explicitly skipped. |
| E-07 | `just quality`; host Python/repository checks. `just docs`; mdBook | **0 failures reported** in `/tmp/pse-wave1-quality-current.log` and `/tmp/pse-wave1-docs-second.log` | Quality includes 14 setup tests; book builds. These receipts predate final source changes and are not Python behavior or native acceptance. |
| E-08 | `cargo xtask golden registry`, `cargo xtask golden minimal_explicit`, `cargo xtask golden registry --check`, `cargo xtask golden minimal_explicit --check`; 512 MiB fixture runtime | **0 failures reported**; check logs `/tmp/pse-wave1-golden-registry-check.log`, `/tmp/pse-wave1-golden-minimal-check.log` | Reopened fresh/stored complete values, sources and SQL agree for that earlier schema. Revisions 29–30 make regeneration/recheck necessary. |

The plan's older 72-test authoring/adapter, 91-test port and 265-test catalog/lifecycle
receipts remain useful scoped history. Their counts must not be combined into a current
workspace pass count, and the older 265-test command had one failure. Likewise the
historical keel ledger records earlier branches/PRs; it is not current remote status.
The earlier strict log `/tmp/pse-wave1-tooling-final-strict2.log` ends with 12 compiler
diagnostics. Source repairs and subsequent successful checks must replace it explicitly;
its existence cannot substantiate a clean final lint claim.

## Terminal acceptance still open

| Required gate | Current state and acceptance condition |
|---|---|
| `just py-sync` then `just doctor` | Current environment blocker; rebuild the editable extension from the final lock/native source and confirm zero blockers. |
| `just governance` | Current registry/generated/provider tests must pass; includes `codegen-check` and `family-check`. Generated files must be refreshed and tracked through normal repository workflow. |
| `just test-package pse-authoring -p pse-relations` | DSL/source/rename tests exist; also run `--features pse-authoring/arbitrary` to exercise the actual AST property instead of only the default text property. |
| `just test-package pse-tests-conformance -p pse-relations` | Run final semantic/canonical/quantity/invariant matrix after all source changes; old 588-pair receipt is insufficient for a changed registry. |
| `just test-package pse-tests-engine -p pse-relations` | Must include changed-input Driver coverage and pass without timeout in the required mode. |
| `just test-package pse-tests-lifecycle -p pse-relations` | Complete fault and three budget fixtures after A-04 and review fixes. |
| Both golden write commands and both `--check` commands | Regenerate from final declarations; complete source/row/query equality must pass under admission, with no hash-only replacement oracle. |
| `just codegen-check` | Current dirty/untracked generated outputs and post-generation changes preclude a final claim. |
| `just py-test` and `just quality` | Run after native refresh and final generation; generated contract/golden behavior plus static/import checks must pass. |
| `just parity-container` | Execute generated compatibility dictionary/class/discretization tests in pinned IDAES 2.12.0 environment. Missing solver/version fails rather than skips. Numerical parity remains outside Wave 1. |
| `just bench-smoke` | Run final real benchmark inventory; no timing gate is implied. |
| `just ci-pr` | Final composite default Rust compile/lint/test/doctest, governance, policy, rustdoc, benchmarks, quality, ADR lint, docs and Python gate required. No successful final composite receipt is available. |
| `just features-powerset` | Once at exit; actual feature combinations and no-default-features compilation required. Not supplied by default test success. |
| `just test-release` | Once at exit with release Cargo profile and force-validate; optimization-dependent paths not established by dev tests. |
| Independent identity vectors | A-07, required by ADR-0050; final framing receipt needed. |
| Implementation review, this audit, plan outcome | Resolve findings, record deliberate deviations and corrected mistakes, and complete formal decision lifecycle. Only then assess `status: done`. |

## Deliberate deviations and stale descriptions

- Combined branch execution supersedes historical parallel worktree/PR topology. The
  historical topology is not an unfulfilled production requirement.
- ADR-0056 restricts executable production passes to P0–P3 and qualifies P10 against a
  complete predecessor fixture. Blueprint inventory of later passes is not executable
  availability. Optional B-evidence/R-3 deferrals are recorded as R-28/R-29.
- ADR-0057 makes EquationFamily/EquationRole authority self-contained in blueprint §7.5;
  the missing `semantic_math_basis` source is resolved by an authored narrow contract.
- ADR-0053/0054/0055/0058/0059 replace historical interface sketches for change sets,
  physical/indexed payloads, reservation ownership, units and identity projection.
  These changes have substantive source consumers; proposed status still requires the
  formal decision process rather than a silent accepted flag.
- Historical test filenames were consolidated. Hostile YAML tests reside in the parser
  module; semantic admission in relation/catalog tests; policy conformance in math tests;
  demand seed checks in `pse-compiler/tests/normalization.rs`. Those are valid mappings
  where the actual behavior is asserted.
- `pse-schema/src/catalog/mod.rs` still says the section modules are empty and
  `codegen/mod.rs` says phase 0 emits nothing. Current bodies contradict those comments.
  `tests/engine/tests/phase0_placeholder.rs` still describes superseded plan-byte equality
  and a survivor-only pushdown oracle. The placeholder tests in the test layers are not
  evidence of their prose obligations and must not inflate completion claims.

The final completion verdict remains **incomplete**, with real implementation progress,
the missing material packet, explicit acceptance gaps and terminal gates listed above.

## Bounded correction addendum — 2026-09-14 11:56 UTC

**Scope:** rereview of A-01, A-03, A-04, A-05, A-06 and A-07 only, at the parent's
request. Live sources and the receipts below were inspected while compiler/source-memory
integration continued. This addendum does not requalify A-02, A-08, A-09 or any terminal
gate; the preceding initial findings remain historical, not assertions that corrected
files still contain their earlier defects. No Cargo/native process or production edit
was run by this auditor. Only this audit document was updated.

| Finding | Current correction and actual consumer evidence | Bounded verdict |
|---|---|---|
| A-01 — material helpers | `pse-material/src/element.rs` now admits element IDs and finite positive atomic masses; `molecular_weight` consumes explicit elemental counts, refuses malformed/duplicate/unknown inputs and recomputes after actual mass changes. `phase_validity.rs` distinguishes null defaults, explicit empty lists, actual species membership and unresolved facts using three-valued conjunction. `stoichiometry.rs` computes ordered phase/species element residuals from explicit molar terms and compositions, returns unbalanced residuals as data, and refuses unsupported basis conversion. Public exports and feature-gated `testdata` call these implementations. Unit tests and `tests/phase_validity.rs` exercise the actual arithmetic, row mutations, ordering and refusal controls. | **Implemented and Tested** by C-01. Original missing packet is **closed at this bounded scope**; broader final gates remain open. |
| A-03 — exact rule-head acceptance | `pse-rules/tests/head_conversion_exactness.rs` now calls actual `compile` and `execute`. Four tests exercise both exact numeric conversion directions and boundaries, typed null/nullability narrowing, quantity mismatch, same-spelling different enum contracts and an accepted target enum literal. The first actual execution exposes failures in three tests: accepted integer-to-float, float-to-integer and enum-literal output paths fail semantic admission. | Test omission **corrected**, required behavior **still failing**. C-05 is **3 passed / 3 failed / 0 skipped**; A-03 remains **open** pending implementation correction and successful rerun. |
| A-04 — every publication object | `publication_each_object.rs` records a successful eight-write trace: three relations, each IPC plus Parquet, followed by manifest and ref. It interrupts every observed position on local storage, asserts the exact trace prefix and that the fault fired, then reopens all three previous relations and compares every old cell. It checks both the observed-reference reservation baseline and zero after final owners drop. This now covers later writes after earlier immutable objects succeeded, not only the first prefix match. | **Implemented and Tested** by C-02. Original per-object coverage gap **closed at this bounded scope**. Power-loss behavior remains outside this test. |
| A-05 — generated canonical properties | `pse-ids/tests/canonical_properties.rs` generates arbitrary floating bit patterns, validity masks, null garbage and reordered split batches. `tests/conformance/tests/canonical_properties.rs` independently constructs dictionary codes, nested masked lists/structs, row permutations and slices; it first compares decoded logical values, then complete preimages. Its second property changes a visible nested value and requires a different preimage. Three properties run 128 cases each, with bounded shrinking; default Proptest failure persistence remains enabled. | **Implemented and Tested** by C-02, **384 generated cases** plus existing fixed controls. Original property-coverage gap **closed at this bounded scope**; this is bounded testing, not a proof for every layout. |
| A-06 — actual stage benchmarks | Production `canonicalize` and feature-gated `canonicalize_stages` share `canonicalize_observed`. Six instrumentation boundaries surround actual preflight, admission/order, normalization, metadata construction, finished IPC streams and frame/hash work. Each benchmark iteration still executes the complete reserved pipeline; Criterion receives the selected stage duration. The instrumentation regression compares complete preimages and sorted output with ordinary execution, checks ownership release and refuses a one-byte budget. Benchmarks cover the whole operation plus six stages at two row counts. | **Implemented and Tested** by C-02 instrumentation regression and C-03's **14 successful smoke cases**. Original benchmark inventory gap **closed at this bounded scope**. No timing/performance conclusion is claimed. |
| A-07 — independent identity vectors | `docs/capability-maps/evidence/identity-vectors-adr0050.md` and its JSON receipt preserve the literal preimages, derive contexts, XOF lengths, expected/actual output, GCC command/version and six official C source/header digests. The recorded oracle separately compiles portable official C from the resolved BLAKE3 1.8.7 source, with SIMD disabled, and reports all 11 vectors matching. `b3sum` was absent; this is an independent language implementation/invocation from the same upstream project. The auditor checked all six cached source/header bytes against recorded provenance and independently reconstructed all 11 ADR literal preimages, comparing bytes directly with zero discrepancies. | **Tested receipt inspected; preimages and source provenance Interface-checked.** Original missing independent-confirmation receipt **closed at this bounded scope**. Auditor did not rerun the native C oracle. This confirms framing/cryptographic output only, never semantic validity. |

### Correction receipts

All test failure baselines are zero. These receipts apply to their named source
boundaries and do not establish final workspace acceptance.

| Receipt | Exact command, mode and result | Scope / limitation |
|---|---|---|
| C-01 | `just test-package pse-material -p pse-mathir -p pse-quantity -p pse-catalog -p pse-relations --lib --test phase_validity --test indexed_driver --test physical_edges --test standard_package --test inference_contracts --test session_validation_budget --test session_admission --test session_codec --no-fail-fast`; default nextest, force-validate; **220 passed / 0 failed / 0 skipped**, run `24a9c7a1-0553-4add-9fa7-b97067a64e84`; `/tmp/pse-wave1-leaf-review-final.log` | Includes the real material helper and phase predicate tests. Other tests in this command are not a fresh broad design review by this auditor. |
| C-02 | `just test-package pse-catalog -p pse-ids -p pse-relations -p pse-tests-conformance -p pse-tests-lifecycle --features pse-ids/bench-instrumentation --test store_protocol --test canonical_properties --test canonicalization --test canonical_null_equivalence --test encoding_roundtrip_identity --test publication_fault_matrix --test publication_each_object --no-fail-fast`; default nextest, force-validate; **34 passed / 0 failed / 0 skipped**, run `54e996ab-14fe-4556-af57-da67b6718024`; `/tmp/pse-audit-catalog-focused3.log` | Includes three generated properties, production instrumentation equivalence/resource control and interruption at every observed object write. |
| C-03 | `just bench-smoke` expands to `cargo test --benches -p pse-benches -p pse-relations --locked --features pse-relations/force-validate`; test/dev mode; **14 benchmark cases successful / 0 failed**; `/tmp/pse-audit-canon-bench-smoke.log` | Whole canonicalization and each of six actual stages at 1,000 and 10,000 rows. Smoke execution establishes operability, not measured performance. |
| C-04 | `cargo clippy -p pse-ids -p pse-catalog -p pse-benches --all-targets --locked --features pse-ids/bench-instrumentation,pse-relations/force-validate -- -D warnings`; default feature mode; **0 diagnostics**, successful dev completion in `/tmp/pse-audit-catalog-strict-core.log` | Scoped strict check; does not replace both workspace Clippy legs or final source checks. Exact invocation supplied by parent because the captured log begins with build output. |
| C-05 | `just test-package pse-schema -p pse-rules -p pse-relations --test compiled_contract --test head_conversion_exactness`; default nextest, force-validate; **3 passed / 3 failed / 0 skipped**, run `8b991e73-5f44-464f-aa0e-c722cd9073ac`; `/tmp/pse-wave1-contract-head-corrections.log` | Two compiled-descriptor tests and nullable-head test pass. Accepted numeric conversions and enum output fail; no successful rule-head qualification is claimed. |
| C-06 | Official C compilation and per-vector replay commands in [the repository identity-vector receipt](../capability-maps/evidence/identity-vectors-adr0050.md); GCC 13.3.0, portable C BLAKE3 1.8.7; recorded **11 passed / 0 failed** | Exact source/preimage/output inventory is inspectable. Current rereview independently checked input/provenance bytes, not native oracle execution. |

The stale catalog/codegen module prose identified in the initial audit has been corrected,
and the obsolete engine placeholder test has been removed. Those documentary corrections
do not supply runtime evidence. No additional substantive omission was found in the six
bounded corrections beyond A-03's now-exposed execution failures.

**Current bounded verdict:** A-01/A-04/A-05/A-06/A-07 are corrected with the scoped
evidence above. A-03 remains open. A-02/A-08/A-09 and the complete final acceptance
matrix are not closed by this addendum. Wave 1 status remains **incomplete**.

## Bounded evidence refresh — 2026-09-14 12:25 UTC

**Scope:** corrected rule-head behavior, shared validation allocation accounting,
revision 31 source-projection/invariant integration, owned authoring, and internal
terminal classification. This is a source-and-receipt refresh requested by the parent;
it does not repeat the complete packet audit or qualify source changes made afterward.
No Cargo/native command or production edit was run by this auditor. The original
checkpoint and C-05's three failures remain historical evidence; D-01 explicitly
supersedes C-05 for the corrected head implementation.

| Plan boundary / finding | Actual implementation and consumer evidence | Current bounded state |
|---|---|---|
| A-03 — exact rule-head acceptance | `pse-rules/src/plan/head.rs::prepare` first checks literal representability and destination meaning. It lowers the accepted concrete literal to a scalar, casts that checked value to the destination storage type, and constructs the expression with the destination field metadata. Column expressions still require the actual logical type and quantity contract, with no nullability narrowing. `exec/mod.rs::admit` compares the retained field with the declared head and validates each output array against that full field. The four direct compile/execute tests cover numeric boundaries, typed nulls, quantities and distinct enum contracts, including accepted outputs. | **Implemented and Tested**, D-01 and D-02. A-03 **closed at this bounded scope**: all four head tests and both descriptor tests pass. Metadata or storage agreement alone is not the oracle. |
| Validation allocation accounting | `pse-ids/src/validation_extent.rs` computes checked visible-buffer, recursive-slot and dictionary-expansion extents. Its documentation explicitly distinguishes a reservation forecast from semantic admission. Catalog membership delegates to this shared helper, and candidate/plan admission obtains scratch reservations before decoding. D-02 exercises encoding admission and tiny-budget/session ownership boundaries, including reservation release and cancellation. | **Implemented and Tested** for the named boundaries; D-03 supplies a scoped strict check. This does not establish every resource path or replace the independent semantic validators. |
| Source/row correspondence at publication and reopen | `pse-compiler/src/validator/sources.rs::validate` merges the actual admitted rows, reparses retained owned source documents, runs P0 resolution and P1 staging, and refuses any resulting operation in the applicable Model/Case snapshot class. `CompilerValidator::validate_snapshot_sources` wires this into catalog admission. `tests/engine/tests/source_projection_admission.rs` accepts unchanged matching sources/rows, refuses a real package-name row mutation with unchanged source at publication, and refuses reopening a fully re-encoded/rehashed manifest whose rows disagree with its source. | **Implemented and Tested**, D-04. The adversarial fixture recomputes valid content identities; refusal therefore exercises actual source correspondence rather than hash inequality. This test is not A-02's changed-input incremental Driver scenario. |
| Current registry invariants and P3/P10 fixtures | The live fixture tree contains **595 invariant directories and 1,190 YAML files**, with a valid/violating pair in every directory. `tests/conformance/tests/invariant_fixtures.rs` asserts exact equality between the registry-derived fixture-directory set and the actual set, then executes both cases and compares complete finding keys. D-04 also exercises the real P3 normalization and P10 complete-predecessor quantity fixtures. | **Implemented and Tested**, D-04: the passing current matrix qualifies **595 pairs**. Earlier 588-pair receipts remain historical. The inventory count alone is not the behavioral evidence, and P10 fixture qualification does not implement P4–P9. |
| Owned authoring and rename | `document/owned.rs` retains document bundles, bindings and reservations behind shared immutable owners; clones share those owners. Registry validation compares actual declarations and selected document definitions. D-05 exercises source-loader budgets, allocation lifetime, hostile stale declarations/rows and both text/AST properties. D-06's `owned_rename_and_amend_retain_allocations_and_refuse_stale_sources` calls the owned rename/amend path and checks retained allocations and actual stale-source refusal. | **Implemented and Tested** at these boundaries. D-05 is a successful behavioral receipt with two unique compiler warnings; it is not a zero-warning quality receipt. The parent reports those qualification warnings corrected before D-06, whose log contains no warnings. |
| Failed/cancelled attempt records | D-06 runs the internal `records::tests::all_actual_cancellation_leaves_remain_cancelled` and `mixed_aggregate_preserves_every_actual_finding_and_first_non_cancel_class` tests, exercising actual nested failure classification and retained findings. | Internal classification **Tested**. Public Driver terminal-attempt publication tests were still being added at this cutoff; their consumer path and receipt remain **unverified** here. No terminal-record acceptance closure is claimed. |

### Refresh receipts

All failure and warning baselines are zero. These source-sensitive receipts are not
additive workspace totals. Captured temporary logs are local evidence, not committed
reproducibility artifacts for the concurrently modified checkout.

| Receipt | Exact command, mode and result | Evidence limits |
|---|---|---|
| D-01 | `just test-package pse-schema -p pse-rules -p pse-relations --test compiled_contract --test head_conversion_exactness`; default nextest, `pse-relations/force-validate`; **6 passed / 0 failed / 0 skipped**, run `239da197-f1fa-422e-aece-da5d7049527f`; `/tmp/pse-wave1-contract-head-corrected.log` | Two compiled-descriptor and all four rule-head tests pass. Replaces C-05's failed execution for this boundary. |
| D-02 | `just test-package pse-catalog -p pse-ids -p pse-rules -p pse-relations --lib --test session_validation_budget --test head_conversion_exactness --test encoding_admission`; default nextest, `pse-relations/force-validate`; **190 passed / 0 failed / 0 skipped**, run `0e78c9fb-7708-46dc-9b64-c07b835d82c9`; `/tmp/pse-wave1-shared-allocation-head.log` | Includes a second passing execution of all four head tests plus the named catalog/session allocation and encoding boundaries. |
| D-03 | `cargo clippy -p pse-ids -p pse-catalog -p pse-rules -p pse-relations --all-targets --locked --features pse-relations/force-validate -- -D warnings`; default feature mode; **0 diagnostics**, successful dev completion; `/tmp/pse-wave1-shared-head-strict-final.log` | Exact command supplied by parent because the log captures build output only. This is a scoped check; a later workspace `just clippy` reportedly found four test-lint issues being corrected, so no current workspace-clean claim follows. |
| D-04 | `just test-package pse-tests-engine -p pse-tests-conformance -p pse-compiler -p pse-relations --test source_projection_admission --test invariant_fixtures --test normalization --test quantity_relations`; default nextest, `pse-relations/force-validate`; **21 passed / 0 failed / 0 skipped**, run `2094b0a9-19c5-4532-b9ed-dcbc7f03d9bb`; `/tmp/pse-wave1-rev31-semantic-integration.log` | Actual source mismatch publication/reopen refusal, current 595-pair invariant matrix, P3 normalization and P10 fixture behavior. The matrix is one test in this 21-test count, not 1,190 additional nextest tests. |
| D-05 | `just test-package pse-authoring -p pse-relations --features pse-authoring/arbitrary --no-fail-fast`; default nextest, `pse-relations/force-validate`; **54 passed / 0 failed / 0 skipped**, run `87091011-fa06-4a98-a5fc-d2d83106978f`; `/tmp/pse-wave1-owned-authoring-final2.log` | Both DSL properties execute. The log reports **2 unique unnecessary-qualification warnings**, duplicated for lib/lib-test builds; the zero-warning baseline was not met by this receipt. |
| D-06 | `just test-package pse-authoring -p pse-compiler -p pse-relations --lib --test rename --no-fail-fast`; default nextest, `pse-relations/force-validate`; **19 passed / 0 failed / 0 skipped**, run `8e4713f7-9558-4edc-9de5-7f5951212050`; `/tmp/pse-wave1-terminal-owned-unit.log` | Owned rename/amend and internal terminal classification are exercised; **0 warnings** in the inspected log. Does not rerun all 54 D-05 tests or qualify public terminal-attempt publication. |

A subsequent engine diagnostic is current counterevidence, not a completed gate:
`just test-package pse-tests-engine -p pse-relations --profile ci --no-fail-fast`
(`pse-relations/force-validate`) in `/tmp/pse-wave1-engine-rev31-ci.log` was interrupted
after **18 passed, 3 test failures and 1 SIGINT**, with **12 tests not run**. Nextest
reports **4 failed / 0 skipped** because it counts the interrupted test as failed;
the failure baseline remains zero. The three actual commit failures report the absent
`authored.change_sets@1` dependency while validating a change-operation sidecar. The
parent is correcting the local-artifact validation boundary and adding a production
regression; no correction receipt was available at this cutoff. A-08/A-09 remain open.

**Current bounded verdict:** A-01/A-03/A-04/A-05/A-06/A-07 now have the scoped
corrections and evidence recorded in these addenda. A-02, A-08 and A-09 remain open or
unrequalified here. Public terminal-attempt behavior, final generated/golden equality,
the complete `just ci-pr`, `just features-powerset`, `just test-release` and pinned
parity gates remain open. The packet table's historical states must be read with these
explicit corrections; neither the growing focused test inventory nor valid content
hashes establish terminal acceptance. **Wave 1 remains incomplete.**

## Appendix B scope correction — 2026-09-14 13:06 UTC

**Interface-checked:** the first terminal `just ci-pr` attempt exposed an Appendix B
registry-coverage omission reported by the parent. This bounded review classifies the
14 actual missing relation names against the current blueprint, plan packets A-1–A-4
and R-2, the plan's incomplete-contract exception, and ADR-0056. It does not supply a
new executable receipt or a complete count for that terminal command. No Cargo or
production edit was run. The original packet audit did not independently expand and
compare every literal Appendix B name; its registry-completeness assessment was too
broad. The explicit omissions below correct that assessment.

ADR-0056 distinguishes **deferred executable pass registration** from **unspecified
relation contracts**. A complete relation declaration remains required even when its
producing pass is outside Wave 1. Conversely, the approved incomplete-contract exception
permits an explicit reasoned deferral when authoritative meanings are absent; it does
not authorize invented fields, opaque JSON payloads, guessed enum members or hash-derived
replacement identities. The existing eight-row deferred file does not yet account for
these omissions. A listed reason is governance bookkeeping, not behavioral validation.

| Missing name | Authority and contract completeness | Required scope disposition |
|---|---|---|
| `authored.measurement_models` | Blueprint §6.10 names `measurement_id`, `observation_id`, DSL `model_expression`, `error_model` and `parameters`. The `parameters` representation is absent and no `ErrorModel` member list exists in the inspected design/registry. §19.4 explains estimation behavior without completing those contracts. | **Permitted incomplete-contract deferral**, with these exact missing meanings stated; otherwise author a narrow contract before implementing A-4. Estimation being a later capability is not itself the reason. |
| `authored.tags` | Blueprint §19.2 lists six display fields and two group defaults, but supplies no complete tag key/ownership declaration or representation/binding for the group defaults. The expression's document/owner contract is also absent. Appendix A identifies intended phase-1 coverage but is not a row declaration. | **Permitted incomplete-contract deferral** of the unresolved authored/group contract, or narrow authoring completion. Do not fabricate a tag ID, entity kind or group payload simply to satisfy the index. |
| `inferred.phase_equilibrium_species` | Blueprint §9.5 supplies schematic `(pair, j)` and `role ∈ {vl, henry, l_only, v_only}`. The actual §6.5 pair relation uses the complete `(property_package_id, phase_a_id, phase_b_id)` key, not a scalar `pair_id`. §9.5 also says membership is the phase intersection while including one-phase role values, leaving the row universe/classification relationship unresolved. | **Permitted incomplete-contract deferral**, or narrow clarification of the complete pair key and role/membership semantics before A-1 declaration. A fabricated hashed pair identity is not a valid repair. |
| `inferred.state_flash_required` | Blueprint §9.2 explicitly declares the unary `state_instance` fact and its rule; §6.7/§7.6 distinguish decided-true heads from `inferred.undecided`. There is no unspecified payload requiring a new semantic contract. | **Missing required declaration** under the A-1/A-2 registry inventory. Register the unary decided-true relation and its actual reference/key contract; executing the later inference rule is a separate scope. |
| `inferred.tear_candidates` | Blueprint §6.7 names the relation with no columns. §12.5 explains feedback-arc selection and authored tear costs; §17.4 explains consumption. None specifies candidate/family identity, row shape or selection representation. | **Permitted incomplete-contract deferral**. The specific reason is absent row contract, not deferred topology execution. |
| `inferred.connection_equations` | Blueprint §12.3 gives the complete four-field mapping `(connection_id, member_ordinal, index, equation_id)`. The related §6.7 port-member and §6.9 indexed-equation contracts supply the existing identity, ordinal and index meanings. | **Missing required declaration** under A-2. Preserve that mapping and actual typed references. P8/P12 execution remains deferred separately. |
| `inferred.initialization_order` | Blueprint §17.3 gives `(instance, ordinal)` and its ordering meaning, with preparation forward and finalization reversed. The referenced instance identity and ordered relation meaning are specified. | **Missing required declaration** under A-2's inferred inventory. Declaring this ordered fact does not claim to execute initialization plans. |
| `compiled.math_alternative_sets` | Blueprint §6.9 supplies a name and optionality purpose only; §19.7 explains GDP lowering without supplying row columns or membership semantics. | **Permitted incomplete-contract deferral** under A-3/ADR-0056. |
| `compiled.math_alternatives` | Same §6.9 declaration gives no alternative row, set-membership key or activation/equation payload contract. | **Permitted incomplete-contract deferral** under A-3/ADR-0056, separately named from its set relation. |
| `compiled.math_static_analysis` | Blueprint §6.9 literally begins the row with `...`; only `parameter_dependence : list<sid>` is specified. §7.5 and §18.7 explain classification semantics but do not complete the per-equation/per-variable row shape. | **Permitted incomplete-contract deferral** under A-3, preserving the parameter-dependence obligation for its eventual contract. |
| `compiled.scaling_plans` | Blueprint §6.11 names `plan_id`, `case_id`, scaler `template_id` and `options`, but never defines the `options` payload. Other relations' key/value options are not an authority to copy silently. | **Permitted incomplete-contract deferral**, or narrow explicit options contract before A-4 declaration. |
| `compiled.solve_plans` | Blueprint §6.11 supplies all row fields and their meanings; §18.7 supplies the solve classes `SQUARE_NLE`, `NLP_LOCAL`, `DAE_INTEGRATE`, `MINLP`, `GDP` and explains justification/modifiers. | **Missing required declaration** under A-4. The complete schema/dictionary must exist even though P15 selection and solver execution do not. |
| `runtime.solver_events` | Blueprint §6.13 supplies the complete four-field row `(run_id, ordinal, kind, message)`. However, `SolverEvent` has no member list anywhere in the inspected blueprint, ADRs or handwritten registry. Backend-output prose in §18.6 supplies no closed dictionary. | **Narrow dictionary authority gap**, not an absent row list. Prefer authoring that dictionary and implementing the A-4 declaration; an explicit incomplete-contract deferral is permissible only if it names this actual unresolved enum. Do not guess members or cite later runtime execution as the reason. |
| `provenance.closure_report` | Blueprint §14.5 specifies nine categories, closed/open status and a list of open items, but no complete open-item element type, subject association or relation key. The plan explicitly includes this name in A-4. | **Permitted incomplete-contract deferral under ADR-0056**, but record it as a deviation from A-4's named deliverable with the missing association/payload contract stated. Alternatively author that narrow contract. P14's deferral alone is insufficient. |

`pse.expr_dsl` in the normalized Appendix B cell is an inline logical-type annotation
inside the explanation of expression families. It is not a relation named
`normalized.pse.expr_dsl`; correct the inventory parser instead of registering or
deferring that fabricated name.

**Bounded verdict:** four names have sufficient existing row contracts and remain
required missing registry declarations. Eight have explicitly incomplete relation
contracts. The remaining two, `phase_equilibrium_species` and `solver_events`, require
particular care over an unresolved membership/key specification or absent dictionary
authority. These are **14 distinct omissions**. Every resulting declaration must reach assembly, generated
contracts, membership and invariant fixtures; every permitted deferral must state its
actual missing authority. No source correction or successful governance rerun is
claimed here. **Wave 1 remains incomplete.**

## Terminal evidence refresh — 2026-09-14 14:47 UTC

**Scope and verdict:** the current source and latest supplied logs now establish the
previously missing changed-source execution, public terminal-attempt controls and
Appendix B accounting. A-01–A-08 are **closed at their reviewed scope**, with the
conditions and deferrals below. A-09 remains **open**. The latest complete Rust test leg
passes, but the composite PR command failed afterward; its generator correction has
only scoped requalification. Pinned parity currently fails during import. Feature
powerset, release and a successful final composite command remain unverified.

This is a bounded refresh of the earlier step/packet audit, not an exhaustive new
column-by-column blueprint review. The auditor read source and logs and updated only
this artifact; the parent ran every native command. Concurrent source changes after
each receipt still require the applicable checks. Logs under `/tmp` are local receipts,
not a committed reconstruction of this dirty checkout.

### Current combined-sequence assessment

| Step | Current source and executable evidence | Status / limit |
|---|---|---|
| 1 — environment and governance | Local Python 3.14.7 unit/component execution passes 60 tests; quality emits no lint/type findings under its configured suppressions. Current pinned-container execution fails importing generated contracts, and the local quality/doctest receipts include tool warnings described below. | **Partial**; local success does not qualify the pinned parity environment or a zero-warning final command. |
| 2 — K-1–K-5 corrections | The current workspace force-validate run exercises canonical framing, identity-independent descriptor/value admission, diagnostic classes, rationals, indices and ownership controls. | **Implemented and Tested**, F-01; earlier scoped implementation evidence remains applicable, not a formal proof. |
| 3 — registry/rules/migrations | All four sufficiently specified Appendix B omissions now have real declarations in `s6_5_property.rs`, `s6_7_instances.rs` and `s6_11_numerical.rs`, including `SolvePlanClass`. Assembly calls these functions; generated relation contracts exist. The corrected index parser excludes only explanatory inline types. Both F-01 and F-03 pass exact registered-or-deferred coverage. The deferred file now contains **18 explicit unresolved contracts**, with the ten additions and original A-4 closure-report deviation recorded in ADR-0056. | **Implemented and Tested within explicit scope**. The ten incomplete contracts are deliberate deferrals, not implemented relations or a reason to expose their later producers. |
| 4 — semantic admission and generation | Actual generated field/row validators run in F-01/F-03. The first narrow governance graph exposed JSON object ordering dependent on `serde_json/preserve_order`, despite workspace regeneration passing. `codegen/jsonschema.rs` now recursively sorts objects after adding DSL syntax metadata; arrays retain declaration order. Its new regression checks nested serialized definitions and required-column ordering. | **Corrected and scoped Tested**, F-02/F-03. A final composite run after this correction remains open. |
| 5 — reservations/canonicalization | F-01 reruns actual owned-buffer admission, hidden/null canonical cases, generated properties and finish failure controls. The prior six-stage benchmark and independent official-C identity receipts remain scoped evidence. | **Implemented and Tested**; hash equality remains supplementary to actual typed values and ownership controls. No new performance claim. |
| 6 — catalog/session/lifecycle/budgets | F-01 reruns all-object publication interruption, local refs, actual query admission, detached-result ownership and terminal recording/storage/CAS controls. `query_memory_budget.rs` retains its **512 MiB shared pool / 64 KiB headroom** stress controls. | **Implemented and Tested** for the exercised boundaries. Hardware power loss remains outside this suite. |
| 7 — quantity/material/math | F-01 reruns the quantity, material, ordered/guarded math, actual P3 normalization and P10 fixture tests. The current invariant tree has **600 valid/violating pairs (1,200 YAML files)**. Its passing matrix asserts exact registry-to-directory equality and executes both cases against complete expected finding keys. | **Implemented and Tested**, F-01. Earlier 588/595-pair counts are historical; no numerical IDAES equivalence follows. |
| 8 — authoring/rules/P2 | F-04 exercises all 57 current authoring tests, including the feature-gated AST property and retained-source allocation controls. F-01 runs actual unpublished P2 admission and public failure/cancellation tests. | **Implemented and Tested** on the Rust side. Generated Python import compatibility still blocks pinned parity. |
| 9 — actual Driver/memo/goldens | `incremental_equals_clean_p0_p3.rs` adds oxygen in exactly one retained source document, commits through real P0/P1/P2, requires P3 `Ok` on changed input and `Reused` on unchanged input, then compares all decoded Model/P3 rows and exact source text against a separate uncached fixture. F-01 passes this test in **211.587 s**, and the complete in-process/durable/uncached P3 control in **152.557 s**. Four F-07 golden commands publish/reopen both fixtures and compare complete admitted rows, source bytes and SQL results. | **Implemented and Tested**; closes A-02 and A-08 at this scope. Required later-pass production execution remains deferred. |
| 10 — terminal acceptance and outcome | F-01's 659-test workspace leg and all executable doctests passed, followed by **47 passed / 1 failed** in narrow governance. F-03 corrects that specific failure. F-08 pinned parity then fails before test collection. No successful complete `ci-pr`, feature-powerset or release receipt is yet available. | **Incomplete**; A-09 remains open. Review/audit disposition and plan outcome must remain conditional. |

Ordinary complete workflow tests and golden execution now share the single **32 GiB
ceiling** in `tests/support/workflow_budget.rs`, consumed by the actual engine and
golden environment constructors. It is a ceiling, not an eager allocation or measured
requirement. Production `ResourceBudget` stays caller-configured. The default nextest
profile has a documented 360-second allowance only for the two measured long engine
tests; no failing test is silently rerun under a different profile. Small resource
refusal tests retain their deliberate local limits.

The public `terminal_attempts` fixture retains all production relation/enum/document
contracts and exactly seven terminal invariants, plus explicit fixture predicates. Its
eight tests cover pre/mid cancellation, pass findings, joined preconditions, P0/P2
refusal, storage/budget failure during recording and final CAS failure. The separate
full-registry sidecar tests exercise local-artifact scope and rehashed invalid terminal
rows. Actual `driver/commit.rs` persists the successful P2 record before `move_commit_ref`;
a subsequent CAS error retains that successful record/output as a distinct publication
outcome. These are exercised consumer paths, not only classification-unit tests. This
closes the earlier public-terminal evidence limitation without extending the fixture's
scope to later passes.

### Latest receipts and non-additive counts

The baseline is **zero failures and zero warnings**. Configured tool suppressions are
stated explicitly where visible; successful commands with warnings are not described
as warning-free. All Rust nextest/test invocations below enable
`pse-relations/force-validate`.

| Receipt | Exact command, mode, result and provenance | What it establishes |
|---|---|---|
| F-01 | `just ci-pr`; its default `cargo nextest run --workspace --locked --features pse-relations/force-validate` leg: **659 passed / 0 failed / 0 skipped**, run `bb27ca10-5ad9-4625-92f2-5ec94ebef05d`, 666.656 s. `cargo test --doc --workspace --locked --features pse-relations/force-validate`: **33 passed / 0 failed**, with **1 Cargo warning** that `pse-py`'s cdylib does not support doctests. The subsequent governance test leg: **47 passed / 1 failed / 0 skipped**, run `2a2dbccf-729d-40e6-b3e3-94fb454c6d87`. `/tmp/pse-wave1-ci-pr-workstation.log` | Whole default Rust behavior and both strict Clippy legs pass at that checkpoint. The composite command fails at `registry_generated_files_match_current_declarations`; later recipe dependencies do not receive a successful composite receipt. |
| F-02 | `just test-package pse-schema -p pse-relations --test codegen_determinism`; default nextest: **3 passed / 0 failed / 0 skipped**, run `8053d7b0-0ac0-4793-896d-6d0407bba309`; `/tmp/pse-wave1-order-narrow.log`. `just codegen`: **0 failures**, all three schema targets regenerated; `/tmp/pse-wave1-order-codegen.log` | Recursive JSON object ordering correction, complete generation and retained DSL grammar. Optional bindgen remains deferred explicitly. |
| F-03 | `just governance` (`cargo xtask governance`); default nextest subcommand: **48 passed / 0 failed / 0 skipped**, run `ac032915-195f-4598-8c48-6268cb443532`, followed by successful three-target `codegen --check` and family checks; `/tmp/pse-wave1-order-governance.log` | Corrects F-01's narrow-feature generation failure and rechecks Appendix B, generated admission, family pins and evidence-lock agreement. Does not replace a full post-correction PR gate. |
| F-04 | `just test-package pse-authoring -p pse-relations --features pse-authoring/arbitrary --no-fail-fast`; default nextest: **57 passed / 0 failed / 0 skipped**, run `a5351f53-d800-4f1a-ad1e-7ba8d48813ba`; `/tmp/pse-wave1-authoring-retained-exit.log` | Current text/AST properties, loader, retained-source and rename controls; no compiler warning in the inspected receipt. Supersedes D-05's warning-bearing authoring checkpoint for these tests. |
| F-05 | `just test-package pse-tests-engine -p pse-relations --test memo_dependencies --no-fail-fast`; default nextest: **3 passed / 0 failed / 0 skipped**, run `74fd5f32-1df4-4c6b-9581-68582c823add`; `/tmp/pse-wave1-memo-owned-corrected.log` | Exact dependency/lineage/absence, retained source and engine-profile comparisons under deliberately equal lookup keys. The separate changed-source Driver test in F-01 supplies the full consumer evidence. |
| F-06 | `just quality`: **0 reported findings**, Pyrefly **0 diagnostics (3 suppressed)**, import contracts **4 kept / 0 broken**, setup tests **14 passed / 0 failed**; `/tmp/pse-wave1-quality-warnings.log`. Zizmor reports **1 offline-mode warning**, with 23 ignored and 18 suppressed findings. `just py-test` (`uv run --no-sync pytest -m "unit or component" -n auto`): Python 3.14.7, 32 workers, **60 passed / 0 failed**, 3.00 s; `/tmp/pse-wave1-py-test-warnings.log` | Local Python behavior and configured static checks. Neither receipt establishes pinned parity; the quality receipt does not meet an unqualified zero-warning claim. |
| F-07 | `cargo run --quiet --package xtask --locked --features pse-relations/force-validate -- golden registry`, then the same command with `--check`; repeat for `minimal_explicit`. **Four successful commands / 0 reported failures**, 32 GiB shared workflow ceiling. Logs `/tmp/pse-wave1-golden-{registry,minimal}-workstation-{write,check}.log`; exact invocations confirmed by parent because logs capture application output only | Registry fixture publishes/reopens two snapshots; minimal fixture publishes/reopens three plus its explicit P10 predecessor boundary. Both checks reopen committed and fresh stores and compare complete decoded values, sources and SQL. `golden::compare` uses complete observation equality; a changed-value/unchanged-identity negative control also passes in F-01. |
| F-08 | `just parity-container` (`./scripts/parity-container.sh`); pinned container/parity recipe requests Python 3.13. `/tmp/pse-wave1-parity-exit.log` ends **exit 4, import/collection error, no parity tests executed**. It also reports an incompatible-environment warning from `uv --no-sync`, which requests Python 3.14.7 | Generated `values.py` eagerly evaluates `attrs.Attribute[object]` and raises `TypeError: type 'Attribute' is not subscriptable`. This is a current cross-version generated-contract blocker, not a skipped or passing parity suite. |

### Remaining closure blockers

1. Correct the generated Python runtime annotation and the parity interpreter-selection
   mismatch, regenerate normally, and qualify the pinned `just parity-container` gate.
   Local Python 3.14 success does not remove F-08's counterevidence.
2. Complete a successful final `just ci-pr` after the generator and command-surface
   corrections. F-03 closes the specific ordering regression; it does not certify the
   remaining policy/rustdoc/bench/quality/ADR/docs/Python legs as a completed composite.
3. Run `just features-powerset` and `just test-release` against the final source.
   Their feature combinations and optimized execution are distinct from F-01.
4. Resolve avoidable command warnings explicitly. `pse-py` already declares
   `doctest = false`; the explicit workspace `--doc` invocation still selects its
   unsupported cdylib. The targeted recommendation is `--exclude pse-py` on that
   doctest command, retaining all supported Rust doctests and Python boundary tests.
   Make local Zizmor's existing offline coverage explicit with `--offline`; the
   `repo-hygiene` workflow already supplies its token for online audits. These are
   command-selection corrections, not suppression of emitted warnings. Official
   [Cargo package selection](https://github.com/rust-lang/cargo/blob/master/doc/book/src/commands/cargo-test.md)
   and [Zizmor operating modes](https://github.com/zizmorcore/zizmor/blob/main/docs/usage.md)
   were checked through Context7; source edits belong to the parent lane.
5. Reconcile the final implementation review, these retained audit findings, deliberate
   ADR-0056 deferrals and the plan's outcome only after the remaining executable gates.
   Formal ADR acceptance follows the documented decision-PR lifecycle; proposed
   status is not silently converted by test success.

No additional confirmed production-contract omission was found in this bounded refresh
beyond the generated parity compatibility failure and the open final checks above.
This statement does not erase the earlier inspection limits or the 18 named incomplete
contracts. **Completion remains conditional and Wave 1 remains incomplete.**

### Subsequent scoped corrections — 2026-09-14 14:50 UTC

The following inspected source and receipts supersede the corresponding 14:47 blockers
without closing the final composite/feature/release gates.

| Boundary | Current evidence and disposition |
|---|---|
| Generated Python annotation | `codegen/python/values.py.template` now types the unused attrs callback context as `object`, while retaining exact value-type/range/finiteness/UTC checks. `test_validator_annotations_evaluate_on_supported_python_versions` forces `get_type_hints` on the factories and callbacks so Python 3.14 also detects eager-annotation compatibility problems. The pinned Python 3.13.15 execution below passes this regression. F-08's import defect is **corrected and Tested**. |
| Pinned compatibility roster | The HenryType parity assertion explicitly accounts for the sole upstream-only `Dummy` sentinel, asserts its actual upstream presence, requires exact equality for every remaining physical member, and requires the generated dictionary to reject `Dummy`. Blueprint §6.14 still owns the four physical members; no production enum was expanded. This is a narrowed, explicit upstream compatibility assertion, not a general allowance for extra members. |
| Pinned parity receipt | `just parity-container` in `/tmp/pse-wave1-final-parity.log`: Python **3.13.15**, **116 passed / 0 failed**, 7.54 s, including the preflight and compatibility tests; no tests skipped. It still emits **1 uv incompatible-environment warning** because sync requests 3.13 while the following `uv run --no-sync` inherits the repository's 3.14.7 request. The actual tested interpreter is 3.13.15. Behavioral parity is **Tested** at the exercised Wave 1 scope, but this command is not yet warning-free. |
| Quality and command warnings | `just quality` in `/tmp/pse-wave1-final-quality.log` completes with **0 reported diagnostics/findings and 0 warnings**, under the same explicit Pyrefly/Zizmor suppressions noted in F-06; setup tests **14 passed / 0 failed**. The local Zizmor command now selects `--offline`, and the doctest recipe uses `--workspace --exclude pse-py`. Source for both command corrections is **Implemented**; the corrected doctest invocation still needs its next actual receipt. |

The remaining parity warning has a concrete interpreter-selection correction: specify
the same Python 3.13 request on `uv run` as on `uv sync`. The current script still pins
only the sync command at this cutoff. Official [uv interpreter selection](https://github.com/astral-sh/uv/blob/main/docs/guides/scripts.md)
was checked through Context7. This requests the already intended interpreter rather
than hiding a warning. A post-correction receipt, the full PR command, feature powerset,
release execution, and final review/outcome reconciliation remain open. **Wave 1 remains
incomplete; no new confirmed production-contract omission remains from this bounded
refresh.**

**Additional inspected component receipt:** `/tmp/pse-wave1-final-pr-preflight.log`
records successful `cargo deny --all-features --locked check`, `cargo audit --deny
warnings`, and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`.
`cargo test --benches -p pse-benches -p pse-relations --locked --features
pse-relations/force-validate` runs all **14 benchmark smoke cases successfully**, plus
**1 relation test passed / 0 failed**. The subsequent quality commands report zero
diagnostics/warnings under the stated suppressions, setup tests **14 passed / 0 failed**,
ADR lint **59 records valid**, register lint **30 rows valid**, and `mdbook build docs`
completes. These are successful component gates with a zero-failure baseline, not a
completed full PR command. The bounded independent rereview of the JSON ordering,
unused Python callback annotations, and exact HenryType sentinel handling found no
concrete defect in those corrections.
