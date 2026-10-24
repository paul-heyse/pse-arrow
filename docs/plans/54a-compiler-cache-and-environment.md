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

Extend the existing diagnostic-tool declaration with sccache 0.19.0, the official
x86_64-unknown-linux-musl archive, its selected member and verified archive SHA-256.
The verified official archive digest is
aa73c7270c997894038c0d8f0d297865f57569e1687883322bc431713d4bb9dd. Use the
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

sccache 0.19 uses Rust cache-key version 7 and C-family version 13. Populate new Rust
entries under the selected release; no compatibility with earlier Rust keys is assumed.
This is not an instruction to clear shared caches, delete native preparations or force an
otherwise unnecessary Cargo rebuild. The selected official executable supports local disk
storage and client-side compilation.

**Acceptance, proposed:** extend the existing diagnostic installation/bootstrap/doctor and
scripts/tests/test_build_environment.py controls for a valid managed install, missing tool,
wrong version or changed installed bytes, competing PATH binary, explicit real-binary
override, disabled/custom wrapper, and nested environment composition. Execute the actual
selected 0.19.0 binary with just build-cache-probe: one eligible miss then hit and incremental
pass-through on a disposable endpoint. This establishes transport/selection, not workspace
benefit. Run just ready after installing/changing the selected tool as the environment route
requires. No wholesale dependency update or shared cache clearing is part of A01.

## A02 — Preserve compiler ownership under logging and mode changes

Fix CB09 at both build_environment.configure and compiler_cache.supervised_compile.
The tagged [sccache configuration](https://github.com/mozilla/sccache/blob/v0.19.0/src/config.rs#L1412-L1419)
checks presence of LOGGING_ENV, defined as
[SCCACHE_LOG](https://github.com/mozilla/sccache/blob/v0.19.0/src/lib.rs#L60-L62),
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

The exact [Rust key construction](https://github.com/mozilla/sccache/blob/v0.19.0/src/compiler/rust.rs#L1839-L1882)
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

**Implemented:** the selected tool is sccache 0.19.0 through the existing versioned installer.
The declared official archive digest is verified above; executable SHA-256 is
0b44834be857e21b026f7db49a7762023fedb5eb5ed60725a66f698d8d79a34b.
Installation/doctor verify the managed bytes, and ordinary service admission reuses its
existing digest calculation to refuse drift before allocation. Explicit foreground probes
bypass that service-admission boundary. cargo-shear remains exactly 1.14.0.

**Implemented:** shared mode eligibility covers logging and scheduler controls at environment
composition and final managed launch. Automatic direct fallback preserves caller settings;
explicit cache requests refuse unsupported modes. Both agent defaults leave CARGO_TERM_COLOR
unset, with explicit caller values preserved. Current guidance lives in the
[build-performance](../dev/build-performance.md) and [environment](../dev/agent-environment.md)
guides. Existing sessions require refresh.

**Tested:** zero-failure baseline, current Linux host and pinned nightly, actual sccache 0.19.0:

- `scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_diagnostic_trace scripts.tests.test_setup.ConfigurationTests` — 24 passed, zero failures, including verified install/default/override controls.
- `just diagnostic-tools sccache` and `just build-cache-probe` — passed; eligible Rust one miss/one hit and incremental pass-through, zero cache errors.
- `just ready` — passed, zero failures; doctor selected the verified managed default. Evidence: `build/assessment/20261024T200730.588851Z-ready-3240367-9090bc/`.
- `scripts/pse-env -- .venv/bin/python /tmp/pse-real-cache-acceptance.py` — four real isolated control groups passed, zero failures. Empty/nonempty SCCACHE_LOG refused before service descriptors; repeated compilation produced one miss/one hit. Actual rustc processing the requested source remained in the caller's allocation, outside the daemon's allocation. SIGTERM returned 143 and drained observed descendants while the daemon survived. Owned service/unit/allocation cleanup completed.
- `scripts/pse-env -- .venv/bin/python /tmp/pse-a03-callers.py` — passed, zero failures. Actual terminal PTY and fresh ephemeral Codex session selected 0.19.0, with identical rustc argv and relevant CARGO inputs. CARGO_TERM_COLOR was absent in both; final statistics one miss/one hit, zero cache errors. The private server was reaped. A fresh Claude runtime was not separately exercised.

Current scripts/receipts are retained under `build/assessment/plan54a-20261024/`; superseded
active version evidence was replaced. Tagged 0.19 source retains SCCACHE_LOG-presence and
scheduler restrictions. Its Rust dep-info is generated locally. Client-side statistics can
show compile_requests=0 while requests_executed=2; measurements use actual hit/miss counters
and executed requests rather than treating compile_requests as a universal denominator.

**Measured:** eligible controlled reuse only. No workspace speedup, relocation remedy or
whole-product qualification is claimed. The remaining 54b–54d scope owns actual graph/freshness,
cache economics and joined qualification. This companion remains for those active readers.
