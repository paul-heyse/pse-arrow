---
title: "Plan69b: Codex workstation and preparation"
status: complete
date: 2026-11-07
adrs: []
doc_role: plan
doc_owner: docs/plans/69b-codex-workstation-and-preparation.md
doc_topics: [current-work, design, deployment, validation]
---

# Codex workstation and preparation

This companion owns [O07](../functional_reviews/intent-first-cli-and-codex-workflows-2026-11-07.md#O07).
The [coordinator](69-intent-first-cli-and-codex-workflows.md) owns assembled acceptance. The
current guide records a working `systemd-run --user --scope` invocation with a shell parent
that waits while the controller moves into its own workload scope. `scripts/pse` currently
fails early if the prepared host is absent. `scripts/bootstrap-host` already supports native
generation preparation and using a prepared source checkout. These are useful integration
boundaries. The review distinguishes the actual manager membership refusal, stale generation,
and socket pathname failures from text mentions and from the unrelated compiler crashes.
[Environment guide](../dev/agent-environment.md) · [Launcher](../../scripts/pse) ·
[Bootstrap](../../scripts/bootstrap-host)

## B01 — Establish activation and authenticated placement once

Trace a Codex command from the effective shell and approved direnv configuration through the
existing bridge, launcher and native admission. Compare hosting the waiting parent in the
bridge with using the existing launcher. Choose the boundary that can verify the checkout and
current manager membership before the first effect, preserve an existing valid placement, and
hold the parent for the original child. Prefer the existing launcher when both routes meet
these guarantees without nesting scopes. If launcher reuse would re-enter the boundary, keep
the bridge as the Codex-specific adapter and leave native admission authoritative.

Preserve the chosen local unbounded ordinary mode and exact explicit finite and scientific
selections. Scope or checkout uncertainty produces a refusal with the canonical recovery action.
Keep passive help and reads independent of placement. Verify effective hook and shell behavior
on the workstation before changing configuration. Hooks provide advice and retain the existing
native trust boundary. Replace the documented manual scope sequence once the supported route
passes the cold and prepared checkout cases. Keep it available as a diagnostic repro if needed.

## B02 — Explicit preparation and continuation

Classify the required host or provider as absent, stale, or mismatched. Return the required
preparation and its exact recovery action. The cold launcher must expose minimal help and the
explicit preparation entry even when the native parser is unavailable. Prefer a fixed handoff
to `scripts/bootstrap-host` for the missing-host case. Keep the shim small and delegate full
catalog and runtime parsing to the native host once present. Use a matching verified seed when
the bootstrap owner accepts equivalence. Use checkout-local preparation otherwise. Report the
actual source and exact prerequisite that failed. On failure preserve the previous usable
selection and return a deliberate retry. Recheck generation and intended placement when
continuing the selected operation. Repeat preparation reuses verified state.

If a prepared continuation uses layered settings, consume A02's validated selection
and its identity or revalidate the premise under the preparation owner. Keep native
environment and path inputs byte-exact. Receive A03's prerequisite code and exact
retry action. Before the native parser is available, the cold launcher supplies its
fixed handoff. After preparation, F01 keeps child options out of the outer format
and execution-mode selection.
[Resolved context and diagnostics](69a-cli-contracts-configuration-and-presentation.md) ·
[Argument boundary](69f-parser-startup-and-argument-forwarding.md#f01--strict-parse-before-usage-format-recovery)

Exercise cold linked checkout and warm main, missing provider, stale host, incompatible seed,
and two explicit requests that encounter the same prepared selection. Assert that help and
inspection do not prepare state, selection and mode survive continuation, and a warm command
does not restage the tool. Use a child format option and an inherited preference in the
receiving check. Measure cold and warm only if claiming improvement beyond the
eliminated manual steps. Reuse Plan67’s captured tool and compiler cache paths.
[Preparation and custody](67b-invocation-custody-and-developer-history.md) ·
[Build reuse](67d-cargo-variants-and-sccache-reuse.md)

### Exact v4 prerequisites

The live fixture continuation in [I02](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I02)
uses the current Study module preparation owner. The earlier v3-shaped receipt was refused.
Use `scripts/pse fixture prepare study-module` to select the verified v4 inputs and publish
the private pointer. The owner derives an immutable key from the selected server, source,
locked dependencies and build inputs, reuses an exact receipt when present, and builds only
on a miss. Check the installed extension's artifact digest, capability scope and original
`ext::` readiness through the canonical fixture. A server version or listener readiness
probe is a separate preflight. Check refusal of the old receipt, successful explicit
continuation, and warm reuse. D01–D03 consume the prepared receipt independently of E03.
[Fixture preparation](../../crates/pse-host/src/fixtures/execution.rs) ·
[Module preparation and reuse](../../crates/pse-host/src/study_module.rs) ·
[Installed extension validation](../../crates/pse-operations/src/canonical_study_module.rs) ·
[SurrealDB v4 readiness probe](https://surrealdb.com/docs/reference/cli/db/commands/isready)

If [E03](69e-source-model-and-live-schema-operations.md#e03--inspect-preview-apply-and-recover-live-schema)
selects the standalone tool, receive [I07](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I07)
before exposing rollout apply. The prepared SurrealKit rc.1 owner checks source, lock and
executable identity. Its current `verify` follows `PSE_SURREALKIT_BIN` while `install`
targets the default generation. An invalid existing default generation also fails install's
verification. Project three distinct A03 outcomes. An absent default can use explicit
`diagnostic-tools install surrealkit` followed by verify. A rejected override requires
selection correction followed by verify. An invalid existing generation needs an exact
protected-owner repair where available, otherwise an unresolved prerequisite with the original
identity. Preserve prior usable generations. Verify passive inspection and a successful
second verification for each supported correction, plus a corrupt-generation refusal that
preserves the bytes and offers no ineffective retry. The upstream version report provides
version, OS and architecture for diagnosis. The checkout receipt establishes provenance.
[Tool selection, verification and install](../../crates/pse-host/src/surrealkit.rs) ·
[Pin](../../.config/surrealkit.toml) ·
[Pinned version report](https://github.com/surrealdb/surrealkit/blob/625e264c4e134def842bfe8ac9d2f918a42bb6c3/crates/surrealkit/src/output.rs)

## B03 — Remove repeated staging and operational sequences

When a managed service needs a socket, allocate a short private scratch root through its
current owner and validate the full derived pathname before launch. Surface an exact refusal
when a required path cannot work. Verify no service is started for a rejected path and the
owner releases scratch after actual drain. Expose failed-stage identity and bounded, redacted
compiler/native prerequisite or crash details through A03. A deliberate exact retry reuses the
selected intent. The log evidence does not establish a cause for compiler crashes.

Use the existing native Study fixture for the live invalid-socket refusal and owner-drain
controls. A rejected pathname must leave launch unstarted. For admitted work, check the
original process forest settlement before releasing its scratch and retained diagnostics.
SurrealKit's logical database teardown is a separate assertion.
[Native fixture owner](../../crates/pse-host/src/fixtures/execution.rs) ·
[I02 receiving boundaries](../plan_improvement_reviews/plan69-surrealdb-and-rust-tooling-2026-11-08.md#I02)

Inventory repeated fixed sequences from the review, including targeted dependency refresh
followed by host preparation. Add a `just` recipe only if its components already expose
the required fixed effects and the sequence removes a repeated handoff. Keep typed admission,
effect and recovery with native owners. Use the existing Codex bridge and advice hook to point
to the resulting commands. Remove redundant orchestration and caller-managed scratch in
migrated workflows. For checkout retirement, keep ordinary Git and exact owner release. Add
previews or exact release surfaces only for an observed gap. If the explicit route cannot
authenticate and settle its owners reliably, bring the combined mutation to R-56 for decision.
[Checkout retirement](67e-resource-retirement-and-storage-verification.md#e02--worktree-identity-and-retirement) ·
[Decision register](../adr/register.md)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| B01 | complete | Existing Codex shell, bridge and native admission | The launcher authenticates manager membership and enters a waiting-parent scope when unplaced. Cold help and catalog refusal left the target absent. Current activated foreground probes observed the configured unbounded mode and the explicit light mode's 2 GiB ancestor limit, with complete settlement in both cases. |
| B02 | complete | B01 placement, A02 and A03 for prepared context and recovery, F01 for the prepared argument boundary | Explicit preparation replaced the refused v3-shaped fixture selection and two consecutive preparations reused the same v4 Study generation and digest. The live project fixture received the prepared module. Focused owner and CLI checks verified the distinct SurrealKit preparation refusals, safe next action and matching-generation recovery. |
| B03 | complete | B01, A03, and existing scratch and retirement owners | The Study fixture checks both full paths before launch. The focused native control rejected an actual overlong parent and left no scratch. The foreground forest drain and three bounded native recovery and interruption controls passed. `just upgrade-host` composes the repeated refresh and preparation sequence. |
