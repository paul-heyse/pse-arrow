# Test code design principle audit

## Scoped refresh: study execution and database boundary

The [2026-10-28 scoped account](study-execution-and-database-boundary.md) examines
`64a217de2` plus current operations inline-child corrections/tests and the concurrent
Plan57j checkpoint. It reconciles selected fixture/route/custody/worker evidence and
the proposed Python/database boundary, without replacing the whole-account baseline
below. Existing DTA-050/055/058 retain primary ownership; no new ID is allocated.
The refresh ran no tests, builds or services and makes no Plan57 completion claim.

This is the living diagnostic account for test architecture, evidence and shared support.
Start with the [integrated architectural account](architecture-and-assurance.md): it describes the
system as built, six integrated conclusions, their interactions and contrasting strengths. Topic pages
hold the primary findings. This account does not select remedies, authorize implementation or
qualify the test suite.

The [skill](../../../.codex/skills/design-principle-audit-tests/SKILL.md) owns the method;
the [binding](../design_principles/binding/pse-arrow.md#test-code-design-principle-audit) owns
invocation, finding identity and follow-up routes. The test companion interprets the full standard;
it does not bound the assessment.

## Baseline and method

- **Audited source:** `87f2eb9050ae24976956628c16e0e7376190be7a`, clean when fixed.
- **Date:** 2026-10-27. Publication is a subsequent commit, recorded separately by Git.
- **Included uncommitted changes:** none.
- **Previous baseline:** `05e6a2b8a` (2026-10-23), published through `cbf45ccc9`. About 480
  commits intervened, including Plan 53 B–F, 52l, 54, 55 and 56; Plan 53 scheduled every earlier
  finding and records most as resolved.
- **Selected standard:** Core 3.4; efficiency heuristics 1.0; test-assessment companion 1.2;
  process-simulator profile 1.5; pse-arrow binding through
  [standard.toml](../design_principles/standard.toml). Only the binding changed since the previous
  baseline, outside the test-audit route.
- **Lens:** holistic and integrative, with particular attention to efficiency-oriented principles
  through static review. Every core foundation remains applicable.

The refresh ran in two waves, then a challenger:

1. **Revalidation.** Five read-only streams checked Plan 53's claimed closures against current
   source: scientific evidence; support and input closure; invocation and build; lifetimes and
   scheduling; tooling-test population. Each also looked for new causes.
2. **Fresh, unanchored investigation.** Five further streams worked without reading this account or
   Plan 53, on system questions the earlier account had not asked: contract-to-evidence portfolio
   and routes; technique and library fit; determinism, time and isolation; test-code structure and
   production seams; physical build structure of test code.
3. **Challenger.** A [fresh challenger](#challenger-review) reviewed the drafted account for
   omissions, misreadings and overstatement.

The coordinator verified each consequential claim directly before publication, including:

- seed-conformance's absence from assessment checks and exclusions;
- unconditional extension publication and the local generation count;
- nextest's process-per-test model against `OnceLock` reuse;
- the absence of any route enabling `pse-authoring/arbitrary`;
- the qualification-receipt `expect` in ordinary store tests;
- the absence of references to route-less drivers;
- the deferred cleanup loop and the unwired `reclaim_reports`.

All judgments are **Interface-checked**. No tests, generators, benchmarks or profiling were run.

## Integrated synthesis

The earlier account's conclusion, that no new structural cause beyond its 48 had reached the
threshold, does not hold at this source. Plan 53 corrected many joins, but about a third of its
claimed closures are partial, overstated or regressed. The fresh pass found ten new structural
causes, DTA-049 to DTA-058. In short:

1. **The scientific basis has weaker routes than the machinery around it.** The reference library has
   no qualification route, no full pass is recorded (its last full run failed), and its generators
   cannot detect stale inputs (DTA-049, DTA-054).
2. **Execution meaning is held beside tests, keyed by location and history.** Requirement tables,
   features, filters and commissioning plans determine what runs. Moves, renames and retirements
   silently change execution, and nothing checks that declared tests execute anywhere (DTA-016,
   DTA-050, DTA-051, DTA-027).
3. **Focused feedback is sized by the largest units, then re-verified.** This covers whole-crate and
   variant builds, the workspace-wide validity bracket, per-run extension generations and per-process
   capsule decode (DTA-053, DTA-039, DTA-033, DTA-030).
4. **Lifecycles are built for protection, not convergence.** Unreferenced generations, capsules,
   candidates and passed resources have no reachable retirement owner (DTA-052, DTA-038, DTA-048).
5. **Production and test structure entangle.** Choreography seams, a shipped fixture drain,
   `Debug`-substring observers and divergent helpers couple the two (DTA-055, DTA-009, DTA-010,
   DTA-028).
6. **Techniques are strong where used and thin at universal claims.** Identity and pruning,
   cross-process cancellation, and layer and governance descriptions are the gaps (DTA-056,
   DTA-057, DTA-058).

## New findings

| Finding | Primary home | Diagnosis |
|---|---|---|
| [DTA-049](oracles-and-acceptance.md#dta-049) | Oracles | The authored scientific reference library has no route in ordinary or comprehensive qualification |
| [DTA-050](evidence-routes-and-execution.md#dta-050) | Routes | No owner establishes that each declared test executes on some route with its prerequisites |
| [DTA-051](doubles-and-coupling.md#dta-051) | Support | Test evidence and support are placed by creation history rather than enduring responsibility |
| [DTA-052](execution-efficiency.md#dta-052) | Efficiency | Prepared artifacts and test evidence are published without content reuse or a convergent lifecycle |
| [DTA-053](execution-efficiency.md#dta-053) | Efficiency | The focused loop is scoped by crate- and variant-wide build units rather than the responsibility under test |
| [DTA-054](oracles-and-acceptance.md#dta-054) | Oracles | Reference provenance binds frozen outputs but not generator inputs or expectation-level derivation |
| [DTA-055](doubles-and-coupling.md#dta-055) | Support | Production types carry test choreography and fixture seams across several vocabularies |
| [DTA-056](oracles-and-acceptance.md#dta-056) | Oracles | Universal identity and pruning claims are challenged by curated examples where invariance relations and existing oracles would discriminate |
| [DTA-057](evidence-routes-and-execution.md#dta-057) | Routes | Cross-process native cancellation journeys infer solver entry and interleavings from elapsed time |
| [DTA-058](oracles-and-acceptance.md#dta-058) | Oracles | Layer and governance descriptions claim more than their checks discriminate |

## Revalidation of DTA-001 to DTA-048

"Established" means current source establishes the correction Plan 53 records, within its stated
scope. Each page holds the decisive source. [Plan 53](../../plans/53-test-architecture-target.md#finding-dispositions)
owns disposition; this table reports diagnosis only.

| Finding | Refresh outcome | Current residual |
|---|---|---|
| [DTA-001](oracles-and-acceptance.md#dta-001) | Withdrawn, unchanged | "Disproved" overstates an unproven boundary; nothing changed |
| [DTA-002](oracles-and-acceptance.md#dta-002) | Established | Premise literals → DTA-054 |
| [DTA-003](oracles-and-acceptance.md#dta-003) | Established, caveat | Input staleness → DTA-054; A04 report discovery |
| [DTA-004](oracles-and-acceptance.md#dta-004) | Established | — |
| [DTA-005](oracles-and-acceptance.md#dta-005) | Established | — |
| [DTA-006](oracles-and-acceptance.md#dta-006) | Established | — |
| [DTA-007](oracles-and-acceptance.md#dta-007) | Established | — |
| [DTA-008](doubles-and-coupling.md#dta-008) | Withdrawn (2026-10-23) | — |
| [DTA-009](doubles-and-coupling.md#dta-009) | Partial | `Debug`-substring matching; missing positive controls allow vacuous passes |
| [DTA-010](doubles-and-coupling.md#dta-010) | Partial | Two divergent residual-body helpers remain |
| [DTA-011](doubles-and-coupling.md#dta-011) | Established | Wider seam pattern → DTA-055 |
| [DTA-012](evidence-routes-and-execution.md#dta-012) | Established | — |
| [DTA-013](evidence-routes-and-execution.md#dta-013) | Established | General cause recurs → DTA-050 |
| [DTA-014](evidence-routes-and-execution.md#dta-014) | Established | — |
| [DTA-015](evidence-routes-and-execution.md#dta-015) | Partial | Bare nextest and assessment `test` routes run without groups; bare pytest `worksteal` ignores `xdist_group` |
| [DTA-016](evidence-routes-and-execution.md#dta-016) | Overstated | Prefix-keyed, fail-open native demand; dual definitions; hand-listed exclusivity; Python file-name inference |
| [DTA-017](evidence-routes-and-execution.md#dta-017) | Partial | Capsule capture builds xtask without leaf proof |
| [DTA-018](execution-efficiency.md#dta-018) | Established | — |
| [DTA-019](evidence-routes-and-execution.md#dta-019) | Established | — |
| [DTA-020](oracles-and-acceptance.md#dta-020) | Established | — |
| [DTA-021](oracles-and-acceptance.md#dta-021) | Established | Library-wide variants → DTA-049 |
| [DTA-022](test-execution-system.md#dta-022) | Established | — |
| [DTA-023](test-execution-system.md#dta-023) | Partial | File-name capsule relevance; dead recipe filter |
| [DTA-024](test-execution-system.md#dta-024) | Established | — |
| [DTA-025](evidence-routes-and-execution.md#dta-025) | Established | Non-fixture preconditions → DTA-050 |
| [DTA-026](test-execution-system.md#dta-026) | Established (scoped) | Lazy-import cycles remain; DA-037 owns |
| [DTA-027](doubles-and-coupling.md#dta-027) | Partial | Demand lists, live readers, unchecked text adaptation |
| [DTA-028](doubles-and-coupling.md#dta-028) | Partial (scope) | Resolved only for migrated distinctions |
| [DTA-029](execution-efficiency.md#dta-029) | Established | — |
| [DTA-030](execution-efficiency.md#dta-030) | Overstated | Process-local reuse is cancelled by nextest; full capsule decode per process |
| [DTA-031](execution-efficiency.md#dta-031) | Established | — |
| [DTA-032](execution-efficiency.md#dta-032) | Established (narrow) | Variant multiplicity → DTA-053 |
| [DTA-033](execution-efficiency.md#dta-033) | Superseded (ADR-0231), residual | Identical-image republication and path-keyed stubs → DTA-052 |
| [DTA-034](execution-efficiency.md#dta-034) | Established | — |
| [DTA-035](execution-efficiency.md#dta-035) | Established | Live readers → DTA-027 |
| [DTA-036](execution-efficiency.md#dta-036) | Established (mechanism) | Declared consumer edges → DTA-027 |
| [DTA-037](execution-efficiency.md#dta-037) | Established | — |
| [DTA-038](execution-efficiency.md#dta-038) | Partial | Repeated digests under service-wide admission; O(M²) member rewrites |
| [DTA-039](execution-efficiency.md#dta-039) | Partial | Third identity pass; workspace-wide refusal scope |
| [DTA-040](execution-efficiency.md#dta-040) | Holds (corrected judgment) | — |
| [DTA-041](execution-efficiency.md#dta-041) | Withdrawn (2026-10-23) | — |
| [DTA-042](execution-efficiency.md#dta-042) | Partial | Deferred cleanup stops early → DTA-052 |
| [DTA-043](execution-efficiency.md#dta-043) | Established | — |
| [DTA-044](execution-efficiency.md#dta-044) | Established (narrowed) | A guard checks a group that no longer exists |
| [DTA-045](execution-efficiency.md#dta-045) | Established | Side effect → DTA-016 |
| [DTA-046](execution-efficiency.md#dta-046) | Established (scoped) | Joint demand remains open |
| [DTA-047](execution-efficiency.md#dta-047) | Partial | Candidate writes without a reader → DTA-052 |
| [DTA-048](execution-efficiency.md#dta-048) | Partial | Checkpoints re-serialize accumulated state |

**Plan-state discovery.** Plan 53's handoff names six `seed*.arrow` reports for its pending
simultaneous-controller consumer and asks for their identity to be verified. Those reports were written
45 seconds after handoff commit `25b75d338` and record the fixture as `inconclusive` on its preparation
deadline. If their identity is confirmed, A04/I10/G01 hold an inconclusive result rather than a
pending one. Details are on the [DTA-003 record](oracles-and-acceptance.md#dta-003); Plan 53 owns
reconciliation. Similarly, Plan 56f F05 is recorded complete although its stated reclamation
obligation has no implementation ([DTA-052](execution-efficiency.md#dta-052)).

## How the standard informed this refresh

**AP-01: responsibility and change locality.** For a native-module rename, a test move, a plan
retirement and a scientific-package edit, the refresh asked which owners must change. The answer is
several tables, plans and lists that the tests do not show (conclusion 2, DTA-016/050/051/027). It
also examined crate-sized compile units, which AP-01 does not require to follow responsibilities but
which govern the feedback loop (DTA-053).

**AP-02: consumed contracts.** It examined private query names, `Debug` substrings and wording
(DTA-009/028); divergent helpers against production construction (DTA-010); patch targets in tooling
tests; and the shipped fixture API (DTA-055). Typed assertions dominate and are a strength.

**AP-03: composition.** It examined how managed, bare and assessment routes compose requirement,
scheduling and activation meaning (DTA-015/016/050), and how tooling tests compose through imports
of plan drivers and other test modules (DTA-051).

**AP-04: semantic model and authority.** It examined who owns reference truth and staleness
(DTA-049/054), attribution granularity, one authority for scientific-input demand (DTA-027), and the
gap between what a layer description claims and what it establishes (DTA-058). Independent oracles
remain explicitly allowed and valued.

**AP-05: explicit structure and constraints.** It examined fail-open classification, preconditions
without roles, compiled-out bodies and dead filters (DTA-016/050), and the converse correspondence that
the C02 catalogue lacks.

**AP-06: local reasoning and independent testability.** It examined what a focused check must
instantiate (DTA-053/039/030), choreography seams in production types (DTA-055), interleavings forced
by hooks versus elapsed time (DTA-057), and contract families gated behind infrastructure
(DTA-050).

**AP-07: execution fit.** The governing efficiency criterion, applied with DP-09/10/19/20 and
H2/H5/H12–H14/H16/H19/H20/H22–H27. Source establishes the following repeated or unnecessary work:

- whole-crate and variant compilation;
- triple workspace identity and double binary hashing;
- per-run image publication and stub regeneration;
- per-process capsule decode;
- repeated registration and checkpoint digests;
- writes without readers.

It establishes no dominant bottleneck or measured magnitude.

**Operational refinements.** The refresh applied these where they actually bear on the evidence:

- DP-01 (one authority): requirement and demand tables; the reference library.
- DP-11 (determinism): native thread and seed ownership is a strength.
- DP-13/14/16 (library first, minimal bespoke machinery): mutation, coverage and fuzz tooling are
  library-first; an Arrow data generator and proptest are unused for universal claims; the
  hand-coded HTTP/2 probe has no route.
- DP-17 (module boundaries): plan-keyed placement.
- DP-18 (effects declared): production seams.
- DP-19/20 (workspaces, outcomes, bounded concurrency): lifecycles and exclusivity.
- DP-21/22/23 (reproducibility, labelled claims, verification matching risk): reference provenance,
  layer claims, and route-and-risk fit.
- DP-24 (discoverable contracts): route discoverability.

DP-02 to DP-08 and DP-12 were not separately at issue in test scope beyond these.

**Profile.** These obligations drove the scientific findings:

- PS-13 (shared checks and references): DTA-049 and DTA-054.
- PS-10 (truthful outcomes): pruning, cancellation, setup-versus-product failure.
- PS-03 (conservation), PS-07 (derivatives and scaling), PS-08 (initialization and recycle) and
  PS-11 (dynamics): their real-model evidence lives in the unrouted reference library and native-only
  journeys.
- PS-04 (structural analysis): exhaustively tested at the algorithm level, with its end-to-end refusal
  gated behind a solver.

The refresh did not renew scientific qualification of any model family.

## Challenger review

A fresh challenger received the drafted account and source, not the coordinator's reasoning, and was
asked to find where it was wrong, overstated or incomplete. It raised fourteen corrections. The
coordinator verified the consequential ones and integrated all fourteen:

- **Wrong:** the draft said a full reference-library pass predated recent edits. No full pass is
  recorded, and the last full run (Plan 28e, 2026-10-11) failed (DTA-049). The surrealism
  qualification tests' module path was also wrong. The correction exposed a dead `RUST_OWNERS`
  prefix, which is now added to DTA-016 as evidence.
- **Overstated:**
  - DTA-033 was labelled a regression, although ADR-0231 deliberately rejects freshness inference.
    It is now relabelled as superseded, with its residual cost.
  - DTA-052 read the five local generations as republication of identical content. They are distinct
    images.
  - DTA-053 claimed "no owner assesses" the tradeoffs, although Plan 54c and build-performance.md
    decided them. It is reframed as a gap in their decision evidence.
  - DTA-049 said the omission was undeclared, although §24.2 declines whole-library claims. The gap
    is now stated as missing only from the assessment report.
  - The DTA-058 error-taxonomy point and the plan-state discovery are now hedged.
- **Under-supported:** the Arrow `data_gen` route in DTA-056 needed the workspace feature-unification
  caveat. The reclamation obligation is now cited from Plan 56f F05.
- **Omissions integrated:**
  - bare pytest's `worksteal` mode ignores `xdist_group` (DTA-015);
  - the tooling tests have no focused route (DTA-053);
  - the nextest exclusivity description and a misleading reference-set test name (DTA-058);
  - seed packages are loaded by the seed-transport family, which exercises them without their oracle
    fixtures (DTA-049 footprint).
- **Merged:** the dead recipe filter's evidence now lives only in DTA-050.

The challenger tried and failed to break DTA-016's de-grouped native modules, DTA-030, DTA-039,
DTA-050's compiled-out and receipt-dependent tests, DTA-055, DTA-057, DTA-009 and the smaller
residuals. Every link and anchor resolved.

## Scope, strengths and limits

**Inspected scope:**

- crate tests across `pse-runtime`, `pse-backend-native`, `pse-math`, `pse-compiler`,
  `pse-operations`, `pse-columnar`, `pse-ids`, `pse-structural`, `pse-authoring` and `pse-admission`;
- `tests/*` and `tests/support`/`fixtures`;
- `python/pse/tests` and `parity`;
- `scripts/tests` (whole population mapped);
- the reference generators and `packages/reference` manifests;
- the execution half (`test_invocation`, `test_requirements`, `native_tests`, `python_tests`,
  `python_extension`, `artifact_publication`, `scientific_inputs`, `resource_store`,
  `test_resources`, `test_run`, `validation`, `assessment_store`);
- `.config/nextest.toml`, `.cargo/config.toml`, the Cargo profiles and the justfile test recipes.

Sampled rather than read are:

- most of the roughly 5,000 Rust tests;
- Python scientific journeys;
- Plan 56 production changes;
- `evidence_store`, `test_archives` and `resource_conversion`;
- most of `surreal_server`.

The strengths to preserve are listed in the
[integrated account](architecture-and-assurance.md#contrasting-strengths).

**Material unresolved questions:**

- whether the reference library's recorded 2026-10-11 failures (nine failed, eleven inconclusive)
  have since been repaired;
- whether bare-route native concurrency is actually unsafe;
- the measured cost of the `pse-runtime` edit loop and the O2 tradeoff;
- whether static nextest overrides could carry requirement meaning to every route on the pinned
  version;
- joint simultaneous canonical demand;
- the A04 report identity.

No elapsed time, speedup or capacity was measured. Local sizes of ignored build directories are
observations, not measurements of cost.

This account establishes no current scientific or product qualification. It does not certify
complete alignment with every principle and does not claim to have found every shortfall.

## Reading and ownership

- [Integrated architecture and assurance](architecture-and-assurance.md): system as built, six
  conclusions, interactions, scenarios, strengths.
- [Oracles and acceptance](oracles-and-acceptance.md): DTA-049, 054, 056, 058; records 001–007, 020, 021.
- [Routes, activation and outcomes](evidence-routes-and-execution.md): DTA-050, 057; records 012–019, 025.
- [Support ownership and coupling](doubles-and-coupling.md): DTA-051, 055; records 008–011, 027, 028.
- [Execution system](test-execution-system.md): records 022–026.
- [Efficiency principles](execution-efficiency.md): DTA-052, 053; records 018, 029–048.

DTA-001 to DTA-058 are allocated; DTA-008 and DTA-041 are withdrawn and never reused. The next
available ID is DTA-059.

Shared causes already in the broad account keep their identity and home. In particular,
[DA-037](../audit/development-tooling.md#da-037) owns the cyclic tooling and supervisor coupling
referenced by DTA-026 and DTA-055. The broad account was consulted for identity and overlap, not
adopted as this audit's evidence.

The account owns diagnosis at its stated source. Reviews own remedy alternatives; implementation
plans own scheduled disposition and closure. Git preserves prior publications and their evidence
conditions. No parallel status ledger, recurring gate or automatic remediation follows from this
refresh.
