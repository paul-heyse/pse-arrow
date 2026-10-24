---
title: "Plan 54a: Compiler cache and environment"
status: complete
date: 2026-10-24
adrs: []
review_sources: [docs/design_review/reviews/design_review_rust-build-reuse_2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/54-rust-build-reuse.md
doc_topics: [current-work, environment, preparation, libraries]
---

# Plan 54a: Compiler cache and environment

## Target and foundation

**Implemented:** one deliberate compiler-cache executable and execution mode for ordinary
developer, agent and nested workflow invocations. The [coordinator](54-rust-build-reuse.md)
owns source dispositions, scenarios S01–S03 and combined completion. This companion owns
the package state below. It changes neither Cargo's artifact authority nor provider-cache
ownership and introduces no replacement daemon manager.

The maintainer authorized execution of this companion on 2026-10-24. A01–A03 are complete
at the bounded acceptance below. The enclosing series and its broader qualification remain
open; this companion is retained for those active consumers.

**Implemented:** scripts/build_environment.py selects the declared managed sccache and
routes it through scripts/compiler_cache.py; that owner retains private endpoints,
executable/configuration generations and independent cache-service lifetime. Tool versions
and binary assets already have an established declaration in .config/diagnostic-tools.toml.
Bootstrap and doctor consume that declaration. The installed Cargo tool inventory is not
sufficient to verify a prebuilt non-Cargo binary.

## A01 — Declare, install and actually select the pinned tools

Extend the existing diagnostic-tool declaration with sccache 0.18.0, the official
x86_64-unknown-linux-musl archive, its selected member and verified archive SHA-256.
The source review reports archive digest
45f1447fbe231e3037bde351ef70677dd212216c8d62ae7ca409fecc4d6acc89;
verify it against the downloaded official asset before publishing the declaration. Use the
existing versioned diagnostic-tool installation machinery, extending its supported tool
names/bootstrap route as needed. Do not create an independent installer or cargo-install
pin for the same binary. Record installed executable identity through that owner so doctor
can detect altered bytes as well as a wrong version. Pin cargo-shear 1.14.0 in the existing
cargo-tools table and consume that version through existing cargo_tools selection.

Make the shared environment select the managed installed executable by default, rather
than allowing an unrelated PATH sccache to win after installation. Bootstrap, doctor and
actual RUSTC_WRAPPER/PSE_SCCACHE_BINARY selection must agree. Preserve deliberate explicit
choices: an empty RUSTC_WRAPPER disables wrapping, a custom wrapper remains explicit, and
PSE_SCCACHE_BINARY selects an alternative real ELF executable when the managed sccache
route is selected. An alternative is reported as an override with its actual version/path,
not passed off as the pinned default. Ordinary explicit cache=on still requires availability;
ordinary cache=auto retains supported direct compilation when the optional tool is absent.
Doctor diagnoses missing/drifting managed installations and does not silently install them.

The cache owner already snapshots the exact executable/configuration into its service
generation. Keep that boundary: activation of a new binary readmits the appropriate owned
generation, without killing another operation's daemon or flushing the shared cache. Do not
use per-build full archive verification; installation/doctor establish the managed asset,
and the existing executable-generation checks protect actual use.

sccache 0.18 retains Rust cache-key version 6; the original review measured compatibility
on its selected Rust probe. C-family key version changes from 12 to 13. This makes old
C/C++ compiler-cache entries unavailable to new matching invocations; it is not an instruction
to delete existing native preparations or force an otherwise unnecessary rebuild. Backend
features of the installed release must support the selected local cache/client-side path.

**Acceptance, proposed:** extend the existing diagnostic installation/bootstrap/doctor and
scripts/tests/test_build_environment.py controls for a valid managed install, missing tool,
wrong version or changed installed bytes, competing PATH binary, explicit real-binary
override, disabled/custom wrapper, and nested environment composition. Execute the actual
selected 0.18.0 binary with just build-cache-probe: one eligible miss then hit and incremental
pass-through on a disposable endpoint. This establishes transport/selection, not workspace
benefit. Run just ready after installing/changing the selected tool as the environment route
requires. No wholesale dependency update or shared cache clearing is part of A01.

## A02 — Preserve compiler ownership under logging and mode changes

Fix CB09 at both build_environment.configure and compiler_cache.supervised_compile.
The tagged [sccache configuration](https://github.com/mozilla/sccache/blob/v0.18.0/src/config.rs#L1412-L1419)
checks presence of LOGGING_ENV, defined as
[SCCACHE_LOG](https://github.com/mozilla/sccache/blob/v0.18.0/src/lib.rs#L60-L62),
before enabling client-side compilation. An empty SCCACHE_LOG also disables it. A configured
distributed scheduler disables it too. SCCACHE_ERROR_LOG is separately restricted by the
repository's existing policy; do not misdescribe the two variables as identical upstream
switches merely because both are incompatible with the supported ordinary route.

Ordinary cache=auto with an incompatible selected mode uses the documented direct compiler
fallback, including native wrapper selection as needed. Explicit cache=on refuses before
compiler/service work with a meaningful explanation. Do not silently remove a caller's
logging setting or let an explicit experiment measure server-side compilation under a
client-side label. The final managed invocation boundary must reject incompatible settings
even if the caller bypassed environment composition. Reuse a single eligibility decision
where that removes duplicated policy; endpoint/owner observations remain at their boundaries.

Extend the existing control of configured scheduler selection to the final invocation seam
so supported direct invocations cannot bypass mode guarantees. Do not infer a safe mode
from SCCACHE_CLIENT_SIDE=1 alone. Ordinary diagnostics can use a separately owned foreground
probe server in explicitly labeled server mode; they must not reconfigure or restart the
shared developer daemon. Keep the current non-direct C/C++ policy, caller cancellation/drain,
and independent cache-service allocation.

**Acceptance, proposed:** targeted existing build_environment/compiler_cache tests cover
SCCACHE_LOG absent, empty and nonempty; existing SCCACHE_ERROR_LOG restrictions; selected
scheduler configuration; ordinary fallback; explicit cache refusal; and bypass of initial
composition. A real isolated client/server control observes the intended compiler parent/
allocation and cleanup during interruption. Reuse existing supervision tests and native
cache-owner fixtures rather than inventing another process manager. An observed safety
gap is not closed by a mocked assertion that merely repeats the new predicate.

## A03 — Remove accidental caller differences

Remove CARGO_TERM_COLOR=never from the Claude and Codex environment defaults. Cargo's
automatic presentation mode already handles ordinary nonterminal agent shells. New sessions
must receive the updated environment; deleting configuration does not change existing
processes. Preserve explicit human presentation choices, and distinguish their requested
semantics from repository-created defaults when reporting key differences.

The exact [Rust key construction](https://github.com/mozilla/sccache/blob/v0.18.0/src/compiler/rust.rs#L1554-L1595)
hashes dep-info environment inputs and almost all CARGO_* variables. CARGO_TERM_COLOR and
CARGO_TARGET_DIR are not excluded. Inspect actual equivalent caller inputs to identify
other repository-created accidental differences, but do not add a blanket environment
scrubber or normalize meaningful profile/validation/coverage settings. env!/option_env!
can make a value observable even when its conventional purpose is operational. D01 owns
measurement-specific target selection; A03 does not redesign explicit PSE_CARGO_TARGET_DIR.

**Acceptance, proposed:** compare one refreshed agent and terminal invocation of the same
eligible selection using a recording wrapper/probe-owned endpoint. Account for effective
rustc argv and relevant environment, including absent versus explicitly set variables.
Populate the controlled cache and demonstrate reuse by the other equivalent caller.
Report avoided duplicate key variants without claiming a twofold end-to-end speedup.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| A01 | complete | 54a | Declared installation, default/override selection and actual selected-binary canary; Outcome below |
| A02 | complete | 54a | Logging/scheduler handling at both boundaries and actual ownership/interruption control; Outcome below |
| A03 | complete | 54a | Refreshed equivalent caller inputs and eligible cross-caller reuse; Outcome below |

A01 and A02 establish the basis for later cache experiments. The implementation and acceptance
below do not close the remaining series packages or qualify scientific/product workflows.

## Verification and Outcome — 2026-10-24

**Implemented:** A01 uses the existing diagnostic installer and a versioned managed path,
never implicit PATH discovery for the default. The downloaded official archive matches the
SHA-256 above; its extracted executable matches the declared binary SHA-256
973cb15f6a986d84ca334bbed3bbe2eb8f1ee8fd81bf9e115b8539a293bf8d59.
The executable digest is declared beside the archive rather than recorded in another mutable
receipt. Installation and doctor verify those bytes; ordinary cache admission reuses its
existing digest calculation to reject drift before service allocation. Explicit foreground
probes bypass service admission and are not evidence that every control command verifies bytes.
The declaration also pins cargo-shear 1.14.0; it was already installed at that version.
No lockfile or dependency graph changed.

**Implemented:** A02 shares one logging/scheduler eligibility decision at environment
composition, ordinary managed entry and final launch. Automatic fallback preserves logging
settings; explicit caching refuses incompatible modes. A03 removes CARGO_TERM_COLOR from both
agent defaults while preserving explicit callers. Enduring operation/selection guidance lives
in the [build-performance](../dev/build-performance.md) and
[environment](../dev/agent-environment.md) guides. Existing sessions retain inherited values.

**Tested:** zero-failure baseline; current Linux host, pinned nightly, actual sccache 0.18.0:

- `scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_diagnostic_trace scripts.tests.test_setup.ConfigurationTests scripts.tests.test_cargo_tools scripts.tests.test_build_environment.BuildEnvironmentTests scripts.tests.test_compiler_cache scripts.tests.test_build_cache_probe scripts.tests.test_pse_env.PseEnvTests scripts.tests.test_semantic_navigation scripts.tests.test_codex_session` — 106 passed, zero failures. Covers installer archive/member/byte controls, missing/default/override/disabled/custom selections, nested composition, logging/scheduler refusals, ownership fixtures and adjacent consumers.
- Independent implementation review found doctor would execute a non-ELF override that the compiler boundary rejected. Both now use the same stdlib-only real-executable validator. `scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_setup.ConfigurationTests` after the final doctor integration — 14 passed, zero failures, including refusal before override execution.
- Scope-end Ruff inspection of the eight changed Python files initially reported eight findings (configuration error handling, bootstrap import placement and type-only/test imports). Corrected all eight without weakening the checks. `scripts/pse-env -- .venv/bin/ruff check scripts/build_environment.py scripts/compiler_cache.py scripts/diagnostic_trace.py scripts/doctor.py scripts/tests/test_build_environment.py scripts/tests/test_compiler_cache.py scripts/tests/test_diagnostic_trace.py scripts/tests/test_setup.py` — passed, zero findings. `scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_build_environment.BuildEnvironmentTests scripts.tests.test_compiler_cache scripts.tests.test_setup.ConfigurationTests` — 42 passed, zero failures after these changes, including unsupported configuration table shapes using the same fallback/refusal contract.
- `just diagnostic-tools sccache` — passed; actual official binary installed through the declared owner. `just build-cache-probe` — passed; one eligible Rust miss then hit and incremental pass-through on a disposable foreground endpoint, zero cache errors.
- `just ready` — passed, zero failures; compiler-cache doctor reports the verified managed default. Evidence: `build/assessment/20261024T195248.125584Z-ready-3187178-7e515f/`.
- `scripts/pse-env -- .venv/bin/python /tmp/pse-real-cache-acceptance.py` — four isolated real-binary control groups passed, zero failures. Empty/nonempty SCCACHE_LOG refused with status 125 before service descriptors; repeat compilation produced one miss and one hit. Observed rustc processing the requested source in the caller's cgroup, separate from the daemon. SIGTERM returned 143, drained observed descendants and left the daemon alive. Cleanup stopped only the probe's service, removed its authenticated unit and released its allocation.
- `scripts/pse-env -- .venv/bin/python /tmp/pse-a03-callers.py` — passed, zero failures. A real terminal PTY populated a private foreground cache; a fresh ephemeral Codex CLI session invoked the same pinned rustc selection and reused it. Exact argv and relevant CARGO environment matched, CARGO_TERM_COLOR absent in both, terminal TTY true and agent TTY false; final statistics one miss/one hit, zero errors. Only the launch parent dropped the old inherited repository default; the helper did not scrub caller CARGO inputs. Both repository config defaults were removed; a fresh Claude runtime was not separately exercised.

Probe scripts and bounded receipts are retained under `build/assessment/plan54a-20261024/`.
The first ownership observer failed because it scanned only a process's main-thread child
list. Its diagnostic is retained; scanning every thread's children corrected the observer.
A subsequent stricter observation selected rustc processing the requested source, excluding
compiler-discovery subprocesses. This was an acceptance-probe correction, not a product repair.

**Measured:** controlled eligible invocation reuse only. No complete-workflow timing, cache
economics, relocation support, workspace speedup or twofold benefit is claimed. F01/OUT_DIR,
Cargo freshness/dependency changes, campaign target normalization, strict shear adoption and
assembled qualification remain with 54b–54d. Scope-end hygiene and comprehensive product
qualification are not run here; D02 owns joined series acceptance.

**Tested:** `just turn-end` passed with zero failures (ADR index and formatting); evidence
`build/assessment/20261024T195559.129698Z-turn-end-3200205-462645/` after the scoped repairs.
`just docs` published 653 chapters with scoped search; `scripts/pse-env --docs -- python3 -m scripts.document_lifecycle validate`
passed with zero errors and 1,355 native scope items validated in the concurrent working tree.
No completed document is retired: this companion is consumed by active 54b–54d and the
coordinator, while the broader source reviews retain unresolved work.
