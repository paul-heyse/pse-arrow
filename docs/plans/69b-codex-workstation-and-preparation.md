---
title: "Plan69b: Codex workstation and preparation"
status: in-progress
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

## B03 — Remove repeated staging and operational sequences

When a managed service needs a socket, allocate a short private scratch root through its
current owner and validate the full derived pathname before launch. Surface an exact refusal
when a required path cannot work. Verify no service is started for a rejected path and the
owner releases scratch after actual drain. Expose failed-stage identity and bounded, redacted
compiler/native prerequisite or crash details through A03. A deliberate exact retry reuses the
selected intent. The log evidence does not establish a cause for compiler crashes.

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
| B01 | in progress | Existing Codex shell, bridge and native admission | The launcher authenticates manager membership and enters a waiting-parent scope when unplaced. Assembled checkout and selected-mode evidence remains. |
| B02 | in progress | B01 placement, A02 and A03 for prepared context and recovery, F01 for the prepared argument boundary | Cold help, fixed preparation, verified seed selection and warm receipts are implemented. The assembled prerequisite and continuation journey remains. |
| B03 | in progress | B01, A03, and existing scratch and retirement owners | The Study fixture owns a short private scratch path and checks it before launch. Live drain and bounded recovery remain to be exercised. |
