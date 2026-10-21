# Structure and invariants

Findings whose structural cause is a representable invalid state, a lifecycle or outcome
distinction carried by convention, or an invariant without an enforcement point. Baseline
and limits are in the [collection README](README.md). Citations are `path:line` at source
commit `a791c533e`.

## <a id="da-007"></a>DA-007 — Canonical lifecycle state is stored as untyped text and flag bags, and the registry's lifecycle enumerations do not govern it

**Principles:** DP-02, DP-03, DP-19, AP-05, DP-01; G1, G2.

**Diagnosis.** The registry's substrate catalog declares every lifecycle column of the
canonical store as `text()` or `flag()`. `substrate.rs` contains no `enumeration(...)` call.
For example, attempts carry independent `ingestion_open`, `closed` and `terminal` flags
plus an `outcome` text (`crates/pse-schema/src/catalog/substrate.rs:536-544`), and
artifacts carry `active` and `retiring` flags (`:759-760`, `:845-846`). The same registry
does declare `AttemptState` (nine members) and `StudyPointState`
(`crates/pse-schema/src/catalog/operations.rs:8-28`), but no substrate table uses them.

Each value set is therefore restated independently:
- SurrealQL literals in the `pse-codegen` templates (`crates/pse-codegen/src/codegen/surreal_execution.rs:566`; `surreal_studies.rs:370-371`).
- Hand-written Rust enums with `as_str` (`crates/pse-operations/src/canonical_execution.rs:2100-2121`).
- Rust string sets (`crates/pse-operations/src/canonical_files.rs:634`).

Combinations no owner defines remain representable, for example `terminal` while
`ingestion_open`, or `active` together with `retiring`. Server-function checks enforce
some of them on write, such as `IF $outcome NOT IN [...]` (`surreal_execution.rs:566`).
These are real enforcement points, but they are spread across functions rather than
declared once per state machine.

The same cause covers other store-column vocabularies:
- **Owner kinds.** Owner-kind columns are `text()` in file instances, history obligations
  and artifact obligations (`substrate.rs:221`, `:648`, `:788`), with a hand Rust enum in
  `crates/pse-operations/src/canonical_history.rs:44-57`. The artifact owner-kind value set
  exists only as a literal list in generated SurrealQL (`crates/pse-operations/src/generated/surreal.surql:2478`).
- **Terminal class.** A hand-written store enum.
- **Artifact kind.** Persisted as text (`crates/pse-runtime/src/workflow/artifacts.rs:1033-1036`).

The owner-kind invariant has a visible gap. Store `release` guards `study` owners but has
no guard for `analysis` owners (`surreal.surql:2487-2499`). The public path
`ScientificArtifact.release(kind, owner)` → `CanonicalStore::release_artifact` adds no
check (`crates/pse-runtime/src/workflow/artifacts.rs:403-409`;
`crates/pse-operations/src/canonical_artifacts.rs:528-540`). The coordinator and the
challenge review both confirmed this path. That a public caller can release an obligation
held by an active analysis is therefore source-established and not *Tested*.

The projections between parallel vocabularies are non-exhaustive (coordinator-confirmed):
- `durable.rs:1875-1880` maps unlisted `AttemptState` members to `TerminalClass::Failed`.
- `study.rs:1570-1574` maps them to `StudyPointState::Completed`.

They disagree only on the five `AttemptState` members (`planned`, `queued`, `running`,
`stale`, `superseded`) that are never produced outside tests. The declared vocabulary does
not govern behaviour, and a newly produced state would default silently in each projection.

Partiality itself is carried as a flag conjunction rather than a lifecycle state: a
`Completed` point with `scientific.usable == false` (`study_execution.rs:1572-1589`;
`study.rs:1577`). That is consistent across the local and recovery paths, but it is a
convention, not a type.

**Consequence.**
- Adding or renaming a state or owner kind is a coordinated hand edit across Rust,
  SurrealQL and Python, with no compile-time or regeneration check.
- A reader can meet a combination of flags that no owner defines.
- A new state silently takes a wildcard default.

**Footprint.**
- Known: canonical runs, attempts, files, artifacts, analyses, studies and history owners;
  owner kinds; terminal class; persisted artifact kind.
- Suspected: study readiness flags (`dirty`, `notify`, `rebuild_pending`) and opaque
  `facts` / `outcome` bytes.

**Counterevidence.**
- `HistoryOwner`, `HistoryPurpose` and `SeedEligibility` are typed in Rust.
- SCHEMAFULL tables reject undeclared fields, though not undeclared values.
- The fenced claim / close / seal protocol is strong; see the README's strengths.

