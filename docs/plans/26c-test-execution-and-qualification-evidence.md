---
title: "26c: Test execution and qualification evidence"
status: done
date: 2026-10-05
adrs: [ADR-0092, ADR-0122, ADR-0143]
review_sources: ["../design_review/reviews/design_review_testing-architecture_2026-10-05.md"]
scenario_sources: ["../design_review/reviews/design_review_testing-architecture_2026-10-05.md#revealing-changes"]
---

# 26c: Test execution and qualification evidence

## Purpose and foundation assessment

Run the necessary obligations with appropriate dependencies and compose their evidence once.
The [coordinator](26-testing-architecture.md) owns F03–F06 dispositions, decision routes and
overall qualification. This companion owns C1–C5 progress. Targets/checks below were
**Proposed** at authoring; source observations were **Implemented/source-inspected**,
not timing measurements. The Outcome records implemented delivery and scoped acceptance.

The current recipe-leaf deduplication, nextest/pytest selectors, failure-preserving assessment,
native binary/link identity, selected-versus-terminal reconciliation and reviewed transfer are
useful foundations. Keep them. Duplicate full feature-graph execution, package-wide resource
assumptions, native inventory calls and Python report parsing have no automatic justification.
Do not replace them with a universal test registry, bespoke scheduler or test-impact engine.

The current context snapshot and native identity are also useful, but snapshot equality across
plans/reviews/prose is unsuitable as every numerical claim's input contract. Workload-specific
prerequisites should not require static/docs/Python checks unrelated to the measurement.

Relevant authorities are blueprint §24.1/§24.3, ADR-0092/ADR-0122/ADR-0143 and the qualification
command guide. Their actual-effect, ownership and evidence changes follow coordinator P0.

## Selected contracts

### Explicit effects, fixtures and resource classes

Keep exactly one Python responsibility category per test: `unit` for local policy/codec logic
without a native attempt or inspection store; `component` for an actual owned boundary with
native/storage dependencies; `integration` for a composed workflow; `performance` for a
separately selected scale/timing claim. Keep separate parity opt-in. Correct actual native
study/event/fitting cases currently labelled unit instead of relaxing unit's meaning.

Replace session-autouse all-extension IPC with one explicit boundary test/requested fixture.
Consumers depend on the registration they actually require. Do not retain a shared fixture
whose primary purpose is repeatedly asserting every extension round trip. Production import
registration remains production-owned; the dedicated IPC test independently checks it.

Runtime, cancellation, writable store/database and attempt state are independently owned per
test/journey. Immutable input/preparation may be shared within a justified fixture scope.
Nextest isolates processes per test, so a static singleton cannot share preparation between
separate tests. Where setup is substantial and several assertions concern one journey, use
one constructed journey with distinct assertions rather than a process-spanning cache.

Move Rust resource-sensitive controls into identifiable owner-local namespaces for native
attempts, store operations and engineering journeys; use nextest filtersets/groups against
those responsibilities. Remove blanket grouping of entire conformance/engine/lifecycle/backend
packages when their pure tests need no such budget. Keep explicit thread requirements,
timeouts, store connection limits and the native memory-capped recipe boundary for actual
heavy operations. No second resource classification registry is added.

Do not make unit selection publish an inspection fixture. Linked component/integration setup
creates its publication once before workers; only explicit fixture consumers depend on it.
An unrelated pure test can run if publication setup fails. Mutable fault state cannot leak
between attempts. Shared immutable fixture scope remains process-local under pytest workers.

### Owned effects replace checkout attribution

Delete `VerifyCleanup`, per-test Git polls and registration hooks whose purpose is checkout
attribution. Replace that policy with explicit output roots, temporary directories and unique
test stores/databases. Production/tool operations that consume an output root receive the
owned destination explicitly. Tests release their owned resources; they never clean unrelated
checkout changes or reset Git state.

For a tool with a prohibited-write guarantee, exercise it in a disposable repository/input
fixture and compare actual file bytes/modes across that owned boundary. Include an already
dirty input; its unchanged porcelain category must not hide a byte change. Use existing guard
tests for enforcement behavior where appropriate. Do not introduce per-test whole-checkout
snapshots, a sandbox framework or another automatic causal attribution mechanism.

Any optional operator checkout observation records context only. An authorized concurrent
agent editing an unrelated file cannot fail a product test. Marker/parity validation and
selected identity capture remain; removing Git attribution does not delete those contracts.

### Common obligations once; distinct modes explicitly

