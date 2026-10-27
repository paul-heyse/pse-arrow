# just

Realizations considered: installed `just` 1.58.0, a host tool. No upgrade is needed for
anything below. Every feature cited is at or below 1.58.
Research date 2026-10-27. Sources:
- Context7 `/casey/just` README: attribute table with version markers; `[arg(ARG, long=..,
  short=.., value=.., pattern=.., help=..)]` from 1.45 and 1.46; modules.
- Local `just --dump --dump-format json` over the repository justfile:
  - 231 recipes;
  - 229 carry `[group]`, 204 `[doc]`, 102 `[script]`, 4 `[confirm]`;
  - no `mod` or `import` statements and no `[arg]` attributes.
- `set shell` and `set script-interpreter` route every recipe through the environment
  launcher.

## Types

### just/declarative-recipe-interface
- **Need it serves:** A discoverable command catalogue whose names, parameters, options,
  documentation and grouping are declared once and readable by humans (`--list`,
  `--usage`) and machines (`--dump --dump-format json`).
- **Recognizable code shapes:**
  - Large justfiles of one-line recipes forwarding `"$@"` to scripts.
  - Repeated long command prefixes.
  - Option vocabulary re-declared in each forwarded script.
  - Tools that scrape recipe bodies.
- **Mechanisms and APIs:**
  - Grouping and documentation: `[group(..)]`, `[doc(..)]` and `[metadata(..)]`.
  - Parameters:
    - `[arg("p", long, short="x", value=.., pattern='re', help=..)]` turns parameters into
      flags and options with validation (1.45 and 1.46+);
    - `min` and `max` for variadics (1.56);
    - `[positional-arguments]`.
  - Modules and composition:
    - `mod name` loads `name.just` or `name/mod.just` as a namespaced sub-module
      (`just name::recipe`);
    - `import` merges files.
  - `[script(..)]` and `set script-interpreter`.
  - `[env(NAME, VALUE)]` (1.47).
  - `[parallel]` dependencies.
  - Machine-readable interfaces: `just --usage <recipe>` and
    `just --dump --dump-format json`, which gives recipes, parameters, attributes and doc
    strings.
- **Obligations discharged:**
  - Argument and flag parsing and pattern validation at the recipe boundary.
  - Help and usage.
  - Grouping and namespacing.
  - A machine-readable command inventory.
- **Residual caller obligations:**
  - Script-side parsing remains for anything the script also exposes directly.
  - Recipe bodies stay shell.
- **Semantic conditions:**
  - `[arg(.., long)]` changes a positional parameter into an option. Callers passing it
    positionally break.
  - Variables and recipes in modules are namespaced.
- **Version / build conditions:** Versions as marked in the README table. All are at or
  below 1.58.
- **Limits and negative results:**
  - Not a typed CLI for the scripts themselves.
  - No reuse of one parameter declaration across recipes, beyond variables and recipe
    dependencies.
- **Evidence:** Interface-checked (Context7 README attribute table; local JSON dump
  statistics).

## Current applicability

### [CBO-052](../../../library_leverage/codebase/developer-tooling.md#cbo-052) — Several command surfaces without a shared declaration
- **Types:** `just/declarative-recipe-interface`.
- **Realization:** Installed 1.58.0.
- **Conclusion:** Fits with conditions, narrowly.
  - The repository already uses `[group]`, `[doc]` and `[script]` almost universally.
  - The capability inspection module already consumes `just --dump --dump-format json` as
    the recipe declaration.
  - The remaining just-side options are `mod` files to split the 231-recipe file, which
    affects organization rather than declaration count, and `[arg]` flags. Flags conflict
    with the "recipes stay positional" constraint.
  - Collapsing the 17 nextest-wrapper recipes into one parameterized recipe would change
    their positional interfaces.
- **What it would discharge / what remains local:**
  - Discharged: help and usage text and grouping (already discharged); file
    modularization if wanted.
  - Remains local: the preset definitions themselves, the environment-launcher modes and
    script argv parsing.
- **Prerequisites, configuration or extension:** None for modules. For `[arg]`, acceptance
  of option-style recipe parameters, which the preservation constraint currently forbids.
- **Semantic gaps against the CBO's preservation constraints:**
  - *Recipes stay positional:* conflicts with `[arg(long)]`.
  - *Admission through the launcher:* already enforced by `set shell`.
- **Unsettled premises and what would settle them:**
  - Whether the JSON dump, already consumed, can become the source from which the
    capability or MCP surface derives recipe entry points end to end. That is a reading
    of the capability inventory's use of the dump, beyond source-path extraction.
- **Evidence:** Interface-checked; application Proposed.