**Evidence strength.** *Interface-checked* and *Implemented*, from source reading. A G5
exposure is a hypothesis.

**Related.** [DA-004](semantic-model-and-authority.md#da-004),
[DA-008](#da-008), [DA-025](execution-fit.md#da-025).

## <a id="da-008"></a>DA-008 — Persistence refusals lose their typed cause, so retry and recovery decisions depend on prose and incidental shape

**Principles:** DP-21, DP-02, DP-19; G3, G5.

**Diagnosis.**
- **Refusals are prose.** The generated SurrealQL functions refuse with
  `THROW '<prose>'` (305 occurrences at baseline; coordinator count), for example
  "cancellation wins terminal seal" and "study decision premise changed".
- **Rust collapses them.** Each refusal becomes `CanonicalError::Driver`. `CanonicalError`
  classifies every variant except `Codec` as `RuntimeInfrastructure`
  (`crates/pse-operations/src/canonical.rs:176-183`; coordinator-confirmed). Definite domain
  refusals, transport timeouts and lost acknowledgements therefore share one code.
- **The operation stage is logged, not attached.** The stage name passed to
  `operation_context` (`crates/pse-runtime/src/workflow/diagnostics.rs:128-144`) does not
  travel with the error.
- **A production retry decision matches rendered messages.**
  `crates/pse-runtime/src/workflow/worker.rs:1697` strips `"An error occurred: "` and
  compares the remainder to three literal strings (coordinator-confirmed).
- **Retry classification depends on wrapper shape.** It marks a directly wrapped
  `WorkflowError::Canonical(_)` as transient, but the same cause inside
  `WorkflowError::Shared` as deterministic (`durable.rs:2062-2099`).
- **The effect field carries no information.** `detail()` always records
  `effect: EffectState::Absent`.
- **Worker loss is mislabelled.** It is encoded as
  `CanonicalError::Configuration("worker authority expired …")` and recorded under
  `TerminationCause::Error` (`durable.rs:505-538`).
  `TerminationCause::Infrastructure` is matched (`study.rs:406`;
  `crates/pse-py/src/workflow.rs:235`) but never constructed.
- **Recovery infers worker loss from incidental fields.** Study recovery reads "lost writer"
  from the combination completion=None, retry=Transient, effect=Absent (`study.rs:1694-1700`),
  and `may_retry` grants a retry on exactly those fields
  (`crates/pse-model/src/study_policy.rs:554-560`).

**Consequence.**
- A tool cannot classify a persistence failure without parsing prose.
- Worker loss, a definite refusal and an infrastructure fault are indistinguishable in the
  durable termination record.
- Retry permission rests on facts the producer does not establish.
- The declared contract says the opposite: §23.2 "upper owners retain the typed cause
  without … rendered-message interpretation"; §20.6 "truthful interruption/worker-loss
  completion".

**Counterevidence.**
- `BoundaryDiagnostic` is a genuinely structured envelope (`crates/pse-model/src/diagnostic.rs:215-242`).
- Fence-related cancellation refusals are typed (`CanonicalError::StudyCancellation`).
- Whether `effect = Absent` is ever wrong depends on whether all durable effects are
  attempt-scoped. That is not established.

**Evidence strength.** *Implemented*, from source reading. The message-matching retry and
the classification collapse are coordinator-confirmed.

## <a id="da-009"></a>DA-009 — Operation kind is encoded as parallel closed sum types whose pairing is not structural, and the set of operations has no single owner

**Principles:** AP-03, AP-05, DP-02, DP-16, DP-01; G9 through PSE-S03.

**Diagnosis.** Every route's outcome flows through `RunResult`. It holds `RunRequest` and
`Result<RunReport, …>` as two independent five-variant enums whose variants must
correspond (`crates/pse-runtime/src/workflow/run.rs:201-233`; coordinator-confirmed).
Consumers match on the request, then match on the report expecting the same variant, and
fall through to `_ => None` on a mismatch. An example is `completion.rs:228-232`. A
mismatched pair is representable and is silently tolerated: accuracy goals or native
reports disappear instead of the mismatch being rejected.

Per-kind facts are re-extracted independently by each consumer:
- `completion.rs:406-431`, `:648-656`
- `diagnostics.rs:165-240`
- `attempt_documents.rs:446-544`
- `numerics.rs:611-613`
- `fitting/results.rs:1149`

Three further closed operation-kind enums have different variant sets:
- `OperationRequest` and `PreparedStudyOperation` (`study_operations.rs:313-364`): four kinds, no Shooting.
- `pse-py`'s own `PreparedOperation` (`crates/pse-py/src/workflow.rs:2427-2431`): three kinds.

The same cause also shows in `RunResult`'s horizon-only `Option` fields (`run.rs:246-252`).

**Consequence.**
- A new analysis kind must be threaded through every consumer arm. Production code has
  about 61 `RunReport` kind sites in 17 files and about 106 `RunRequest` sites in 12 files.
- A consumer that is missed falls into a silent `_ => None` path rather than a compile error.
- No single owner answers which operations exist.

**Counterevidence.**
- Consumers delegate to kind-owned data rather than recomputing it.
- `usable()` and `assessments()` are shared accessors.
- Studies dispatch to the same operation supervisors.

**Evidence strength.** *Implemented*. The silent-mismatch consequence is inferred from the
code shape; no mismatching pair was constructed.

**Related.** [DA-012](boundaries-and-composition.md#da-012) (lifecycle copies per kind).

## <a id="da-010"></a>DA-010 — A covariance propagated across runs is admitted as certified on its face

**Principles:** PS-12, DP-02, DP-03, DP-21; G7, G3.

**Diagnosis.** Solve-level uncertainty propagation passes the request's covariance
straight into propagation as `Ok(&propagation.covariance)`
(`crates/pse-runtime/src/workflow/local_analysis.rs:281`; coordinator-confirmed).

The `ParameterCovariance` document holds only `run_id`, `parameters` and `values`
(`crates/pse-runtime/src/math/settings.rs:91-149`). It has no approximation label, validity
state, units or fit-revision lineage. Admission checks symmetry and a nonnegative diagonal.
It never resolves `run_id`. The type is a generated msgspec struct that Python can construct
(`python/pse/contracts/documents/__init__.py:7419`).

The propagated result's `local_validity` therefore reflects only the sensitivity side. A
stale covariance from another revision, a foreign one or a hand-made one is published as a
valid first-order result. That contradicts §19.8 ("valid only where every upstream validity
row holds").

In-process, `RunResult::parameter_covariance` returns `Err` when the covariance was
withheld (`workflow/uncertainty.rs:248-291`). Converting to the document erases that
distinction.

**Counterevidence.**
- Propagating an externally supplied covariance can be a legitimate request, and the
  document's comment says the result's validity is the covariance's
  (`math/settings.rs:98-100`). The defect is narrower than "every non-fit covariance is
  invalid": `run_id` is an unverified provenance claim, and the result does not label the
  covariance as external or uncertified (DP-21 lineage).
- Typed withholding reasons (`FitWithheld`).
- At-bound detection uses KKT bound activity.
- A fit propagating to its own predictions keeps in-process validity and appears sound.

**Evidence strength.** *Implemented*, from source reading. A forged-covariance probe would
make it *Tested*.

## <a id="da-011"></a>DA-011 — Document versions and native-to-document type pairing are declared by hand across two crossing conventions

**Principles:** DP-01, DP-24, DP-16.

**Diagnosis.** Documents cross the Python boundary in two conventions:
- **Bytes in both directions.** About 37 native methods return `bytes` and about 52 take
  `bytes`. The Python wrapper chooses the decode type by hand, for example
  `codec.decode_json(self._handle.completion(), Completion)` (`python/pse/_runs.py:96`, `:414`).
- **Named Python types.** `DocumentValue` / `DocumentInput` name the Python type through
  about 71 hand-written `DocumentType::NAME` strings, and Rust imports `pse.codec` at call
  time (`crates/pse-py/src/documents.rs:200-232`). Native code calling back into Python
  contracts makes the dependency cyclic.

Versions are declared twice: `RecordedDocument::VERSION` constants in `pse-py` restate the
owners' `Version<N>`.

The import-time compatibility check verifies the package version and the registry
fingerprint only (`python/pse/_build.py:110-120`), not the document schema set. Because msgspec encodes every
generated default explicitly, the generated Python defaults are what Rust receives.

**Consequence.**
- A new result accessor needs a Rust method, a hand-written Python wrapper and a
  hand-chosen type pairing.
- A version bump in the owner has a second place to update.
- Drift in document schemas is not caught at import.

**Suspected wider reach.** Several Rust documents evolve through `#[serde(default)]` with
no version. `Completion` (`crates/pse-runtime/src/workflow/completion.rs:15-23`) was
confirmed by the slice reviewer. A broad count was reported but not re-verified by the
coordinator.

**Counterevidence.** Mismatches fail closed: decoding is strict, and version-first checks
refuse unknown versions (`python/pse/codec/__init__.py:185-260`;
`crates/pse-py/src/documents.rs:46-60`).

**Evidence strength.** *Interface-checked*.
