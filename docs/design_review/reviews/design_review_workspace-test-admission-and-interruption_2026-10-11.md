# Workspace test admission and interruption design review

## Assessment and scope — slots 1 and 12

**Decision: Revise the test-admission and local-test boundaries. Preserve the production interpretation refusal.** The completed workspace report contains 2,363 cases: 2,256 passed and 107 failed. Every failure has the same prerequisite cause: the current reader requires `pse.substrate.v3`, while the selected service configuration declares `pse.substrate.v2`. All five tests interrupted in the earlier run now pass. The evidence supports a shared environment/admission correction and specific test-isolation improvements; it does not support weakening scientific assertions, accepting the old interpretation, or redesigning producer identity because of those SIGINT statuses.

The principal review is accountable to the coordinating Codex agent. A fresh `design-reviewer` independently inspected the failure boundary, fixture dependencies and five interrupted tests, then reassessed the completed JUnit report. The coordinator inspected the decisive source and integrates that advice here. No remediation is implemented by this review.

| Field | Scope |
|---|---|
| Standard | Core 3.4, Heuristics for Efficient Architecture 1.0, ProcessSimulator 1.5, `pse-arrow` binding; `design-review` and `design-review-process-simulator` skills |
| Tier / purpose | Design / target; bounded subsystem review of test admission, fixture composition, interpretation evolution and interruption |
| Functional target | Reliable development and qualification of the simulator's modeling, preparation, fitting, study and publication behavior, with failures attributed to their actual owner and local policies testable without unrelated infrastructure |
| Baseline | `e403c66c81f73fbc45f61542df8b7504a87c38eb`; the maintainer's uncommitted `.config/nextest.toml` changes add default JUnit output and extend the default termination threshold from 120 to 360 seconds |
| Primary execution evidence | Maintainer's `cargo nextest run --workspace --no-fail-fast`, recorded at `2026-10-10T23:47:34.478-04:00`, UUID `be5befc4-5042-418f-a924-6e0baf76d132`, 105.262 seconds reported test-run duration |
| Artifact | `target/nextest/default/junit.xml`, SHA-256 `192d72b1235a54c9ed786cc309f7fbf1295a993eef2ba280911409aebe215940`; original report left unchanged |
| Suppliers and consumers | Registry interpretation declaration and generated projections; Rust canonical options/store; Python environment and service admission; runtime workflow fixtures; Nextest configuration; producer-identity and physical-codegen controls; existing test-resource/terminal owners |
| Exclusions | Whole-product numerical qualification, native feature campaign, Python and external-reference parity, managed-primary acceptance, full persistence/recovery audit and throughput benchmarking |
| Follow-up ownership | Not yet scheduled. A subsequently authorized remediation plan should own these finding references and their dispositions. This review does not add scope to Plan 28 or reopen completed Plans 33–35. Existing campaign status remains at the [current-work owners](../../plans/README.md). |

The relevant workload is a mixed workspace run containing many independent tests, a subset of canonical consumers and several nested compiler controls. Multiple repositories share the host, so concurrent load can affect elapsed time. This review makes no performance regression claim from a single timing sample. Its execution-fit concern is avoidable setup and failure amplification, not a numerical capacity SLA.

## What the completed run establishes — slots 5 and 10

The XML was parsed across every `testcase`, rather than extrapolated from the pasted terminal tail. There are 107 `failure` elements, zero `error` elements and zero `skipped` elements. All 107 failures are in the `pse-runtime` library suite. Every one contains both the v3/v2 interpretation error and `crates/pse-runtime/src/workflow/tests.rs:118`. Other reported cases have no failure/error element. JUnit's zero skipped elements does not establish that every available test was selected: the workspace configuration excludes managed-primary cases, ignored tests are separate, and Nextest does not run doctests.

| Failed area | Count |
|---|---:|
| Mathematical solve preparation and strategy | 3 |
| Binding admission | 8 |
| Completion projection | 1 |
| Data-document admission and source loading | 11 |
| Fitting covariance and oracle | 18 |
| Modeling cases and certificates | 7 |
| Modeling conformance | 14 |
| Modeling diagnostics, declarations and engines | 5 |
| Modeling imports | 6 |
| Modeling preparation and reuse | 23 |
| Modeling source export and generic documents | 3 |
| Modeling result publication | 1 |
| Strategies | 3 |
| Related-case study | 1 |
| Workflow factory composition and durability | 3 |
| **Total** | **107** |

