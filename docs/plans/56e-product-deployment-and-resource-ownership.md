---
title: "Plan 56e: Product, deployment and resource ownership"
status: draft
date: 2026-10-25
adrs: [ADR-0229]
review_sources: [docs/design_review/audit/README.md, docs/design_review/audit/system-architecture.md]
scenario_sources: [docs/design_review/audit/system-architecture.md]
doc_role: plan
doc_owner: docs/plans/56-design-principle-alignment.md
doc_topics: [design, environment, storage, libraries]
---

# Plan 56e: Product, deployment and resource ownership

## Purpose and current foundation

**Proposed:** dependencies follow product/development responsibility; deployment/resource meaning
has an owned contract; process observations are shared without erasing distinct policies; supplier
records truthfully describe retained edits. This companion develops DA-019/020/036/037/041.
[Plan 56](56-design-principle-alignment.md) owns decisions, scope and status.

The current supervisor, original admission and report custody are real foundations. Intervening
service telemetry selection/settlement and cross-checkout custody corrections through `56d4bf158`
remain preserved; they do not remove mirrored deployment documents, conformance-owned production
settings, catalogue-induced product dependencies or repeated process observations. DA-024 remains
a corrected optional-producer scope account, not authority to delete portable APIs or outer lineage.

## Target and alternatives

### Separate declaration/publication by responsibility

Under proposed RC02/RC03, scientific registry vocabularies stay canonical, while owner-local
boundary document vocabularies and development relation/catalogue contributions live with their
responsibility. Reuse `RegistryBuilder::{declare_relation,declare_enum,declare_identity,build}`
and existing document generation; compose product/development catalogues only in the generation
entry point. A schema publication build must not enable development analytics in normal product
consumers just to see their Rust types.

Use the existing xtask evidence composition for development evidence/knowledge storage, queries
and document contributions. Remove those family implementations from unconditional product
`pse-operations` exports, product/foundation generated types and ordinary runtime/Py dependencies.
Extend the existing generator's output-family selection instead of copying a generator or adding
a new crate. Runtime's catalogue remains product-owned; xtask can aggregate independent compiled
contributions during generation, then publish separate product and private/tooling outputs.

Product observability, owner metrics, checkpoints, finite capture, scientific inspection and
shared IPC admission/copy operations remain product capabilities. Move common scientific IPC/
contained-copy primitives beneath both project portability and diagnostic review consumers, with
product names and narrow interfaces. Development review/corpus behavior must not be an input to
product import or a root semantic enum merely because a generator can reach it.

A whole-module `tooling` extraction is unselected: it would move product behavior for a misleading
name. A new evidence crate/service is also unselected absent a separate distribution responsibility.
A feature that every normal consumer enables is not a partition. Check the actual supported Cargo
closure and generated publication roots, including workspace feature unification.

### Product deployment and operator composition

Rust serde documents own deployment/profile/context/receiver/readiness/observation meaning and
protocol revision. Generate bootstrap Python/schema projections with F01; use one Rust relational
validator for materialized allocation. Python owns environment/systemd realization and uses the
schema plus bounded product-operator validation before publishing configuration. Do not re-create
relational rules independently in Python, native deployment and several store readers.

RC04 proposes moving actual production worker/project composition to binary targets in the existing
`pse-runtime` package, without a new package. Split the mixed recovery helper: product offline
payload operations consume C04; fixture seeding/signal/failure controls remain in xtask. The
I24 exploratory `pse-native-lane` stays in tooling. Replace fixture-named feature dependencies for
ordinary product workers with explicit product target requirements.

Receiver generations derive the owned Python import closure from selected product entry points,
including regular-package parents and generated telemetry bindings, plus required configs and
executable/provider identities. Validate that closure in an isolated frozen import probe before
publication; unknown dynamic owned imports require explicit declaration or refusal. An unrelated
development script must not change receiver identity. Keep immutable generations, original locks,
context association and explicit drain/readmission; do not reinterpret old live generations.

### Production profile authority and owner observations

Under RC05, separately typed production execution-profile sections in the existing
`.config/agent-capacity.toml` supply selected worker/pool/lane/headroom values; the product Rust
profile contract validates/materializes them, and conformance selects a profile instead of defining
it. Host admission policy stays in that file with existing precedence. Keep host envelope,
controller/observer, receiver and per-worker scopes explicit; different legitimate totals are not
forced equal merely because they are all memory values. Materialize a typed admitted allocation once;
supervisor, runtime, worker and host defaults consume it instead of repeating numeric rules.
Preserve every selected profile's values, including the reference sixteen lanes, distinct
functional/wide fixture lanes, and the physical/logical CPU and uncapped ordinary build behavior. Remove the 48/32-GiB setup coupling and repeated finite headroom/store literals only
when generated/materialized profile consumers replace them. No smaller host partition is introduced.

