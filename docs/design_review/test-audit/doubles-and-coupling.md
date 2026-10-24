# Doubles, seams and contract coupling

Findings about what tests observe relative to the contract they protect. They cover
test-only paths and seams in production source, and fixtures that must be kept in step with
production by hand. Baseline and limits are in the [collection README](README.md). Citations
are `path:line` at source commit `cbda2efc6`.

## <a id="dta-008"></a>DTA-008 — Static routing tests select through a test-only eligibility composition instead of the production assessment path

**Principles:** AP-02, AP-06; companion §4, §5.

**Claim at risk.** The static class/rank routing rules hold on the production selection path,
and stubs "route through the adapter table".

**Diagnosis.** `crates/pse-backend-native/src/routing.rs:1363-1414` defines three `#[cfg(test)]`
methods: `policy_eligibility_for_test`, `policy_decision_for_test` and `policy_select_for_test`.

They differ from production in these ways:
- They rebuild `Eligibility` from `admit(...)` alone and mark every admitted adapter `Ready`.
- `policy_decision_for_test` also clears evidence and forces `Ready` on a `Constant` decision.
- Production `BackendExecution::assess` (`execution.rs:767-792`) goes further. It applies
  `assess_static` (`routing.rs:896-975`), which demands structure evidence when the context
  has none. It then applies settings-specific capability, context refusals, `admit_settings`,
  `assess_representation` and `finish()` (`routing.rs:522-533`).
- `decide_assessed` (`routing.rs:1104-1112`) gives pending entries no route.
- On the production path a `Constant` decision demands structure evidence
  (`routing.rs:1137-1150`).

Where the test-only path is used:
- Across the 18 tests in `routing.rs` `mod tests`, the test-only path is called 24 times and
  the production path once.
- `stub_backend_routes_through_adapter_table` (`execution/tests.rs:244-270`) selects through
  the test-only path.
- The `LexProbe` stub (`execution/tests.rs:1229-1248`) re-implements the same reduced rule.

**Consequence.**
- These tests are evidence for `admit`, `policy_candidates` and `decide_assessed` given `Ready`
  inputs. They are not evidence for the composed production decision; some asserted routes,
  for example `caller_inventory_limits_automatic_selection` (`routing.rs:1453-1484`), would be
  pending without structure on the production path.
- A new contextual step in `assess` that changes static outcomes would not be detected.
- The test composition must be kept in step with production by hand.

**Counterevidence.** `routing/contextual_tests.rs` exercises production
`decision`/`eligibility` with supplied structure in 13 tests. The comment at
`routing.rs:1366-1367` states the narrower intent.

**Probe.** Rerun the static tests through the production decision with a minimal structure
witness and compare the selected routes.

## <a id="dta-009"></a>DTA-009 — Incremental-reuse tests count private query names in Salsa's `Debug` text, and some "nothing re-ran" assertions have no positive control

**Principles:** AP-02, DP-09, DP-23; companion §4, §5.

**Claim at risk.** Memo preservation and backdating: which compiler phases re-execute after an
edit.

**Diagnosis.**
- Observers push `format!("{:?}", event.kind)` for every `WillExecute` and count substrings
  naming private tracked functions (`"typed"`, `"resolved"`, `"effective_members"`, …)
  (`crates/pse-compiler/src/workspace/modeling/granular_tests.rs:25-40`).
- There are about 14 such sites:
  - in `crates/pse-compiler/src/workspace/modeling/`: `reuse_tests.rs`, `granular_tests.rs`,
    `document_tests.rs` and `tests.rs`;
  - `shared_packages.rs`.
- Pinned salsa 0.28.5 prints the function name only through an attached database. Otherwise
  it prints `DatabaseKeyIndex(IngredientIndex, Id)` (`salsa-0.28.5/src/key.rs:94-106`).
- `relocating_unchanged_declarations_preserves_type_and_member_memos`
  (`granular_tests.rs:376-400`) asserts only that four counts are unchanged, with no positive
  count.