Select the linked, force-validated workspace as the canonical common Rust execution graph for
comprehensive local qualification. Remove the second full default-workspace execution and
the separate governance-test invocation already covered by the workspace. Preserve `just test`
as an explicit developer command; calling it independently is not another mandatory campaign.

Put genuine feature-absence behavior into a focused `feature_absence` owner-local namespace
and run that selection in the default graph. These controls assert actual unavailable/refused
capabilities, not duplicate common mathematical/codec outcomes. Native controls stay in the
linked graph. If a consequential mode difference is discovered, give it a focused control;
equal names across graphs do not prove behavioral equivalence. No full powerset is inferred
from these selections; supported powerset checks remain explicitly selectable.

Run linked Python unit/component/integration selection once in comprehensive scope. Keep
pure targeted `py-unit` use and distinct reference/performance opt-ins. Retire overlapping
assessment Python selections/recipes that merely repeat this comprehensive execution; retain
explicit smaller selectors where they serve local work. Default unit selection has no
inspection-publication setup.

Continue letting recipes own tool paths, feature strings, profiles and native environment;
framework filtersets/markers own selected cases. The scope declaration composes invocations,
not exact test names or a second definition of validity. Keep recursive leaf deduplication.
Static/manual aggregate overlaps also execute each required leaf once in the final campaign:
governance test coverage comes from the native workspace, while remaining freshness/family
obligations consume their own actual checks. Do not silently equate different generators or
mode-dependent checks to suppress a required operation.

### One inventory and terminal composition

Within assessment, `validation.py` owns terminal parsing, selection reconciliation and check
status. The native Rust wrapper owns its one explicit nextest JSON listing, uses it for actual
binary/link provenance and writes the raw selection artifact alongside provenance. Assessment
consumes that artifact; it does not make an earlier listing call. The wrapper checks listing
failure before execution. Default feature-absence invocation uses the existing assessment
listing path once. Nextest's intrinsic collection is not misrepresented as removable work.

Python persists selected node identities at collection in the running pytest invocation.
Its raw JUnit is parsed/reconciled once by assessment. The native Python wrapper performs no
additional parsing when assessment owns the invocation. For standalone native Python use,
the wrapper calls the same terminal-composition operation after execution, enforcing its
current empty/skip/malformed-report refusal. Make ownership explicit in invocation options;
do not maintain two implementations or accept contradictory status from wrapper versus parent.

Shared terminal composition consumes the invocation identity/start, raw report, expected
selection and actual provenance. It retains raw artifacts and produces the existing factual
check record. Missing, duplicate, unexpected, skipped, failed, truncated and unexecuted
selected cases remain visible; nonzero process exit cannot be overwritten by a report.
Collection failure, timeout/interruption and setup failure retain their truthful states and
only block actual dependents. Retry/flaky policy does not launder a failed qualification.

### Claim inputs and contextual snapshot

Keep one contextual source inventory before/after an assessment for provenance, including
Git revision/patch information. Reuse that hashed map for scope projections; do not rescan
or rehash the tree separately for every claim. Scope projections preserve additions,
deletions, modes and symlink identities, not only contents of currently existing files.

Extend the existing `Gate` declaration with an explicit input-scope identity. The selected
scope families are:

| Scope | Relevant inputs |
|---|---|
| Rust product | Rust manifests/lock/toolchain, `.cargo`, all workspace crate/test/vendor source and package data, native/build configuration and actually used runner/recipe files |
| Python product | Rust product for the compiled native boundary, Python source/contracts/tests, Python manifest/lock and pytest/extension setup |
| Tooling | Tool source/tests, invoked recipes/configuration and the fixture/policy inputs the named tool actually reads |
| Documentation | Publishing/checker source/configuration and consumed document collections; generated docs also consume their declared generation inputs |
| Generation | Authoritative declarations/generator source, target configuration and emitted boundary; applicable native generation inputs remain explicit |

These are conservative recipe/subsystem scopes, not inferred per-test dependency graphs.
Tooling or publication that reads product/document data includes those inputs explicitly.
Unknown scope defaults to the whole captured input set and forbids narrower automatic reuse;
absence of a declaration is not evidence of irrelevance. Ordinary runtime claims exclude
unconsumed plans/reviews/prose. Executable policy/data expressed in documentation remains a
claim input where the operation actually consumes it.

Each record retains its canonical invocation/selection, feature/profile/tool identity,
input-scope version, scoped path hashes, actual relevant environment and native identity where
required. Scope changes invalidate automatic reuse. Equality of a digest does not establish
scientific applicability. Contextual changes can be recorded without invalidating an unrelated
claim. A relevant input changing during execution prevents its qualification; unrelated
concurrent prose changes do not. Preserve the contextual change record without blaming a test.

