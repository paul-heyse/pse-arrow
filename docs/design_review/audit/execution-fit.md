# Execution fit

These findings concern work, movement, live state or lifecycle machinery that the supported
workloads do not appear to need: edit → re-solve, studies, recycles, dynamics and extension
(process-simulator functional target). They are assessed qualitatively under AP-07/G9.
Static evidence of avoidable work or amplification counts as architectural evidence. No
quantitative speed or capacity claim is made without a measurement, and none was taken.
Baseline and limits are in the [collection README](README.md). Citations are `path:line`
at source commit `a791c533e`.

## <a id="da-025"></a>DA-025 — Persistence machinery is sized for a more demanding deployment than the system's actual participants and recovery obligations

**Principles:** AP-07 (H1, H14, H25, H26, H27), DP-16, DP-19 ("without gratuitous leases or hostile-operator assumptions").

**Diagnosis.** The persistence boundary carries the following machinery.

- **Bookkeeping tables.** About 27 of the 62 canonical tables track lifetime, protection,
  retention, history, staging or coordination, by the slice reviewer's classification.
  Examples: file owners, borrowers, instances and admission; manifest tickets; protections
  and result protections; artifact obligations, reads and releases; history obligations,
  counts, identities and locators; reclaimed ranges; guards; roots; stages; retirements.
- **Server functions.** 116 hand-authored server functions.
- **Owner identity.** File owners are bound to Linux boot ID, PID namespace, PID, process
  start time and cgroup (`crates/pse-operations/src/canonical_files.rs:213-251`).
  Nonexpiring read tickets are released only on "authenticated owner death". Yet the online
  physical collector those protections arbitrate, `CanonicalStore::collect_file`
  (`canonical_files.rs:625`), has no production caller. Every caller is a test module
  (coordinator-confirmed).
- **Patched SDK.** A patched SurrealDB SDK is vendored (`vendor/surrealdb/PATCH.md`;
  `Cargo.toml:158`). The diff against crates.io 3.3.0 is several thousand lines; it
  carries deadlines, admission, replay and response limits.
