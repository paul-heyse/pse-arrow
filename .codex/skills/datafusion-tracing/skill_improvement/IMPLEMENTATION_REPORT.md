# DataFusion tracing skill implementation

Implemented 2026-09-18; final validation receipts use UTC timestamps on 2026-09-19.
Scope is the portable skill. Subject packages remain pinned at 55.0.0. The work changes no
consumer application, installed skill configuration, or host service architecture.

## Delivered reference

The [entry point](../SKILL.md) follows the companion DataFusion skill's short routing style.
Task, crate, representation, lifecycle and symptom routes lead to 14 reviewed briefs. Each
brief separates source interpretation, exact contracts, alternatives, consumer observations
and remaining uncertainty. Direct files remain usable alongside `find`, `show` and `compare`.

The contract layer retains 3,504 records across 636 pages, including 92 module records, full
upstream docs and type trees, source spans, implementation contexts and artifact identities.
Hosted and private rustdoc remain distinct. Public builder access routes do not become invented
imports of internal types. The original 502-item compact inventory, 49 macro arms, 11-crate
scope, syntax rules, corpus and inherited probes remain available.

Bounded retrieval carries a result fingerprint and lossless UTF-8 byte continuation. Maintained
annotations do not overwrite upstream text. There are 212 explicit contract diagnostics for
unresolved documentation links or display limitations; these are not treated as absent APIs.

Corrected guidance includes default preview formatting, cross-batch/partition accumulation,
preview syntax hints, seven source links, the store route's unrelated planner-probe citation,
options required by the INFO execution macro, and historical dependency combinations. Snapshot
aggregates remain `recorded`; one assertion no longer promotes a whole span row. Compatibility
is described through manifests, resolved features and actual profiles, without a minor-version
arithmetic rule.

## Plan delivery

| Stage | Delivered outcome |
|---|---|
| 1. Preserve contracts | Full rustdoc records, artifact-scoped identity, raw preservation checks, corrected conflicting guidance |
| 2. Route discovery | All 11 crates routed; tasks, representations, lifecycle and symptoms link to reviewed depth |
| 3. Establish semantic depth | 14 authored briefs with claims, alternatives, exact operations, composition and limits |
| 4. Strengthen evidence | 12 public-consumer tests, structured captures, named assertions, locked dependency/feature profile |
| 5. Bound retrieval | Portable standard-library reader, alias/member resolution, compare, UTF-8 continuation and fingerprints |
| 6. Evaluate | Frozen baseline/candidate and 32 tasks per condition; independent judgments retained separately |
| 7. Package and maintain | Reader/research archives, manifests, notices, invalidation controls, isolated reader transfer and research reproduction |

These are completion statements about this implementation scope. They do not imply exhaustive
characterization of the indexed crates or satisfaction of every possible downstream integration.

## Verification

| Check | Result and scope |
|---|---|
| Contract preservation | Passed: 3,504 records compared to 13 raw capture profiles; 1,494 resolved documentation targets checked |
| Bounded reader | Passed: aliases, five task routes, comparison and reconstruction of a multibyte payload across 73 fragments |
| Deterministic regeneration | Passed: prior generated hashes compared with a fresh offline build, separately from stored digest integrity; provenance generation date is run metadata |
| Structural/navigation checks | Passed: 11 syntax rules, 23 navigation checks, 18 legacy routes, maintained links and registered inventory counts |
| Public consumer runtime | Passed: all 12 tests; result/control assertions plus capture hashes and compiler/dependency profile |
| Rust fixture lint | Passed: pinned stable Clippy for the contracts integration test with warnings denied |
| Python lint | Passed for changed generators and all new reader/maintenance scripts |
| Reader/research transfer | Passed: copied bundle lookup from `/`, empty environment, isolated Python and no source-checkout/network access |
| Invalidation | Passed: changed authoring and a newly added candidate operation both trigger the expected review hints |
| Research reproduction | Passed: copied fixture/lock/runner reproduce all 12 consumer tests from an external capsule, reusing an external dependency/target cache |

The runtime tests distinguish default/custom/disabled/error previews; partition accumulation;
live stream, EOF adapter release and retained plan ownership; early drop and repeated execution; independent
execution groups; declared fields and metrics; phase selection and rule parentage; a controlled
later plan rewrite; default/explicit targets and filters; spawned parent/dispatch propagation;
wrapped store result/error/stream behavior; local capture versus SDK sampling; and owned-provider
flush/shutdown. Fields recorded after span construction are inspected at completion.

