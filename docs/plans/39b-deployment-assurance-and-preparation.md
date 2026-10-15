---
title: Deployment assurance and preparation
status: done
date: 2026-10-14
adrs: [ADR-0164, ADR-0183, ADR-0189]
review_sources: [docs/design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md]
doc_owner: docs/plans/39-implementation-friction-and-codex-effectiveness.md
doc_topics: [architecture, preparation, validation, environment]
---

# Plan 39b: Deployment assurance and preparation

## Purpose and boundary

This companion to [Plan 39](39-implementation-friction-and-codex-effectiveness.md)
owns the corrections for the friction review's [F01](../design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md#f01),
[F02](../design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md#f02),
[F03](../design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md#f03)
and [F06](../design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md#f06).
Worker and imported Python startup must consume the same owned observation operation;
completed strict role captures must remain independently reviewable after later role
preparation; installation must consume the selected Python realization; and nested build
diagnostics must reach the existing live log consumer. Producer assurance must state
which guarantee its actual consumers require before choosing a narrower mechanism.

The coordinator owns rule decisions and finding dispositions. This document owns its
packet state. [Plan 39d](39d-qualification-and-plan38-cutover.md) owns final assembled
qualification, including every remaining Plan 38 obligation, after the selected new
Plan 39 functional work and investigations are complete.
The target below is now implemented for the selected execution slice; its bounded Outcome separates targeted evidence from 39d acceptance.
Blueprint §5.3 owns producer identity and the separate deployment-local guarantee,
blueprint §14.3 owns preparation/reconstruction trust and blueprint §24 owns the
qualification boundary; this plan changes their realization through the decision route.

## Current scope decision

The maintainer withdrew the environment/provenance proof targets on 2026-10-14. Fresh all-role strict capture, source-completeness investigations, executor-input projections, compiler/launcher/installer phase associations and exact installer/environment reproduction are excluded from Plan39 and its inherited Plan38 acceptance. These targets are eliminated, not deferred; no revisit trigger or future campaign is retained. Existing producer code/tests and runtime safeguards remain. Scientific/model/result provenance, explicit Arrow force-validation, physical correctness and truthful cancellation/drain remain required. The completed implementation and its historical test outcomes below remain evidence for their original scope.

## Authoring foundation and focused assessment

Authoring source baseline: `36f15656d`. The review's diagnosis was made at `d5e8606bb`;
its retained historical results remain evidence for that scope, rather than qualification
of the authoring baseline. The source routes below describe that authoring baseline; delivered behavior is recorded in the Outcome. Core 3.4 and Efficient Architecture 1.0 support changing the physical operation
boundary while keeping scientific identity, deployment compatibility, exact artifact
association and outer provenance distinct. Process Simulator 1.5's scientific checks
remain with their existing owners; an associated executable never proves physical or
numerical validity.

| Current owner | Established foundation and remaining gap |
|---|---|
| `crates/pse-runtime/src/math/portable.rs` | `observe_deployment_runtime` already admits receipt/namespace work, checkpoints the original stop/deadline and joins native work. Its retained lease currently covers the complete startup work envelope rather than only escaped observation data. `observe_local_runtime` already has retained-result accounting. |
| `xtask/src/bin/support/deployment.rs` | Actual worker root verifies its executable, but calls synchronous deployment observation, receipt reading and artifact verification before creating the local-only startup interrupt owner. The strict branch bypasses the owned runtime operation. |
| `crates/pse-py/src/workflow.rs::open_recording` | The actual imported root already calls the owned deployment operation and independently verifies the loaded image. Its startup observation lease lives through canonical connection/open. |
| `xtask/src/producer_identity.rs` | Actual selected unit graph, compiler dep-info, context, source consistency and reviewed executor channels support fail-closed strict capture. `macro_caller_input_basis` deliberately includes generated suppliers' compilation closures. Whole-caller grants repeat manual I/O review after unrelated consumed code changes. |
| Same capture owner | `selected_build_output_observations` preserves selected logs and `.d` files under build-script output directories. It does not preserve every rustc unit dep-info and generated caller input needed by independent reconstruction. Later package-family cleanup can remove them. |
| `scripts/producer_deployment.py` and `scripts/python_extension.py` | Capture selects each role and validates actual association. Correctness installation separately invokes Maturin; Python capture later builds independently. `bind_python_artifact` already independently imports the module and checks exact admitted RPATH replay. |
| Capture and validation logs | The outer capture streams to role logs, and validation/result readers already consume live logs. The selected Cargo build inside `producer_identity::run` uses `Command::output()`, withholding child diagnostics until completion. |

The [product inquiry](../design_review/evidence/implementation-friction-and-codex-effectiveness-2026-10-14/product-inquiry.md)
and [tooling inquiry](../design_review/evidence/implementation-friction-and-codex-effectiveness-2026-10-14/tooling-inquiry.md)
supply the wider observations. Their useful distinctions survive this plan: no permanent
Python startup reservation leak was established; increasing a truthful namespace bound
does not itself violate ownership; the historical numeric LLVM suffix cause remains
unproved; and a successfully reproduced warm installer transformation does not prove a
fresh compiler realization. Ordinary development uses the current effective environment.
Historical evidence does not authorize silently restoring an earlier PATH or wrapper.

## Rule decisions and producer guarantee comparison

The original RC01/RC02 selections led to the completed B1–B6 implementation described in the Outcomes. Their further strict campaign, source-completeness and installation-proof targets are withdrawn by the maintainer. Ordinary deployment-local workflows and current runtime checks remain; no cross-build equivalence is claimed by ordinary functional acceptance.

## Target contracts

### Owned startup observation and retained results

The actual composition root supplies loaded artifact identity, actual role, selected receipt
and one original startup deadline/cancellation owner before substantial observation starts.
Worker strict and local branches use `MathService::observe_deployment_runtime`; local
replay admission remains its separate receiving guarantee. Python uses the same operation
while keeping loaded-image checks before and after observation. Store connection/open
remains an explicit subsequent effect with its actual cancellation/acknowledgement owner.
Observation does not mint store permission or publish readiness.

The runtime preflights receipt extent and admits complete work, then reads, parses and
verifies through the checkpointed native owner. Malformed/unknown/changed receipts, role
mismatch, changed loaded bytes/current inputs, insufficient allowance, expired deadline or
cancellation cannot mint a producer or readiness. Preserve the distinction between a
supported unqualified outcome and an operational error; do not turn every startup failure
into `None`. Strict receipt selection must not fall back to local permission after refusal.

Use the existing retained-result job capability to release receipt/namespace scratch only
after actual native completion/drain. Retain only escaped observation strings/hashes and
optional producer data under their actual allocation owner. A cancelled future requests
stop and keeps work ownership until join; finishing the startup signal listener does not
prove that native work drained. Python's retained result should survive connection/open
with this smaller result lease; the former full work-envelope lease is removed. Preserve
original typed stop cause and actual loaded-root safety restrictions.

### Completed preparation and installation changes

B3/B4 and the finite preparation operation are implemented as recorded in the Outcomes. Preserve their current safeguards and historical evidence without scheduling additional capture/source qualification, phase-specific installer proof or complete environment reproduction. Ordinary current installation/import behavior is exercised by 39d.

### Diagnostics reach current consumers

The nested capture build emits named preparation/configuration, refresh, build, sealing and
association transitions to its existing diagnostic stream. Forward Cargo stderr as it is
produced and preserve exact raw stdout JSON for evidence parsing; tee stdout only where an
existing consumer needs it. Never mingle human stage text into the machine evidence stream.
Keep final child status and refusal details even when output is partial. Drain/reap owned
children and close logs on cancellation or failure before releasing process/evidence owners.
A quiet link is not a new hang inference or deadline policy. Existing role logs, validation
live output and result readers remain the consumers; no dashboard or event store is needed.

## Execution packets

| Packet | Delivered behavior | State |
|---|---|---|
| <a id="b1"></a>B1 — Settle selective channel-binding uncertainty | Guarded pure owner selection; original bounded evidence in the Outcome | Done |
| <a id="b2"></a>B2 — Common startup owner and scratch lifetime | Owned worker/Python startup, cancellation and native drain before scratch release | Done |
| <a id="b3"></a>B3 — Protect role evidence and artifacts | Implemented historical evidence protection; further completeness proof withdrawn | Done |
| <a id="b4"></a>B4 — Select and install Python realization | Implemented actual-child capture/association; further installer/environment proof withdrawn | Done |
| <a id="b5"></a>B5 — Apply selected assurance route | Guarded owner behavior and conservative existing strict entry; further strict campaign withdrawn | Done |
| <a id="b6"></a>B6 — Stream nested child diagnostics | Live diagnostics with actual status, partial output and drain | Done |

## Verification and completion

The Outcomes retain commands, conditions and zero-failure results actually observed. They do not establish whole-product qualification. Plan39d owns remaining ordinary runtime/worker/Python, scientific/recovery/interruption and measurement acceptance. Explicit force-validation and current runtime safeguards remain. Additional environment/provenance proof is excluded rather than an unresolved acceptance prerequisite.

## Selected execution checkpoint — 2026-10-14

The authorized execution slice is B1–B6 and C1–C5, including their supporting production changes. 39a and the full 39d/remaining Plan38 campaign remain deferred. B2 startup ownership and retained-result changes, B3 protected historical evidence, B4 actual-Maturin-child capture and B6 raw streaming are integrated; targeted checks and bounded actual composition probes are complete, with the unexamined acceptance stated below.

B1's real pinned-rustc Paste probe demonstrated generated environment selection absent from compiler dep-info. Stock Paste has no pure switch; stock Pest cannot provide the current py_literal file workflow through a pure entry point. The operator explicitly selected guarded pure owners on 2026-10-14. [ADR-0189](../adr/0189-pure-macro-producer-channels.md) and the [independent bounded target review](../design_review/reviews/design_review_plan39-pure-macro-channels_2026-10-14.md) precede dependent assurance changes. Paste refuses every Env segment before access; Pest validates actual literal inline attributes and calls the existing generator with inclusion disabled; py_literal preserves its exact published grammar in one inline authority. Tokens-only includes supplied compiler token/span context and still requires current exact source/helper/feature review. Relevant caller code remains scientific identity. Unknown or effectful owners retain conservative rules. Exact local transitive source pins are governed by matching vendor manifests and selected lock entries; replacing the old blanket patch ban is recorded as RC03 in the bounded review.

Independent implementation review found and corrected missing cross-root alias/context evidence, a dry-run observer configuration, a release-only parser regression and Rustup proxy invocation ambiguity. Historical namespace inventories explicitly retain topology/digest framing; only separately consumed raw members are copied. Historical success never proves current freshness. Final all-role current association may still refuse an earlier role after a later executor refresh; no further strict campaign is scheduled; ordinary functional acceptance does not claim strict current association.

## Selected execution outcome — 2026-10-14

**Implemented:** B2 removes synchronous worker receipt/observation and store-first composition. Worker strict/local startup shares the admitted runtime owner with Python, under one original cancellation/deadline before observation. Native work joins before scratch is released; only escaped hashes/strings/result ownership remains charged through subsequent store open. Strict refusal never falls back to Local. Python keeps independent loaded-image checks and cancellation checkpoints around store effects.

**Implemented:** B3 seals a versioned historical sidecar with raw unit graph/metadata/argv/streams/status, per-unit compiler dep-info associations, consumed generated inputs including outside package roots, exact original versus protected file identities, selected source/helper reviews, protected role bytes, complete namespace topology/digest framing, cross-root aliases and entry/target absence. Only consumed raw members are copied; namespace inventories are explicitly not full raw namespace copies. The historical reader reconstructs those premises after original Cargo paths disappear. Deployment validation and RPATH extraction consume the protected context, with no live legacy-JSON fallback. Current artifact/source/configuration/environment qualification remains independent and conservative.

**Implemented:** B4 makes Maturin's actual Cargo child the sole selected Python compilation. One private single-use adapter records actual argv/cwd and allowlisted SDK/compiler/provider selections, protects explicit PyO3 configuration, and requires qualified sealed raw evidence before installation continues. Actual sole-cdylib graph validation supports Maturin's manifest-selected omission of `--crate-type`; explicit conflicting or unknown contexts refuse before build. Cargo proxy invocation identity is preserved. Independent loaded import/full SHA and exact RPATH-only replay remain mandatory, with transformation evidence retained. The superseded strict pre-install/independent-recapture ordering is removed; ordinary editable sync is unchanged.

**Implemented:** B5 installs exact pinned guarded Paste/Pest/py_literal sources and source-qualified `tokens-only.v1` review for the three current strict role contexts. Whole-caller grants disappear only for these current exact pure owners/helpers/features. Stock/unknown/changed owners remain conservative. Caller-body changes still rekey scientific identity; owner/helper/feature changes revoke the grant. Paste's unchanged compiler-version build effect retains its separate review. No native/tool completeness was inferred from observed hashes. The root local-source policy checks vendor containment, exact package/version and unique actual lock selection.

**Implemented:** B6 concurrently drains/spools exact child stdout/stderr, forwards stderr and rendered ordinary Cargo JSON compiler diagnostics before exit, and preserves real status/partial streams. Maturin receives the original JSON without duplicate local rendering. Preview limits do not validate or discard raw evidence. Existing live/result logs remain the consumers; incomplete capture never seals successful history.

**Tested, zero-failure baseline:**

- `PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw just unit-package pse-runtime deployment_startup`: six passed, 491 outside the filter. Includes original expiry, cancellation/abandonment through actual native drain, receipt bounds/stop and exact retained extent.
- Force-validated worker/xtask build (`scripts.arrow_validation build -p xtask --bins --no-default-features --features package-fixtures --locked`) passed. Force-validated `check -p pse-py --no-default-features --locked` passed. These compile selected consumers, without qualifying their scientific journeys.
- Force-validated xtask Nextest filter `test(producer_streamed_child_) | test(producer_historical_) | test(producer_observed_installer_arguments_) | test(producer_tokens_only_) | test(producer_dynamic_macro_review_) | test(producer_pest_preferred_)`: 14 passed, 53 outside the filter. Includes actual tiny worker/cdylib executor refresh, generated inputs and macro caller reconstruction after original dep-info/workspace/legacy JSON deletion, plus substituted associations, tampering/absence and live diagnostic ACK controls. These captures deliberately remain unqualified for native execution.
- `RUSTC_WRAPPER= scripts/pse-env --recipe-default-class compile -- python3 scripts/tests/fixtures/pure-macro-channels/qualify.py`: 42 tests and twelve expected compiler refusals across debug/release, zero warnings/failures. The exact published py_literal grammar SHA is independently checked. This isolated pure closure has no Arrow or solver dependency and uses its own target.
- Explicit-light unittest selection for `test_producer_cargo`, `test_producer_deployment`, `test_python_extension` and `test_validation`: 90 passed. Additional correction controls cover actual Cargo jobs/color, same-inode proxy ambiguity and deleted legacy-context inspection. Expected refusal fixtures are assertions, not failing qualification.
- Force-validated `nextest run -p pse-tests-governance --test no_patch_tables --locked --no-fail-fast`: two passed. Exact local source/lock policy and existing CPU/linker constraints survive.

**Tested, actual worker composition:** an owned private profile and counting socket endpoint exercised the built worker. A malformed receipt returned the receipt parse error with zero connections. A separate 128 MiB sparse receipt was interrupted only after `/proc/<owned-child>/fd` showed its actual receipt reader open; the worker returned `math evaluation cancelled`, with zero connections. Both enclosing admission scopes completed successfully after owned process drain. Evidence is `/tmp/pse39-worker-startup-vgsfrwk_` and `/tmp/pse39-worker-cancel-m92cpz9b`. No canonical server was started or altered.

**Tested, actual installer capture/refusal:** actual pinned Maturin/PyO3 with the current capture tool compiled a small nested extension, then reused it Fresh. The command was `scripts/pse-env --resource-class compile -- .venv/bin/python /tmp/pse39-maturin-real-lo0g0_cm/run-probe.py compiling-nested fresh-nested`, with PyO3 0.29.2, the pinned Rust toolchain, isolated Python 3.14 and explicit empty Rust wrappers. Both Cargo children succeeded, each sealed historical closure independently verified (20 unit associations, 1206 protected members), and both preserved the same raw artifact/identity. Deliberately incomplete native/macro declarations made the strict adapter refuse installation; post-refusal import remained unavailable. The first probe exposed Maturin's omitted crate-type switch and was preserved separately before correction. Evidence is `/tmp/pse39-maturin-real-lo0g0_cm/probe-observations.json`. This establishes compiling/Fresh selection, raw protection and truthful refusal, not successful qualified installation.

**Interface-checked:** source-grant expansion in the current runtime, worker and Python producer-profile unit graphs selects the guarded owners without caller grants or macro-channel refusals, while overall eligibility remains false without complete native/current-input evidence. No broad native completeness grant or historical reinterpretation is minted.

Remaining functional acceptance is with 39d: ordinary installed Python workflows, assembled worker/Python/store journeys, selected hygiene/manual/scientific/parity checks and measurements. Strict qualified import, all-role association and environment/source-completeness proof are withdrawn. In particular, later executor refresh may invalidate an earlier role's current inputs even though historical evidence remains intact. These results do not close the principal findings or the whole Plan39/remaining Plan38 campaign. 39a was not implemented in this slice. The implemented source/form restrictions remain in proposed ADR-0189; no additional proof/revisit target remains; ADR status is unchanged.

Final selected integration: after root formatting, the combined affected operational/adapter/validation unittest selection passed **207 tests**, and the same force-validated xtask filter passed **14 tests** again, including the actual omitted-crate-type regression. `just ready` and `just turn-end` returned zero failures. Documentation publication rendered/indexed **441 chapters**; native scope extraction for Plan39 reported **11 complete, 17 open, zero unknown**, with no errors. These are selected code/document checks; full 39d static/scientific qualification remains unexecuted.

## Remaining-design execution outcome — 2026-10-14

**Implemented:** the finite preparation operation replaces destructive per-role executor refresh. The proposed ADR-0189 amendment and [independent preparation target review](../design_review/reviews/design_review_plan39-preparation-operation_2026-10-14.md) precede this correction. Target-local exclusion is acquired before dependency setup; the selected capture tool is bootstrapped before the source/native baseline. The operation refreshes the bounded selected executor-family union once, then tracks actual exact executor compilations. A first context requires non-Fresh successful compiler evidence. Later Fresh reuse requires preceding compilation within this operation and unchanged dependency-complete source, reviewed inputs/environment/native/absence and original executable/output premises. Maturin's actual child context remains authoritative; unknown families refuse. Selected writer descendants retain the guard; it neither replaces host admission nor excludes arbitrary bare Cargo.

The original output/evidence premises are checked before a known executor can run again and after capture. Failed/incomplete and completed operations are terminal. Historical sidecars retain preceding compilation and its original premises before operation state commits; original-path current verification remains independent. Build-script output JSON alone is not proof that the script executed in that child. The implementation keeps full native/SDK distinctions conservatively; it does not guess a projection to make roles share a context or grant native completeness.

**Tested, zero-failure baseline:** `PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw scripts/pse-env --resource-class compile -- .venv/bin/python -m scripts.arrow_validation nextest run -p xtask --bin xtask --no-default-features --locked -E 'test(producer_preparation_) | test(producer_streamed_child_) | test(producer_historical_) | test(producer_observed_installer_arguments_) | test(producer_tokens_only_) | test(producer_dynamic_macro_review_) | test(producer_pest_preferred_)' --no-fail-fast` passed **23**, with **53 outside the filter**. Nine new operation controls exercise finite sharing, terminal/concurrent ownership, guard descendants, source/native/absence/output mutation and failed children; fourteen retained controls preserve historical reconstruction, installer selection, macro qualification and diagnostic streaming. Tiny actual compiler fixtures deliberately remain native-unqualified. The force-validated `scripts.arrow_validation check -p pse-runtime -p xtask --all-targets --no-default-features --locked` also passed for the selected integrated packages before the final targeted corrections.

**Tested:** the explicit-light combined unittest selection for producer adapters, Python extension, validation, environment, activity, host admission, manager/drain observation and the two startup-status controls passed **211**. The command was `PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw scripts/pse-env --resource-class light -- .venv/bin/python -m unittest scripts.tests.test_producer_cargo scripts.tests.test_producer_deployment scripts.tests.test_python_extension scripts.tests.test_validation scripts.tests.test_pse_env scripts.tests.test_activity scripts.tests.test_host_admission scripts.tests.test_native_operation.ManagerObservationTests scripts.tests.test_native_operation.DrainObservationTests scripts.tests.test_surreal_server.SurrealSupervisorTests.test_new_launch_reobserves_unavailable_listener_under_original_clock scripts.tests.test_surreal_server.SurrealSupervisorTests.test_validation_launch_keeps_published_admission_closed`. Negative fixture diagnostics were expected assertions; the enclosing test command exited successfully. No canonical server or fixture was activated.

**A mistake made and corrected:** the first 23-control run passed 22 and failed one: a tampered cached build-script executable reached Cargo before the later consistency check rejected it. The check now precedes child effects, and mutation controls require absence of a second stdout spool, status and raw build evidence. The complete selected rerun passed 23. Independent implementation review also corrected late capture-tool bootstrap, missing native review assertions and descendant guard retention.

**Deliberate boundary at this historical slice:** actual all-role PSE captures, strict qualified installation/import and source/native/SDK completeness were not run; those additional proof targets are now withdrawn. Remaining functional/scientific/recovery/signal checks and measurements belong to 39d. Existing strict mechanisms may still truthfully refuse cross-role reuse; resolving that refusal through further source-qualified assessment is excluded from D1. This completes the remaining preparation design correction, not F02/F03 closure or 39d acceptance. Formal ADR status is unchanged.

Root `just turn-end` exited zero after integration (ADR index and formatting). Its advisory Ruff auto-fix stage printed **19 remaining diagnostics** against the zero target; that stage uses `--exit-zero`. These remain for D3's explicitly deferred static repair/qualification, rather than becoming a quality baseline or a claim of lint success. Four owned Python files received formatting and one equivalent literal-concatenation safe fix. No deferred static/manual or scientific qualification was established.
