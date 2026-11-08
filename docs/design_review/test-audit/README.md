# Test code design principle audit

This is the living diagnostic account for test architecture, evidence and shared support. Start
with the [integrated architectural account](architecture-and-assurance.md), then follow each
finding to its topic page. The [skill](../../../.codex/skills/design-principle-audit-tests/SKILL.md)
owns the method and the [binding](../design_principles/binding/pse-arrow.md#test-code-design-principle-audit)
owns invocation and finding identity.

## Baseline and method

- **Audited source:** `3419aca2a356a700a351c1706ba629c8ca61bfbc`, reviewed 2026-11-07. The
  checkout was clean at selection. The audit document edits are outside the source baseline.
  Git records publication separately.
- **Previous source:** `7bd341f0b` with the included changes cited at `f5b436bbf`. The
  [October publication](https://github.com/paul-heyse/pse-arrow/blob/24852b36d2552709a8d304faf15e24157e2abdda/docs/design_review/test-audit/README.md)
  records that moving baseline, its exclusions and the original challenger review. The prior
  scoped study and database examination used `64a217de2`.
- **Standard:** Core 3.4, efficiency heuristics 1.0, test-assessment companion 1.2, process-simulator
  profile 1.5 and the pse-arrow binding, selected through [standard.toml](../design_principles/standard.toml).

Three parallel source investigations first traced current execution, fixture and scientific
evidence paths. They then used the 928 commits after `f5b436bbf` as leads to changed test and
production contracts and followed affected callers into unchanged code. The coordinator sampled
xtask, governance and the diagnostics process supervisor, and reconciled the evidence across tracks.
Current judgments are **Interface-checked**. Earlier execution and measurements retain the
commands, conditions and limits in their original records. The audit used source and recorded
results for its diagnoses.

### Selection and result refresh for the 2026-11-08 review

**Interface-checked · scoped source `dab55ccca2dd177792bea5469ce7798f93ad2deb`.** This
refresh covers the committed Plan69 selection, nextest policy and result-custody delta. Pending
changes in `pse-host/src/managed/workflow.rs` and `managed/workflow/borrowed.rs` are excluded.
The November 7 examination remains the source of evidence for the other areas and claims.

- Ordinary selection now excludes workspace packages with no default test, doctest or example
  target from its intended root set and names packages whose intended root is absent from the
  admitted graph. Nextest has a dedicated `managed-primary` profile. This narrows the earlier
  selection diagnosis, while actual selected cases and their fixture readiness remain separate.
  The scientific grouping and qualification-only prerequisite findings still apply. [Selection](../../../crates/pse-host/src/selection.rs) ·
  [Nextest profiles](../../../.config/nextest.toml) ·
  [DTA-015](evidence-routes-and-execution.md#dta-015) · [DTA-050](evidence-routes-and-execution.md#dta-050)
- Detached and borrowed test-result classification now consumes authenticated original drain
  and the paired original result before compact expiry begins. Plan69 owns its integrated result
  acceptance. This further bounds the retired verdict mechanism and prior retention diagnosis.
  Opaque target-owned setup can still fail inside the test body. [Result custody](../../../crates/pse-host/src/invocation/compact.rs) ·
  [DTA-012](evidence-routes-and-execution.md#dta-012) · [DTA-025](evidence-routes-and-execution.md#dta-025) ·
  [DTA-052](execution-efficiency.md#dta-052) · [Plan69c](../../plans/69c-ordinary-testing-and-managed-results.md)

**Publication reconciliation · `72ce83228`.** The pending workflow changes landed as
`be095a207`. They release unpublished inventory images for an interrupted listing only after
the owner can establish paired settlement and the absence of a published inventory. Plan69c
records the named receiving result and retains source/build image capture as an explicit design
boundary. DTA-052's remaining test-owner release question and DTA-039's repeated verification of
selected images are distinct from the corrected interrupted-listing path.

## Integrated synthesis

- Managed correctness binds an admitted Cargo graph, source-built inventory, fixture selection and
  original nextest results. Selected runtime cases now have a scientific group. Generic Cargo
  execution and some specialist cases still carry different runner or prerequisite guarantees.
- SurrealKit covers selected schema and actor behavior on a test-owned endpoint. Rust retains
  physical migrations, reopen, caller semantics and child drain. Fixture consolidation removed
  several feature and preparation variants, with residual duplication in target-owned service
  construction and a distinct cleanup path for borrowed fixtures.
- Native parity and authored conformance routes now exist. Oracle isolation and the resulting
  values are stronger in the ordinary Proto path. Some allowances, reference setup rules and
  historical compatibility inputs still depend on the system being checked or its current shape.
- Focused work still inherits the scope of larger units. The same captured inventory can be hashed
  repeatedly inside one command. Selected scientific inputs can rebuild an entire catalog within
  each consuming test process. Membership controls request the resolved dependency graph several
  times. These are structural observations about work performed.

## <a id="revalidation-outcomes"></a>Revalidation and primary homes

Each topic page begins with current evidence and dispositions. Historical sections keep their
original conditions and anchors. A retired mechanism can leave an assurance obligation in a
successor route. A residual names the cause and its current footprint. An inherited result keeps
its original evidence date and scope.

| Topic | Current focus |
|---|---|
| [Oracles and acceptance](oracles-and-acceptance.md) | Candidate-derived allowances, route coverage, historical inputs, and assertion discrimination |
| [Routes, activation and outcomes](evidence-routes-and-execution.md) | Actual selection, prerequisites, setup verdicts, and timing controls |
| [Support ownership and coupling](doubles-and-coupling.md) | Fixture and reference construction, shipped seams, and host evidence |
| [Execution system](test-execution-system.md) | Native and generic route contracts and old tooling retirement |
| [Execution efficiency](execution-efficiency.md) | Repeated validation and preparation, resource lifetime, and the new findings below |
| [Scoped study and database follow-up](study-execution-and-database-boundary.md) | How the October evidence applies to current owned, borrowed and prepared lanes |

Each structural cause keeps one primary home. The test account links the broad account's
[DA-037](../audit/development-tooling.md#da-037) for the historical tooling cycle and does not
refresh that baseline. [Plan53](../../plans/53-test-architecture-target.md#finding-dispositions)
preserves historical dispositions. [Plan66l](../../plans/66l-required-population-and-assembled-acceptance.md)
owns remaining scientific acceptance. [Plan67c](../../plans/67c-surrealkit-and-rust-test-consolidation.md)
records the selected tested cutover. [Plan69c](../../plans/69c-ordinary-testing-and-managed-results.md)
owns ordinary testing and managed results. The audit owns diagnosis.

[DTA-071](execution-efficiency.md#dta-071) records repeated dependency-graph acquisition by
workspace membership controls. [DTA-072](execution-efficiency.md#dta-072) records reference
output that can grow on disk before its readback limit applies. DTA-001 through DTA-072 are
allocated, including the withdrawn records. The next available identity is DTA-073.

## How the standard informed this refresh

Changes to fixture inputs and provider preparation showed where construction responsibilities
remain local and where callers depend on incidental setup details (AP-01–AP-03, DP-13–DP-18).
The scientific traces followed units, input identities, reference calculations, acceptance
allowances, and observed result versus asserted outcome (AP-04–AP-06, DP-01–DP-12 and DP-21–DP-24
where relevant). The authored conformance and reference cases supplied examples of physical and
numerical discrimination under the process-simulator profile. A supported population or solver
claim still needs the corresponding case, correct source, and actual verdict.

For AP-07/G9, the investigations followed focused and broader routes through admission,
build and list, immutable preparation, execution and failure settlement. In particular, H5,
H12, H18–H21 and H26 distinguish necessary graph scope, authentication, resource ownership
and independent evidence from work repeated for incidental command or process boundaries.
The [integrated account](architecture-and-assurance.md) explains these tradeoffs and preserves the
real-service, process and oracle checks that give the retained evidence its meaning.

## Challenger review

A fresh reviewer checked the draft against the same source revision. The review identified the
ordinary Python controls omitted by `reference-unit` and the documented manual selections that
limit that diagnosis. It separated SurrealKit's setup-result finality from proof of the exact
seed failure, verified which clock governs service shutdown, and distinguished full Cargo
metadata requests from Cargo's unmeasured internal work. It also separated the ordinary wire
response bound from the scratch diagnostic writer and traced both repeated executable-verification
paths. The coordinator checked the source and incorporated those corrections in the topic records.

## Scope, strengths and limits

Inspected representative pure and Arrow-consuming selections, doctests, a native scientific route,
host execution and provider controls, SurrealKit and native operations fixtures, isolated reference
exchanges, authored conformance, and selected owning-crate inline, solver, runtime and governance
tests. The coordinator sampled xtask documentation, conformance and thermodynamic campaign controls,
and the diagnostics process-supervision tests. These traces do not establish a denominator for every
test, exact feature/profile combination, provider, authored scientific model or platform. Doctest
admission and the selected graph checks were read in source. Their live behavior was not sampled.
Prior closures outside these traces retain their evidence limits. Runtime interference, time and
storage growth remain unmeasured in this refresh.
