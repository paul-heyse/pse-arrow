<!--
Thanks for contributing. Fill every section — a blank box is not an answer, but
"no ADR needed because …" and "N/A because …" are. See CONTRIBUTING.md §3.

If this pull request is squash-merged, its title becomes the changelog entry. Use a
Conventional-Commit subject:
  <type>(<scope>): <summary>
types: feat fix perf refactor docs test build ci chore deps adr design
scopes are free-form (crate or area short names).
An ADR pull request is titled exactly `adr: ADR-NNNN <title>` and labeled `adr`.
-->

## Summary

<!-- What changes and why, in a few sentences. Lead with the behaviour, not the diff. -->

## References

- **Implements:** <!-- ADR-NNNN, or "no ADR needed because …" (see CONTRIBUTING.md §4) -->
- **Plan:** <!-- docs/plans/NN-*.md, or N/A -->
- **Review / finding / scenario:** <!-- links for architectural changes; N/A otherwise -->
- **Current disposition owner:** <!-- link the plan/packet; do not copy its live status -->
- **Closes:** <!-- #123, or N/A -->
- **Register rows touched:** <!-- R-NN, or none -->

## Architecture impact

<!-- For architectural changes: expected change boundary, consumed contract, composition and
local testability; material library integration cost. Link the review for architectural and
behavioral verdicts. A bounded change can say why this is N/A. Do not repeat the whole review. -->

## Evidence

<!--
Label every claim with the §D vocabulary and NAME the test or benchmark:
  Proposed | Interface-checked | Implemented | Tested <test name>
  | Measured <bench name> + conditions | Formally established <argument + assumptions>
The canonical definitions are in the core design principles §D; a passing check does not
certify unexamined architecture or promote a historical observation into a current result.
"Tests pass" is not evidence. "Tested: tests/engine/spill.rs::spill_bounded_by_limit"
is. Say what you did NOT verify too.
-->

- 
- **Not verified:** 

## Checklist

- [ ] An ADR is referenced above, **or** I have stated why no ADR is needed.
- [ ] Generated sources were regenerated with `just codegen` and committed
      (`rust / codegen-diff` is green), or this PR touches no generator input.
- [ ] Lockfiles (`Cargo.lock`, `uv.lock`) changed only deliberately, by a targeted
      `cargo update -p … --precise …` / `uv lock --upgrade-package …`, and the reason is
      in the summary — no bare `cargo update` / `uv lock`.
- [ ] If one of the four pinned families was **majored** (arrow, datafusion, pyo3,
      object_store): the ADR exists, the capability map under `docs/capability-maps/` was
      regenerated, and `just family-check` passes. *Adding a dependency needs none of
      this — see `docs/dev/dependency-policy.md`.*
- [ ] If the IDAES parity pin moved: GOVERNANCE.md §5 was followed and every tolerance
      change is called out individually above.
- [ ] No edits under `docs/generated/`, `crates/*/src/generated/`,
      `python/pse/contracts/`, `external/`, `build/`, or `target/`.
- [ ] Documentation updated where an enduring contract, explanation or workflow changed; new files follow the naming and
      citation conventions (CONTRIBUTING.md §7).
- [ ] Any checks chosen for this review and their scope are reported above. CI is
      optional and manually initiated; no full qualification is required for this PR.

## Legal

- [ ] I license my contribution under **MIT OR Apache-2.0**, and every file I authored
      carries the two-line SPDX header.
- [ ] **Clean-room attestation.** I did not copy code, comments, or docstrings from
      `external/idaes-pse` or any other reference implementation, and I did not paste
      from any source whose license is not MIT- or Apache-compatible. Where I read a
      reference implementation, I read it to understand behaviour and wrote the
      implementation myself.