The resolved consumer profile uses DataFusion **55.1.0**, Arrow **59.3.0**, object_store **0.13.2**,
tracing **0.1.44**, tracing-subscriber **0.3.23**, bridge **0.32.0**, and OpenTelemetry API/SDK
**0.31.0**. It is distinct from the subject version and the rustdoc producer profile. SDK
`testing` enables the in-memory exporter; retained records are inspected before its default
shutdown reset. Primary source verification and actual runtime observations remain separate.

The original 12 probes retain their original dates and verdicts: 11 `confirmed`, one `recorded`.
This implementation validates their retained artifacts but does not report them as freshly run.

## Comparative evaluation

The paired evaluation uses the plan's E01–E16 contrasts: 32 reference-only design/debugging tasks
for each frozen reference, the same 100–160 word answer budget, local tools, no external docs,
and no task oracle available to the answering agent. An independent judge can inspect the
oracle, source, fixtures and answers. No evaluator-produced sketch is claimed compiled.

The independent judge found 30 passing and two partial cases in each condition. E08A and E09B
in both answers missed the outer stream adapter's EOF-triggered destruction. Discovery,
contract fidelity and diagnosis showed no candidate regression or decision-score improvement.
New assertions strengthened cited behavioral support in 26 cases (six ties), with per-case assessments in
the judgment artifact. Measured read-command counts were 13 versus 14; elapsed intervals had
different start boundaries and do not establish a speedup. Bytes/tokens were not measured.

Post-evaluation refinements corrected the macro-family count and filter note, and extended the
lifetime test to settle the shared missed boundary. A source-only inference that retaining the
public adapter retains its inner recorder failed. Exact DataFusion 55.1.0 source and the passing
replacement control show close events at EOF while public handles remain alive. The failed
hypothesis, source and final capture are retained. The final brief states this profile-specific
behavior. These refinements have source/runtime verification; the full 32-case pilot was not
rerun against them.

Final independent results are recorded in
[evaluation/paired-evaluation.json](evidence/implementation/evaluation/paired-evaluation.json).
The pilot has one pass per condition. It supports case-level observations, not a general
performance or accuracy claim; repeated trials would be needed to assess variance.

## Retained artifacts

- [Validation results](evidence/implementation/verification.json) and
  [runtime receipt](evidence/implementation/probe-results.json).
- [Resolved profile](evidence/implementation/probe-profile.json),
  [consumer fixture](../build/fixtures/probe-crate/tests/contracts.rs), and
  [structured captures](evidence/implementation/captures/).
- [Exact-release source verification](evidence/implementation/upstream-verification.json) and
  [license manifest](../content/licenses/manifest.json).
- [Baseline answers](evidence/implementation/evaluation/baseline-answers.json),
  [candidate answers](evidence/implementation/evaluation/candidate-answers.json),
  candidate snapshot/manifest and task file under `evidence/implementation/evaluation/`; the
  baseline archive/manifest are directly under `evidence/implementation/`.
- Reader and research archives under `evidence/implementation/bundles/`, with transfer receipts
  and observed file/network traces alongside them. Archive hashes identify qualified copies.
- [Maintenance instructions](../maintenance.md) for regeneration, probes, invalidation and packaging.

## Deliberate limits

No OTLP receiver or backend was qualified. The local SDK observations establish export into an
in-memory exporter, not transport acknowledgement, retention, process-exit delivery, or timeout
behavior. Store probes invoke the wrapper directly; registration through a DataFusion runtime
remains source-backed integration guidance. SDK sampling is tested; every possible per-layer
filter interaction is not. Query-stream failures, remote store providers/multipart, arbitrary
concurrent workloads, missing native metrics and schema-dependent memory/overhead bounds remain
useful additional characterization work. The final adapter observation was cross-checked through the DataFusion skill
(`df.consume`, the exact `poll_next` contract and matching 55.1.0 archive bytes).
The callable-builder/internal-type distinction is
covered without attempting to import a private implementation type.

Offline reading needs Python 3.11+ only. Research regeneration also needs the recorded external
tools; compilation needs its pinned compiler and dependencies. Transfer proves independence
from the original checkout for the observed commands, not compilation without a toolchain/cache.
Archives contain no compiled targets or hidden acquisition caches and install nothing globally.
