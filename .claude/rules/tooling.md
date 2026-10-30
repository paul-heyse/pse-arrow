---
description: Recipes, scripts, build configuration and agent configuration
paths:
  - "justfile"
  - "scripts/**"
  - ".config/**"
  - ".cargo/**"
  - ".envrc"
  - ".claude/settings.json"
  - ".codex/**"
  - "rust-analyzer.toml"
---

# Changing the command surface and agent configuration

The mechanics are in [the agent environment guide](../../docs/dev/agent-environment.md);
keep it accurate when you change them.

- **One environment boundary.** Recipes, direnv, the Claude SessionStart hook and the native
  helpers all take their environment from native `pse-host::environment` through
  `scripts/pse-env`. Add defaults there, not in a recipe or a second script, and keep the
  precedence: command options, caller values, `.envrc.local`, repository defaults. Explicit
  unsupported resource selections refuse before command execution; no Python fallback is
  permitted.
- **Recipes are thin.** A recipe composes the real tool, passes `*args` through, prints what
  it runs when it builds a command, and leaves selection visible to Cargo/nextest. Native
  recipes declare capabilities with `[script("bash", "scripts/pse-env", "--native=…", "--",
  "bash", "-euo", "pipefail")]`; never source environment files inside a recipe.
  Forward arbitrary arguments as positional data, never interpolated shell source. A composed
  recipe honors its supported mode across all stages or rejects it before effects.
- **Fixed observation.** `just activity` runs its read-only observer directly, without workload
  admission or compiler preparation. It accepts observer options, never an arbitrary executable;
  report unavailable observations as unavailable, not empty success.
- **Placement owns resource policy.** The selected ordinary native path uses configured
  unbounded sharing, all 16 physical cores and 32 logical compiler jobs without a slot ledger
  or new OS memory cap. Explicit finite selections retain original physical containment and
  charge until actual drain; native providers select only completed preparation. Unsupported
  selections refuse rather than silently downgrade. The environment guide records current
  acceptance and remaining Plan58 obligations.
  Native scopes and original kernel population establish physical ownership/release. Nested
  commands reuse actual authenticated native containment, never an environment marker alone.
  `scripts/bootstrap-host` explicitly publishes the completed immutable native tool set;
  read/help/explain verbs never rebuild it, and top-level effectful `pse` verbs rebuild a stale
  host and fail if that rebuild fails (Plan59 R3). See the environment guide for supported host premises.
- **Status meanings.** `125` is the boundary's own failure, with a `pse-env:` line naming the
  fix; keep a command's own exit status and signals intact.
- **Hooks.** `scripts/pse-host agent-hook` owns edit guarding and session activation
  (`just lint-agents` checks the wiring). Prepare the native host explicitly before use.
  End-of-turn automation stays removed (ADR-0161).
- **Edits.** Prefer `apply_patch`/`Edit`. Repeated structural Rust rewrites use ast-grep, for
  example `ast-grep run -l rust --pattern 'checkout_root()' --rewrite 'ctx.root()' --update-all
  crates/pse-host/src`. A scripted replacement asserts its count first:
  `assert s.count(old) == 1, s.count(old)`.
- **Checks.** Exercise a changed capability once, in its revealing case; the script tests run
  with `just setup-test`, `just native-setup-unit` and `just surreal-test`. Do not add lints.