This groups one observed cause without erasing the tests' different responsibilities. Their intended assertions were blocked at fixture construction. A failed cache test in this cohort is not evidence of a cache invalidation defect; a failed conformance test is not evidence of numerical disagreement with an oracle. Those behaviors require compatible-state execution before they can be assessed.

The maintainer's command supplies no explicit Arrow `force-validate` activation. The XML does not record the resolved feature graph, so it cannot settle whether some ambient/workspace path activated it. Treat the report as diagnostic execution evidence, not proof of the repository's complete Arrow correctness-check contract. The focused diagnostic described below explicitly activates scoped validation.

### The five former SIGINT cases

The earlier interruption was the coordinator's authenticated SIGINT to the bare Nextest run during the maintainer-authorized shutdown for a restart. It was not a test assertion or a demonstrated self-inflicted timeout. The completed report now independently resolves their behavior in that workspace run:

| Test, under `xtask::bin/xtask` | Completed result / seconds | Inspected operation |
|---|---|---|
| `producer_identity::tests::producer_review_catalog_only_grants_current_selected_owner` | Passed / 8.879 | Temporary Cargo projects; selected unit graph, metadata and compiler-host queries; review applicability by owner, target, features, files and environment |
| `producer_identity::tests::producer_pest_preferred_grammar_revokes_review_and_requires_actual_consumption` | Passed / 9.828 | Seven captures distinguish a newly preferred grammar, a stale Cargo artifact and actual recompilation/consumption |
| `producer_identity::tests::producer_dynamic_macro_review_binds_actual_caller_inputs_and_environment` | Passed / 9.891 | Four captures across proc-macro/helper inputs, changed caller closure and renewed review |
| `codegen::physical::tests::altered_physical_facts_are_admitted_from_values_or_refused` | Passed / 10.862 | Five ordinary production package admissions: unchanged facts, invalid unit scale, invalid reference pressure, changed unit composition and changed dimension |
| `producer_identity::tests::producer_actual_cargo_consumed_edit_rekeys_uncompiled_test_edit_does_not` | Passed / 11.893 | Seventeen captures across production/test-only edits, executor refresh, binary selection and cdylib selection |

The producer controls use independent temporary project roots, the pinned toolchain and offline Cargo. `cargo_output` and actual-build capture in [producer identity](../../../xtask/src/producer_identity.rs) synchronously capture child output. Their quiet execution is consistent with waiting for legitimate compiler work. The inspected path does not recursively build the outer workspace target and does not establish a target-lock deadlock. An externally imposed shared target directory could change that premise, but it is not established for the earlier run.

The [physical-facts control](../../../xtask/src/codegen/physical/tests.rs) follows [physical fixture admission](../../../xtask/src/codegen/physical.rs): a local Tokio runtime, shared engine/session, authored package parsing, checked relations, registry materialization, production invariants and `PhysicalInventory` admission. It does not call the canonical fixture helper or launch Cargo in this path. Its declared 32-GiB memory budget is a ceiling, not evidence of that much resident allocation.

No hang, solver failure, child leak or insufficient timeout is established by these five cases. The 8.879–11.893-second completions do not justify raising a 120-second timeout. The maintainer's timeout change is preserved; this review neither reverts it nor claims it fixed these tests.

Actual compiler consumption is the independent challenge these producer tests supply. Replacing the builds with fabricated compiler messages or changing the preferred-grammar expectations to accept a stale artifact would defeat that challenge. Similarly, keep the physical test's ordinary admission path rather than replacing it with a test-only validator. Smaller local controls can supplement these integration controls where they exercise independently meaningful behavior.

### Focused compatible-state diagnostic

