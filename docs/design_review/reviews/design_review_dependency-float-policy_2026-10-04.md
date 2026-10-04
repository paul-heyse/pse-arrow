# Dependency float policy governance review — 2026-10-04

## 1. Scope, drivers and coverage

| Field | Assessment |
|---|---|
| Subject | Proposed [ADR-0159](../../adr/0159-dependencies-float-exact-pins-need-a-reason.md) and its implementation in commits `92110fe97` (ADR, ADR-0122 supersession, blueprint §3.1/§3.3.2, revision 120), `5262cf987` (Rust carets, `[workspace.metadata.pse.pins]`, inverted `dependency_pins`, `dependency_floors` message), `febd9710d` (Python `>=` floors, `uv.lock`) and `edb9411dd` (`just upgrade`, permissions, policy text, DP-14/DP-15 wording) |
| Standard | Core/template 3.3; process-simulator 1.4 (PS-13 only); `binding/pse-arrow.md` |
| Tier / purpose | Design / target, bounded to dependency governance (AGENTS.md: governance change, ADR plus review) |
| Reviewer | Independent delegated design reviewer, 2026-10-04 |
| Baseline | `main` at `edb9411dd` (four commits ahead of `origin/main` `c0221f8b6`, unpushed). Another session's uncommitted work under `crates/`, `docs/plans/` and `python/pse/tests/` is excluded |
| Decision | Behavioral/semantic adequacy: satisfied, with F04 required. Architectural fitness: satisfied. Overall: **Accept-scoped**: F01, F04 and the ADR-text corrections in F03 land in the acceptance change; F02 should land with it; F05–F07 follow |
| Disposition owner | The ADR-0159 acceptance change (coordinator) for F01, F03 (ADR text), F04; a follow-up tooling/`design:` change for F02, F03 (test), F05–F07. No active plan is claimed |

**Not under review.** The operator's 2026-10-04 policy: agents add libraries freely; dependencies
float under carets or `>=` floors; committed lockfiles record versions and `--locked` gates stay;
upgrades are at the agent's discretion through `just upgrade`; an exact pin or git revision needs an
overt, dependency-specific reason recorded beside it; families that must resolve to one version stay
pinned. This review asks whether ADR-0159 and its implementation realise that policy soundly.

**Functional target.** Version constraints in the manifests express only what must hold. Resolution
lives in the lockfiles, and every exceptional constraint states why it exists. The guarantees the
exact pins used to stand beside stay intact: one type universe, reproducible runs, byte-stable
generated output, stable reference oracles and complete identity keys.

**Inspected.** ADR-0159, ADR-0122, ADR-0066 (Scope, Compensating controls, Status history),
`scripts/adr.py` (supersede, lint), blueprint §3.1, §3.2 and §3.3.2, `docs/dev/dependency-policy.md`,
AGENTS.md, CLAUDE.md, GOVERNANCE.md §5, root `Cargo.toml` (`[workspace.dependencies]`,
`families`, `pins`), `pyproject.toml`, `tests/governance/tests/dependency_pins.rs` and
`dependency_floors.rs`, `xtask` `family_check`/`check_family`/`compare_evidence`, the `upgrade`
recipe, `.config/after-turn.toml`, `crates/pse-workspace-hack/Cargo.toml`, `crates/pse-buildinfo`
(`build.rs`, `identity.rs`, `lib.rs`) and its consumers in `pse-runtime` (`workflow/completion.rs`,
`workflow/publication.rs`), `crates/pse-ids/tests/golden_vectors.rs`, `xtask` codegen
(`codegen/queries.rs`, `codegen/schemas.rs`, `feos_reference.rs`),
`crates/pse-operations-queries/Cargo.toml`, `scripts/vendor-delta.py`,
`tests/conformance/Cargo.toml`, `packages/reference/data/oracles/`, the capability-map evidence
lockfiles, `.github/dependabot.yml`, `.github/ISSUE_TEMPLATE/library-upgrade.yml`, the
`edb9411dd` diff of core DP-14/DP-15 and the salsa 0.28.4 and feos/feos-core/quantity registry
manifests.

**Executed probes (2026-10-04, crates.io and PyPI index state of that date).**
`cargo metadata --locked` passed. `uv lock --check` passed (uv 0.12.22).
`cargo update --dry-run [--verbose]` would move 57 packages and no family member; 44 stay
behind latest. `uv lock --upgrade --dry-run` would move 18 packages, among them msgspec
0.21.1 → 0.22.0 and ruff. The lock-to-evidence comparison script found no family package
differing between `Cargo.lock` and either evidence lockfile; the support lock differs on 14
shared non-family packages, among them salsa 0.28.2 against 0.28.4.

