# Dependency admission and licence policy review

## 1. Decision and scope

**Accept-scoped.** Reviews ADR-0066 and the blueprint §3.1/§3.3.2/§24.1 amendment against
the Data Model–Based Design Charter. Method: inspected `deny.toml`, the `pse-tests-governance`
crate, `xtask` `family-check`, `pyproject.toml` import-linter contracts, the `justfile`
recipe graph, `.github/workflows/rust.yml`, `.github/setup/ruleset-main-full.json`, and the
six documents carrying the ADR-requirement table. Scope is **admission policy**: which
third-party libraries may enter the graph and under which licences. Reproducibility,
the one type universe, and the semantic contracts of the platform are explicitly out of
scope and are asserted to be unchanged.

This review does not establish that any particular library is appropriate. It establishes
that refusing libraries *by policy* was costing more than the risk it addressed at this
phase, and that the mechanisms which answer the deferred question remain installed.

## 2. Authority and lifecycle map

Blueprint §3.1 remains the **pin** authority: the version a named crate must declare.
It ceases to be the **admission** authority. New §3.3.2 owns admission policy and states
it once; `docs/dev/dependency-policy.md` is its operational projection; `deny.toml` is the
configuration of a report, not a gate. `tooling_deps.toml` degrades from an exemption
allowlist to documentation of why sixteen crates are not platform commitments. Register
row R-31 owns the lifecycle: the policy is dated, triggered and re-openable, not deleted.

## 3. Semantic contracts and invariants

Three invariants were separated from the policy they had been bundled with.

- **One type universe** (blueprint §3.1) was enforced twice: precisely by
  `cargo xtask family-check`, and incidentally by `deny.toml`'s `multiple-versions = "deny"`
  plus nineteen `skip` entries. Only the first states the invariant; the second stated
  "no crate in the graph may appear twice", which is a different and much broader claim.
  Removing the second loses no coverage of the actual hazard.
- **Pin drift** was enforced in the same test as admission. They are now distinct arms:
  drift fails, an undescribed dependency is reported.
- **Import boundaries** (numpy/scipy, pyomo/pint, `pse.contracts`) are architecture and are
  untouched. The one contract that was dependency hygiene — `pandas`, `pydantic`, `sympy`,
  `networkx`, `matplotlib`, `click` — is removed; `idaes` is retained inside it because its
  reason is a packaging constraint (uv workspaces enforce a single `requires-python`), not
  hygiene.

## 4. Derivation and execution design

No derivation, pass, schema or execution path changes. The change is entirely in
governance configuration and prose. The single code change is
`tests/governance/tests/pins_match_blueprint.rs`, where one match arm moves from `problems`
to an `eprintln!` note and the stale-exemption assertion becomes the same.

## 5. Representative journeys

An implementer needs a crate for a phase-1 capability. Before: the crate fails
`pins_match_blueprint`, so they amend blueprint §3.1 under `PSE_DESIGN_EDIT=1` in a
`design:` PR, write an ADR, request a design review, and if the crate pulls a duplicate
`hashbrown` they also add a dated `skip` entry. After: `cargo add`, and `just governance`
prints a note naming it. If that crate is AGPL-3.0, `just deps-report` says so and nothing
blocks. If it drags a second `arrow-schema` major, `just family-check` still fails — the
hazard that actually breaks `downcast_ref` is unchanged.

## 6. Acceptance gates

| Gate | Result | Evidence and scope |
|---|---|---|
| G1 Authority | Pass | §3.1 keeps pins; §3.3.2 gains admission; `docs/dev/dependency-policy.md` is the single operational statement and the six repeated tables cite it rather than restating it. |
| G2 Semantic fidelity | Pass | The one-type-universe invariant is stated by `family-check` alone, where it was previously spread across two mechanisms with different meanings. |
| G3 Validity | Pass | Drift still fails. `cargo deny --all-features --locked check` reports `advisories ok, bans ok, licenses ok, sources ok` on the current lockfile. |
| G4 Hidden behavior | Pass | The lifted bans' reasoning is retained in `deny.toml` prose and the dev doc; the three semantic traps are named explicitly rather than silently dropped. |
| G5 Consistency and recovery | Pass | R-31 makes the deferral recoverable with a trigger that can fire and a check that can run; `just policy` is retained unchanged as that check. |
| G6 Transformation and reuse | Not applicable | No derived artifact, hash framing or reuse decision is touched. |
| G7 Truthful claims | Pass | SECURITY.md and `docs/dev/ci.md` are corrected in the same change, because leaving them would make a public statement false. |