The coordinator created an explicitly selected, separate v3 service at `/home/paul/.local/state/pse-arrow/surreal-review-nextest-20261011`, port 18253, using the existing setup/start owners. The previous default and campaign states were not migrated, deleted or reconfigured. The diagnostic selects the exact 107 failed names from the supplied XML, uses `pse-runtime --lib --locked`, scoped `pse-runtime/force-validate`, zero retries and a separate `review-admission-20261011` Nextest profile/report. It does not enable native-solvers or claim the complete campaign feature graph.

**Diagnostic result: pending execution at drafting; replace with the completed result before publishing.**

## Responsibilities and consequential distinctions — slots 2 and 3

The production boundary already distinguishes concepts that the test-entry experience should preserve:

| Responsibility / owner | Consumed meaning and owned effect | Required local context |
|---|---|---|
| [Substrate declaration](../../../crates/pse-schema/src/catalog/substrate.rs) | Defines the current interpretation; generation supplies Rust and Python-consumed projections | Registry declaration and generator; no second handwritten version table |
| [Canonical options](../../../crates/pse-operations/src/canonical.rs) | Reads bounded private owned configuration; requires the current interpretation before credentials/connection; database opening also checks actual persisted interpretation | Selected private state and reader contract; semantic mismatch is a typed refusal |
| [Service supervisor](../../../scripts/surreal_server.py) | Owns lifecycle, resource placement and authenticated protocol readiness; generic configuration permits consistent historical interpretation for administration | Exact selected service, manager/process identities and lifecycle operation |
| [Environment admission](../../../scripts/pse_env.py) | Composes explicit/local/default selection; `require_store` checks serving/admission and may perform admitted readiness/start operations | Caller environment, selected state and current allocation |
| [Canonical fixture support](../../../crates/pse-operations/src/testing.rs) | Selects a UUID database on a compatible service; owns connections and reports drain; terminal owner separately decides retention/reclaim eligibility | Compatible service profile; isolated fixture database, not a preinitialized main application database |
| [Runtime composition](../../../crates/pse-runtime/src/workflow/mod.rs) | Validates factory service ownership, then constructs runtime with mandatory canonical scientific deployment and independently selected result durability | Factory ownership check itself consumes only `SharedRuntime` and `EngineFactory`; complete runtime construction additionally consumes canonical deployment |
| [Nextest scheduling](../../../.config/nextest.toml) | Selects tests and coordinates store/scientific classes, deadlines and output | Real test inventory and the effect being protected |
| [Test-resource owner](../../../scripts/test_resources.py) and [runner](../../../scripts/test_run.py) | Separate test outcome, completion, physical drain, references and disposal eligibility | Selected invocation and terminal evidence; destructor completion is not a passing outcome |

`profile_version: 2` is the supervisor configuration format. It is not `pse.substrate.v2`. A valid current store can have profile version 2 and substrate interpretation v3. Likewise, the default directory's `surreal-functional-v2` spelling does not itself make a store incompatible: its actual declaration does. Renaming a directory is not a semantic migration.

The current composed environment selects `/home/paul/.local/state/pse-arrow/surreal-functional-v2` when no caller/local override exists. Read-only inspection found both interpretation fields set to v2 and admission quiesced. This is consistent with the reported refusal. The JUnit file contains neither the selected path nor the original shell environment, so the exact original selection remains unproven. The stronger established fact is that every failing reader observed v2 in its selected configuration.

**Physical semantics within this review:** the incompatible-store refusal occurs before authored quantities enter evaluation. The physical-codegen control does test malformed physical declarations, but this review does not independently audit all their scientific meaning.

| Element | Dimension / unit | Basis and convention | Validity / authority |
|---|---|---|---|
| Substrate interpretation and service profile | Not physical quantities | Distinct semantic compatibility and configuration-format domains | Registry interpretation versus supervisor profile owner |
| Unit scale and composition in the physical control | Authored physical declaration | Existing package definitions; no new convention introduced here | Nonpositive scale and incompatible composition must be refused by ordinary admission |
| Reference pressure in the physical control | Authored pressure declaration | Existing declared reference convention | Invalid negative reference value is refused; no new pressure conversion is proposed |
| Fitting level and profile-point controls | Dimensionless statistical/control domains | Existing `FitUncertainty` / `ProfileControls` definitions | Decoding constraints are independently testable; source/profile identity remains an integration obligation |