A small leaf observation module supplies authenticated boot/start/namespace/invocation/executable/
cgroup facts and explicit uncertainty. Launcher liveness, unresolved manager authority and actual
descendant drain remain different owner policies. Native file custody keeps C04's Rust authority;
matching cross-language documents do not imply one universal `alive()` predicate.

Expose service borrowers through a narrow existing admission/resource authority interface,
independent of report/fixture custody. Test/report owners register through it; the supervisor no
longer imports their internals to discover drain obligations. Preserve current cross-checkout
custody, exact transitions and recovery without a second ledger or resetting reports. Compose
admission and supervision at entry points, breaking reverse imports and the runtime cycle.

### Truthful recoverable supplier edits

Keep exact upstream origin/version/revision/archive digest with a reproducible complete delta
against the tracked edited tree. Select truthful edited-tree records plus reproducible diff,
using the existing patch-series form where it is already the owner; do not mandate one patch format
for every supplier. Reconcile SUNDIALS/SDK/engine-API/server records rather than append a new
contradictory inventory. Separate substantive and whitespace changes, include added/deleted files,
and record the real upstream-defect workaround through R-41 where applicable. The numerical IDAS
correction is already declared/tested historically; I23's retained numerical patches are not
reopened. No OTel family/version split or numerical defect is inferred solely from missing prose.

## Packages

### E01 — Responsibility-owned declarations and product dependency partition

Implement the accepted RC02/RC03 catalogue/registry split with F01's enum/constraint lowering.
Migrate evidence/knowledge implementations, schemas, private Python outputs, CLI consumers and
generation aggregation together. Move the shared product IPC/copy primitives and product
observability out of development-review ownership. Delete forced normal `evidence-query`
dependencies, foundation tooling vocabulary and unconsumed product-only projections once callers
use their correct family. Preserve actual product diagnostics.

### E02 — Deployment contract, product roots and frozen closure

Generate deployment documents and one validator, move selected product targets under RC04, split
recovery/fixture responsibilities and make Rust/Python readers consume the same protocol. Derive
and validate the receiver closure including generated imports, then migrate launches/observations,
configs, entry recipes and tests together. Retire mirrors, stdout-string interpretations and broad
`scripts/*.py` generation identity after all readers/launchers migrate. C04 supplies file authority.

### E03 — Production profile and materialized allocation

Under RC05, put production profile selection upstream of conformance; feed host admission,
supervisor, native validation, worker and MCP defaults from its admitted document. Remove repeated
finite literals and stale error-text constraints. Explicit operator overrides retain precedence;
invalid relational combinations refuse through the same validator before setup effects.

### E04 — Shared observations and acyclic borrower layering

Extract leaf observations, make owner policy inputs explicit, and provide the service borrower
interface through existing authority. Migrate supervisor, admission, resource/test custody,
capability specialists and service owners in coherent slices. Keep PID reuse/uncertainty protective
and drains authoritative. Delete reciprocal imports and obsolete duplicated observation fragments;
do not replace them with a service locator or generic ledger framework. Finalize E02 closure only
when its now-bounded dependencies are consumed.

### E05 — Supplier record reconciliation

Reconstruct exact supplier deltas from pinned origin and tracked edits, correct false provenance
claims and add the real R-41 entry where required. Verify full reconstruction including added/
deleted/whitespace bytes, then schedule affected native controls for any actual supplier change.
No supplier version bump, patch rewrite or permanent proof/check apparatus is selected merely to
repair truthful records. Archive diffs are useful evidence, not a new per-command ledger.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| E01 | implemented; targeted controls passed, assembled qualification pending | 56e | Normal product closure excludes development-only families/queries; all catalogue consumers use their owner |
| E02 | product protocol/targets/frozen launch implemented; assembled journeys pending | 56e | One deployment protocol/validator and working product targets/frozen receiver closure replace mirrors |
| E03 | production declaration/consumers implemented; targeted role controls passed, wider journeys pending | 56e | Production profile drives conformance and all allocation readers without repeated finite rules |
| E04 | service/observation slice implemented; E03 profile edge remains | 56e | Leaf observations and borrower interface remove cyclic owner knowledge while preserving distinct policies |
| E05 | implemented; exact source reconstruction tested | 56e | Truthful supplier origin/delta/workaround record reconstructs actual bytes without a mandated format |

