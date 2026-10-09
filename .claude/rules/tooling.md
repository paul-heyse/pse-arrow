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
  helpers all take their environment from `scripts/pse_env.py`. Add a default or a correction
  there, not in a recipe or a second script, and keep the precedence: command options,
  caller values, `.envrc.local`, repository defaults. A value the boundary must replace is
  reported (`pse-env: refused …`), never silently overwritten.
- **Recipes are thin.** A recipe composes the real tool, passes `*args` through, prints what
  it runs when it builds a command, and leaves selection visible to Cargo/nextest. Native
  recipes declare capabilities with `[script("bash", "scripts/pse-env", "--native=…", "--",
  "bash", "-euo", "pipefail")]`; never source environment files inside a recipe.
  Forward arbitrary arguments as positional data, never interpolated shell source. A composed
  recipe honors its supported mode across all stages or rejects it before effects.
- **Fixed observation.** `just activity` runs its read-only observer directly, without workload
  admission or compiler preparation. It accepts observer options, never an arbitrary executable;
  report unavailable observations as unavailable, not empty success.
- **Placement owns memory caps.** Workload commands run in their own capped scope; do not
  add another wrapper. Builders call `pse_env.placement()` per command (a transient unit name
  cannot be reused). Host admission uses short kernel-held allocation metadata exclusion;
  never hold a machine-wide or fd-inheriting lock through a workload, RPC or drain.
- **Status meanings.** `125` is the boundary's own failure, with a `pse-env:` line naming the
  fix; keep a command's own exit status and signals intact.
- **Hooks.** `scripts/agent-hooks.py` is the only hook script (`just lint-agents` checks the
  wiring). End-of-turn automation stays removed (ADR-0161).
- **Checks.** Exercise a changed capability once, in its revealing case; the script tests run
  with `just setup-test`, `just native-setup-unit` and `just surreal-test`. Do not add lints.