**Well-posedness and numerical stages:** variable roles, degree-of-freedom analysis, guards, derivative order, scaling, class-specific solver selection, tolerances and post-solve qualification are not independently audited here. None was reached by the 107 original failure traces. The proposed corrections must preserve their normal owners and force the existing assertions to execute; neither a compatible service nor a passing fixture constructor establishes scientific adequacy.

## Change and failure scenarios — slot 4

| ID / kind | Stimulus | Required response and inspected implication |
|---|---|---|
| S01 / contract evolution | Run current canonical consumers against an older selected interpretation | Refuse before relying on it, with selected-state and expected/observed interpretation context. Preserve per-use Rust checks after any earlier preflight. |
| S02 / local responsibility | Exercise factory ownership/width or uncertainty decoding with no store available | Exercise those operations from their actual inputs. Keep canonical runtime/package integration tests separately. |
| S03 / composition and growth | Select many canonical tests, pure tests, or a mixed workspace | Pure-only selection has no store prerequisite; canonical selections admit their shared prerequisite once before fan-out. Per-test databases remain isolated. |
| S04 / mechanism substitution | Use bare Nextest rather than a repository-composed runner | Reuse one compatibility operation through a small runner adapter; no separate migration policy, test framework or forced service start. |
| S05 / intentional invalid input | Select canonical configuration/refusal controls with deliberately incompatible test inputs | Ambient preflight must not veto pure negative controls whose subject is the invalid configuration itself. A compatible ambient service is needed only when the control actually consumes it. |
| S06 / interruption | Stop nested compiler tests or an in-flight canonical test | Runner interruption is distinct from an assertion failure. Child process drain and fixture outcome/retention remain independently owned. The five completed rerun results settle the observed earlier uncertainty. |
| S07 / scheduling evolution | Rename or narrow a store fixture's assertions | Reconcile the effect-based scheduling policy with actual inventory and current effects; an obsolete exact-name rule must not silently express a protection it no longer supplies. |

New physical models and solver substitution are not the variation axis of these corrections. Their scientific owners should require no edits merely to improve test admission. The relevant mechanism substitution is the test-entry route, and the relevant domain extension is explicit consumer compatibility requirements for a selected service; neither requires a new production store abstraction.

## Findings — slot 7

### <a id="f01"></a>F01 — Consumer compatibility is discovered after shared test setup has fanned out

**Evidence:** the completed run demonstrates 107 identical fixture-construction refusals. `CanonicalOptions::from_state_for_database` enforces the current reader interpretation. `canonical_fixture_options` calls it before choosing the UUID database. `workflow::tests::canonical_deployment` unwraps the error. In contrast, `pse_env.require_store` checks service admission/readiness without requiring the consuming reader's interpretation, and the generic supervisor `config_for` requires consistency between its two interpretation fields rather than equality to the current reader. These are different responsibilities; generic lifecycle readiness is not proof of consumer compatibility.

**Consequence / principles:** one incompatible ambient prerequisite becomes a large body of tests that never exercise their subjects. Callers must know that an owned, configured or protocol-ready service may still be unusable by the selected consumer. S01/S03/S04 expose AP-05 and DP-21/DP-24 gaps in test admission. H18 and H24 favor settling the shared uncertainty before fan-out while retaining checks at later trust transitions. This is not a G3 failure of the Rust reader: its refusal is sound.

**Proposed correction:** extend the existing admission boundary with a read-only, consumer-specific compatibility operation. It consumes the explicitly selected state, the reader's generated interpretation requirement and the intended operation class; it produces usable compatibility or a bounded diagnostic naming selection, expected and observed interpretation and the explicit recovery choices. Repository execution and a small native Nextest setup adapter can consume that operation. They must not each encode another version policy.

Keep service ownership/profile validation, consumer compatibility, protocol readiness and actual database interpretation distinct. The preflight should precede expensive test fan-out and any optional readiness effects. It must not read or print secret values, migrate state, choose a different store, create application schema or start a service merely because inspection was requested. Preserve the Rust read/open checks because configuration or persisted state can change after preflight.

