---
title: Rust-owned host and execution boundaries
date: 2026-10-28
standard: {core: "3.4", process-simulator: "1.5"}
tier: design
purpose: target
focus: combined
doc_role: review
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [design, environment, validation]
---

# Rust-owned host and execution boundaries

## 1. Scope, basis and conclusion

**Accept at the Proposed target-design evidence level**, with the library and existing native
interfaces below **Interface-checked**. The bounded subject is
[ADR-0233](../../adr/0233-rust-owned-host-and-execution-boundaries.md): the lean native host
root, first-party operating boundary and its composition with native work, canonical write
admission, fixtures, compiler caching and observation. No material MUST gap or SHOULD deviation
was found in that target. This decision does not accept an implementation or qualify the
whole simulator. In the template's terminology this is a bounded **Accept**, rather than
**Accept scoped**, which denotes recorded SHOULD deviations.

This is an independent, combined-focus, design-tier target review. The library focus is the
selected zbus/systemd/Tokio route and native compiler-cache composition; the principle focus is
ownership, explicit lifecycle authority, local construction/testability and proportionate
execution. The criterion is Core 3.4, its efficiency companion, and the applicable Process
Simulator 1.5 obligations. The test-assessment 1.2 companion informs fixture and evidence
reasoning. Current repository decisions explain the baseline; agreement with them is not the
acceptance criterion.

