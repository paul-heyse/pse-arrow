# Structure and invariants

**Interface-checked. Refreshed 2026-10-29 at source `f83bac039` plus the working tree.** Decisive
files were clean.

This page holds the primary causes in which a vocabulary, identity or transition is declared in
one place but enforced, or re-spelled, by convention elsewhere:
- canonical store vocabularies ([DA-007](#da-007));
- canonical transition authority ([DA-051](#da-051));
- identity derivation ([DA-050](#da-050));
- failure classification ([DA-039](#da-039)).

The [integrated account](system-architecture.md) groups them under its cause about central
vocabularies applied at consumers.

## <a id="da-051"></a>DA-051 — Canonical transition authority is split across four representations and invoked through hand positional strings

**Principles:** AP-04, AP-05, AP-02, DP-01, DP-02, DP-03, DP-24.

**Status:** new on 2026-10-29. Two unanchored streams, durable state (U3) and structure (U4),
reported it independently. It is distinct from [DA-007](#da-007), which owns re-spelled
vocabularies and keys. This finding owns where a transition's *rule* lives.

**Claim at risk.**
- §0.6: every decision vocabulary that crosses a boundary is a registry enumeration with one Rust
  type, and operational state is registry-generated.
- Plan58 58d D03: "repeated action interpretation deleted".

**Structural cause.** One transition, such as an attempt phase change, a claim or a study-point
settlement, is expressed in four places:

| Representation | Where it lives |
|---|---|
| Registry enumerations and raw SQL assertion strings | `pse-schema/src/catalog/substrate.rs:650-659` |
| Guarded SurrealQL function bodies, authored as string constants in the generator crate and published as the "generated" protocol | 16 `pse-codegen/src/codegen/surreal_*.rs` files → 205 `DEFINE FUNCTION` in `pse-operations/src/generated/surreal.surql` |
| Hand-written positional invocation strings with bind names in the operations crate | 155 distinct `fn::pse_*::*` calls across about 26 files. For example, `canonical_execution.rs:5564` invokes `claim` with ten positional arguments defined at `surreal_execution.rs:340`. The generated Rust (`generated/surreal.rs`) contains no `fn::pse_` call bindings |
| Rust choreography | `workflow/durable.rs`, `workflow/worker.rs`, `workflow/study.rs` |

Beyond function invocation, `pse-operations` holds about 300 hand SurrealQL statement literals
(`SELECT`/`UPDATE`/`CREATE`/`DELETE`/`UPSERT`/`LET`, inline tests included) across 28 files. They
use hand field lists decoded positionally; for example, a 16-field select at
`canonical_studies.rs:2461` feeds `decode_scope_point`.

**Point settlement: the trust boundary is decided; one binding gap remains.**
- `record_study_attempt` evaluates `may_retry` client-side (`workflow/study.rs:2164`) and passes
  `settled`, `facts` and `outcome` to the `unsafe fn observe_study_point`
  (`canonical_studies.rs:2470-2486`).
- That is the writer-trust boundary that proposed ADR-0164 designs: "the occurrence observation
  writer similarly requires actual admitted kernel facts or an effect-free shared-policy refusal".
  About twenty such `unsafe` authority entries exist, for example in `canonical_execution.rs` and
  `canonical_selection.rs`. This audit does not treat the decided trust model as a defect. A later
  review may challenge it.
- The verified gap is narrower. `fn::pse_study_v1::observe` (`surreal.surql:3228-3258`) does not
  bind the study-module hash, although progression and prepared claims do
  (`surreal.surql:3324-3330, 3394-3405`). After a module change, settlement can therefore run
  under a policy that claims would refuse.

**Consequence.**
- Adding a phase or outcome means coordinated string edits that only canonical tests against a
  live store catch.
- Signature or field-order drift between an invocation string and its function surfaces only at
  runtime. No static check tying call sites to definitions was found, though an empty search does
  not prove absence.
- 58d D03's deletion of repeated interpretation holds for progression, and D03 remains the live
  owner of that seam. The module-hash binding gap is outside D03's stated scope.

**Counterevidence.**
- DDL, codecs and field constraints really are generated from the registry. Registry limits are
  substituted into templates from one owner (`surreal_execution.rs:6-37`).
- All three placements share `pse_model::study_policy`.
- Server-side SurrealQL cannot import Rust enumerations except through generation.
- Store-side guarded functions are a maintainer premise. 58d names `surreal_control.rs` and
  `surreal_studies.rs` the "one generated canonical policy owner".

## <a id="da-007"></a>DA-007 — Canonical vocabularies and key schemes are declared but re-spelled in hand SurrealQL and Rust

**Principles:** DP-01, DP-02, DP-04, AP-05.

**Status:** reopened against Plan56's "resolved — CA02 Implemented/Tested" disposition. The
original manifestations are narrowed: typed key kinds and declared prefix constructors now exist
(`pse-schema/src/model/canonical.rs:105-137`; `pse-model/src/generated/canonical.rs:183-200`),
and the admission prefix branch is gone.

**Remaining manifestations:**
- **Ordinary run keys** are still minted and parsed by prefix with no declared constructor:
  - `strip_prefix("run:")` in `workflow/worker.rs:1829-1840` and
    `workflow/study_execution.rs:250-257`;
  - also `study.rs:844`, `durable.rs:1274` and `research_publication.rs:204`.
- **Hand SurrealQL re-spells enumeration subsets.**
  - The terminal-phase pair `['terminal','terminal_compacted']` appears about 25 times across 13
    hand-authored generator and operations files, plus `testing.rs`, rather than coming from the
    generated `terminal()` (`pse-model/src/generated/enums.rs`). The registry's own occurrences
    are not counted.
  - The outcome set `['succeeded','failed','partial','cancelled']` is spelled in
    `surreal_execution.rs:729` and `surreal_studies.rs`.
  - About 75 literal `IN [...]` lists appear in total, for example `surreal_files.rs:32, 64, 71`.
- **A hand inverse projection.** The registry declares `CanonicalTerminalOutcome` with forward
  projections (`pse-schema/src/catalog/operations.rs:71-110`). The runtime hand-writes the inverse
  `AttemptState → TerminalClass` (`workflow/durable.rs:2195-2202`).
- **Literal membership.** Artifact-profile membership by literal names (formerly DA-032) persists.

**Consequence.** Adding a member to a declared enumeration silently leaves hand subsets stale.
**Counterevidence:** the field `TYPE` assertions are derived, and refusal codes have a registry
(`pse-schema/src/catalog/canonical_refusals.rs`).

## <a id="da-050"></a>DA-050 — Identity derivations are catalogued per frame but minted inline at consumers through uncataloged sub-contexts

**Principles:** AP-04, AP-05, DP-01, DP-04, DP-09, DP-24.

**Status:** new on 2026-10-29 (U6). It also holds the name-encoded `f_<hex>` function identity
noted under [DA-003](semantic-model-and-authority.md#da-003): an identity carried in a name and
parsed back (`pse-modeling/src/specialize/reductions.rs:30-36`; `check.rs:948, 983`).

**Claim at risk.**
- The frame catalog: "every context is declared here and nowhere else", and "a new meaning or a
  new derivation version is a new variant" (`pse-ids/src/derive.rs:36-43`).
- §5: golden vectors freeze the frame spellings and digests.

**Structural cause.** A few umbrella frames act as generic derivation buckets, and consumers
distinguish meanings inside them with free-form `.str("…")` tags. Those tags are not catalogued,
have no owner and often carry no version. Immediate in-band tags, counted conservatively:

| Frame | Tags |
|---|---|
| `DerivedBindingV1` | 33 |
| `EngineeringGoalV1` | 26 |
| `CanonicalPayloadV1` | 26 |

`CanonicalPayloadV1` is documented as "exact source payload bytes" (`derive.rs:476-477`). It also
derives:
- attempt, result-set and study population keys;
- scientific attempt lineage;
- physical row and path keys.

The same identity is minted inline at several consumers:
- **Attempt lineage.** `scientific.attempt.lineage.v1` is derived independently at recovery
  (`workflow/durable.rs:692-695`) and at claim (`workflow/worker.rs:1837-1840`). Nothing ties
  the two.
- **Point identity.** `target::point_identity` (`math/strategy/target.rs:555-561`) is
  re-implemented in `math/staged/driver.rs:1181-1184`.
- **Bound-goal normalization.** The `BoundGoal` normalization identity is built separately in
  `kkt_accuracy.rs:262-266`, `engineering_accuracy.rs:159-162` and `derived_accuracy.rs:152-155`,
  each with a slightly different product hash.

The catalog's ownership metadata is also wrong:
- `Canonical*`, `Math*` and `ProducerV1` sit in the native-solver block (`derive.rs:424-534`) but
  are used by operations, runtime, compiler and buildinfo.
- About 70 frame/crate pairs are used outside their declared owner, and the generated frame
  reference projects that unchecked metadata.

§5.1's "other crates hash only through its framing APIs" is overstated. SHA-256 byte digests
appear in `pse-operations` (`canonical_migration.rs:69`, `canonical_study_module.rs:335`,
`native_service.rs:246`), `pse-host` and `pse-buildinfo`, without a declared second digest family.

**Consequence.**
- Changing a derivation means finding every inline copy. Divergence would silently change
  identities or break attempt correlation without touching the catalog or its golden vectors.
- Domain separation between meanings rests on string uniqueness across crates.
- "Which identity is this?" cannot be answered from the catalog.

**Counterevidence.**
- Framing is length-prefixed, so the tags give real domain separation.
- blake3 has one owner, enforced by a governance test.
- `Frame` is a compile-checked enum.
- §5 allows "a new projection or context string".
- Many operations tags carry `.vN`.

The problem is ownership and discoverability of derivations, not hash soundness. Tag counts come
from an immediate-call regex, so true counts are higher. Whether the three `BoundGoal` hashes
differ intentionally is undetermined.

## <a id="da-039"></a>DA-039 — Failure classification collapses at the widest error carriers

**Principles:** AP-04, AP-05, DP-02, DP-21, PS-10.

**Status:** reopened against Plan56's "resolved — CA01 Implemented/Tested" disposition, and
widened, on 2026-10-29. Plan56 fixed the producers it named and added
`PoisonedOwner`, `Integrity`, `Internal`, `Resource` and `Unsupported` classes
(`workflow/mod.rs:528`). Two unanchored streams found the default still decides the class, and
the same collapse in canonical and host errors.

**Claim at risk.** §23.2:
- "request admission (`WorkflowError::Input`), internal defects and typed source failures have
  separate contracts";
- "workflow boundaries preserve resource, cancellation, incompatible and infrastructure
  distinctions";
- "formatting an error into a string does not establish its class".

**Structural cause: runtime.**
- The crate-wide helper `contract(msg) → WorkflowError::Input(String)` (`workflow/mod.rs:555-557`)
  has about 2,390 non-test call sites, against roughly 80 constructions of the typed-category
  variants.
- About 270 of those calls are `contract(e.to_string())` or `contract(format!(…))`.
- `Input` carries non-input facts, and all of them project as `DiagnosticCode::WorkflowInput`:
  - internal invariants: "worker idle lane completed" (`worker.rs:359`);
  - integrity mismatches: "physical document manifest paths differ";
  - resource refusals: "chunk admission exceeded" (`worker.rs:1403-1431`).
- `pse-math`'s error has `Contract` with no internal or integrity variant (`pse-math/src/error.rs:26`).

**Structural cause: canonical store.**
- `CanonicalError`'s `TypedDiagnostic` maps every variant except `Codec` to `RuntimeInfrastructure`
  (`pse-operations/src/canonical.rs:468-475`).
- The same owner's `is_infrastructure()` (`:336-371`) excludes `Refused`, `Configuration`,
  `Interpretation`, `PayloadLimit`, `StudyCancellation` and `OperationReused`, so the owner's two
  classifiers disagree.
- The registry `RefusalClass` on `Refused` never reaches the diagnostic.
- Canonical `Timeout`, `PayloadLimit` and `StudyCancellation` surface as infrastructure, not as
  timeout, resource limit or cancellation (`workflow/diagnostics.rs:10-57`).
- `CanonicalError::Configuration(String)` has about 337 construction sites, including serde
  failures.

**Structural cause: native service and host.**
- `NativeServiceError` (`native_service.rs:47-49`) and `HostError` (`pse-host/src/lib.rs:61-64`)
  are blanket infrastructure.
- Invalid host CLI input travels as `HostError::Ownership(String)`.

**Consequence.** The persisted and exported diagnostic envelope tells consumers "invalid input" for
store-integrity or internal faults, and "infrastructure" for cancellation, timeout or payload
limits. Classification needs prose parsing, against §23.2 and DP-21.

**Counterevidence.**
- Operational *policy* uses precise typed predicates rather than the class: `effect_is_unknown`,
  `retryable`, `needs_fresh_premise` and `proves_source_refusal` (`canonical.rs:374-460`).
  Behavior is therefore mostly sound; the defect is at the diagnostic and contract surface.
- Native solver failures are exhaustively typed.
- The governance `error_taxonomy` test checks only that `Diagnostic` exists, not adequacy.

## Resolved identities

| ID | Outcome at `f83bac039` | Decisive evidence |
|---|---|---|
| <a id="da-008"></a>DA-008 | Resolved (2026-10-25) | Typed store refusals from thrown registry codes |
| <a id="da-009"></a>DA-009 | Resolved | Per-kind `RequestFacts` (`workflow/run_outcome.rs:33-75`); tagged `ReplayConstructor` (`replay.rs:439, 487`; `8943cf557`). The remaining wildcards match outcome variants within one kind. A workspace-wide arm recount was not repeated |
| <a id="da-010"></a>DA-010 | Resolved (2026-10-25) | Covariance origin, PSD and authentication |
| <a id="da-011"></a>DA-011 | Resolved (2026-10-25) | Generated document catalogue. Its Python consumers are retiring with the Python generator (dirty tree) |
| <a id="da-040"></a>DA-040 | Resolved | Enum-keyed metric attribute values (`tooling/metric_declarations.rs:351-371, 433-438`); estimators refused by declared kind and temporality (`scripts/telemetry_investigation.py:271-300`) |

Earlier full text is at source `682fc3830`.