**Revealing legitimate cases:** an isolated fixture needs a compatible service profile and creates its own schema; it does not need the deployment's main database initialized. Administrative inspection/rebuild must remain able to inspect an older owned interpretation. Pure tests deliberately reading incompatible temporary configuration must still execute their refusal assertions. These cases rule out globally tightening `surreal_server.config_for`, requiring canonical initialization for every fixture, or attaching a blanket store gate to every workspace case.

**Verification:** a canonical selection against v2 gets one prerequisite refusal before creating fixture databases; a pure-only selection with no selected store runs; intentional invalid-configuration controls execute; v3 fixture creation works without a main application database; changing state after preflight still receives the Rust refusal. A mixed Nextest setup failure aborts that run, rather than guaranteeing that its unrelated tests continue—make that behavior explicit.

**Diagnosis maturity:** Tested from the supplied XML plus Interface-checked source. The remedy is Proposed; compatible-state diagnostics establish only the existing behavior they exercise.

### <a id="f02"></a>F02 — Local ownership and decoding assertions inherit canonical setup unnecessarily

**Evidence:** `Runtime::check_deployment` consumes `SharedRuntime` and `EngineFactory` and checks pool, spill manager, native cache manager, object-store registry, cache-service and CPU ownership/width. The two `workflow_composition_*` tests first construct full canonical runtimes; the foreign-owner control constructs two. Thus their intended checks cannot run when the unrelated selected store is incompatible.

`workflow::fitting::covariance::tests::fit_uncertainty_admission` similarly prepares a real package before reaching plain `serde_json` assertions about invalid levels, zero profile points and default profile controls. It also contains legitimate preparation/source/profile identity assertions, which should retain their integration context.

**Consequence / principles:** S02 violates AP-06; AP-01 and H5/H28 support separating the actual local responsibilities. Backend setup failure obscures local behavior and unnecessarily creates connections/databases when exercising ownership or decoding alone. No timing threshold is needed to establish that dependency is unnecessary.

**Proposed correction:** exercise the existing private ownership operation directly with shared-runtime/factory fixtures, covering each consumed owner and configured width. Retain focused end-to-end `Runtime::from_shared` construction coverage, including preservation of custom extensions and partition demand. Split independently meaningful uncertainty decoding assertions into a local control, leaving package preparation and source/profile identity checks in their integration control. No public test-only interface, optional production canonical field, fake scientific store or new crate is needed.

Do not label every failed test pure. Binding tests assert actual physical contracts, revision attribution and member identity after real preparation. Completion coverage intentionally obtains real source lineage before testing publication projection. Many reuse/import/export controls also genuinely consume canonical state. Their dependency is justified by their current assertions; simplifying them requires preserving those obligations, not mechanically replacing the helper.

**Verification:** local ownership/decoding selections run with `PSE_SURREAL_STATE` absent or incompatible, retain the same refusal/identity assertions, and register no canonical fixture resources. Separate runtime/package integration controls still exercise the production composition and source-authority path. Ephemeral result durability must continue to use canonical scientific source admission.

**Diagnosis maturity:** Interface-checked, with its manifestation present in the supplied failed run. The split and its behavior remain Proposed.

### <a id="f03"></a>F03 — An exact-name scheduling rule no longer selects its fixture control

**Evidence:** the exclusive store override names `testing::canonical_server_unit::fixture_connections_preserve_external_and_deployment_databases`; the implemented control is `fixture_connections_preserve_deployment_database`. It therefore falls through to the broader one-slot store rule rather than consuming the documented sixteen-slot exclusion. The current control checks that a non-owning deployment connection cannot remove its deployment database; it no longer expresses the former name's complete scenario.

**Consequence / principles:** S07 exposes AP-05 and DP-20/DP-24 policy drift. The written protection is disconnected from the executing subject. This is independent of the v2/v3 mismatch and did not cause the five SIGINT statuses. The canonical-tests feature control is also not established as selected in the supplied default-feature run.