- **Python supervisor.** An 8,874-line Python supervisor, `scripts/surreal_server.py`
  (see also [DA-019](boundaries-and-composition.md#da-019)), plus host admission,
  native-operation and maintenance scripts.
- **Operational protocols.** Restore, rollout and incarnation-rotation protocols exist,
  although §20.5 describes the design-phase store as disposable and regenerated.

Fine-grained allocator charging is a separate cause, covered in
[DA-022](local-reasoning-and-assurance.md#da-022).

**Consequence.**
- A single-user local simulator pays a client-server, multi-owner recovery protocol on
  every durable operation.
- Representative persistence changes (PSE-S04, PSE-S06) cross the SDK fork, the Python
  supervisor, generated SurrealQL and several protection tables.
- One layer's benefit cannot be realized in production, because there is no online
  collector. If physical payload reclamation is simply absent in production, payloads may
  also grow without bound. That is unresolved.

**Counterevidence (material).**
- Real concurrent writers exist: 16 worker lanes, the Python caller, and study summary
  writers.
- Crashes are real, and H25 warns against assuming they cannot happen.
- Attempt leases, fences and acknowledgement settlement protect DP-19 truthfulness.
- The remote server was a deliberate ADR-0164 tradeoff with stated costs.
- Logical forget and reclaim are reachable from Python and arbitrate through tickets.
- The fixed durable overhead in the edit → re-solve loop has not been measured.
  `canonical_joined_publication_progress_and_ordinary_cost_probe`
  (`workflow/durable.rs:4309`) is the existing probe for it.

**Evidence strength.** The inventory and the missing production caller are source-established.
The disproportion is a qualitative judgment. The cost is a hypothesis.

**Related.** [DA-007](structure-and-invariants.md#da-007), [DA-024](local-reasoning-and-assurance.md#da-024).

## <a id="da-026"></a>DA-026 — Result selection is a bespoke index and predicate engine whose transactional index writes grow with result rows, while DataFusion sees an unpruned serial scan

**Principles:** DP-13, DP-14, DP-16 (§E), AP-07 (H4, H7, H8, H16, H27); PSE-S04.

**Diagnosis.** For solve variables, solve constraints, fit parameters and fit observations,
the append protocol requires one `canonical_result_rows` locator per row. Each locator is
written into SurrealDB inside the guarded append transaction. This is on top of the Arrow
IPC payload, the block zone maps and the output descriptors
(`crates/pse-operations/src/generated/surreal.surql:1012-1060`).

Which relations, fields and owner columns are indexed is hand-coded across pse-operations
and runtime, not declared in the registry. The relation-name aspect is covered by
[DA-032](semantic-model-and-authority.md#da-032)
(`crates/pse-operations/src/canonical_execution.rs:2345-2360`;
`crates/pse-runtime/src/workflow/result_projection.rs:28-140`).

The query surface is a bespoke single-field predicate (`minimum`, `maximum`, `missing`)
evaluated with hand-built Arrow kernels (`result_projection.rs`, scalar predicate;
`connected_results.rs:140-155`). The DataFusion `SnapshotProvider` meanwhile reports every
filter as `Unsupported` and exposes one partition behind a mutex-held decoder
(`workflow/result_snapshot.rs:39-50`, `:686-727`). General DataFusion queries over results
therefore never reach the zone maps or the row index.

**Consequence.**
- Durable write work and the number of guarded round trips scale with variables plus
  constraints, not with useful payload. Per-row metadata extent drives how appends are
  split.
- Adding an indexed relation or field edits about four places across two crates.
- Pruning that the libraries provide natively (Parquet statistics, page index, bloom
  filters with DataFusion pruning) is re-implemented in the catalog.

**Counterevidence.**
- Exact bits, signed zero and missingness are preserved deliberately.
- Locators reference IPC rows rather than duplicating payload.
- Each transaction is bounded.
- The single-decoder choice is declared and compared in tests.

**Open library question.** Can the pinned Parquet/DataFusion route satisfy the result
contract? The contract requires exact f64 bits including NaN payloads, signed-zero
equality for numeric access, missingness, and recoverable row coordinates. Parquet float
statistics special-case ±0 and exclude NaN, so pruning correctness under these semantics
needs primary-source confirmation.

**Evidence strength.** *Implemented* for the structure. The magnitude of write
amplification is a hypothesis. Counting locator rows and append transactions for one
large solve would settle it.

## <a id="da-027"></a>DA-027 — Shared-body structure is grouped during specialization, re-expanded for the compiler, then rediscovered under a second identity

**Principles:** DP-06, DP-09, DP-16, AP-07 (H12, H14, H16, H17).

**Diagnosis.** Preparation handles shared bodies in four steps:
1. `group_bodies` scans every instance, then renames, renders and hashes it into dispatch
   groups (`crates/pse-modeling/src/specialize/group.rs:50-223`).
2. Its only consumer, `bound_bodies`, renames each group back into per-instance
   expressions (`group.rs:229-276`; called once at
   `crates/pse-compiler/src/workspace/modeling/executable.rs:401`; coordinator-confirmed).
   A test asserts the result equals `model.equations`.
3. The compiler projection then wraps each scalar output with clones of its transitive
   member bindings (`executable.rs:1257-1272`).
4. For each output it re-normalizes, runs a full physical-only `BodyBuilder` pass to
   capture premises, then renders and hashes. Only after that does the memoized
   `semantic_body` lookup happen (`executable/grouped.rs:938-1023`).

Two independent body-identity mechanisms therefore track the same meaning, and the
modeling-side one has no compact consumer. Proving reuse repeats the physical walk for
every output, however much sharing there is. Several full copies of the model's
expressions stay live during preparation:
- the specialized model;
- the `bound_bodies` maps;
- `Projection.expressions`, retained by a tracked query;
- interned `BodyInput` expressions.

**Counterevidence.**
- Symbolica materialization still runs once per distinct identity.
- `MathService` body retention prevents repeated materialization across workspaces.

**Evidence strength.** *Implemented*. Magnitude unmeasured.

**Related.** [DA-001](semantic-model-and-authority.md#da-001), [DA-030](#da-030).

## <a id="da-028"></a>DA-028 — Single-output bodies recompute shared member computations once per consuming output on every evaluation

**Principles:** AP-07, DP-10 (H12, H16); divergence from declared design.

**Diagnosis.** A compiler body is one scalar output expression (row, term, contribution or
member), with every transitively read member inlined
(`crates/pse-compiler/src/workspace/modeling/executable/grouped.rs:245-265`;
`executable.rs:1257-1272`). A member read by k outputs is therefore computed k times per
case evaluation, and so are its derivatives. Examples are mixing-rule or caloric `let`s
read by every fugacity row (`packages/reference/thermodynamics/models/pr-equilibrium.pse:134-141`)
and replica members read by a quadrature closure row (`projection_tests.rs:54-70`).

`CaseWorker` reuse applies only to identical programs with identical input bits
(`crates/pse-math/src/assembly.rs:3451-3560`), so it does not cover shared intermediates
inside different bodies. §7.1 says "a body covers one definition and its consumed finite
membership". Reading that as a per-definition body makes the code a divergence, but this
is an interpretation of the text. Either way, the per-output choice maximizes program
sharing across instances at the cost of recomputation within an instance, and that
tradeoff is not stated anywhere.

**Counterevidence.**
- Symbolica common-subexpression elimination works within a body.
- Implicit providers reuse their last product at identical inputs
  (`crates/pse-math/src/implicit.rs:2091-2115`).
- Whether term and contribution outputs are evaluated during solver iterations, or only in
  observation demands, is unverified.

**Evidence strength.** *Implemented* for the structure. Magnitude is a hypothesis; operation
counters on a flash fixture would settle it.

## <a id="da-029"></a>DA-029 — Preparation contains whole-model nested scans that grow superlinearly within the supported limits

**Principles:** AP-07 (H9, H10, H11).

**Diagnosis.** Preparation nests linear searches inside whole-model loops. The supported
scope is 100,000 rows, instances and scalars (`crates/pse-math/src/binding.rs:1161-1170`)
and 100,000 specialization items (`crates/pse-modeling/src/specialize.rs:73-82`). These
scans run on every structural edit, and through [DA-030](#da-030) also on unrelated edits.

The scans:
- For every row, a fixed-point loop over all member bindings by name, re-walking ASTs. It
  is not guarded by whether the model has derivatives, so it runs for steady models too; the
  dynamic-row count is checked only afterwards
  (`crates/pse-compiler/src/workspace/modeling/executable.rs:614-655`).
- `rows.iter().find` per output (`:997-1014`).
- `outputs.iter().position` per term or hint (`:1040-1046`, `:1106`).
- Every accumulator's term list scanned per contribution output (`executable/grouped.rs:1076-1077`).
- Per instance, a scan of all model symbols and all equations (`crates/pse-modeling/src/specialize/group.rs:51-74`).
- The full symbol table re-rendered per contribution × coordinate ([DA-001](semantic-model-and-authority.md#da-001)).

The naive closure is incidental: `MemberReads` (`executable.rs:1276+`) computes the same
member-read closure with indices.

**Evidence strength.** *Implemented*, static amplification evidence. No timing was taken. A
probe would run the `projection_tests.rs` `projected(cells)` harness at 500, 2,000 and
8,000 cells.

## <a id="da-030"></a>DA-030 — Compiler reuse stops at package-wide and whole-root granularity, and the runtime layers hand-keyed caches beside Salsa to compensate

**Principles:** DP-09, PS-11, DP-18, H13, H14.

**Diagnosis.**

**Granularity.** Checking is per declaration in Salsa, but the result is published as one
package-wide `Catalog { checked: Arc<CheckedPackage> }` input
(`crates/pse-compiler/src/workspace/modeling.rs:176-182`; coordinator-confirmed).
`projection` reads that input directly (`executable.rs:459`, `:1033`). So any package edit
re-runs projection, including [DA-029](#da-029)'s scans, for every live root, even when
`specialized` backdates. Specialization is one query over the whole root
(`modeling.rs:323-342`), so any structural edit re-specializes the whole flowsheet.

**Capacity.** `semantic_body` is LRU-capped at 64 (`executable/grouped.rs:99`;
`workspace.rs:103`; coordinator-confirmed). When a `MathService` retention is attached,
eviction costs a retention lookup rather than rematerialization (`grouped.rs:107-130`). The
per-output re-normalization and premise walk of
[DA-027](#da-027) still run before that lookup, whatever the LRU size.

**Parallel cross-lane caches.** Because Salsa lanes do not share memos, the runtime adds
cross-lane reuse beside Salsa:
- `MathService` has a weighted cache with 13 product kinds, a DataFusion compiled-program
  cache, five single-flight maps and a products map (`crates/pse-runtime/src/math.rs:365-402`;
  `math/retention.rs`; `math/artifacts.rs`; `math/products.rs`).
- `PhysicalCache` and the engine `ModelCache` are separate.

The tracked `semantic_body` query reads and writes this runtime retention
(`grouped.rs:99-130`; coordinator-confirmed), so the hand-framed `SemanticBodyHash`
overrides Salsa's dependency tracking for that product. Other cross-lane keys are
hand-maintained:
- `BasisKey` uses field-by-field equality.
- `SelectedRequest` and `AdmittedClosureHash` include process pointer addresses, with
  owners retained to prevent pointer reuse (`math/preparation.rs:23-63`;
  `workflow/modeling/canonical.rs:1845-1863`).
- `view_key` lists `Profile` fields by hand (`executable/solve.rs:221-271`).

**Consequence.**
- Edit → re-solve on a structural or unrelated edit repeats whole-model preparation.
- The soundness of each cross-lane product rests on keys maintained by hand that Salsa
  cannot cross-check.
- Each new reusable product needs a key variant, a product variant, size accounting and a
  flight.
- The same product can be found through more than one mechanism, each with its own
  eviction.

**Counterevidence (material).**
- Value-only changes bypass this path: `PreparedCase::rebind` shares plan, structure and
  artifacts (`executable/solve.rs:467-548`).
- Reuse is conservative.
- One identity spans the scopes, and retention hits are checked for identity and physical
  equality.
- `materialize` refuses reuse on incomplete premise coverage.
- Reuse tests are extensive.
- The scopes (lane, process, study lane, durable) are genuinely different. §14.4 declares
  persisted dependencies the reuse authority and Salsa an accelerator.
- No stale reuse was demonstrated.

**Evidence strength.** *Implemented* for the structure. Unsoundness risk is a hypothesis. A
probe would publish an edit to a definition the live root does not reach and observe the
`compiler.query.*` phases.
