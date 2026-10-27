# Cyclopts

Realizations considered: installed `cyclopts` 5.3.0, a direct pin in the `quality`
dependency group, which the default development environment includes. It is not a base
dependency. Its requirements are `attrs`, `docstring-parser`, `rich` and `rich-rst`.
Research date 2026-10-27. Sources:
- Context7 `/brianpugh/cyclopts`: lazy loading with `"module:function"` import paths; shared
  parameters through a flattened dataclass with `@Parameter(name="*")`.
- Installed `cyclopts-5.3.0`: `core.py` `App.command` overloads, `command_spec.py` and
  `loader.py` (lazy specs), and the `config/` package (`_env.py`, `_toml.py`, `_json.py`,
  `_yaml.py`).
- dist-info METADATA.
- One `python -X importtime -c 'import cyclopts'` observation in `.venv`: about 28 ms
  cumulative, against about 3 ms for `argparse`. This is a single run on this host, an
  observation and not a benchmark.

## Types

### cyclopts/typed-cli-from-signatures
- **Need it serves:** Declare a command's options once, as typed function parameters or
  dataclasses, and derive parsing, validation, help and shell completion from that
  declaration. Share option groups across many commands and load command modules lazily.
- **Recognizable code shapes:**
  - Many scripts, each building an `argparse.ArgumentParser` with repeated `add_argument`
    blocks for the same options.
  - Hand-written `type=` converters and range checks.
  - A generator emitting argv parsing from another schema.
- **Mechanisms and APIs:**
  - `App()` with `@app.command` and `app.command("pkg.mod:func", name=...)` for lazy
    import paths.
  - Sub-apps through `app.command(sub_app)`.
  - `Annotated[T, Parameter(name=..., help=..., validator=...)]`.
  - Flattened shared option dataclasses with `@Parameter(name="*")`.
  - `App.meta` for global options.
  - Config sources in `cyclopts.config`: `Env`, `Toml`, `Json` and `Yaml`.
  - Validators, and help rendered with rich.
- **Obligations discharged:**
  - Parsing, type conversion and validation from annotations.
  - Help and completion.
  - Shared option groups.
  - Lazy command loading.
- **Residual caller obligations:**
  - Command semantics.
  - Process admission and environment wrappers.
  - Stable positional contracts for existing callers.
  - Exit codes.
- **Semantic conditions:**
  - Option spelling follows the parameter name (configurable). Migrating from argparse can
    change accepted spellings and error messages.
  - Import of cyclopts, rich and attrs is far heavier than argparse.
- **Version / build conditions:**
  - The 5.x API. Lazy import-path commands are present in 5.3.0 (`command_spec.py`,
    `loader.py`).
  - Available only where the `quality` group is installed.
- **Limits and negative results:**
  - Not stdlib. Unsuitable for cold-start hooks and bootstrap scripts that must run before
    the environment exists.
- **Evidence:** Interface-checked (Context7 docs; installed source file presence and
  `App.command` overloads). Import-cost figure is a single unbenchmarked observation.

## Current applicability

### [CBO-052](../../../library_leverage/codebase/developer-tooling.md#cbo-052) — Several command surfaces without a shared declaration
- **Types:** `cyclopts/typed-cli-from-signatures`.
- **Realization:** Installed pin 5.3.0 in the `quality` group. No version move.
- **Conclusion:** Fits with conditions as the single Python entry-point framework for the
  scripts that run inside the provisioned environment. Ruled out for stdlib-only bootstrap
  and hook scripts. Given "tooling changes stay light", the conversion of 63 argparse
  scripts is a broad change whose benefit is unproven.
- **What it would discharge / what remains local:**
  - Discharged:
    - Repeated `add_argument` blocks for shared options, through a shared option dataclass.
    - Hand converters and range checks.
    - The argparse-shaped part of the capability CLI generator, which already targets
      cyclopts.
  - Remains local:
    - Recipe positional contracts.
    - Environment admission through the launcher.
    - The capability runtime.
    - Stdlib-only scripts.
- **Prerequisites, configuration or extension:**
  - A shared options module.
  - Lazy command registration to bound import cost.
  - Keeping recipes positional (preservation constraint), so cyclopts parameters must
    accept the same positional order.
- **Semantic gaps against the CBO's preservation constraints:**
  - *Admission through the environment launcher:* unaffected.
  - *Recipes stay positional:* achievable, but every recipe's argument order must be
    checked.
  - *Codex and Claude parity:* unaffected.
- **Unsettled premises and what would settle them:**
  - Which scripts actually share option vocabulary. A count of repeated `add_argument`
    names across the 63 scripts would show whether consolidation removes material
    duplication or only changes syntax.
  - Which scripts must stay stdlib-only. The bootstrap and hook paths should be
    enumerated.
- **Evidence:** Interface-checked; application Proposed.