**Not run.** `just governance`, `family-check`, `codegen-*-check`, `dependency_pins` itself (the
implementer reports `just governance` passed; secondhand). No build, test suite or regeneration.

## 2. Decomposition, ownership and authority

| Responsibility | Owner | Notes |
|---|---|---|
| Version constraints | `[workspace.dependencies]`; `pyproject.toml` | Carets and floors by default; members inherit (only the generated workspace-hack declares versions directly, at major granularity) |
| Resolved versions | `Cargo.lock`, `uv.lock` under `--locked`/`--frozen` | Unchanged mechanism; the authority a version is read from |
| Moving resolutions | `just upgrade [package …]` | `uv lock --upgrade` + `cargo update`, or per package |
| Why a constraint is exceptional | `[workspace.metadata.pse.pins]`; comments in `pyproject.toml` | Rust reasons are machine-read; Python reasons are prose |
| Family membership and version | `[workspace.metadata.pse.families]` | Member `=` strings repeat the version; `family-check` reconciles them with the graph |
| One type universe | `xtask family_check` over `cargo metadata --locked` | Every resolved package matching a family glob must share one version equal to the declared one |
| Reason enforcement | `dependency_pins` | Exact `=` comparator or full-rev git source must be a family member or carry a pins entry; moving git and stale entries fail |
| Build/implementation identity | `pse-buildinfo` `BUILD_IDENTITY` | Hashes `Cargo.toml`, `Cargo.lock`, `uv.lock`, `pyproject.toml`, toolchain and build env; enters completion and publication implementation identities |
| Semantic identity contract | `pse-ids`, frozen by `golden_vectors.rs` | Independent of the blake3 crate version by construction |

Strength: the split is clean. Manifests own constraints, lockfiles own resolution, a metadata table
owns the reasons. The two checks read those owners and need no hand-maintained list. Because
`BUILD_IDENTITY` already digests the lockfiles, floating requires no identity change: every resolved
version still enters the implementation key (DP-09 complete key).

## 3. Change scenarios