**Proposed correction:** reconcile the intended effect protection with the current assertion and actual test inventory. Remove the obsolete exclusive entry if isolated fixture consumers cannot change the deployment database during this control; otherwise select the actual test and state the remaining competing effect that requires exclusivity. Do not blindly restore sixteen-slot serialization because an old comment said so. Prefer existing Nextest selectors and effect-based groups; no second inventory manifest or scheduling engine.

**Verification:** inspect the selected inventory under `canonical-tests`, confirm each retained narrow override reaches its intended case, and run the connection control with the actual competing-effect cases relevant to the selected protection. The outcome must preserve deployment isolation while avoiding exclusion unrelated to its effects.

**Diagnosis maturity:** Interface-checked. The exact required exclusion remains a bounded design decision; no race or measured performance loss is claimed.

## Interruption and ownership follow-ups — slots 5 and 10

These observations are separate from the causes established by the completed report. They must not be presented as additional failed tests or prerequisites for fixing F01/F02.

**Standalone terminal ownership.** `scripts/test_run.py` places terminal composition/finalization after synchronous `subprocess.call`; a `KeyboardInterrupt` can bypass that path, and an ordinary nonzero result is initially classified generically as failed. The campaign executor in `scripts/validation.py` already has an explicit interrupt/child-group termination path and interrupted status. Reuse the existing terminal owner when correcting the standalone route, with child termination/drain, interrupted run, individual assertion failures, unattempted cases and missing report represented as separate facts. Do not add a new result ledger. The bare Nextest command in the supplied run did not execute this wrapper, so this is not its causal defect. Existing conservative resource eligibility is a preservation constraint; no false reclamation is established here.

**Nested producer children.** The synchronous Cargo helpers have no local child deadline/guard. That alone does not establish a leak or require bespoke supervision: the outer Nextest runner handles test process groups. If direct xtask cancellation or descendant escape is to be qualified, exercise the actual caller/child boundary and verify authenticated descendants drain, then close only the demonstrated ownership gap. The completed five cases need no timeout increase, global serialization or compiler substitution.

**Temporary review catalogs.** Three producer tests place review catalogs beside their `TempDir` and remove them only at successful completion. Ordinary panic can leave the sibling file. A separately owned temporary catalog directory can preserve the important separation from reviewed source and provide ordinary unwind cleanup. Fatal signals can bypass destructors, so this is not a solution to SIGINT lifecycle ownership. This small hygiene improvement is Proposed, not an explanation for the completed run's failures.

## Libraries, alternatives and remedy integration — slots 8 and 9

The installed runner was checked as `cargo-nextest 0.9.146`, commit `8af696ddcce8fff2962d6a5168b6d138b8616a35`. Context7 and primary Nextest documentation were consulted for the runner capabilities rather than assuming wrapper behavior.

Nextest's [setup scripts](https://nexte.st/docs/configuration/setup-scripts/) can associate one setup operation with matching selected tests. They are experimental in the inspected release; a failed setup aborts the run. A pure-only selection need not trigger canonical setup. This is a candidate thin admission adapter, not a service lifetime owner or an instruction to provision services during every bare run. Its selection must exclude standalone negative configuration controls and track actual canonical effects.

