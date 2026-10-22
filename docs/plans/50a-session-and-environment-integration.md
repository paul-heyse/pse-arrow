---
title: "Plan 50a: Session and environment integration"
status: draft
date: 2026-10-22
adrs: []
review_sources: [docs/functional_reviews/codex-effectiveness-and-diagnostics-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/50-codex-effectiveness-and-diagnostics.md
doc_topics: [environment, libraries, diagnostics, validation]
---

# Plan 50a: Session and environment integration

## Purpose and foundations

Deliver [O01/O02](../functional_reviews/codex-effectiveness-and-diagnostics-2026-10-22.md#O01):
fresh Codex commands select the intended checkout and preserve the complete workload environment;
useful semantic/database capabilities attach to the actual selected session. Package state and
shared decisions belong to the [coordinator](50-codex-effectiveness-and-diagnostics.md#package-state).
**Interface-checked:** `bb61ae3f5`; **Proposed:** this design and acceptance. Existing review probes
did not reproduce the historical cache-variable loss and do not qualify the replacement.

The workstation injects `~/.config/codex/bash-env.bash` into each Codex Bash launch. It unloads
direnv, restores workstation selections and activates the checkout, but leaves `BASH_ENV` inherited.
Just and the managed sccache wrapper start more Bash shells. `pse_env.compose/execute` preserve
the hook alongside compiler/native variables. Reinitialization after workload composition is
the seam to remove; jobs/cache/native policies already have owners.

Both rust-analyzer and native SurrealDB MCP are already configured and documented as opt-ins.
The database helper fixes a particular state/port, while `.envrc.local` and caller selection can
choose another. Its strict identity/authentication checks are a strength. Reuse them without
weakening endpoint validation or silently following changing selections.

## Target behavior and interfaces

Machine activation occurs once at the beginning of each top-level Codex command. After successful
activation the bridge unsets its startup hook so nested shells consume the selected mapping.
Codex's configured parent injects the bridge again for the next command, including another
checkout. `scripts/pse-env` remains the workload owner; neither activation nor a session capability
read starts servers, builds extensions, installs tools or reconciles resources.

Add a fixed, admission-free `just codex-session` entry point. It resolves an explicit optional
`--state` selection, otherwise the existing environment owner, and passes literal trailing Codex
arguments. It selects the existing database MCP and rust-analyzer capabilities when requested;
the normal task-oriented preset requests both, with explicit opt-outs. An unavailable optional
capability is reported with its existing repair/start route; no fallback store is chosen. An
explicit required capability fails before launching an apparently equivalent session.

The launcher supplies configuration overrides as argv data, not shell interpolation. Resolve
the helper's pinned checkout interpreter and absolute selected state; construct the exact
loopback endpoint from validated supervisor metadata. Factor common selected-service validation
from `mcp_authorization`, while retaining endpoint-specific `/mcp` checks. Never expose its header
in a preview or log. User-provided explicit Codex overrides retain precedence and are reported
as changing the preset selection rather than silently being overwritten.

Keep native `info/list/query` scope explicit per call. Resources/prompts are optional additional
context where the actual client exposes them. Distinguish configured, requested/enabled,
attached, scope-selected and unavailable. The launcher can establish selection/readiness;
actual client attachment is reported by the client interaction, not inferred from a successful
process spawn. Credential/state changes require reconnection. Standard Codex invocation remains
supported; the new route does not replace the coding agent runtime.

The machine-local bridge and project defaults have different ownership. The implementation
updates the actual local bridge and its maintained workstation instructions, then records the
portable behavior in the environment guide and isolated fixtures. Do not put a home-directory
path into a new reusable library skill or commit private configuration. No shared shell policy
is broadened beyond the selected workstation behavior.

## Packages

<a id="A01"></a>

### A01 — One initial activation

Change the bridge after successful activation to remove inherited `BASH_ENV`; preserve fail-closed
direnv behavior and existing workstation/compiler selections. Test fresh checkout activation,
main→worktree commands, nested Just/sccache and native child propagation. Exercise caller values,
local defaults, missing venv and unapproved/failed activation. Include the historical cache
variable case with a controlled input; do not claim a real compiler failure was reproduced from
the review's successful one-level probe.

Use isolated temporary shell/environment fixtures, not edits to protected product files. Inspect
the actual child mappings selectively; do not dump the full environment. Remove recurring recipe
guidance requiring manual `env -u BASH_ENV` only after the selected nested routes work.

<a id="A02"></a>

### A02 — Freeze the selected session attachment

Implement the fixed session entry point and supervisor validation refactoring. Give the recipe
an explicit direct-script Just exception, like the fixed `activity` observer: use
`[script("bash", "-euo", "pipefail")]` rather than the default workload shell/interpreter.
Otherwise Just enters `scripts/pse-env` and may acquire admission before the launcher runs.
Keep the pinned checkout interpreter for Python helpers. The selected service context carries
state, endpoint, server instance/unit generation, binary/module and credential identities through
the existing helper. It is connection context, not scientific result qualification. No arbitrary
URL/header delegation is introduced. Literal argument forwarding preserves spaces and special
characters without evaluation.

First consumers are one real Codex inspection of a canonical table and one Rust definition/
implementation/reference query. Test stopped service, wrong endpoint, changed instance,
credential rotation and explicit selection mismatch. Confirm that failure creates no database
or service. B01 later supplies the Python navigation adapter; it is not a prerequisite for
the initial Rust/database session route.

<a id="A03"></a>

### A03 — Report effective capability/configuration state

Provide a bounded nonsecret session description from the resolved arguments and actual available
client observations. Link native readiness/repair commands. A configuration file's existence is
not attachment evidence. Do not claim hook enforcement from a filename or matching alias alone;
use existing setup controls and the actual client load/trust view when that matters.

Reconcile the guide's `local-env` profile and "without direnv" text with promoted workstation
defaults and current Codex support. Keep the documented pinned toolchain, sixteen jobs, unbounded
ordinary builds, explicit scientific modes and fresh-session opt-in behavior accurate. Remove
hard-coded selected-store instructions superseded by A02 while preserving standard direct use.

## Acceptance and boundaries

**Proposed targeted evidence:** shell propagation/argument tests, selected-service/helper refusal
controls and explicit real Codex interactions. A mocked MCP initialization is not the real-client
acceptance. External tool availability and client resource/prompt support remain distinct.

This companion changes tooling and local activation, not model semantics or native scientific
admission. It neither changes approval/sandbox policy nor enables arbitrary CRUD MCP tools.
Do not run a broad native suite to validate environment formatting or session selection.

Sources: `scripts/pse-env`, `scripts/pse_env.py`, `scripts/build_environment.py`, `scripts/sccache`,
`scripts/surreal_server.py::mcp_authorization`, `.codex/config.toml`, `rust-analyzer.toml` and
[environment guide](../dev/agent-environment.md). Preserve the review's exact source evidence and
[official Codex configuration](https://developers.openai.com/codex/config-reference/),
[MCP guidance](https://developers.openai.com/codex/mcp/) and
[hook behavior](https://learn.chatgpt.com/docs/hooks) when adapting actual client capabilities.