## 7. Principle findings

| Finding | Principles | Evidence or gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|
| Admission machinery exceeded the risk it addressed at this phase | DM-56, DM-58 | Adding a crate required amending an off-limits design document through a `design:` PR; the project publishes nothing and every crate is `publish = false` | Implementation slows for a licensing risk that cannot yet materialize | Make admission advisory; keep every mechanism installed and runnable | `just deps-report`, `just policy`, `rust / deny` present and `continue-on-error` |
| One invariant was enforced by two mechanisms with different meanings | DM-31, DM-56 | `family-check` asserts one resolved version per family; `deny.toml` `multiple-versions` asserted no duplicate crate at all, behind nineteen `skip` entries | The broader mechanism blocks unrelated additions while adding nothing to the real hazard | Retire the broad one; keep `family-check` required | `rust / family-check`; a forced off-family pin still fails |
| The deferred licensing question needed an owner, not a deletion | DM-31, DM-59 | Removing the licence allowlist without a trigger would silently convert a deferral into an omission | Copyleft accumulates unexamined until a release forces an audit with no tooling | Register row R-31 with a firing trigger and `just policy` as its check | `scripts/check_register.py --lint`; 31 rows OK |
| Public statements would have become false | DM-59, G7 | SECURITY.md promised `cargo deny`/`cargo audit` "fail on … disallowed licenses" on every PR; `docs/dev/ci.md` listed `rust / deny` as required | A truthful-claims violation in a security-facing document | Rewrite both to describe the advisory posture accurately | Inspection; `just docs` link check |

Applicability: the authority, proportionality and truthful-claims groups apply directly.
Numerical, solver, backend-binding and recovery groups are outside this change.

## 8. Alternatives and architectural leverage

| Alternative | Risk | Cost | Decision |
|---|---|---|---|
| Keep the closed list, add blueprint rows per library | Every library is a `design:` PR against an off-limits file | The exact cost the decision exists to remove | Reject |
| Delete the machinery outright | The licensing question becomes unanswerable without rebuilding tooling | Low now, high at release | Reject |
| Keep every mechanism, make it report; own the deferral with a dated trigger | Copyleft accumulates unexamined until R-31 fires | One register row, one dev doc | Select |
| Relax admission but keep `multiple-versions = "deny"` | Ordinary additions still blocked over hashers and proc-macro plumbing | A growing hand-maintained `skip` list | Reject; `family-check` covers the real hazard |

## 9. Verification and measurement plan

**Implemented:** the configuration, test and documentation changes.
**Tested:** `tests/governance/tests/pins_match_blueprint.rs` (both tests pass with the
drift arm intact); `python/pse/tests/conftest.py::no_numpy_on_import`; `just lint-imports`
(4 contracts kept, 0 broken); `cargo deny --all-features --locked check` (all four checks
ok). **Proposed:** the pre-release licensing audit that R-31 triggers. No performance
claim is made and none applies.

Negative control: forcing an off-family pin must still fail `just family-check`. Positive
control: a dependency absent from blueprint §3.1 must pass `just governance` with a note.

## 10. Exceptions and unresolved decisions

No SHOULD deviation is introduced. The licensing question is deferred, not answered —
that is the substance of the decision and the reason R-31 exists. ADR-0066 remains
`proposed` until its decision PR; ADR-0065 covers the Arrow/DataFusion subset of the same
argument and is likewise proposed.

## 11. Decision and implementation changes

Accept-scoped. The scoping is that this verdict covers **admission policy only**: it does
not certify any library, does not relax reproducibility, and does not survive the trigger
in R-31 without being revisited.