Nextest's [signal handling](https://nexte.st/docs/design/architecture/signal-handling/) forwards an interrupt to test process groups, consistent with the earlier shutdown. The [installed revision's displayer](https://github.com/nextest-rs/nextest/blob/8af696ddcce8fff2962d6a5168b6d138b8616a35/nextest-runner/src/reporter/displayer/status_level.rs) intentionally suppresses captured output on interrupt; `failure-output = "immediate-final"` alone does not guarantee a final output dump in that condition. The maintainer has now configured [JUnit](https://nexte.st/docs/machine-readable/junit/) and [native recording](https://nexte.st/docs/features/record-replay-rerun/). These are useful existing diagnostic capabilities; no separate reporting project is needed to address the demonstrated causes.

| Alternative | Fit and cost | Recommendation |
|---|---|---|
| Keep current setup and manually select a compatible state | Smallest immediate operational correction; Rust remains safe, but repeated prerequisite panics and local-test coupling remain | Use for diagnosis; insufficient final architectural correction |
| Existing compatibility operation plus scoped Nextest/repository adapters and local-test split | One meaning owner; early shared refusal; local checks lose unnecessary effects; native scheduling and reporting remain library-owned | Preferred direction, with experimental setup behavior and selection tested explicitly |
| Improve the fixture error alone | Better per-case context without a runner dependency; still repeats a shared failure and still blocks local checks | Useful supporting diagnostic, or fallback adapter if native setup cannot satisfy the entry contract |
| Require a repository wrapper for every test invocation | Existing wrapper can supply lifecycle ownership, but creates a new restriction on ordinary bare-tool use and does not itself fix local coupling | Not selected; no need to change the bare-tool policy |
| Reject old interpretations in all supervisor configuration reads | Simple comparison in the wrong owner; breaks historical-state inspection and explicit maintenance | Reject this remedy |
| Accept v2, relabel it v3, bypass canonical runtime setup, or reduce assertions | Apparent green results with weaker interpretation/source/coverage guarantees | Reject these remedies |
| Add a new test scheduler, report ledger, store compatibility service or compiler mock | Duplicates adopted runner/owner capabilities and obscures the real cause | No demonstrated need |

F01 and F02 complement one another: isolate genuine local responsibilities first or alongside admission work, so a shared gate applies only to consumers that need the service. F03 is independent configuration repair. Neither early admission nor a new service selection permits deleting per-use Rust checks or merging test databases. Existing source-authority, connection/drain and terminal-outcome distinctions must survive every correction.

## Foundation and gate judgments — slot 6

Verdicts apply only to the inspected test/admission boundary. A product-wide foundation verdict cannot be inferred from this bounded review.

| Foundation | Verdict | Grounding |
|---|---|---|
| AP-01 Separation of concerns | Violated | F02 couples owned factory/decoding checks to canonical runtime construction; split those responsibilities while retaining integration coverage |
| AP-02 Stable contracts | Satisfied within inspected production boundary | Reader interpretation mismatch is typed and explicit; no silent compatibility fallback. F01 improves admission discoverability rather than redefining stored meaning |
| AP-03 Composition | Satisfied within inspected boundary | Existing runtime, canonical fixture, service and runner owners can compose the correction; no copied end-to-end subsystem is required |
| AP-04 Domain model and semantic authority | Satisfied within compatibility/lifecycle scope | Interpretation, profile format, result durability, fixture drain and terminal outcome have consequential distinct meanings and owners; no whole scientific-model judgment |
| AP-05 Explicit structure | Violated | F01's shared compatibility prerequisite is late in the entry route; F03's selector no longer enforces its documented protection |
| AP-06 Local reasoning/testability | Violated | F02's local operations cannot be exercised through their current controls without unrelated store setup |
| AP-07 Execution fit | Violated in local test setup; no producer timing violation established | F02 performs unnecessary canonical work; F01 amplifies one shared prerequisite into 107 blocked tests. Quantitative savings are unmeasured |

| Gate | Judgment | Evidence / limits |
|---|---|---|
| G1 Authority | Pass, scoped | One registry interpretation feeds derived consumers; no competing semantic version authority found |
| G2 Semantic fidelity | Pass, scoped | v2 is refused, not reinterpreted; profile-format and interpretation domains remain distinct |
| G3 Validity | Pass at the inspected reader boundary | All 107 reject incompatible input before fixture creation/scientific evaluation; preflight must preserve this enforcement |
| G4 Hidden behavior | Pass for inspected production entry contracts | Canonical construction has explicit effects; F02 is an unnecessary test dependency, not a discovered silent pure-production effect |
| G5 Consistency/recovery | Unresolved for interruption follow-up | Completed five cases pass; standalone wrapper interrupt/finalization behavior needs an executed control before qualification. No observed corrupt publication or unsafe reclaim established |
| G6 Transformation/reuse | Unresolved beyond examined producer controls | Five original controls pass; the blocked cache/reuse tests and full receiving/replay semantics need their own execution/qualification |
| G7 Truthful capability claims | Pass for this review's scoped claims | Interrupted, prerequisite-blocked and passed cases are distinguished; bare workspace execution is not claimed as full native/scientific qualification |
| G8 Library leverage | Pass, scoped | Nextest already owns scheduling, signals and reports; proposed adapters fill application compatibility, without duplicating those mechanisms |
| G9 Architectural fitness | Fail | S02 violates AP-06/AP-01; S01/S03/S07 expose AP-05 and avoidable setup/failure amplification |
| PS-G1 Physical fidelity | Unresolved for affected scientific scope | Physical-fact refusal control passes, but this review does not independently qualify all quantities, bases, reference states or closure checks |
| PS-G2 Structure/well-posedness | Unresolved for affected scientific scope | Original 107 fail before their scientific subject; no general structural audit performed |
| PS-G3 Numerical fidelity | Unresolved for affected scientific scope | No independent derivative, solver outcome, initialization-restoration or post-solve qualification audit |

The unresolved scientific rows are coverage limits, not newly discovered numerical defects. The architectural decision is already **Revise** from the proven local-test and admission findings; passing downstream tests cannot remove those structural findings by itself.

## Verification, rule impacts and next decision — slots 10–12

| Claim | Evidence | Conditions / result |
|---|---|---|
| Completed workspace failure census and five prior SIGINT outcomes | Tested, maintainer-supplied execution artifact | Command and UUID above; all reported XML cases inspected; 107 interpretation refusals, 2,256 passes, five named controls pass |
| Refusal occurs before UUID fixture creation, connection and science | Interface-checked | Canonical options, fixture support and workflow helper inspected |
| Current default-selected configuration is v2 | Interface-checked live observation | `pse_env.base_environment(Path.cwd(), os.environ)` composition and nonsensitive selected config fields; does not recover the original Cursor environment |
| Factory ownership and decoding dependencies are unnecessarily broad | Interface-checked | Direct consumed inputs versus fixture construction inspected; split not implemented |
| Exact-name scheduling selector is stale | Interface-checked | Current config and actual source test name inspected; no race/performance claim |
| Consumer preflight, local split and interruption improvements work | Proposed | Require targeted controls stated in findings; no remediation implemented |

**Rule impacts: none for the selected recommendations.** The design preserves exact interpretation refusal, mandatory canonical scientific admission, force-validation, isolated fixtures and bare-tool availability. Native setup's experimental capability is an implementation tradeoff to assess, not a changed architectural rule. No ADR change, compatibility exception or historical-result baseline is proposed. If a later design instead mandates a wrapper, changes fixture disposal eligibility or removes scientific admission, that is a different recommendation and must identify its own rule impact before adoption.

The next consequential decision is the consumer-admission operation and its native Nextest adapter, coupled with the small local-test split. The selected store can be repaired operationally by explicit selection or fresh current setup; an older authoritative state needs the existing explicit maintenance/rebuild route if its contents must be reconstructed. Do not mutate configuration strings to simulate a migration. Historical local run values are not required oracles: preserve current invariants, actual compiler-consumption challenges, malformed physical-fact refusal and explicit external-reference checks where relevant.

| Priority | Proposed correction / decision | Closure obligation | Follow-up owner |
|---|---|---|---|
| High | F01: settle consumer compatibility before canonical test fan-out | Correctly scoped negative/positive entry controls and retained Rust late refusal | Future authorized remediation plan |
| High | Execute compatible-state diagnostic and assess newly exposed failures individually | All 107 intended behaviors reached, with precise feature/store conditions | This review's diagnostic evidence; later campaign remains separately owned |
| Medium | F02: isolate actual local ownership and decoding checks | Store-independent local controls plus retained end-to-end coverage | Future authorized remediation plan |
| Medium | F03: reconcile obsolete effect scheduling | Actual inventory and justified competing-effect control | Future authorized remediation plan |
| Follow-up | Standalone interruption and temporary catalog ownership | Targeted ownership/interrupt/unwind controls; preserve conservative retention | Future authorized remediation plan, if adopted |

Behavioral adequacy is established for the interpretation refusal and the five completed controls under the supplied run's conditions, not for the entire simulator. Architectural fitness needs revision at the identified boundaries. The strongest correction removes unnecessary dependencies and bounds shared prerequisite failure while preserving the existing source and scientific guarantees.