Unchanged-input reuse requires equivalent recorded invocation, scope definition, relevant
inputs/environment and applicable native identity. Reviewed transfer remains a separately
labelled explicit act with origin/reason/changes; it is never automatic changed-code reuse.
Interrupted/not-run/failed evidence is not transferable as a pass. Missing relevant identity
or untrustworthy input coverage requires fresh execution or explicit reviewed disposition.

Replace the current version-4-only report assumptions with one new current representation;
do not add old report readers, repair old receipts or reinterpret their provenance. Current
receipts keep executed/reuse/transfer/not-run distinctions and original observation links.

### Selected measurement prerequisites

Replace `case_measure.require_functional`'s equality against `comprehensive()` and global
source seal. Measurement declarations own functional prerequisite scopes for their workload
groups, referencing existing recipe/mode selections rather than exact-name test manifests:

- Process execution, fitting, study and publication cases require the native runtime and
  conformance boundary selections for their production operations.
- Preparation cases require native runtime/compiler preparation and the applicable provider
  composition controls; admission cases require authored loading/admission controls.
- Resource-pressure, cancellation and durable cases include their actual corresponding
  lifecycle/enforcement prerequisites, rather than inheriting a pure preparation pass.

Represent these as small named functional-scope selections at the existing scope/workload
owners. A selection specifies recipe, graph/profile and framework filter; workload entries
reference that identity. Their definition and relevant model/data/limits are input-keyed.
Choose the targeted namespace controls delivered by 26a/26b/C1, retaining broader conservative
package selection only where narrower coverage is not yet trustworthy. Do not invent a
second registry of required individual tests or infer equivalence from test-name substrings.

The functional invocation must be complete and satisfy its independently specified behavioral
controls; a smoke build or benchmark readiness pass is not functional qualification. Accept
either the exact declared functional invocation identity, or the explicitly declared covering
full-workspace native invocation with complete terminal evidence in the matching graph. That
covering case includes all normally selected prerequisite controls; requirements for ignored or
otherwise separately selected controls remain explicit. No arbitrary filter-subset equivalence
is inferred from names, and no new filter parser is written. If coverage cannot be established,
require the selected functional invocation instead of accepting an unrelated broad pass.

Before measuring, validate only selected prerequisite claims and relevant product/model/policy/
lock/native-provider inputs. Measurement code, declared cases and parameters also enter the
measurement's own identity; changing them does not automatically erase an unaffected product
observation. Benchmark binaries and functional test binaries are different artifacts: retain
each identity and require compatible source/feature/profile/native-provider conditions for
the consumed claim, not byte equality between unrelated executables.

Keep fresh-process sampling, Criterion-owned statistics and actual native provenance. Keep
smoke explicitly untimed and separate from measurement, unknown-case refusal, build diagnostics,
resource refusals and report limits. No sampler or statistical engine is rewritten.

## Packages

| Package | Required capability and delivery | Deletion and targeted acceptance | Status |
|---|---|---|---|
| <a id="c1"></a>C1 Fixtures and actual resource classes | Existing production operations and category contract; explicit IPC/publication dependencies, correct Python tiers and Rust namespaces/groups | Remove unrelated autouse/package-wide setup. Pure tests run without native/publication; real native/store work remains capped | implemented; assembled acceptance complete |
| C2 Owned effects | P0 owned-effect policy; explicit outputs and disposable tool fixtures | Delete Git attribution/plugin helpers and obsolete tests. Dirty-input writes are caught at owned boundary; concurrent unrelated edits are permitted | implemented; assembled acceptance complete |
| C3 Selection and terminal ownership | C1 categories; B4 freshness replacement where its recipes retire | Remove full-graph/common repetition, duplicate native enumeration and nested report parsing; preserve complete failures and mode distinctions | implemented; assembled acceptance complete |
| <a id="c4"></a>C4 Relevant-input evidence | C3 current truthful invocation records; scoped input/environment declarations | Delete global reuse equality and old-version receipt coupling. Relevant changes invalidate; unrelated prose does not; transfer stays explicit | implemented; assembled acceptance complete |
| C5 Measurement readiness | C4 scoped claims and actual workload prerequisites | Delete mandatory comprehensive prerequisite. Selected functional claims admit only matching workloads; smoke never supplies qualification | implemented; assembled acceptance complete |