E01 and F01 share one generator/output-family decision, with root ownership of the declaration
surface. E03 and E02 consume its emitted bootstrap documents. E04's leaf/borrower slice enables
E02's bounded closure; E02 protocol work need not wait for every observation migration. E05 is
largely independent. No whole-document cycle or new crate is needed.

## Acceptance

**Planned:** a development corpus/enum change does not modify scientific foundation contracts or
introduce analytics into normal operations closure; generated schemas and intended host imports
remain complete. Receiver closure stays unchanged after an unrelated script edit and includes a
new required generated import. Launch/restart/refusal controls bind exact receiver/context/image.

Change a profile input and verify materialization, setup and runtime admission agree without
secondary literal edits; preserve selected sixteen lanes/host mode. Exercise PID reuse, uncertain
owners, launcher death with surviving descendants, repeated drain and report-custody conversion.
Probe product recovery without fixture imports and keep exploratory/fixture roots available under
their existing scopes. Supplier reconstruction compares complete trees; any numerical behavior
change separately requires its native/scientific controls. These are future checks, not authoring
qualification or a measured rebuild-time claim.

## Selected 56c prerequisites — 2026-10-26

**Implemented/Tested:** [56c Outcome](56c-canonical-contracts-and-effects.md#verification-and-outcome--2026-10-26) owns the selected product recovery binary in existing `pse-operations`, separate xtask fixture controls, native offline file authority and closed current-artifact readmission. This delivers only E02's helper-root prerequisite. Worker/project root relocation, the complete deployment protocol/validator and receiver closure remain E02 work; RC04 is selected only for the helper. RC02/RC03 catalogue separation and RC05 production profile authority remain pending, with existing numeric capacities preserved.

## Execution checkpoint — 2026-10-26

All packages in this companion are authorized for joint 56d/56e execution. The coordinator's
[joint checkpoint](56-design-principle-alignment.md#joint-56d56e-execution-checkpoint--2026-10-26)
records selected rule changes, dependency order, preservation and qualification boundaries.
Contract publication and independent E04/E05 work have begun; package completion remains open.

## E05 supplier reconstruction — 2026-10-26

**Implemented:** reconciled SDK/engine API 3.3.4 gRPC origin, SUNDIALS numerical edits and the
existing server patch series. Complete archive deltas in `vendor/deltas/` retain substantive,
whitespace, added/deleted path and executable-mode meaning. Existing supplier implementation
bytes and patches were preserved; the source record and R-41 now identify the retained IDAS
quadrature-history workaround. Producer review hashes remain explicitly historical evidence.

**Tested:** the complete reconstruction block in `vendor/deltas/readme.md`, executed through
`scripts/pse-env -- .venv/bin/python`, verified archive SHA256, packaged revisions and exact
tracked path/byte/executable-mode equality for all seven edited suppliers: zero failures against
the zero baseline. The twelve existing server patches applied with `git apply --index` to pinned
revision `1ac2b1dba7cd177621a4cf5d507caebeb2414395`; resulting tree
`120161c0f4138d7036961888ca60186dda195218` (65 changed paths). The pinned DiskANN archive
digest and existing patch also verified. These controls establish source reconstruction, not
numerical, producer-context or product requalification. No native implementation changed.

## E04 service and observation checkpoint — 2026-10-26

Commit `d7d0169af` separates authenticated process observations, operation lifetime, metadata and
systemd readback into leaf owners. Existing resource authority supplies service-facing borrower
obligations. Admission consumes explicit service policy; supervision no longer imports test/report
custody, and the native-operation/environment reverse edge is removed. Missing service policy
retains service-affiliated rows conservatively. Cross-checkout custody and actual descendant drain
remain distinct from launcher liveness. E03 still owns the `reference_execution` reverse edge;
setup-specific native-cache/build projections were not declared universally acyclic.

**Tested:** targeted `scripts/pse-env -- .venv/bin/python -m unittest` selections passed 95
host/conversion/borrower/custody/zombie controls, 87 observation/telemetry/environment/manager/drain
controls, 122 adjacent consumer controls and 71 non-real native-operation controls, against the
zero failure target. All twelve real native lifecycle controls passed across split runs; one
production-observer local-name collision failed first and its exact corrected rerun passed. The
managed-study fixture script (`scripts/pse-env -- .venv/bin/python
python/pse/tests/test_managed_study_fixture.py`) passed thirteen tests. Original supervisor
286-test run had ten errors across nine functions; the corrected exact nine-function rerun passed.
An earlier broad native attempt was interrupted with exit 130. These are scoped controls, not a
clean monolithic supervisor/native result or product qualification. Detailed logs remain at
`/tmp/e04-observations.log`, `/tmp/e04-consumers.log`, `/tmp/e04-native-unit.log`,
`/tmp/e04-native-real.log`, `/tmp/e04-native-remaining.log`,
`/tmp/e04-supervisor-rerun.log` and `/tmp/e04-managed-fixture.log`.

E02/E03 now consume the owned leaf/import boundary and retain complete frozen-closure, shared
validation, profile propagation and end-to-end receiver obligations.


## E01–E03 product ownership checkpoint — 2026-10-26

**Implemented:** runtime relations and their generated outputs exclude the separately owned private
evidence/knowledge catalogue and query backend. Product allocation materialization/validation,
worker execution and project commands use product targets in existing crates; the replaced xtask
worker/project targets, modules and dispatch have been deleted. Ordinary canonical initialization
uses the lower product initializer's versioned description/initialization protocol. The private
bootstrap family projects twelve actual Rust roots, including local enums and the native recovery
checkpoint; no full runtime configuration schema is redeclared in Python.

The production section of `.config/agent-capacity.toml` owns the unchanged six execution roles,
the distinct default 48/16/32 GiB service role, and the historical capability defaults. Host
admission, conformance-facing requests and supervisor consumers select that owner through the one
Rust allocation validator. Host command placement remains usable before the product validator is
built. The host-to-supervisor reference-policy reverse import and Python finite relational rules
are removed. Module build placement now uses a recipe default, preserving explicit caller choices
and the workstation's selected ordinary unbounded mode.

Frozen receiver publication derives its owned Python import closure, retains generated models,
configuration and the selected helper, and probes copied sources in an isolated interpreter. The
v2 manifest binds interpreter/provider inputs; v1 generations remain available for observation and
drain but require explicit readmission for current startup. Pure producer artifact observations
have a leaf owner, so the receiver closure no longer imports test/report or producer capture
orchestration. Source-side setup/readmission requires selected prepared inputs before publication.
A compiled Rust-owned runtime-input descriptor retains selected nonsecret values and original
managed-provider receipt hashes. Frozen primary launch selects its helper beside the admitted
supervisor script and uses the runtime-only branch: the existing cache owner verifies and pins
actual managed generations without compiler discovery, Cargo or provider preparation. Secrets are
inherited at invocation and thread budgets remain with the admitted allocation. Missing or changed
preparation refuses protectively; no development producer verifier was relocated. The conservative
AST closure still includes setup definitions; the runtime path does not execute them.

**Tested:** focused supervised Python controls passed 63 producer/provider/resource/closure/progress
cases and 15 migrated composer/custody cases. The current publication/image/progress/recipe
selection passed 29 cases; the actual product profile, ordinary backup-resume and reference
initializer selection passed 17. A new recipe initially lacked catalogue classification; the
classification was repaired and its affected controls passed. The first reference selection
exposed the new preceding operator-build failure result; the test now exercises both operator and
initializer build refusal. The combined Rust product-owner selection passed 48 of 51 initially;
three new checkpoint fixtures correctly failed private-directory admission, were corrected to
explicit 0700 fixtures, and their exact rerun passed all three. The composite therefore covers all
51 selected controls, while retaining the original failures. All counts use the zero failure
target; these are focused controls, not assembled/native/scientific qualification. Current scoped
logs include `/tmp/e02d05-product-protocol-tests.log`, `/tmp/e02d05-publication-tests.log` and
`/tmp/pse-56de-product-ownership-units.log` with its exact checkpoint rerun.

The prepared-input/isolated-closure/product-protocol selection additionally passed 28 supervised
Python controls against the zero failure target, including actual managed-generation corruption
refusal and guards against entering preparation. This exercises the component seam, not a complete
native worker launch.

Next acceptance is the actual prepared frozen receiver launch without provisioning, remaining
host/supervisor propagation and lifecycle controls, and the coordinator's selected native canonical
and product journeys. Existing producer reviews/receipts remain historical; source relocation and
these targeted checks do not requalify an assembled producer deployment or close E01–E03.
