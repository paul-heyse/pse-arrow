<!--
Thanks for contributing. Fill every section — a blank box is not an answer, but
"no ADR needed because …" and "N/A because …" are. See CONTRIBUTING.md §3.

The *squash title* of this pull request becomes the changelog entry and is checked by
`governance / pr-title`. Use a Conventional-Commit subject:
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
- **Closes:** <!-- #123, or N/A -->
- **Register rows touched:** <!-- R-NN, or none -->

## Evidence

<!--
Label every claim with the §D vocabulary and NAME the test or benchmark:
  Proposed | Interface-checked | Tested <test name> | Measured <bench name> + conditions
  | Observed <link to a CI run or log>
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
- [ ] If a dependency family moved (arrow, datafusion, pyo3, object_store): the ADR
      exists, the capability map under `docs/capability-maps/` was regenerated, and
      `just family-check` passes.
- [ ] If the IDAES parity pin moved: GOVERNANCE.md §5 was followed and every tolerance
      change is called out individually above.
- [ ] No edits under `docs/generated/`, `crates/*/src/generated/`,
      `python/pse/contracts/`, `external/`, `build/`, or `target/`.
- [ ] Documentation updated where behaviour changed; new files follow the naming and
      citation conventions (CONTRIBUTING.md §7).
- [ ] `just ci-pr` is green locally.

## Legal

- [ ] I license my contribution under **MIT OR Apache-2.0**, and every file I authored
      carries the two-line SPDX header.
- [ ] **Clean-room attestation.** I did not copy code, comments, or docstrings from
      `external/idaes-pse` or any other reference implementation, and I did not paste
      from any source whose license is not MIT- or Apache-compatible. Where I read a
      reference implementation, I read it to understand behaviour and wrote the
      implementation myself.