**Consequence.**
- *Silent:* renaming a tracked function, or a Salsa format or attachment change on upgrade
  (PSE-S06), makes the counts zero, and equality-only assertions keep passing.
- *Spurious:* wherever positive or exact counts exist (e.g. `reuse_tests.rs:1729-1752`), a
  behavior-preserving split or merge of queries turns into test edits.

**Counterevidence.**
- Many tests do carry positive controls (`reuse_tests.rs:58` asserts `first > 0`), so names
  are printed today.
- Every test also compares incremental output against a clean or fresh result (`assert_fresh`),
  so semantic correctness is challenged independently of the counters.
- A typed ingredient lookup is already used at `granular_tests.rs:752` (`debug_name()`).

**Footprint.** *Established:* the sites above. *Suspected:* other equality-only count
assertions in `reuse_tests.rs`, which were not individually audited.

## <a id="dta-010"></a>DTA-010 — Implicit-body test builders are copied across crates because `cfg(test)` helpers are crate-private, and the copies mirror production construction by hand

**Principles:** AP-02, AP-06; companion §2, §5.

**Diagnosis.**
- pse-math defines `#[cfg(test)]` builders: `source_body_for_test`, `finish_body_for_test` and
  `residual_bodies_for_test` (`crates/pse-math/src/implicit.rs:2350-2420`). They are documented
  as the "same preparation and all-input/all-output compilation as `BodyBuilder::finish`". The
  production builder releases the prepared source, so tests cannot bind fixtures through it.
- pse-backend-native keeps its own copies (`crates/pse-backend-native/src/implicit.rs:505-596`),
  which already differ slightly in cancellation and flag handling.
- pse-math has no test-support feature that could share them.

**Consequence.**
- A change to how `SourceBindings`/`SourceBody` are built must be made in production and in
  both copies. There are about 60 call sites, roughly 40 of them in pse-math `implicit_*`
  files.
- Drift between the copies would give the two crates different fixture semantics, and no test
  would fail.

**Uncertainty.** Whether the current differences are intentional was not established.

## <a id="dta-011"></a>DTA-011 — Canonical tests run a structurally different `CanonicalStore`, because the fixture lifetime is a cfg-gated field threaded through production drop and release paths

**Principles:** AP-02, AP-06, DP-19; companion §5.

**Claim at risk.** Canonical store tests show how the production store behaves on drop, protection
release and cleanup.

**Diagnosis.**
- `CanonicalStore` gains `fixture_lifetime: Option<Arc<testing::FixtureLifetime>>` under
  `cfg(any(test, feature = "test-support"))` (`crates/pse-operations/src/canonical.rs:1059-1062`).
- `canonical.rs` refers to it at 18 sites, with more in `canonical_files.rs`,
  `canonical_results.rs` and `canonical_lifetime.rs`.
- Under test, protection cleanup tasks retain the fixture (`canonical.rs:852-871`).
- `ProtectionOwner::drop` substitutes a store without the lifetime (`canonical.rs:910-920`).
- The final-owner seal decision depends on the lifetime's strong count
  (`canonical.rs:2236-2251`).
- Payload withholding requires the lifetime (`canonical_files.rs:428-433`).
- The struct's drop order is itself part of the design: a comment at `canonical.rs:1059-1060`
  says the fixture's final drain must run after both client aliases release.

**Consequence.**
- Tests exercise drop ordering and release sequencing with an extra owned field and extra
  branches that production builds do not contain.
- A production-only regression in drop or release ordering would not be observed through
  these tests.

**Counterevidence and bounds.**
- The hooks are not in the production extension graph: `cargo tree -p pse-py -e normal,build`
  shows no `test-support`.
- Each fixture is a separate `canonical_test_<uuid>` database.
- The drop-order intent is documented.
- `canonical_transport_tests.rs:679` constructs a store with `fixture_lifetime: None`, which
  partly covers the production shape.
- Fault and tamper hooks in `crates/pse-operations/src/testing.rs` refuse non-fixture databases
  and serve recovery claims a simple fake could not.

**Probe.** Identify which tests run protection release or drop with `fixture_lifetime: None`.