Editing ownership includes `scripts/validation_scope.py`, `validation.py`,
`validation_receipts.py`, `native_tests.py`, `case_measure.py`, root/Python conftests,
nextest/pytest configuration, affected test namespaces/markers, workload declarations and
`justfile`. One assigned writer owns shared recipe/configuration changes and integrates
26b freshness/crate retirement. Do not remove useful runner-negative tests merely because
their implementation moves to a shared operation.

## Verification

The checks below were **Proposed** at authoring; executed evidence is recorded in the Outcome. Tooling changes use the focused stdlib runner controls through
`just unit-consolidation-tools` and relevant setup/guard fixtures. Extend existing controls for
exact argument/filter preservation, nonzero exit, stale/malformed/empty/truncated reports,
collection/interruption, failed setup blocking only dependents and origin-preserving transfer.

Add discriminating controls for one explicit native listing, one nested Python report parse,
truthful feature-absence versus linked scope, and a pure fixture selection without publication
or native dispatch. Verify category enforcement still rejects zero/multiple categories.
Dedicated owned-effects tests exercise dirty-file byte changes and unrelated concurrent edits.

Input/reuse tests cover relevant additions/deletions/modes/symlinks, changing locks/native
libraries/scope definitions, unrelated prose and unrelated environment, and relevant inputs
changing during execution. Prerequisite tests reject incomplete, wrong-workload, wrong-mode,
changed-model/provider evidence while accepting an unchanged selected claim after a prose edit.
Keep measurement case-selection/build-error/CSV-unit controls; no real timing run is required
for a parser or qualification-contract change.

After functional integration, coordinator Q1 runs the actual non-overlapping Rust/native/Python
selections and static/manual leaves once, under the new composition. Standalone recipes remain
available on demand. Record actual command/mode/scope and zero failure baseline; no current
campaign, speed improvement or scientific qualification is claimed by these documents.

## Outcome

### What was built

**Implemented**, 2026-10-05: explicit fixtures own publication, IPC extension inputs, mutable
runtime facades and isolated operational databases. Git-based per-test attribution is deleted.
The common linked Rust graph runs once, with separate default-feature absence controls;
nextest/pytest own actual selection and terminal results. The thin composer retains one native
inventory, terminal reconciliation, mandatory native identity and origin-preserving transfer.
Version-5 evidence uses relevant input/environment scopes; no historical receipt adapter exists.
Native Python provenance resolves the actually imported checkout extension, hashes only that
binary and its linked closure, and verifies the same loaded origin inside pytest before
collection. Database support controls inherit the existing `store_tests` scheduling namespace.
Measurement workloads name functional prerequisites and accept exact matching invocations or
the declared complete linked covering invocation, without inferring arbitrary filter equivalence.

**Tested**, zero failure baseline: `just setup-test-report build/plan26-q1-tools-final` passes
129/129 controls, with no failures or skips. After annotation/lint repairs, seven affected
terminal/provenance/transfer/measurement controls pass again. `just lint-py` reports zero
findings. The separate default-feature absence selection passes 2/2. Coordinator Q1 owns
linked native/Python and final static/manual evidence. These tests cover malformed/stale
reports, interruption and collection, actual categories/effects, relevant source drift,
prose-insensitive reuse, explicit transfer and prerequisite refusal; no timing experiment ran.
Assembled acceptance is complete. The conformance review's two findings are resolved at this
companion's execution owners; its independent remedy follow-up accepts the corrected scope.
Plan 26's Outcome owns the additional 52/52 tool controls, focused database pass, fresh
33/33 linked Python selection and final manual results.

### A mistake made and corrected

Review found that declared tooling/documentation inputs exceeded the snapshot inventory,
standalone Python provenance was optional, and reused transfers could lose their original
reason. Input capture now covers tracked agent configuration and all declared documentation
roots, uses one relevant-environment declaration, always records actual Python native provenance,
and preserves the original transfer observation and rationale. Negative controls exercise each
correction. Fixture lifetime also follows the strong process deployment owner: immutable
settings are shared within the session, while facades and database effects are individually owned.
Final conformance review additionally exposed a database control outside the declared store
schedule and provenance of on-disk extension candidates instead of the consumed import.
The existing namespace and wrapper/session boundaries now enforce those guarantees; hostile
import controls distinguish foreign imports, stale candidates and execution-time divergence.

### Deviations from the plan, deliberate

The production deployment resource service remains strongly process-owned. Replacing it with
a weak registry would permit escaped buffers from an old budget to overlap a new pool and
would mismatch the process executor lifetime. No statistical engine, benchmark sampler or
automatic test-impact inference was added; performance claims remain unmeasured.
