# Design principle audit

This is the living diagnostic account of alignment gaps in the implemented simulator and its
development tooling. Start with the [integrated system account](system-architecture.md), then
follow its improvement questions to the primary findings below.

## Followup conclusion

The preceding audit was **not merely a catalogue of localized defects**. Its authority,
composition and execution-fit diagnoses describe consequential cross-owner problems, and most
survive source revalidation. However, its emphasis and some conclusions needed correction:

- **A missed scientific permission chain changes the picture.** A fit rejected by original
  scientific checks can retain covariance that invocation evidence marks unavailable, while
  result projection and both local and persisted reuse expose it as qualified fit covariance.
  The refusal is absent from propagation output. This widens
  [DA-038](semantic-model-and-authority.md#da-038). Strong component-level science does not
  establish consistent end-to-end permission.
- **Exchange has another physical work problem.** Executable project closure/explanation holds
  aggregate selected source encodings before copying or returning a manifest
  ([DA-045](execution-fit.md#da-045)). Exact source admission and protection remain necessary;
  aggregate residency is a separate choice. Practical severity is unmeasured.
- **Several earlier claims were too broad.** Proposed ADR status and Tested evidence can coexist;
  the current transport supplement is identifiable; differing path consumers and liveness
  policies can be legitimate. Physical collection has a public Rust primitive and no supported returned-space promise.
  Optional strict producer receipts were conflated with mandatory outer lineage. A universal
  lifecycle envelope and automatic backup thaw were not established remedies. The primary
  findings now carry these corrections, not just this summary.

The clearest improvement targets are coherent scientific permission through derived products,
owned operation/meaning contracts at consumed boundaries, deliberate product/tooling ownership,
and complete-workload choices for publication, populations, exchange and agent reads. These are
properties for later solution comparison, not adopted implementation packages or priorities.

## Baseline and method

- **Audited source:** `682fc3830f8a370f0995544af5adf821e4be9ab9`, 2026-10-25; clean when fixed.
- **Publication:** a later commit, recorded by Git. Included uncommitted production changes: none.
- **Input account:** source `b5bdac9da763bf1a70f20b0599de0edad8a955f9`, published at
  `c02f68670`. Earlier account and challenge text remain recoverable there and at `2257311c0`.
- **Intervening committed changes:** audit text, MCP acceptance tests and their assessment
  documentation. Product implementation is unchanged. The new diagnoses and corrections are
  missed or miscalibrated judgments, not new production regressions.
- **Concurrent changes excluded:** later committed and uncommitted codegen Markdown and
  test/build tooling changes,
  including Arrow validation, native-test currency, setup reporting, archives, telemetry/service
  settlement and report custody, and their tests.
  Decisive observations of overlapping files are read against the fixed source revision; those
  changes are not attributed to the audited source or included in the audit publication commit.
  A consuming solution review must reconcile those later owner changes before relying on an
  affected finding.
- **Standard:** Core 3.4; efficiency heuristics 1.0; process-simulator profile 1.5; pse-arrow
  binding through [standard.toml](../design_principles/standard.toml). Test-assessment companion
  1.2 applies where original scientific checks and assurance are examined.
- **Scope:** the whole implemented product and development tooling. The separate test account
  keeps its own baseline; relevant assurance was inspected without refreshing it wholesale.
- **Maintainer premises retained:** canonical SurrealDB; sixteen coordinated lanes; durable work
  across callers/processes without a daemon; one local operator; uncapped ordinary build mode.
  These do not exempt their realization from assessment.

Two fresh, read-only design reviewers first formed independent conclusions without reading the
old audit catalogue: one followed scientific meaning and model-to-result consumers; the other
followed lifecycle, execution, history/exchange and tooling. Their initial conclusions were
returned before they received the catalogue for reconciliation. The coordinator inspected
cross-owner contracts, general findings and decisive sources of the new/widened/corrected claims.

The independent passes found real strengths as well as gaps. Reconciliation covered every active
finding and the bounded bases of resolved/merged findings; decision-closed mechanisms were not
reopened without new evidence. A subsequent fresh challenge examined the resulting account.
Its outcome is recorded under [Challenge reconciliation](#challenge-reconciliation).

**Evidence limits.** This is source-based architectural diagnosis. No product tests, solver runs,
export/backup fault probes, benchmarks or numerical reference campaign were run. Static searches,
source/diff inspection and documentation consistency checks establish their named scope only.
Previous measurements and exact censuses retain their original date/conditions; an unchanged file
is not a newly executed test. Topic pages retain earlier evidence explicitly and identify followup
re-readings. No overall alignment certificate or gate-acceptance verdict is issued.

**Publication checks, separate from source evidence:** the audit-local reference/identity check
found 158 local links, 45 unique DA anchors and zero errors; `git diff --check --
docs/design_review/audit` found zero whitespace errors. `just turn-end --live` exited zero, but
its safe-fix Ruff step printed nine remaining findings in concurrent `scripts/test_resources.py`
against a zero target. That is not a clean lint claim; those findings are outside this diagnostic
engagement and were not repaired or included in its commit. No product qualification follows.

## How the standard informed this followup

All seven architectural foundations remain applicable. The investigation used supported journeys
and credible changes to test separation, contracts, composition, model adequacy, authoritative
behavior, enforceable structure, local reasoning and physical execution fit. The DP refinements
helped distinguish repeated semantic decisions from legitimate projections and distinct policies.

The scientific profile directed attention beyond adapter status maps: original checks, model
validity/applicability, derived-product qualification and downstream interpretation. Sampled
PS-02 envelopes and PS-13 shared conformance machinery have positive source evidence, with no
claim of complete scientific reference coverage. G9 consequences are structural; quantitative
capacity or speed claims remain unmeasured. No principle count or file-size score substitutes for
that judgment.

## Challenge reconciliation

A third fresh read-only reviewer challenged the draft against exact source `682fc3830`, its
strongest legitimacy and the original account. The two principal additions survived source and
call-order challenge. The following material corrections were accepted and integrated in the
primary pages and synthesis:

| Challenge | Reconciled result |
|---|---|
| DA-025 treated a missing built-in composition as a violated supported promise | Public collection and explicit returned-space/no-automatic-collector scope are retained; no active violation established |
| DA-024 conflated mandatory provenance, local admission and optional strict receipt qualification | Separate those routes and their roles; withdraw the broad purposelessness/unconditional-cost claim |
| DA-005 asserted physical meaning could not be recovered from any bundle | Executable bundles preserve authored physical documents; retain the missing direct result-reader projection |
| DA-043 charged Runtime construction to every native call | Supplied-only comparison/profile capture return after currency/import but before Runtime construction |
| DA-041 demanded a patch series/family and called regeneration impossible | Require truthful recoverable edits; retain mechanisms as solution candidates and withdraw impossibility |
| DA-033/037 retained universal-envelope/liveness wording beside their narrowing | Reconcile the earlier paragraphs: shared facts do not erase different lifecycle/owner policies |
| DA-034 labelled a historical ratio's persistence Interface-checked | Current structure is source-established; the current timing ratio remains unmeasured |

The coordinator re-read the load-bearing covariance, executable-closure, optional-receipt,
physical-bundle, native-worker early-return and historical-measurement evidence. The challenger
found no further established substantive omission in its bounded inspection; this is not proof
of absence. No challenge was resolved by product mutation or a qualification claim.

## Revalidation outcomes

| ID | Followup disposition at `682fc3830` | Evidence and limits |
|---|---|---|
| DA-001 | Resolved; residue merged | The typed resolved IR owns syntax; runtime name lookup → [DA-006](semantic-model-and-authority.md#da-006) |
| DA-002 | Resolved; residue merged | Modeling derivations catalogued; runtime re-mints → DA-006; store keys → DA-007 |
| [DA-003](semantic-model-and-authority.md#da-003) | Retained | Handwritten selected-source inventory and distinct lexical binding rules |
| DA-004 | Resolved | Generated artifact and owner enums |
| [DA-005](semantic-model-and-authority.md#da-005) | Retained; export claim corrected | Direct result-reader projection missing; executable bundles retain recoverable authored physical context |
| [DA-006](semantic-model-and-authority.md#da-006) | Retained | Mode target reconstruction, skipped dynamic nominal hints, name scans and identity re-mints |
| [DA-007](structure-and-invariants.md#da-007) | Retained; precision corrected | Store vocabularies and special prefix associations restated; ordinary study membership is explicit |
| DA-008 | Resolved, bounded correction retained | Typed store refusals/effect knowledge remain; no new fault injection |
| [DA-009](structure-and-invariants.md#da-009) | Retained | Kind-specific lifecycle questions independently projected; shared typed supervisor remains a strength |
| DA-010 | Resolved for authentication/PSD | Original fit permission gap is now traced under DA-038, not a reopened forgery diagnosis |
| DA-011 | Resolved, bounded correction retained | Generated catalogue and import fingerprint remain; new catalogue census not repeated |
| DA-012 | Resolved; residue merged | One run supervisor; dispatch strand → DA-009 |
| DA-013 | Resolved | One pure prediction policy |
| DA-014 | Closed by decision | Parameter-valued points; other kinds refused (I22, RC12) |
| [DA-015](boundaries-and-composition.md#da-015) | Retained | Hand host bindings/defaults persist; generated CLI and inspection requests are counterevidence |
| DA-016 | Narrowed and merged | Residue → [DA-017](boundaries-and-composition.md#da-017) |
| [DA-017](boundaries-and-composition.md#da-017) | Retained at sampled seams | KINSOL static/admitted capability split, exposed PETSc refusals and dynamics attribution wildcard |
| DA-018 | Closed by decision (I23) | No new patch-semantics evidence; decision not reopened |
| [DA-019](boundaries-and-composition.md#da-019) | Retained; fidelity wording corrected | Deployment mirrors/frozen glob remain; database destination checksums are not source-byte comparison |
| [DA-020](boundaries-and-composition.md#da-020) | Retained | Product/tooling change propagation established; a particular crate split is not selected |
| [DA-021](local-reasoning-and-assurance.md#da-021) | Retained | Runtime-carrying values and broad workflow visibility; no new exact census |
| [DA-022](execution-fit.md#da-022) | Closed by decision (I16) | No concrete new unsupported-allocation evidence; no renewed sizing campaign |
| [DA-023](local-reasoning-and-assurance.md#da-023) | Narrowed; false grounds withdrawn | Stale current mechanism text remains; proposed/Tested coexistence, transport supplement and differing path roles are legitimate |
| [DA-024](local-reasoning-and-assurance.md#da-024) | Broad diagnosis not established | Optional strict receipts, local admission and mandatory outer lineage differ; public portable composition has a role |
| [DA-025](execution-fit.md#da-025) | No active violation established | Public collector plus explicit returned-space/no-automatic-collector scope; optional operator-composition question retained |
| DA-026 | Resolved | Per-row locators removed; DataFusion pruning |
| DA-027 | Resolved | Group/re-expand cycle deleted |
| DA-028 | Resolved structurally | Multi-output guarded regions |
| DA-029 | Resolved in the compiler; residue merged | Runtime scans → DA-006 |
| [DA-030](execution-fit.md#da-030) | Closed by decision (I08–I10) | No stale reuse shown; cold-worker need remains unmeasured, not an automatically fired trigger |
| DA-031 | Resolved; residue merged | Compiler view policy; inline role admission → DA-006 |
| DA-032 | Narrowed and merged | Artifact-profile literals → DA-007 |
| [DA-033](boundaries-and-composition.md#da-033) | Retained; remedy narrowed | Concrete settlement/effect-knowledge divergence; no proof every owner needs one universal envelope |
| [DA-034](execution-fit.md#da-034) | Retained structurally | Per-relation/chunk fixed publication protocol; I30 stays historical |
| [DA-035](execution-fit.md#da-035) | Retained with scale limit | Population item loops and fixed clock; admission bound is not qualified supported scale |
| [DA-036](development-tooling.md#da-036) | Retained; headline narrowed | Finite profile values restated and conformance settings determine production allocation; existing host policy credited |
| [DA-037](development-tooling.md#da-037) | Retained; policy distinction corrected | Cyclic coupling and repeated observations; different launcher/manager/drain policies are not inherently wrong |
| [DA-038](semantic-model-and-authority.md#da-038) | Widened materially | Missed covariance publication/reuse permission chain; aggregate and composite acceptance seams retained |
| [DA-039](structure-and-invariants.md#da-039) | Retained | Constructor classification reaches lazy scientific admission and agent remedies; no failure injection |
| [DA-040](structure-and-invariants.md#da-040) | Retained | Closed boundary vocabularies, bounds and positional metric meaning restated |
| [DA-041](local-reasoning-and-assurance.md#da-041) | Retained; remedy unselected | Direct prose/bytes contradictions; archive counts inherited, no mandatory patch format/family |
| [DA-042](execution-fit.md#da-042) | Retained, bounded | Prefix-selected span export and unavailable drop observation; permitted loss remains legitimate |
| [DA-043](development-tooling.md#da-043) | Retained; cost attribution corrected | Shared process/currency/import prefix; Runtime construction is operation-specific; no current timing |
| [DA-044](execution-fit.md#da-044) | Retained; remedy narrowed | Fixed verifier clock and repeated passes; safe recovery, not mandatory self-thaw or excluded durability |
| [DA-045](execution-fit.md#da-045) | New diagnosis of unchanged code | Executable closure/export retains all selected source encodings before copying |

DA-001–DA-045 are allocated. Retired IDs are never reused; the next available identity is DA-046.
DA-038 owns the missed covariance-permission diagnosis. DA-010 stays resolved for its narrower
content-authentication/external-origin/PSD correction; this distinction prevents duplicate homes.

## Strengths to preserve

The independent discovery passes directly found governing mechanisms, not merely descriptive
schemas:

- Checked modeling functions retain physical validity, applicability claims and strict domains.
  Definitions, contextual values and temporary initialization overlays remain distinct.
- Typed resolved regions, complete preparation keys and single-flight reuse separate immutable
  preparation from changing values. No new stale-reuse evidence was found.
- Original candidate assessment gates engineering-goal assessment. Dynamics checks original
  initial coordinates and cumulative inventory/flux closure. Their existence does not settle
  every derived-result permission, as DA-038 now demonstrates.
- A shared typed run supervisor and pure study policy serve the supported operations. Adapter
  qualification/status internals keep the preceding audit's bounded evidence; they were not
  comprehensively requalified here.
- Exact immutable effect identities, guarded publication, original-owner protection, indexed
  bounded history compaction and caller/activity/connection separation remain meaningful strengths.
- Public covariance content reconciliation, explicit external provenance and PSD admission are
  real controls, although they do not supply missing scientific permission.
- Generated boundary shapes, strict version admission, protected result decoding, a derived
  capability CLI and native library use reduce hand interpretation at several boundaries.

## Scope, coverage and limits

**Re-read in the followup:** model checking and resolved preparation; representative solve,
initialization, fit, dynamics, assessment and covariance consumers; operation/lifecycle
projections; store declarations and generator/template seams; deployment, retention and physical
collection composition; history compaction and project closure/export; backup freeze/copy/restore;
resource policies and tooling imports; native MCP prerequisites/currency; telemetry declarations
and export selection; current-intent and vendored-record claims.

**Inherited or limited:** exact whole-tree counts, numerical patch semantics, broad adapter status
mapping, comprehensive compiler absence claims and the old archival upstream diff figures. The
preceding 2026-10-25 account's evidence remains available at `c02f68670`; the new source baseline
does not silently renew those measurements or detailed coverage.

**Unresolved/unexamined:** exhaustive numerical correctness and independent reference coverage of
all scientific packages; all property envelopes; server-internal CPU work; full telemetry
alert/investigation/selection and specialist bodies; remaining vendor-tree semantics; current
latency, memory, contention, scale and interruption manifestations. The newly added exchange
finding establishes a structural realization, not its dominant operational cost.

An area without a finding is not thereby aligned. The account states improvement opportunities
without authorizing repairs, selecting a universal registry/framework, changing maintainer
constraints, or starting a solution review or qualification campaign.

## Reading and ownership

| Page | Primary findings |
|---|---|
| [Integrated system account](system-architecture.md) | Consequence-oriented synthesis, scenarios, counterevidence and improvement directions |
| [Semantic model and authority](semantic-model-and-authority.md) | DA-003, DA-005, DA-006, DA-038 |
| [Structure and invariants](structure-and-invariants.md) | DA-007, DA-009, DA-039, DA-040 |
| [Boundaries and composition](boundaries-and-composition.md) | DA-015, DA-017, DA-019, DA-020, DA-033 |
| [Local reasoning and assurance](local-reasoning-and-assurance.md) | DA-021, DA-023, DA-024, DA-041 |
| [Execution fit](execution-fit.md) | DA-025, DA-034, DA-035, DA-042, DA-044, DA-045; closed DA-022/DA-030 |
| [Development tooling](development-tooling.md) | DA-036, DA-037, DA-043 |

This collection owns diagnosis. Reviews own alternative remedies, plans own scheduled work and
closure evidence, and architectural owners/ADRs own accepted design and rationale. The
[audit skill](../../../.codex/skills/design-principle-audit/SKILL.md) and
[binding](../design_principles/binding/pse-arrow.md#design-principle-audit) govern the process.
Refresh in place, preserve identities and lineage, and keep earlier exact accounts in Git.