The refreshed diagnostic input is publication **`8fed0a30cd3fd254049ca800a0ed6800977e7ecc`**, at
[the scoped T01 host-foundation refresh](../audit/development-tooling.md#scoped-t01-host-foundation-refresh--2026-10-28),
whose inspected source is `50f07ea6fe757db362e4e9c1ec0c11e04545d75c`. Relevant diagnosis is
[DA-037](../audit/development-tooling.md#da-037), with the fresh narrowing of
[DA-036](../audit/development-tooling.md#da-036). The old profile literals and direct
test-resource-query account are not current premises. DA-043's whole capability graph and the
separate test account were not refreshed by this engagement.

Reviewed target inputs are the working-tree ADR-0233, blueprint §21.7's explicitly Proposed
boundary, [58b](../../plans/58b-bootstrap-and-host-control.md),
[58d D05](../../plans/58d-canonical-control-and-connectivity.md#d05),
[58f F02](../../plans/58f-development-and-assurance.md), the
[coordinator's T01/T02 checkpoint](../../plans/58-rust-owned-system.md), and the
[improvement review's I01/I03–I05/I10–I12](../../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md).
These documents describe intended behavior; the authorization checkpoint does not make it
implemented. No new production code existed in this reviewed slice.

## 2. Functional intent and responsibility boundaries

Cold construction and useful ownership diagnosis must work when Python, mathematical providers,
the database or workload admission are unavailable. A foreground operation must preserve its
original child outcome and distinguish request acceptance, manager completion, process exit,
descendant drain and resource release. Durable data governs persisted transitions without
becoming an OS-process join service. Pure scientific computations need no canonical store.

The proposed decomposition gives these meanings appropriate owners:

| Owner | Owns | Consumed boundary |
| --- | --- | --- |
| B / lean host root | Environment selection, typed user-manager operations, original OS identities and physical observations | Owned capabilities allow destruction; borrowed observations do not. No database or mathematical types enter this closure. |
| C / native work | Actual preparation, execution, cancellation and joins | Stop intent is not release; finite charges survive until the work drains. |
| D / canonical operations | Schema applicability, authentication, durable transitions, mutable write fences and migration | Process establishment is insufficient for readiness; stale clients cannot retain authority across migration. |
| F / development and fixtures | Native tool selection, actual fixture construction and assurance/report consumers | Scheduling names grant no effects; disposable placement does not authorize existing-store reset. |
| Scientific observation owners | Calculation progress, diagnostics and scientific lineage | Build-stage/freshness observation can retire without erasing scientific outcomes. |

This is a dependency and meaning boundary, not a mandate for a trait or crate per row. Heavy
development operations stay with their real consumers. Application composition connects the
host's process capability to D's readiness/admission capability; the host must not import D to
make that connection itself.

## 3. Contracts and semantic authority

The domain here is host execution, not physical formulation. Its consequential distinctions are
original attempt versus replacement, owned versus borrowed resource, observed absence versus
unavailable evidence, queued/completed job versus drained descendants, and supported schema
versus currently admitted mutation. Private construction and explicit uncertainty are suitable
ways to make those distinctions govern behavior. A Rust handle cannot establish an independent
process's continued identity or a mutable store fence forever.

ADR-0233 binds process/unit/cgroup observations to original identity and refuses destructive
action on unknown ownership. Reconnection observes an uncertain launch; it does not replay it.
The seed constructs and privately copies a completed executable before atomic selection. Reads
do not compile or rescan source. A running process retains its selected executable generation.
These contracts avoid both guessed success and a second freshness authority.

Environment resolution preserves command, caller, personal-local and repository-default
precedence, explicit empty values and secret meaning. Child mappings are explicit; there is no
process-global environment mutation after threads start. The environment guide's secret-safe
explain/deferred-local behavior remains a preservation constraint: a diagnostic must not execute
local shell configuration merely to explain provisional defaults or expose its secrets.

D05's quiescent migration contract is sufficiently explicit for this decision: close new
claims/mutations, permit narrowly defined original-attempt settlement under the old
interpretation, drain writers and native work, then transform and publish the successor.
Unknown drain keeps admission closed. Actual writes recheck mutable canonical fences, and
successor activation fences stale handles and reconstructs capabilities. This avoids the
deadlock of closing the very settlement route needed to drain. It also avoids transforming
under an admitted writer. Supported transformation meaning and the exact permitted settlement
operations remain D's implementation contract, not host policy.

The existing `canonical.rs::WriteAdmission` reads local `config.json`; `ensure_writes` checks it
before operations. That inspected path is useful current admission behavior, **not** evidence
of D05's transactionally effective schema fence. `canonical_maintenance.rs` does not by itself
establish the proposed cross-writer migration journey. B04 cannot close on a proposed D interface.

No physical quantity, property correlation, equation, derivative, scaling convention or solver
outcome mapping changes here. A physical-semantics table and well-posedness analysis would have
no scoped subject. Scientific contracts remain with their existing authorities; this review
does not claim to have checked their numerical realization.

## 4. Revealing change and failure scenarios

| Scenario | Architectural observation and expected response |
| --- | --- |
| S01 — Cold checkout without a venv; inherited Python compiler wrapper | Explicit shell seed builds the locked host package with wrapper recursion bypassed for that invocation; help/diagnosis needs no science or DB. Failed publication retains the last completed selection. |
| S02 — Fast completion, lost launch reply, manager disconnect, or unit/PID reuse | Subscribe before submission, correlate the original job and retain sufficient original unit/result identity. Reobserve rather than launch again. Unavailable identity remains uncertain. |
| S03 — Leader exits while a setsid descendant remains | Manager containment plus bound hierarchical cgroup observation establishes drain; process-group or pidfd completion alone cannot release the charge. Borrowed resources cannot be stopped as if owned. |
| S04 — Migration overlaps original settlement and a stale open client | New mutations close; authorized original settlement completes under the old interpretation; all writers/native work drain before transformation. Stale-handle writes refuse after the successor fence. |
| S05 — Cache service dies around a successful compiler result | Preserve the actual successful result; native tool/compiler errors remain errors. No Python continuity watcher or blind retry redefines the outcome. |
| S06 — Assertion fails in a private disposable fixture | Explicit settlement still runs and retains cleanup failures/diagnostics. Cleanup cannot imply scientific success or confer deletion authority over an existing external store. |
| S07 — Add a retained service or replace a host implementation | The service composes process establishment with its own readiness semantics; manager details remain in B. Scientific algorithms and D's transition meaning do not change merely because the host mechanism changes. |

S07 supplies a relevant domain extension and mechanism-substitution test. A new unit/property
model is outside this bounded review; it should not need host policy changes merely to extend
scientific meaning. Grow/interruption reasoning covers concurrent ordinary builds/operations
and explicitly finite scientific work, rather than a whole-simulator capacity campaign.

## 5. Mechanisms and execution fit

The independent crate is justified by construction and failure-domain needs. Inspected
`xtask/Cargo.toml` still has unconditional codegen/schema/columnar dependencies; `default = []`
does not remove them. `pse-diagnostics` already supplies lightweight process facts and
`SupervisedChild`; its manifest has no Arrow/DataFusion dependency. Reuse those useful facts
without treating its explicitly limited process-group proof as setsid containment. Inspect the
new root's actual normal, build and selected test closure when implemented; importing a heavy
testkit would defeat the claimed independence.

Foreground ownership plus systemd containment avoids inventing a resident admission daemon,
new host database or general allocation ledger for configured unbounded work. Finite scientific
demands and selected reference/timing exclusions still compose with native joins. Ordinary
parallelism remains available. The design removes repeated interpreter crossings, per-compiler
identity polls/hashes and read-time source scans; their absence is a structural proposal, not
a measured latency or throughput benefit.

The complete migration operation deliberately orders only effect-dependent work: short
admission/fence decisions, permitted old settlement, physical drain, transformation and
successor activation. Expensive scientific preparation and execution stay outside transactions.
Interrupted migration stays explicitly pending/closed rather than reopening on restart.
Executable selection, immutable payload visibility and canonical transaction publication keep
their different atomicity promises; one universal publication framework would obscure them.

## 6. Foundation and gate assessment

The following judgments apply to the **Proposed design**, with inspected mechanism evidence;
they do not claim an implemented enforcement point.

| Foundation | Verdict | Decisive scoped reasoning |
| --- | --- | --- |
| AP-01 Separation | satisfied | Host facts, scientific joins, canonical transition policy and assurance consumers have distinct owners; a native launcher alone is explicitly insufficient. |
| AP-02 Stable contracts | satisfied | Original outcome, ownership and uncertainty survive manager replacement and transport loss; consumers need no database types to use host operations. |
| AP-03 Composition | satisfied | Process establishment composes with service-specific readiness; native science can remain direct; optional faces do not impose a universal operation catalogue. |
| AP-04 Domain authority | satisfied | Typed original/owned/borrowed identities and D's actual-write fence govern decisions, rather than names, test groups or permanently trusted handles. |
| AP-05 Explicit structure | satisfied | Admission, settlement, drain and migration order are declared; unsupported/unknown observations refuse rather than masquerade as release. |
| AP-06 Local reasoning | satisfied | Cold root and local policy tests exclude unrelated science/DB; real manager journeys remain necessary for claims about that boundary. |
| AP-07 Execution fit | satisfied | Native containment and explicit setup remove duplicated hot-path custody; joins match work lifetimes; ordinary parallelism survives and migration fences only necessary effects. No unexamined scale premise is used to claim speed or full capacity. |

| Gate | Judgment | Evidence and limit |
| --- | --- | --- |
| G1 Authority | pass | One host owner; D owns canonical meaning; scientific permission and physical release remain distinct. Current DA-037 is not claimed resolved. |
| G2 Semantic fidelity | pass | Original/replacement, borrowed/owned, job/drain and schema/admission distinctions are preserved. |
| G3 Validity | pass | Private capability construction, identity checks, supported-version construction and actual-write fences are assigned enforcement points; their implementation remains required. |
| G4 Hidden behavior | pass | Read/explain do not rebuild or admit work; child environments are explicit; optional telemetry does not gate computation. |
| G5 Consistency/recovery | pass | No automatic uncertain-launch replay; physical drain precedes release and schema transformation; incomplete migration remains closed. |
| G6 Transformation/reuse | pass | Explicit executable generation and schema migration contracts; cache-service continuity is intentionally removed as an outcome premise, not mistaken for equivalence. |
| G7 Truthful capability | pass | Native interfaces supply a credible route; systemd255 runtime qualification and cache-mode controls remain explicitly unperformed. No new numerical or speed claim is made. |
| G8 Library leverage | pass | zbus/systemd own IPC and containment; native sccache owns compiler-cache machinery; no bespoke daemon, retry layer or shutdown hierarchy is selected without need. |
| G9 Architectural fitness | pass | All seven applicable foundations are satisfied for S01–S07 at target-design strength; no averaging of those judgments is used. |
| PS-G1 Physical consistency | not applicable | No physical quantity/property/balance contract changes. |
| PS-G2 Well-posedness | not applicable | No formulation, incidence analysis or solve-order contract changes. |
| PS-G3 Numerical integrity | not applicable | No numerical stage is reviewed. Host cleanup and compiler success cannot substitute for independent scientific acceptance. |

PS-10/PS-12 supply preservation constraints on truthful outcomes; passing the host design does
not establish a scientific result. PS-11's reuse obligations are not weakened by deleting build
observation. No profile MUST is waived.

## 7. Findings and preservation constraints

**No new blocking architectural finding or SHOULD exception.** The current implemented
composition defect remains DA-037 until integrated code/callers/deletions and evidence settle it;
reviewing a replacement does not resolve that diagnosis. DA-036's narrowed historical
manifestations must not be recreated merely to preserve old assertions.

The positive verdict depends on the actual contracts above, especially original-result
retention across fast completion, no automatic replay after reply loss, identity-authenticated
drain, narrowly permitted old-attempt settlement, and fences at actual writes. An implementation
that drops any of those contracts is outside this accepted design. These are preservation
constraints already present in the target, not additional findings or a new status ledger.

## 8. Library fit and ownership cost

Context7 was the first library lookup. Its zbus result describes a moving main branch; exact
release source supersedes it for consequential feature details. The tagged
[zbus 6.1 manifest](https://github.com/z-galaxy/zbus/blob/zbus-6.1.0/zbus/Cargo.toml)
supports a client/Tokio feature selection without default runtime/service machinery; the
[tagged workspace manifest](https://github.com/z-galaxy/zbus/blob/zbus-6.1.0/Cargo.toml)
declares Tokio `1.53.3`. This supports the narrow move from the repository's exact `1.53.2`
pin rather than a second runtime version or wholesale resolution. This is Interface-checked,
not a resolved dependency-closure check.

The inspected published `zbus_systemd 0.26201.0` manifest uses zbus 6 with `proxy`, defaults
disabled, and exposes `systemd1` and `zbus-async-tokio`. Its generated systemd1 source contains
`Manager`, `start_transient_unit`, `subscribe`, `job_removed`, `control_group` and
`invocation_id`. The [versioned crate source](https://docs.rs/crate/zbus_systemd/0.26201.0/source/)
and the [repository overview](../../libraries/zbus_and_zbus_systemd.md) are reproducible
entry points. Library types can remain inside B's integration owner; owned/borrowed authority
and physical-release meaning must remain the application's explicit contract.

The [systemd255 interface](https://www.freedesktop.org/software/systemd/man/255/org.freedesktop.systemd1.html)
provides asynchronous unit jobs and transient-unit construction. Its documented signal ordering
supports subscription before submission and correlation by returned job path. The
[v255 cgroup guidance](https://github.com/systemd/systemd/blob/v255/docs/CGROUP_DELEGATION.md)
supports manager-derived cgroup paths rather than reconstructing them from unit names.
Generated newer proxies do not establish runtime method availability or user-manager authority.
Fast transient-unit collection must not erase the original result before it is retained; B02's
completion-before-reply case is a revealing qualification of that contract.

Native sccache client-side compilation is selected to keep actual compiler work inside workload
ownership, while F owns setup/configuration. Its
[0.19 architecture](https://github.com/mozilla/sccache/blob/v0.19.0/docs/Architecture.md) and
[startup implementation](https://github.com/mozilla/sccache/blob/v0.19.0/src/commands.rs) are
the release-specific qualification route carried by F02. This review does not establish every
logging/distribution mode. Native result validity is the selected premise; deleting a watcher
does not make every cache error successful. `sd-notify` and hierarchical shutdown remain
eligible optional capabilities, with no demonstrated need to add them to this root now.

## 9. Alternatives and tradeoffs

| Alternative | Assessment |
| --- | --- |
| Current Python controller and thin Rust launcher | Retains the import cycles, second host policy and per-compiler custody. Renaming the entry point does not solve the diagnosed boundary. |
| Feature-separated xtask | Viable only after disentangling unconditional scientific/schema dependencies and their imports. It carries broader coupling into cold construction than the dedicated small root. |
| Lean host root plus systemd and existing native facts | Selected. Adds a crate and a seed/update seam, while localizing manager integration and deleting duplicate policy/callers. |
| Resident admission daemon | No named persistent coordination requirement justifies its IPC, restart and authority model. Revisit if a real retained multi-client duty needs it. |
| Process groups alone | Simpler for a deliberately bounded group contract, but insufficient for the selected descendant-release guarantee. Retain as a useful primitive, not a substitute for containment. |
| Global fixture permission/terminal registry | Repeats authority already supplied by real construction/ownership, and can confuse cleanup with science. Shared runtime uncertainty still requires real checks. |

The selected route is also the simplest viable library-owned alternative for this supported
Linux contract. It is not a claim that Rust or systemd inherently produces faster or more
reliable science. If user-manager access cannot supply the retained physical guarantee, reopen
the mechanism choice rather than silently downgrade it to process-group completion.

## 10. Verification and limits

| Claim | Evidence | Result / remaining obligation |
| --- | --- | --- |
| Current host composition and cache-continuity premise | Interface-checked: refreshed audit and direct source reads of launcher/environment, compiler cache, xtask and diagnostics | Current cause confirmed within the refreshed footprint; no timing claim. |
| Proposed host/library composition | Proposed plus Interface-checked tagged manifests, generated proxy interfaces and systemd255 documentation | Credible mechanism and bounded architectural acceptance; no actual manager session exercised. |
| Native migration fence | Proposed; existing `WriteAdmission` and maintenance paths inspected | D05 must implement and challenge transactional closure, old settlement, stale handles and interrupted successor activation. |
| Host/result/drain and fixture behavior | Proposed B01–B05 controls and S01–S07 | Targeted tests and real disposable user-manager journeys remain unrun. |
| Native compiler/cache modes | Proposed F02 source and targeted-control route | Pinned logging/distribution behavior and cache-loss cases remain unqualified. |
| Scientific/numerical behavior or performance | No new claim | No scientific tests, benchmarks or assembled product qualification ran. |

No code, build, test, service action or process-lifecycle mutation was performed for this review.
The failure target for future named checks remains zero; there is no executed test result to
report as passing. Inspection covered the scoped sources named above and the refreshed audit,
not every script import, fixture body, supported platform or capability. That unexamined breadth
does not certify absence of defects elsewhere. Runtime restrictions remain implementation
acceptance obligations rather than evidence furnished by this document.

## 11. Rule impacts and disposition owner

The following changes follow the selected target rather than a demonstrated new defect. Their
operator disposition is recorded in the [Plan58 rule-decision table](../../plans/58-rust-owned-system.md).
ADR-0233 and its architecture route carry the enduring changes; this review changes no rule
or accepted ADR status. The coordinator owns current disposition and dependent work.

| ID | Current rule / owner | Target change | If the rule is retained |
| --- | --- | --- | --- |
| RC01 | Blueprint D12/§21 mandatory Python authoring/result boundary; affected ADR-0164/0214/0228/0229/0232 requirements | Native first-party operations; optional CLI/MCP; deferred PyO3. Preserve unaffected scientific/native rationale. | The full Rust-owned boundary cannot be claimed; a lean host alone could still be delivered under a narrower target. |
| RC02 | Environment guide/§24 host launch, Python wrapper and host-controller mechanics | Native host ownership and independent seed; retain precedence, secret-safe explanation, scientific constraints and physical release. | The Python controller remains necessary and the reviewed host cutover is incomplete. |
| RC03 | Plan54/environment/build-performance cache continuity and per-compiler custody | Native compiler result governs; cache-service death alone cannot poison success. Remove watcher and matching callers/tests. | The watcher or equivalent continuity policy must remain, defeating this selected simplification. |
| RC04 | Existing fixture permission/terminal and success-gated reclamation requirements in validation/resource tooling | Actual owned/borrowed construction; cleanup after pass or failure; preserve diagnostics and real uncertainty. | Fixture cutover retains the second authority/protocol and needs a revised architectural assessment. |
| RC05 | Existing build-stage/wait/freshness observation and dependent alerts/queries | Retire producer and consumers together; retain scientific progress/diagnostics. Collector removal is separate. | Those consumers require a real producer; ordinary tool output cannot be asserted equivalent. |
| RC06 | Current canonical configuration admission and historical recovery expectations, blueprint §20.6 | Minimal native migration and actual-write fences now; broader historical backup/recovery qualification before shared/non-regenerable deployment. | B service startup cannot claim the new readiness/migration capability; applicable durable obligations remain open. |

No new waiver of scientific validity, physical release, authentication or protected-data
ownership is proposed. No existing-data deletion is authorized. No new finding disposition
table is needed because no additional actionable defect was found.

## 12. Decision and next boundary

**Behavioral/semantic adequacy:** Accept as a Proposed bounded contract with a credible,
Interface-checked realization route. **Architectural fitness:** Accept for the seven assessed
foundations and S01–S07. **Overall:** Accept the ADR-0233 target within this scope. No blocking
finding or SHOULD exception was identified.

The next consequential boundary is implementation evidence: B must establish the cold closure
and truthful original unit/process/result/drain behavior; D must establish the working migration
fence needed by B04; F must establish selected native-cache modes and actual fixture cleanup.
Those owners retain their existing package tests and scope-end integration obligations. Review
acceptance neither changes ADR status nor supplies those results, and it confers no whole-product,
cross-platform or numerical qualification.


## 13. Remaining native development and product retirement — 2026-10-29

### Subject and decision

**Accept as a Proposed target**, with inspected responsibility boundaries **Interface-checked**.
This independent native design-reviewer extension assesses ADR-0233's appended target at
`da23cbf57`. It continues this combined-focus T01 engagement, rather than restarting a
new diagnostic audit. Concurrent implementation is outside the assessment.

The subject is final `pse-py` removal after real consumers migrate or explicitly retire,
with Python-only packaging, generation and operating policy. Isolated scientific oracle
drivers and third-party tools remain eligible. Native conformance, paired thermodynamic
campaigns, scientific adoption, retained results and calculation observation remain.
Obsolete measurement drivers may retire while their requirements and original results stay H-owned.

### Architectural assessment

| Foundation | Verdict | Reason |
| --- | --- | --- |
| AP-01 | satisfied | Science, canonical state, physical custody and heavy developer composition keep distinct owners. |
| AP-02 | satisfied | Retained contracts move before deletion; peripheral interface parity and dormant compatibility facades are unselected. |
| AP-03 | satisfied | Developer workflows compose existing native operations rather than duplicate scientific policy. |
| AP-04 | satisfied | Rust/domain declarations and independent oracle interpretation retain their separate authority. |
| AP-05 | satisfied | Migration, explicit loss, stop/join, output custody and qualification boundaries remain distinct. |
| AP-06 | satisfied | Cold host construction remains independent of schema/math/database dependencies. |
| AP-07 | satisfied | Interpreter crossings/projections/currency machinery retire without a universal registry, scheduler or publication protocol. |

At target-design evidence level G1–G9 are satisfied. Scientific gates remain preserved
constraints, not new tested results: no formulation, tolerance, solver or numerical
interpretation change is selected. No runtime science, teardown, migration or speed claim follows.
No blocking finding, SHOULD deviation or MUST gap is identified; this is Accept, not a waiver.

### Material implementation conditions

1. Move actual retained callers and their receiving assertions before deleting a boundary,
   or record the selected loss with its existing owner. API availability alone is insufficient.
2. Delete Python-only emission, not retained Rust/Arrow/canonical/file/wire meaning;
   regenerate through the existing declarations and generator.
3. Retain native joins/unwind containment, scientific qualification, canonical effect identity,
   dynamic write fences and escaped-buffer allocation custody.
4. Before deleting a measurement driver's sole scenario expression, transfer its case/mode/input,
   expected scientific outcome, observation-loss and custody requirements to H. Preserve original
   results. This requires neither new H campaigns nor campaign execution now.
5. Oracle processes cannot import the candidate product; absent/failed references stay failures.

### Rule and architecture applications before dependent removal

These extend the existing reviewed RC dispositions; Plan58 remains their work owner.

| ID | Owner | Required amendment |
| --- | --- | --- |
| RC07 | D12; blueprint §3.1–§3.3 and §21.1/§21.5/§21.6/§21.7 | Remove mandatory Python boundary/crate/family and extension delivery requirements with actual consumers; retain native numerical, identity and Arrow contracts. |
| RC08 | §4.2, §23, §24.1–§24.2 and generator/governance/environment owners | Retire Python-only emissions, installed-extension/stub checks and wrapper fixture/controller requirements with their callers. Retain real external admission, force-validation, independent references and historical qualification limits. |
| RC09 | G01/G03 and existing H scenario/evidence owner | Record selected old-driver retirement and preserve specific scientific requirements/results without H qualification claims. |

Amend frontend-only clauses in §5.3/§20.6 where their mechanisms retire; do not weaken
scientific hashing, producer/reuse compatibility or canonical history. Cargo metadata,
packaging, generators, recipes, agent instructions and environment/validation/interface guides
move through this existing decision route with the architecture revision row. Accepted ADR
rationale is immutable. Formal ADR acceptance, implementation acceptance and full H qualification
remain separate. The next boundary is actual integrated migration/deletion with targeted B–G evidence.
