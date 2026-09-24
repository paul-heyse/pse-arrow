---
title: Current Plan 14 validation and evidence
status: implemented
date: 2026-09-24
---

# Current Plan 14 validation

**Implemented, unqualified:** the assessment runner collects the active Plan 14
campaign. M19–M20 supplies executable cases and evidence controls; M21 must close
implementation/deletions before M22 runs the campaign. The
[acceptance guide](architecture-acceptance.md) gives the sequence. Earlier Plan 10–13
campaigns and seals cannot authorize or satisfy this target.

`just assessment-list --phase functional` and `just assessment-list --phase performance`
print declared gates, profiles, cases and exclusions. Listing reads the manifest;
it does not compile, import `pse`, execute tests, seal sources or create receipts.
`just plan14-discover` is a distinct operation: it compiles/enumerates Nextest tests
and collects pytest identities without executing them.

## Scope and order

`scripts/validation_scope.py` owns explicit functional/performance classifications
and expands aggregate recipes. `scripts/implementation_phase.py` checks closed
inventory rows and a fresh source-bound implementation seal. M00–M21 and X01–X12
must each occur once with status `complete`; text elsewhere in an evidence column
cannot close a package. Native process recipes and aggregate campaigns
use the same phase guard. Isolated units, compilation and pure regeneration remain
available during implementation.

Functional qualification includes current format/lint/governance/generation/family/
documentation checks, workspace and native tests, release/doctest/feature/coverage
profiles, linked Python workflows and surviving inspection/storage consumers. The
runner first installs the native editable Python profile with `py-sync-native`.
The explicit `plan14-native`, `plan14-python` and `plan14-tools` gates own target
case witnesses. Other configurations remain useful repository checks; a default
build cannot stand in for a linked native case. All Rust test profiles use explicit
Arrow force-validation. The failure baseline is zero.

Performance contains only the current complete-process measurements and independent
G1–G7 review collection. It requires successful current-source functional evidence;
a functional campaign containing a benchmark cannot evade that ordering. Historical
benchmarks remain available only under the same phase constraints and do not
substitute for the current process workloads.

The campaign covers the pinned Linux development environment. Remote wheel/platform
matrices, alternative toolchains, IDAES parity environments, solver-image rebuilding,
cloud/destructive operations and unselected mutation campaigns require separate
scope. `assessment-list` carries the exact exclusions. Native capability and scientific
qualification are separate from distribution/platform qualification.

## Evidence and failure collection

Each new gitignored output directory retains scope, per-check logs, fresh reports,
source provenance, `checks.json` version 3, review indexes and artifact hashes.
Existing output directories are never overwritten. Independent gates continue after
failures; interruption leaves incomplete evidence. Compilation and environment
prerequisites can fail, and their dependent outcomes remain visible.

Nextest JSON enumeration and pytest collection establish exact selected identities.
A required case needs every named witness once with passed status, in its declared
mode/profile. Missing, duplicate, skipped, wrong-profile or interrupted results do
not qualify. Requirement coverage is separate from command exit status. Manifest
rows remain `implemented-unqualified`; only authenticated execution proves coverage.

One bounded product-input inventory owns source sealing and continuation. It includes
source, manifests/locks, generated contracts, reference packages, selected governance
skills/rules, plans and design documents. Hashes preserve additions, removals, executable
modes and symlink destinations. Build output, credentials in `.envrc.local`, local
library-reference corpora and unrelated workspace state are excluded. The archive and
diff use the same path scope. Native profile artifacts bind actual executable/extension
and resolved linked-library hashes, toolchain and serial native thread settings.
They never retain the Symbolica license value.

Criterion reports bind 12 distinct cold/warm size/thread workloads, at least ten
samples per workload, confidence intervals, complete operation/teardown timing,
tracked pool peak, process RSS and raw artifacts. Pool memory and RSS have different
scopes. Review artifacts retain independently authored decisions and copies of their
cited evidence; identity, source digest, scope, verdict and unresolved mandatory
findings are checked. Neither collector manufactures a review verdict.

A command can finish while qualification fails. `complete` means every declared gate
was attempted; `required_checks_covered`, exact case coverage, source stability and
artifact authentication must also pass. Advisory dependency findings retain their
advisory status; tool failures remain failures. The zero-finding target is unchanged.

## Repairs and continuation

Use a fresh output directory with `--resume-from`, repeat `--rerun` for all invalidated
gates and state `--change-reason`. A changed implementation needs current M21 closure
and a new seal before qualifying it. Retained results must remain compatible with the
current scope, source changes and their authenticated origin/artifact chain. A prior
passing summary alone is insufficient. Performance receives the complete functional
campaign through `--functional-from`; missing or stale coverage is refused.

The isolated controls are `.venv/bin/python -m unittest
scripts.tests.test_implementation_phase scripts.tests.test_validation
scripts.tests.test_plan14_acceptance`. They test phase, source, exact identity, report,
continuation, measurement and independent-review contracts without running the product.
They establish tooling behavior, not scientific or integrated acceptance.