| ID | Scenario | Observation |
|---|---|---|
| S01 | Add a library (`cargo add`, `uv add`) | Caret/floor, no ADR, no pins entry; `dependency_pins` passes. A *family member* added this way is a caret and also passes (F03) |
| S02 ([PSE-S06](../design_principles/binding/pse-arrow.md#pse-s06)) | Routine `just upgrade` with no arguments | Dry run: 57 Rust and 18 Python moves; the exact family members hold arrow/parquet/datafusion/object_store/pyo3 (pyo3 0.29.3 and arrow 60 are held). The recipe stops there: the workspace-hack can drift (new `synstructure` 0.14, `itertools` 0.12 added, `multiversion` removed) and `family-check` is not run (F02). `BUILD_IDENTITY` changes, correctly |
| S03 | Family patch or major move | GOVERNANCE §5: edit every member `=` and the families entry, `just upgrade <crate>`, `family-check`, ADR for a major. Sound |
| S04 | A floated version breaks something | Add an exact pin with a pins entry; `dependency_pins` demands the reason. A `<`/`~` hold-back is not detected (F03) |
| S05 | Generator dependency move (syn, prettyplease, bindgen, cornucopia, schemars) | Kept exact; a bump is a manifest edit plus regeneration; `codegen-*-check` catches drift. Sound |
| S06 | Reference oracle regeneration or live comparison after an upgrade | FeOS (`feos`, `feos-core`, `quantity`) now floats although it generates `oracles/feos-0.10.1` and is called by conformance (F04) |
| S07 | Identity and hashing | Semantic identities: unaffected; BLAKE3 output is fixed by the frozen golden vectors. Implementation identity: complete; over-inclusive for tool-only `uv.lock` moves (observation, §5) |
| S08 | First publication (R-31) | Wheel metadata carries uncapped floors; the ADR's revisit trigger covers it |

## 4. Assessment by question

### 4.1 ADR-0122: what is superseded, what is carried

In substance, ADR-0159 replaces only ADR-0122's Outcome item 6 ("every external dependency stays
`=`-pinned"). Every other decision named in the brief is carried and not contradicted: the dated
nightly and `rust-version` floor, feature unification and the workspace-hack, per-checkout
`target/`, committed lockfiles with `--locked`, `cargo deny`/`audit`/`shear`, families with the
family-major ADR, `no_patch_tables`, `delta_revisions`, `dependency_floors` and ADR-0066's relaxed
admission. Nothing in the new text contradicts them, and the `dependency_floors` message now
prefers raising the floor over holding a dependency back, which matches the policy.

The mechanism is the problem ([F01](#f01)). `just adr-supersede` marked the whole of ADR-0122
`superseded` while ADR-0159 is only `proposed`, the only such pair in the tree. The live decisions
for the toolchain and the workspace-hack therefore have, as their current record, a one-paragraph
restatement inside an ADR about dependency versions. That restatement carries one of ADR-0122's four
revisit triggers and omits the `workspace_hack_keeps_force_validate_opt_in` control and the hakari
traversal exclusions. Blueprint §3.1 and §3.2, AGENTS.md prime directive 6 and the invariants, and
the `Cargo.toml` comments still cite ADR-0122 as the authority. ADR-0159 itself uses the gentler
route for ADR-0066: it amends one clause and retains the record, following ADR-0066's own precedent
for ADR-0018. However, ADR-0066's accepted text ("`=`/`==` pins … are unchanged") gets no forward
pointer.

### 4.2 One type universe

The guarantee holds. `family_check` takes every package in `cargo metadata --locked`, groups
those matching each family's globs (excluding `datafusion-tracing`), and fails unless the set has
exactly one version equal to the declared one (`match = "minor"` for pyo3). A mixed family without
a duplicate name, the hazard ADR-0018 named, therefore fails, and so does a family moved ahead of its
declaration. Carets elsewhere do not weaken this, because the check reads resolution, not
requirements. The exact member pins add an anchor of their own: `cargo update` cannot move a
family, and undeclared transitive members (`arrow-data`, `arrow-arith`, …) stay put because their
newer releases require newer anchored members. The dry run confirms that no family package moves.
Carets on bridge crates (`pyo3-arrow`, `pyo3-async-runtimes`, `datafusion-tracing`,
`instrumented-object-store`) are acceptable: their family dependencies are what the check sees.

Two gaps remain, both about *when* and *what* is checked, not about the check itself. First,
`family-check` runs only in hygiene and governance, while `just upgrade` makes lock moves routine
([F02](#f02)). A non-family crate that declares a wide range on a family (`>=53, <61`) can pull a
second major into the graph on an upgrade. Tests run in between see `downcast_ref` return `None`
with no compile error, which is exactly the misleading failure the invariant exists to prevent.
Second, nothing requires family members in `[workspace.dependencies]` to be exact, although the
ADR says they stay exact and `cargo add` writes a caret ([F03](#f03)).

### 4.3 Kept and floated pins

| Pin | Judgment |
|---|---|
| Family members; POUNCE/FERAL git revisions; vendored `deltalake` | Sound (type-sharing family; fork at an immutable revision; vendored source) |
| `syn`, `prettyplease`, `bindgen` | Sound: they produce committed generated Rust and bindings; `codegen-*-check` |
| `cornucopia` (flagged) | Sound, and the strongest of the three: the generated crate's header records "Cornucopia 1.0.1", and `codegen/queries.rs` inventories the dependency set Cornucopia emits |
| `schemars` (flagged) | Sound: `codegen/schemas.rs` commits `schema_for!` output as JSON Schemas and the msgspec contract types |
| `salsa` (flagged) | Reason not supported ([F05](#f05)): no use of a `salsa_unstable`-gated item was found; the feature is salsa's own default, re-enabled under `default-features = false` |
| `validator` | Acceptable as vendored-source coupling (`scripts/vendor-delta.py` writes it into the provenance-checked Delta manifest); the reason text could say that |
| `num-dual` | Over-pinned ([F05](#f05)): `feos`, `feos-core` and `quantity` require `"0.14"`, and Cargo never resolves two semver-compatible versions, so a caret already yields one num-dual 0.14 |
| `blake3` | Over-pinned ([F05](#f05)): the identity contract is BLAKE3 as frozen by `golden_vectors.rs`, which states an independent implementation must reproduce it. "Qualified against this implementation" is the excluded "version enters a digest" reason; the golden vectors are what would name a breakage |
| `feos`, `feos-core`, `quantity` (floated) | Wrongly floated ([F04](#f04)): they generate `packages/reference/data/oracles/feos-0.10.1` (`just feos-reference`) and are called live by `tests/conformance` (`feos_coherent_state_multi_output`). The dry run already moves `quantity` 0.14.0 → 0.14.1 |
| `symbolica`, `faer`, `nalgebra`, `pyo3-arrow`, tracing adapters (floated) | Sound under the policy: no dependency-specific reason found; carets on 0.x hold the minor; tests are the check |
| Python: `idaes-pse`, `pyomo`, `pint` (parity); `teqp` (thermo reference); `numpy`/`scipy` (test oracles) | Sound reasons. One consequence goes unstated: a single universal `uv.lock` resolves groups and extras together, so the test-group `numpy==2.5.3` also holds the runtime `array` extra's numpy (F05) |
| Python: `pyarrow`, `attrs`, `cattrs`, `msgspec`, tools (floated) | Sound under the policy. Python floors cross majors where Rust carets do not; msgspec 0.22.0 arrives on the next upgrade and is a Python-boundary codec, so its tests are the ones the move affects |

### 4.4 Blueprint wording

§3.1 and §3.3.2 agree with the ADR on the rule, the families and the check. The ADR-0159 decision
line correctly marks the record as proposed. Residual text still states the old rule or overstates
the new one ([F06](#f06)). §3.1 still says "Read the manifest for a version", although the
lockfile is now where a version is read, as the ADR's own Pros and cons say. The §3.1 and §3.2
decision lines cite ADR-0122, now marked superseded (F01). Outside the blueprint, the same applies to
the library-upgrade issue template, the Dependabot comment and `versioning-strategy: auto`, the
capability-map "pinned"/"pin exactly" wording, and three diverging copies of the list of acceptable
reasons.

## 5. Remaining risks examined

- **Identity hashing.** `BUILD_IDENTITY` digests both lockfiles and both manifests, so it stays a
  complete implementation key under floating. That is a strength, and no change is needed. It is
  over-inclusive: a tool-only `uv.lock` move (ruff, pyrefly, typos) changes the completed-environment
  and publication implementation identities and rebuilds every dependent of `pse-buildinfo`.
  This predates ADR-0159. Routine `just upgrade` makes it more frequent. It is safe (no stale
  reuse), and the policy states that a version entering a key is not a pin reason. Observation, not
  a finding. Semantic identities do not depend on any floated version: `pse-ids` is the sole
  `blake3` dependent (`blake3_owner`), and `golden_vectors.rs` freezes the digests.
- **Codegen bytes.** Every generator whose output is committed stays exact (syn, prettyplease,
  bindgen, cornucopia, schemars, pyo3-introspection as a family member). Floated `quote` and
  `proc-macro2` feed `prettyplease`, which reprints from the AST. The cargo-hakari section is the
  one generated artifact that is a function of the lock itself and is not regenerated by the
  upgrade path (F02).
- **Wheel metadata.** Runtime requirements become uncapped floors at the locked versions. This is
  right for an unpublished package. R-31 and the ADR's revisit trigger cover the first release.
  `maturin>=1.15,<2` in `[build-system]` is a cap without a recorded reason (F03).
- **Evidence lockfiles.** `family-check` compares the capability-map evidence lockfiles with the
  graph for family packages only, as before, and they agree. Non-family drift is unchecked
  (already 14 packages in the support lock) and will grow with routine upgrades. The maps remain
  evidence about their extraction versions, and DP-15 now says behaviour is qualified at the
  resolved version. The ADR's claim that the capability maps "already read resolved versions from
  `Cargo.lock`" is inaccurate (F06).
- **Native build tooling** (`cc` 1.4.6 → 1.6.0 on the next upgrade, `cmake`) floats. A changed
  default compiler flag could alter vendored C numerics. With no named breakage, this is not a pin
  reason under the policy; conformance and parity are the checks. Observation.
- **`uv.lock` revision 3 → 5** was written by uv 0.12.22. CI's `setup-uv` installs the latest uv,
  so CI is unaffected. An older local uv may not read it. Out of scope.

## 6. Architectural assessment and gates

| Foundation | Verdict | Evidence |
|---|---|---|
| AP-01 Separation of concerns | satisfied | Constraints, resolution, reasons, family identity, reason enforcement and type-universe enforcement each have one owner (§2) |
| AP-02 Stable contracts | satisfied | The pin contract is explicit and machine-read for Rust; its enforcement is narrower than its statement (F03), a correction rather than a missing contract |
| AP-03 Composition | satisfied | `just upgrade` composes the two native resolvers; the lock-derived follow-ups belong in the same operation (F02) |
| AP-04 Domain model and authority | satisfied | The model distinguishes requirement, resolution, family, pin and reason, and these govern the checks. The decision-record routing (F01) and one misapplied category (F04) are defects of application, not of the model |
| AP-05 Explicit structure | satisfied | `families` and `pins` are declared tables, consumed by `family-check` and `dependency_pins` |
| AP-06 Local reasoning/testability | satisfied | `dependency_pins` unit-tests its classifier on fixture manifests without Cargo or network; `family-check` needs only `cargo metadata --locked` |

| Gate | Result | Evidence / required action |
|---|---|---|
| G1 Authority | pass, F01 required | One owner per fact; the decision-record route for ADR-0122's surviving decisions must be repaired |
| G2 Semantic fidelity | pass, F04 required | No resolved version moved at adoption (`cargo metadata --locked`, `uv lock --check`); the FeOS oracle must keep the version it is named for |
| G3 Validity | pass | Unreasoned exact pins, moving git sources and stale reasons fail |
| G4 Hidden behavior | pass | The MSRV-aware resolver's hold-backs are ADR-0122 behaviour and are reported by `cargo update --verbose` |
| G5 Consistency and recovery | pass, F02 | After an upgrade, hakari output and the family invariant are consistent only once hygiene runs |
| G6 Transformation and reuse | pass | Implementation identity digests the lockfiles; semantic identity is version-independent |
| G7 Truthful capability claims | pass conditional on F03/F06 text | "A pin cannot be added silently" and the capability-map sentence overclaim; correct them in the acceptance change |
| G8 Library leverage | pass | Cargo caret semantics, `--locked`, `cargo update`, the MSRV-aware resolver, uv floors and cargo-hakari; no bespoke resolver |
| G9 Architectural fitness | pass | Follows the six satisfied foundations |
| PS-13 Reference validation (SHOULD) | deviation, F04 | A reference oracle floats |

## 7. Findings

| ID | Severity | Finding | Correction (summary) |
|---|---|---|---|
| <a id="f01"></a>F01 | Medium | Record-level supersession of ADR-0122 for a one-clause change, before ADR-0159 is accepted; surviving decisions lose triggers and controls; ADR-0066 has no forward pointer | Amend and retain, per the ADR-0066 precedent |
| <a id="f02"></a>F02 | Medium | `just upgrade` moves the locks but neither runs `family-check` nor regenerates the workspace-hack | Finish the recipe with hakari regeneration and `family-check` |
| <a id="f03"></a>F03 | Low | `dependency_pins` covers exact and git pins only: caps and hold-backs, caret family members and Python pins pass; the ADR claims more | Widen the classifier; check family exactness; decide the Python route; correct the ADR claim |
| <a id="f04"></a>F04 | Medium | The FeOS reference oracle (`feos`, `feos-core`, `quantity`) floats | Pin it with the reference-oracle reason |
| <a id="f05"></a>F05 | Low | `num-dual`, `blake3` and `salsa` are kept for reasons that need no exact pin or are not supported by the code | Float, or record the actual dependency-specific reason |
| <a id="f06"></a>F06 | Low | Residual old-rule and overclaiming text; three diverging lists of acceptable reasons | Edit the listed texts; keep one reason list |
| <a id="f07"></a>F07 | Low | The core standard's DP-15 MUST and DP-14 changed wording with no version note | Add a dated revision note; name the edit in ADR-0159 |

### F01 — Record routing for ADR-0122 and ADR-0066

*Principles/gate:* DP-01, G1. *Evidence:* ADR-0122 front matter now has `status: superseded` and
`superseded-by: ADR-0159`, while ADR-0159 is `proposed`. No other retained pair has a proposed
successor. `scripts/adr.py supersede` sets the status unconditionally. ADR-0159's `revisit:` carries
"Cargo stabilizes `feature-unification`" but not ADR-0122's other three triggers: a nightly
regression blocking the date, hakari emitting a `force_validate` or native-solver line, and Cargo
keying path packages by absolute location. Its Verification omits
`workspace_hack_keeps_force_validate_opt_in`. Blueprint §3.1 and §3.2, AGENTS.md prime directive 6
and the feature-unification invariant still name ADR-0122. ADR-0066's Scope and Compensating
controls still say `=`/`==` pinning is unchanged, with no Status-history pointer.
*Consequence:* the record set and the architecture disagree about which record holds the live
toolchain and workspace-hack decisions. Three revisit triggers sit on a record marked dead. If
ADR-0159 were revised or rejected, ADR-0122 would be left superseded by nothing accepted.
*Correction (preferred):* follow the amend-and-retain precedent ADR-0159 already uses for ADR-0066.
The commits are unpushed and `origin/main` still has ADR-0122 accepted, so the lint permits this.
Restore ADR-0122 to `accepted` with `superseded-by: null`, and replace its appended history line
with "2026-10-04 — Outcome item 6 (exact pins) amended by ADR-0159; items 1–5 stand". Append a
matching history line to ADR-0066. In ADR-0159, set `supersedes: []`, say "amends" in Scope, drop
the carried ADR-0122 trigger from `revisit:`, and keep the carried-forward list as pointers. Reword
the §3.1 ADR-0159 decision line and the revision-120 row from "superseding" to "amending".
*Alternative:* keep the supersession, but have ADR-0159 carry all four triggers and ADR-0122's
compensating controls, and move every ADR-0122 citation (AGENTS.md, §3.1, §3.2, `Cargo.toml`) to
ADR-0159. This is heavier and gives the toolchain decision a dependency-titled home.
*Verification:* `just adr-lint` passes. Every ADR-0122 citation resolves to an accepted record.
ADR-0122's four triggers appear exactly once among current records.

### F02 — The upgrade operation stops before its consequences

*Principles/gate:* DP-15, DP-01 (generated output), G5; S02, PSE-S06. *Evidence:* the `upgrade`
recipe runs `uv lock --upgrade` and `cargo update` (or per-package forms) and exits. The end-of-turn
hooks run `fmt` and other generators, but not `cargo hakari generate`. Today's dry run adds and
removes graph packages, including a new semver-incompatible `synstructure`, and hakari's generated
section is a function of that graph. `family-check` runs in hygiene and governance only. The recipe
comment defers checks "when a need emerges". The one-type-universe invariant is that need.
*Consequence:* after a routine upgrade, `codegen-hakari-check` fails later at scope end. More
seriously, a crate whose wide range admits a second family major makes `downcast_ref` return
`None` in the tests the agent runs next, a failure that looks like a logic bug, until hygiene names
it. `cargo update -p syn` is also ambiguous while syn 2 and 3 both resolve.
*Correction:* end `just upgrade` with `cargo hakari generate && cargo hakari manage-deps --yes` and
`just family-check`, failing loudly and printing the moved packages. Optionally add
`cargo update --verbose` so held-back packages are visible. For an ambiguous name, suggest the
`name@version` spec. This adds no hygiene check mid-work: both steps are consequences of the
operation just performed, and GOVERNANCE §5 already requires `family-check` after a family move.
*Verification:* on a scratch branch, `just upgrade` exits non-zero when a family is split, and
`just codegen-hakari-check` is clean afterwards.

### F03 — Enforcement is narrower than the stated contract

*Principles/gate:* DP-23, G7. *Evidence:* `dependency_pins::declaration` treats a requirement as a
pin only when a comparator starts with `=` or the source is git. Its own fixture asserts that
`'>=1.0, <2'` passes. A `~`, `<` or `<=` hold-back is therefore `Floating`. A family member declared
with a caret is never flagged, and `cargo add arrow-json@59.3.0` writes a caret. Python reasons
are comments that no check reads, and `[build-system] maturin>=1.15,<2` carries no reason. ADR-0159
Compensating controls say "a pin cannot be added silently". The ADR, §3.1, AGENTS.md and the policy
all say a cap or hold-back needs a reason. *Consequence:* the cheapest hold-backs, a cap or a
tilde, escape the one check the policy relies on, and the ADR claims coverage it does not have.
*Correction:* classify any comparator that bounds from above (`<`, `<=`, `~`, `=`, wildcard minor)
as a pin. Require each `[workspace.dependencies]` entry that matches a family glob to be exact at
the family's declared version. That turns the duplicated family version into a checked pair at the
manifest, not only after an upgrade. For Python, either move reasons into a `[tool.pse.pins]` table
read by a small governance test, or record in ADR-0159 that Python reasons are instruction-only (a
§H exception with a trigger). Correct the ADR's compensating-control sentence in the acceptance
change. *Verification:* unit cases for tilde, `<`, a caret family member and an unreasoned Python
`==` (if the table route is chosen).

### F04 — The FeOS reference oracle floats

*Principles/gate:* PS-13 (SHOULD), G2; S06. *Evidence:* `5262cf987` turned `feos =0.10.1`,
`feos-core =0.10.1` and `quantity =0.14.0` into carets. `xtask/src/feos_reference.rs` ("Independent
oracle generation only") is run by `just feos-reference` into
`packages/reference/data/oracles/feos-0.10.1/data`, a package named `pse.data.oracles.feos-0.10.1`.
`tests/conformance` enables `feos`, `feos-core` and `quantity` and calls FeOS live
(`math_composition::feos_coherent_state_multi_output`). The policy names "a parity or reference
oracle" as a reason. The Python analogues (`teqp==0.23.1`, the IDAES parity set) were kept, and the
dry run already moves `quantity` to 0.14.1. *Consequence:* after an upgrade, a regeneration writes
values from another FeOS into a package named for 0.10.1. Live conformance comparisons also run
against a reference other than the recorded one. The provenance the reference package states
becomes false without any check noticing. *Correction:* pin `feos`, `feos-core` and `quantity`
exactly, with one pins entry such as "FeOS reference oracle: committed package `feos-0.10.1` and
conformance comparisons are qualified at this release". Fold `num-dual` into that closure or float
it (F05). *Verification:* `dependency_pins` passes, and `cargo update --dry-run` no longer lists
`quantity`. Optionally, a check that the oracle package name matches the resolved `feos` version.

### F05 — Pins whose stated reasons do not require them

*Principles/gate:* DP-15; policy conformance. *Evidence and correction, by pin:*
- `num-dual`: the stated reason ("pse code and feos must resolve the same num-dual 0.14") is
  satisfied by a caret, since its dependents require `"0.14"` and Cargo keeps one 0.14.x. Float it,
  or restate it as part of the F04 oracle closure.
- `blake3`: the semantic identity contract is the BLAKE3 specification as frozen in
  `golden_vectors.rs`. A crate version that changed a digest would fail those vectors, which is a
  named breakage. The stated reason is the excluded "version enters a digest" category. Float it,
  and rely on the golden vectors and `blake3_owner`.
- `salsa`: the reason cites the `salsa_unstable` API. A grep of `crates/**/*.rs` found only
  `tracked`, `interned`, `input`, `db`, `Event`/`EventKind`, `Durability`, `Setter`, `Cancelled`,
  `CancellationToken` and `Storage`, none of them gated by `salsa_unstable` in 0.28.4. No
  `memory_usage`, `IngredientInfo`, `PageInfo` or input `value()` use was found. The search does not
  cover macro-expanded code. Either name the actual dependency-specific reason (for example, if
  reuse tests that count `WillExecute` events are the concern, say so), or float salsa and drop
  `salsa_unstable` if it is unused.
- Python `numpy` test oracle: add to its comment that it also fixes the runtime `array` extra's
  resolved numpy in the universal lock.

*Consequence:* pins kept on weak grounds teach the next agent that the reason bar is low, which is
the drift the policy exists to stop. *Verification:* each remaining pins entry names a mechanism
that a caret would violate.

### F06 — Residual and overclaiming text

*Principles/gate:* DP-01, G7. *Instances:*
- Blueprint §3.1: "Read the manifest for a version" (now the lockfile).
- `.github/ISSUE_TEMPLATE/library-upgrade.yml`: "Every dependency here is version-pinned".
- `.github/dependabot.yml`: the header says the codegen crates are `=`-pinned and refers to
  "exact-pin and family gates". `versioning-strategy: auto` rewrites caret requirements in
  `Cargo.toml` instead of moving the lock (prefer `lockfile-only`), and ADR-0159 does not say how
  Dependabot (ADR-0035) coexists with `just upgrade`.
- `docs/capability-maps/README.md` ("pinned library APIs") and the "pin exactly" advice in
  `supporting-rust-libraries.md` (petgraph, salsa, faer). For 0.x crates a caret already holds the
  minor.
- ADR-0159 Pros and cons: the capability maps do not read `Cargo.lock`. They describe their
  evidence lockfiles, compared for families only.
- AGENTS.md "Latest by default … move to the latest with `just upgrade`": `cargo update` never
  crosses a caret (44 held today, among them quantity 0.15 and diffsol 0.17). Say "latest
  compatible", and name the manifest edit, or `cargo update --breaking -Z unstable-options` on the
  pinned nightly, for majors.
- The list of acceptable reasons appears three times with differences. The ADR has "unstable-API
  use" and lacks "wheel availability". AGENTS.md has the reverse. The policy has unstable-API use.
  Keep one list, in `docs/dev/dependency-policy.md`, and link it.

*Correction:* edit these texts. Blueprint text goes through the `design:` route.

### F07 — Core standard wording changed in place

*Principles/gate:* standard governance (`standard.toml`). *Evidence:* `edb9411dd` changed DP-15
(MUST) "Pin versions and features" to "Qualify behaviour at the resolved (locked) version and the
enabled features", and DP-14 "pinned" to "resolved". The header still reads "Version 3.3 ·
2026-09-30". The same edit is in another repository's copy, so the core is shared. The obligation is
compatible, since a lockfile pins resolution, so this is a clarification. But reviews cite "core
3.3", and the template's own convention is a dated note ("Output and synthesis guidance revised
2026-10-01; assessment rules and slot identifiers unchanged"). *Correction:* add "DP-14/DP-15
wording revised 2026-10-04 (resolved version); obligations unchanged" under the version line in
each copy, and name the edit in ADR-0159's Scope.

## 8. Alternatives

| Alternative | Judgment |
|---|---|
| Baseline: everything exact | Rejected by the ADR and the policy; the lockfiles already give reproducibility |
| Carets everywhere, families included, with `family-check` alone | Viable, because the check reads resolution. But `just upgrade` would then move families and fail the check on every upstream patch. The exact member pins are a cheap anchor. ADR's rejection is sound |
| **Proposed: carets/floors with reasoned exact pins** | Selected; sound with F01–F07 |
| Library-owned: Dependabot or Renovate in lockfile-only mode with family groups instead of `just upgrade` | Complementary, not a replacement: CI runs only on dispatch here, and the policy puts upgrades at the agent's discretion. Configure Dependabot `lockfile-only` or retire it (F06) |

## 9. Decision

**Behavioral/semantic adequacy:** satisfied. At adoption no resolved version moved (Tested:
`cargo metadata --locked` and `uv lock --check`, 2026-10-04). Reproducibility, the type universe
and identity completeness are preserved by mechanisms that read resolution. F04 is required to keep
the FeOS reference oracle's stated provenance true.

**Architectural fitness:** satisfied. Six foundations satisfied; G9 passes.

**Overall: Accept-scoped.** The accepted scope is the dependency policy as implemented for
`[workspace.dependencies]` and `pyproject.toml`, at Implemented evidence (Interface-checked by this
review; probes Tested as listed in §1). Conditions for `status: accepted`:

- F01's record routing, F04's oracle pins, and the ADR-text parts of F03 (the "cannot be added
  silently" claim) and F06 (the capability-map sentence) are in the acceptance change.
- ADR-0159's `review:` cites this document.

The recorded scoped deviation: Rust upper caps and hold-backs, and all Python pin reasons, stay
instruction-enforced until F03's test extension lands (trigger: the first `<`/`~` requirement or
new `==` pin added without a reason, or F03's change).

| Priority | Change | Findings | Acceptance evidence | Owner |
|---|---|---|---|---|
| 1 | Amend-and-retain routing for ADR-0122/ADR-0066 | F01 | `just adr-lint`; citations resolve to accepted records | ADR-0159 acceptance change |
| 1 | Pin the FeOS oracle closure | F04 | `dependency_pins`; `cargo update --dry-run` | ADR-0159 acceptance change |
| 1 | Correct ADR-0159 claims | F03 (text), F06 (Pros and cons) | Reading | ADR-0159 acceptance change |
| 2 | `just upgrade` regenerates hakari and runs `family-check` | F02 | Scratch-branch upgrade | Tooling follow-up |
| 3 | Widen `dependency_pins`; decide the Python reason route | F03 | New unit cases | Tooling follow-up |
| 3 | Re-reason or float num-dual, blake3, salsa | F05 | Pins entries name a caret-violating mechanism | Follow-up |
| 3 | Residual text; one reason list; standard revision note | F06, F07 | Reading; `design:` route for §3.1 | Follow-up |

F02 is the most consequential follow-up. It belongs with the acceptance if routine `just upgrade`
use starts before the follow-up lands. F04 should precede any `just upgrade`, because the next one
moves `quantity`.

**Coverage limits.** This was a static review plus the read-only probes listed in §1. No
governance test, `family-check`, codegen check, build or regeneration was executed by the reviewer.
The salsa finding rests on a source grep that excludes macro-expanded code. Unexamined: whether
floated Symbolica or faer patch releases change numerical results that enter conformance
expectations. The tests the move affects are the policy's check for that.
