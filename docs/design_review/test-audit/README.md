# Test code design principle audit

This is the living diagnostic account for test architecture, evidence and shared support. Start with
the [integrated architectural account](architecture-and-assurance.md). It describes the mid-cutover
system as built, six integrated conclusions, their interactions and the contrasting strengths. Topic
pages hold the primary findings. This account does not select remedies, authorize implementation or
qualify the test suite.

The [skill](../../../.codex/skills/design-principle-audit-tests/SKILL.md) owns the method. The
[binding](../design_principles/binding/pse-arrow.md#test-code-design-principle-audit) owns invocation,
finding identity and follow-up routes. The test companion interprets the full standard; it does not bound
the assessment.

## Baseline and method

- **Audited source:** `7bd341f0bd4a27ad629008abdff56ee2bcf7e9f3`, refreshed 2026-10-29.
- **Moving source:** HEAD advanced three times during the audit, and each commit landed the working-tree
  changes the streams had read:
  - `ef1ce1956` refuses retired Python construction in `scripts/surreal_server.py` and
    `scripts/surrealism_module.py`; [DTA-070](doubles-and-coupling.md#dta-070) depends on it;
  - `adb50cc86` and `f5b436bbf` committed the remaining included changes.

  **Line citations refer to `f5b436bbf`.** Edits made after the streams finished (`crates/pse-host/src/correctness.rs`
  and its tests, `benches/benches/k4/studies.rs`, plan and documentation edits) are not included.
- **Publication:** a subsequent commit, recorded separately by Git.
- **Included uncommitted changes:** at the start, the working tree held 59 modified paths from concurrent
  work, all since committed as above:
  - `crates/pse-runtime/src/workflow/*` and `crates/pse-runtime/tests/native_*`;
  - `crates/pse-py/src/*`;
  - `crates/pse-operations/src/canonical_migration.rs` and `canonical_client/held_transaction.rs`;
  - `python/pse/tests/*`;
  - Python surreal scripts and tests, later committed as `ef1ce1956`.

  Streams compared these diffs while ignoring whitespace. They are mostly rustfmt reflow. Substantive
  edits (narrowed `held_transaction` gates, `expect` messages in migration tests, a rewrite of
  `result_pruning_tests.rs` with the same technique) do not change any conclusion. Decisive dirty files
  are named where they are cited.
- **Previous baselines:**
  - whole account: `87f2eb905` (2026-10-27);
  - scoped study and database page: `64a217de2` (2026-10-28).

  135 commits intervened before `7bd341f0b`. They include the Plan58 B/C/D/F/G slices, the new
  `pse-host` crate, about 28k lines of canonical migration, progression and native-service code, the
  native reference exchanges, the deletion of the Python parity candidates, and about 35k lines removed
  from `scripts/tests`.
- **Context:** Plan58 ([Rust-owned system](../../plans/58-rust-owned-system.md)) is mid-execution and
  paused at its 2026-10-29 closeout. Its companions schedule the removal of much of the Python test
  machinery:
  - [58f](../../plans/58f-development-and-assurance.md) F02, F04 and F05;
  - [58g](../../plans/58g-boundary-retirement-and-optional-consumers.md) G03.

  Plan text is treated as intent, never as implementation evidence.
- **Selected standard:** Core 3.4; efficiency heuristics 1.0; test-assessment companion 1.2;
  process-simulator profile 1.5; pse-arrow binding, all through
  [standard.toml](../design_principles/standard.toml). Nothing in the standard changed since the previous
  baseline.
- **Lens:** holistic and integrative, with particular attention to efficiency-oriented principles through
  static review. Every core foundation remains applicable.

The refresh ran in three waves, then a challenger.

1. **Revalidation, wave 1.** Three read-only streams, each also looking for new causes in its area,
   checked DTA-001 to 058 against current source:
   - oracles and scientific assurance;
   - routes, execution and efficiency;
   - support, fixtures and the database boundary.
2. **Fresh, unanchored, wave 1.** Two streams worked without reading this account or Plan 53:
   - the emerging native test architecture;
   - whether assurance survived the Python-to-native transition, pairing each deleted test with its
     replacement.
3. **Fresh, unanchored, wave 2.** Three further streams covered the populations wave 1 sampled thinly:
   - canonical migration, progression and native-service tests in `pse-operations`;
   - `pse-host` and other changed crates;
   - the remaining Python and `scripts/tests` population, plus the `pse-runtime` inline tests changed
     since the last baseline.
4. **Challenger.** A [fresh challenger](#challenger-review) reviewed the drafted account against source.

The coordinator verified every consequential claim directly before adopting it, including:

- no `test-group` assignment in `.config/nextest.toml`;
- `miri` missing from the adapter's mode allow-list;
- the whole-workstation worker sizing;
- the "frozen" covariance allowances copied from the same run;
- `just parity` reduced to a preflight;
- the absence of a reclaimer for 39 retained `pse-canonical-native-*` stores;
- the "historical" v2 request built by the current serializer;
- `test-support` activation in test-mode builds (`cargo tree --offline`, the only command run beyond
  reading);
- the migration recovery branches;
- the KLU cancellation test calling no product code;
- the vacuous fixture entry;
- the live capacity file read by admission tests;
- the retired `setup` stubbed in orchestration suites;
- `setup-test` running through `test_run`.

All judgments are **Interface-checked**. No tests, builds, generators, services, benchmarks or profiling
were run.

## Integrated synthesis

The cutover improves oracle isolation and physical settlement. It does not yet carry execution meaning,
and it is recreating several established causes in native form.

1. **Better oracles without routes.** Reference comparisons now use isolated oracle processes with
   discriminating controls. They have no route, their acceptance allowances come from the candidate, and
   the parity recipe runs only a preflight (DTA-050, DTA-058, DTA-061, DTA-062). The reference library is
   unchanged (DTA-049, DTA-054). Migration history is checked against the current serializer (DTA-065).
2. **Execution meaning dropped, not moved.** The native route assigns no groups. Feature gates silently
   remove most replacement evidence from ordinary routes. Prerequisites are panicking environment
   variables. Three route families and duplicated policies diverge, and the routes for surviving evidence
   depend on modules scheduled for deletion (DTA-015, DTA-050, DTA-063, DTA-064).
3. **Per-test reconstruction.** Each native test builds a whole runtime, and each canonical native test
   builds a whole service and schema. Feature-set variants multiply binaries, and worker journeys each
   claim the whole workstation (DTA-018, DTA-053, DTA-046).
4. **Settlement converges; disposal does not.** Failed private stores accumulate. Each test body decides
   disposal, and setup refusals report as test failures (DTA-060, DTA-052, DTA-025).
5. **Entanglement recurs.** Test seams sit in the native fixture owner and the migration client. Fixture
   policy is restated per binary. Host tests bind to live operator state. Python orchestration suites
   certify stubbed flows (DTA-055, DTA-059, DTA-069, DTA-070).
6. **Thin at dangerous transitions.** Fault injection, refusal discrimination, provider preparation,
   concurrency and universal claims are thin (DTA-066, DTA-067, DTA-068, DTA-057, DTA-056).

The earlier account's conclusions remain the frame. What changed is where each applies: Python-route
findings stay live until F02/F05/G03 delete their mechanisms, and their native successors already show
the same causes.

## New findings

| Finding | Primary home | Diagnosis |
|---|---|---|
| [DTA-059](doubles-and-coupling.md#dta-059) | Support | Native integration binaries restate fixture policy that drifts from production construction |
| [DTA-060](evidence-routes-and-execution.md#dta-060) | Routes | Native fixture disposal, retention and deadline policy is decided in each test body |
| [DTA-061](oracles-and-acceptance.md#dta-061) | Oracles | Reference comparison allowances cross the oracle boundary from the candidate under test |
| [DTA-062](oracles-and-acceptance.md#dta-062) | Oracles | The reference-environment contract is restated in each oracle and has drifted |
| [DTA-063](test-execution-system.md#dta-063) | Execution system | Coexisting route families hold divergent execution meaning during the cutover |
| [DTA-064](test-execution-system.md#dta-064) | Execution system | Routes for surviving evidence are composed of the machinery scheduled for deletion |
| [DTA-065](oracles-and-acceptance.md#dta-065) | Oracles | Historical-compatibility evidence for canonical migration is built by the current code it checks |
| [DTA-066](oracles-and-acceptance.md#dta-066) | Oracles | Migration fault injection is added per call site and leaves the final-edge recovery branches unstimulated |
| [DTA-067](oracles-and-acceptance.md#dta-067) | Oracles | Native refusal assertions accept any error |
| [DTA-068](doubles-and-coupling.md#dta-068) | Support | Host provider preparation is established by fakes, refusals and a lock test that calls no product code |
| [DTA-069](doubles-and-coupling.md#dta-069) | Support | Real-effect host tests bind to the operator's live admission journal and capacity policy |
| [DTA-070](doubles-and-coupling.md#dta-070) | Support | Python orchestration tests certify flows whose real first step now refuses, against test-authored configuration |

Several native observations recur under existing identities rather than new ones, because their
structural cause is the same: DTA-015, DTA-018, DTA-025, DTA-046, DTA-050, DTA-055, DTA-057.

## <a id="revalidation-outcomes"></a>Revalidation of DTA-001 to DTA-058

Each topic page's refresh table holds the decisive source. The outcome terms mean:

- **Established:** current source establishes the recorded correction.
- **Live on Python:** the mechanism remains and is scheduled for deletion by a Plan58 package.
- **Inherited / recurs:** the native replacement shows the same cause.

[Plan 53](../../plans/53-test-architecture-target.md#finding-dispositions) still owns disposition of
these identities; this table reports diagnosis only.

| Finding | Refresh outcome | Current residual |
|---|---|---|
| [DTA-001](oracles-and-acceptance.md#dta-001) | Withdrawn; premise inherited | → DTA-061 |
| [DTA-002](oracles-and-acceptance.md#dta-002)–[007](oracles-and-acceptance.md#dta-007), [021](oracles-and-acceptance.md#dta-021) | Established, unchanged | DTA-003 caveat: A04 report identity open in 53a |
| [DTA-008](doubles-and-coupling.md#dta-008) | Withdrawn (2026-10-23) | — |
| [DTA-009](doubles-and-coupling.md#dta-009), [010](doubles-and-coupling.md#dta-010) | Partial, unchanged | Native construction drift → DTA-059 |
| [DTA-011](doubles-and-coupling.md#dta-011) | Resolved for the native fixture; legacy residual live | Legacy population has no supported route |
| [DTA-012](evidence-routes-and-execution.md#dta-012) | Established on Python; premise retired on native | — |
| [DTA-013](evidence-routes-and-execution.md#dta-013), [014](evidence-routes-and-execution.md#dta-014), [019](evidence-routes-and-execution.md#dta-019) | Established | — |
| [DTA-015](evidence-routes-and-execution.md#dta-015) | Inherited by native, widened | No group assigned on the native route |
| [DTA-016](evidence-routes-and-execution.md#dta-016) | Live on Python (F02) | Native route has no classification |
| [DTA-017](evidence-routes-and-execution.md#dta-017) | Partial, live | Capsule capture without leaf proof |
| [DTA-018](execution-efficiency.md#dta-018) | Recurs in native replacement | Per-test runtime and full service/schema; claim fusion |
| [DTA-020](oracles-and-acceptance.md#dta-020) | Established; route caveat | Evidence gated behind refused `--store` routes |
| [DTA-022](test-execution-system.md#dta-022) | Established on Python; divergence → DTA-063 | — |
| [DTA-023](test-execution-system.md#dta-023) | Partial | Dead recipe filter |
| [DTA-024](test-execution-system.md#dta-024) | Partially reopened | Native route does not verify nextest identity |
| [DTA-025](evidence-routes-and-execution.md#dta-025) | Inherited by native | Primary home for setup refusals reported as test failures |
| [DTA-026](test-execution-system.md#dta-026) | Established (scoped) | DA-037 owns |
| [DTA-027](doubles-and-coupling.md#dta-027) | Partial, extended | PID owner prefix never matches; `native_dynamic_workflows` unlisted |
| [DTA-028](doubles-and-coupling.md#dta-028) | Partial, wider footprint | Primary home for untyped refusals: host `Ownership(String)`, runtime `Input(String)`; ~22 native wording assertions |
| DTA-029, 031, 032, 035–037, 040, 045 ([page](execution-efficiency.md#residuals-of-earlier-findings)) | Established, unchanged | — |
| [DTA-030](execution-efficiency.md#dta-030) | Overstated, unchanged; footprint grows | PID exchange decodes the whole capsule |
| [DTA-033](execution-efficiency.md#dta-033) | Superseded, residual | `py-unit` still republishes |
| [DTA-034](execution-efficiency.md#dta-034) | Holds on Python | Native counterpart bounded |
| [DTA-038](execution-efficiency.md#dta-038), [039](execution-efficiency.md#dta-039), [042](execution-efficiency.md#dta-042), [047](execution-efficiency.md#dta-047), [048](execution-efficiency.md#dta-048) | Partial, live on Python | Premise of 038 retired on native |
| [DTA-041](execution-efficiency.md#dta-041) | Withdrawn (2026-10-23) | — |
| [DTA-043](execution-efficiency.md#dta-043) | Holds on Python; premise retired on native | — |
| [DTA-044](execution-efficiency.md#dta-044) | Established (narrowed) | Guard checks a nonexistent group |
| [DTA-046](execution-efficiency.md#dta-046) | Escalated on native | Whole-envelope self-sizing, ungrouped |
| [DTA-049](oracles-and-acceptance.md#dta-049) | Live on Python route | Native conformance runs one synthetic package |
| [DTA-050](evidence-routes-and-execution.md#dta-050) | Live, widened | Feature-gated inventory; unrouted replacements; refused `--store` populations |
| [DTA-051](doubles-and-coupling.md#dta-051) | Partial; new footprint | Orphaned parity harness; ~21 unrouted check scripts |
| [DTA-052](execution-efficiency.md#dta-052) | Live on Python; new native footprint | Host generations; failed stores → DTA-060 |
| [DTA-053](execution-efficiency.md#dta-053) | Live, extended | 22 binaries × ~10 feature sets |
| [DTA-054](oracles-and-acceptance.md#dta-054) | Live, unchanged | — |
| [DTA-055](doubles-and-coupling.md#dta-055) | Live; recurs in native owners | Native service, migration client, progression module |
| [DTA-056](oracles-and-acceptance.md#dta-056) | Live | Pruning rewrite keeps the technique |
| [DTA-057](evidence-routes-and-execution.md#dta-057) | Partial, extended | Lease sleep with a hidden lease parameter; no concurrent commit race |
| [DTA-058](oracles-and-acceptance.md#dta-058) | Live, extended | Parity documentation points at a preflight |

The [scoped study and database page](study-execution-and-database-boundary.md) (2026-10-28) is
revalidated in the [support refresh](doubles-and-coupling.md#refresh-at-7bd341f0b--2026-10-29). Its Plan57
receipt-table premise for DTA-058 is retired because Plan58 supersedes Plan57.

## How the standard informed this refresh

| Criterion | Where it was applied | Findings |
|---|---|---|
| AP-01 responsibility and change locality | Fixture policy changes across binaries; retirement of a Python module across surviving routes; orphaned support | 059, 064, 051 |
| AP-02 consumed contracts | Doubles against systemd, the canonical server and Python collaborators (`create=True`); config schema with no producer | 068, 070, 055, 063 |
| AP-03 composition | `#[path]` support composition; route families composing closure, groups, fixtures and verdict | 059, 063, 015 |
| AP-04 semantic authority | Oracle independence of allowances and environment; historical-wire authority; duplicated policies | 061, 062, 065, 063 |
| AP-05 explicit structure | Feature-gated inventory; environment prerequisites; config comments contradicted by routes; untyped refusals | 050, 060, 067, 015 |
| AP-06 local reasoning and testability | What a focused check instantiates; live operator state; time windows; claim fusion | 018, 069, 057, 053 |
| AP-07 execution fit (with G9) | Per-test service and schema reconstruction; feature-set variants; whole-envelope self-sizing; route-less preparation (a full extension build for a preflight) | 018, 053, 046, 051, 052 |
| Efficiency heuristics | H2 (units vs kinds), H12/H14 (preparation lifetime, reuse), H19 (carrying established validity), H21 (capacity across layers), H25 (proportionate machinery), H26/H27 (auxiliary and lifecycle cost) | as above |
| DP-01 one authority | Python and Rust closure, environment and receipt twins; eight config copies | 063, 070 |
| DP-10, DP-20 bounded work and concurrency | Ungrouped whole-envelope journeys; concurrent service units | 046, 018 |
| DP-16, DP-17, DP-18 library first, module boundaries, declared effects | Production fixture installers and test dispatch; nextest delegation is library-first (a strength) | 055 |
| DP-19, DP-22, DP-23 recovery, labelled outcomes, verification matching risk | Migration fault coverage; setup vs product failure; vacuous passes | 066, 060, 068 |
| DP-21, DP-24 reproducibility, discoverable contracts | Hand-exported checkpoint commands; parity documentation | 050, 058 |
| PS-13 shared checks and references | Exchanges, oracle environment, reference library | 061, 062, 049, 054 |
| PS-10 truthful outcomes | Settlement branches; setup refusals; refusal discrimination | 066, 060, 067 |
| PS-03/07/08/11 | Real-model evidence still sits in the unrouted library and native-only journeys | 049, 050 |
| PS-04 | Unchanged since the last baseline | — |

DP-02 to DP-09 and DP-11 to DP-15 were not separately at issue in test scope beyond these. Native
thread and seed ownership (DP-11) remains a strength.

## Challenger review

A fresh challenger received the drafted pages and source, not the coordinator's reasoning, and was asked
where the draft was wrong, overstated or incomplete. It raised 22 corrections. The coordinator verified
the consequential ones and integrated all of them:

- **Wrong:**
  - DTA-050 claimed the `native_service` target hides its `test-support` cases. Workspace feature
    unification activates `test-support` in every test-mode build (`cargo tree -e normal,dev`). That
    claim was removed; the same evidence settled DTA-055's open reach question.
  - "18 of 20" feature-gated targets is 16 of 22 binaries (16 of 20 declared).
  - DTA-065 cited a digest move predating schema versioning as an unbumped change. The sentence was
    removed; the absence of a literal pin for the current digest stands.
  - DTA-067 counted the typed helper's calls as 10; there are 4. It also omitted the I/O-free vocabulary
    test, so the exposure is narrowed to a call-order regression in `run_cli`.
  - DTA-057 misdescribed the host kill timing.
- **Overstated:**
  - "Competing workers" is a deterministic interleaving, not a timing window; only a concurrent race is
    missing.
  - Only one journey's deadline exceeds the default termination, and only on the default profile.
  - DTA-070's "schema with no producing owner" applies only to Python-only fields; a typed native
    producer of the core exists.
  - DTA-018's journey length.
  - DTA-053's rebuild cost depends on separate invocations.
  - DTA-061's consequence: the configured fraction itself is checked.
  - DTA-068: a real failed-publication test exists.
  - DTA-046 is reframed under the chosen uncapped policy.
- **Under-supported:** the Python-runner routes are runnable only after full-profile preparation.
- **Identity:**
  - Untyped refusals take their primary home in DTA-028; DTA-067 keeps the test-side discipline.
  - Setup reporting returns to DTA-025, and failed-store accumulation to DTA-052. DTA-060 narrows to
    per-body disposal, retention and deadline policy.
  - DTA-064 states its lineage from DTA-026/DA-037.
- **Upgraded:** DTA-027's PID prefix is established as never matching.
- **Omission integrated:** the `pse-operations` publication controls (`canonical-tests` plus
  `native-service`) have no recipe.
- **Editorial:** baseline currency (HEAD advanced to `f5b436bbf`), counts and wording.

The challenger tried and failed to break DTA-015's absent group assignment and the absence of any
supplier for the reference interpreters. It also confirmed DTA-061, DTA-062, DTA-065's
current-serializer construction, DTA-066's unstimulated branches, DTA-068's lock test and vacuous
fixtures, DTA-069, DTA-070's stubbed `setup`, DTA-063's unverified nextest, and the DTA-018/046 counts.

## Scope, strengths and limits

**Inspected scope:**

- `crates/pse-host` (tests and inline tests, read in depth);
- `crates/pse-operations` (migration and native-service tests in depth; conflict and transport tests
  sampled);
- `crates/pse-runtime/tests` (all targets inventoried; about 15k lines sampled);
- the changed `pse-runtime/src/workflow` inline tests;
- `tests/reference/oracles`;
- `python/pse/parity`;
- the remaining `python/pse/tests` and `scripts/tests` population (classified in full, read selectively);
- `.config/nextest.toml`, Cargo manifests and features, and justfile recipes;
- sampled tests in `pse-math`, `pse-backend-native`, `pse-ids`, `pse-diagnostics` and `pse-model`.

The strengths to preserve are in the
[integrated account](architecture-and-assurance.md#contrasting-strengths).

**Not inspected or only skimmed:**

- `xtask/src` tests;
- generated runtime files in `pse-model`;
- `pse-diagnostics::supervised_child` beyond test names;
- most Python scientific journeys;
- `host_admission.py` against `pse-host/src/admission.rs`;
- `thermodynamic_campaign.py`.

**Material unresolved questions:**

- How does real nightly Cargo report host units in `--unit-graph` output? The adapter's detection tests
  use synthetic graphs.
- Do the 39 retained private stores come from failures that have since been repaired?
- Is a pre-existing Python-supervised store still the intended host for `canonical-tests` until the D/B
  cutover?
- Which route is meant to carry parity evidence?
- How strongly does the engine serialize concurrent claims? This decides whether the missing concurrent
  commit race (DTA-057) matters.
- Has the reference library's 2026-10-11 failure been repaired?
- What is the A04 report identity?

No elapsed time, speedup or capacity was measured. Directory counts and line counts are observations,
not cost measurements. This account establishes no current scientific or product qualification. It does
not certify complete alignment with every principle and does not claim to have found every shortfall.

## Reading and ownership

- [Integrated architecture and assurance](architecture-and-assurance.md): the mid-cutover system, six
  conclusions, interactions, scenarios and strengths.
- [Oracles and acceptance](oracles-and-acceptance.md): DTA-061, 062, 065, 066, 067, 049, 054, 056 and 058;
  records 001–007, 020 and 021.
- [Routes, activation and outcomes](evidence-routes-and-execution.md): DTA-060, 050 and 057; records
  012–019 and 025.
- [Support ownership and coupling](doubles-and-coupling.md): DTA-059, 068, 069, 070, 051 and 055;
  records 008–011, 027 and 028.
- [Execution system](test-execution-system.md): DTA-063 and 064; records 022–026.
- [Efficiency principles](execution-efficiency.md): DTA-052 and 053; records 018 and 029–048.
- [Scoped study and database boundary](study-execution-and-database-boundary.md): the 2026-10-28 scoped
  account, revalidated above.

DTA-001 to DTA-070 are allocated. DTA-008 and DTA-041 are withdrawn and never reused. The next available
ID is DTA-071.

**Disposition ownership.** [Plan 53](../../plans/53-test-architecture-target.md#finding-dispositions)
(in progress) owns disposition of DTA-001 to DTA-058. Plan58 reconciles "affected Plan53/54 interfaces"
(T01) but adopts no DTA disposition, and F02/F04/F05/G03 schedule work that reaches many of these
findings. DTA-059 to DTA-070 have no disposition owner. This account selects none.

Shared causes already in the broad account keep their identity and home. In particular,
[DA-037](../audit/development-tooling.md#da-037) owns the cyclic tooling and supervisor coupling
referenced by DTA-026 and DTA-055. The broad account was consulted for identity and overlap, not adopted
as evidence.

The account owns diagnosis at its stated source. Reviews own remedy alternatives; implementation plans
own scheduled disposition and closure. Git preserves prior publications and their evidence conditions.
No parallel status ledger, recurring gate or automatic remediation follows from this refresh.
