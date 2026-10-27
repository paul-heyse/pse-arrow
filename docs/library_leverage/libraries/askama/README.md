# Askama

Realizations considered: not a dependency. Upgrade route `askama` 0.16.1 (released
2026-09-04, MSRV 1.88). Its derive macro crate builds on the ecosystem `syn` 2 line. That
is a separate copy from any workspace-pinned `syn` 3 used by local generators, and is
compile-time only.
Research date 2026-10-27. Sources: docs.rs `askama` 0.16.1 crate page (fetched
2026-10-27); crates.io API. The askama book was not read.

## Types

### askama/compile-time-checked-templates
- **Need it serves:** Text templates whose variables are type-checked when the program is
  compiled, so a missing or misspelled substitution is a compile error rather than a
  generator-run error.
- **Recognizable code shapes:** The same as other text-template opportunities (placeholder
  `replace` chains and `format!`-heavy emitters), with the added wish that drift fails the
  build.
- **Mechanisms and APIs:**
  - `#[derive(Template)]` with `#[template(path = "x.sql")]` or
    `#[template(source = "...", ext = "txt")]` on a struct or enum whose fields are the
    context.
  - `template.render()` returns a `String`. `write_into` streams the output.
  - Jinja-like syntax with loops, conditionals, macros and filters.
  - Escaping is selected by extension or configuration. The `|safe` filter bypasses it.
- **Obligations discharged:**
  - Compile-time binding of every template variable to a typed field.
  - Templates compiled to Rust.
- **Residual caller obligations:**
  - Target-language escaping. Only HTML-style escapers are built in; others are custom.
  - Unused-field detection is ordinary dead-code linting, if it applies.
  - Whitespace control for byte-exact output.
- **Semantic conditions:**
  - Rendering is pure Rust formatting, and therefore deterministic for deterministic input.
  - An extension not mapped to an escaper renders unescaped. This is documentation
    knowledge and was not verified at 0.16.1.
- **Version / build conditions:**
  - Proc-macro build cost.
  - Templates are read at compile time, so a template edit triggers recompilation of the
    owning crate.
- **Limits and negative results:**
  - The fetched page does not confirm an `escape = "none"` attribute value at 0.16.1.
  - No target-language syntax awareness.
- **Evidence:** Interface-checked: crate page (inline `source`/`ext`, auto-escaping, `|safe`,
  `askama_macros`). Other details come from documentation knowledge.

## Current applicability

### [CBO-046](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-046) — SurrealQL and Python emitted through string templates (templating side)
- **Types:** `askama/compile-time-checked-templates`.
- **Realization:** Upgrade route `askama =0.16.1`, a generator-only dependency.
- **Conclusion:** Fits with conditions, and is stronger than a runtime engine on the
  drift-safety axis. A placeholder without a field fails compilation, and a field without
  use can be linted. It shares the same escaping gap, and the same weak case: few
  placeholders, mostly integers.
- **What it would discharge / what remains local:**
  - Discharged: the 18 SurrealQL `replace` placeholders and the 2 Python `@X@` templates,
    which become typed structs.
  - Remains local:
    - Escaping.
    - The textual rewrites over generated Python, which are not placeholders.
    - Large `push_str` emitters, where moving logic into template control flow is a
      lateral move.
- **Prerequisites, configuration or extension:** Template files beside the generators, or
  inline `source`; a non-escaping extension.
- **Semantic gaps against the CBO's preservation constraints:** Byte-deterministic output
  needs whitespace-control discipline. The existing regeneration check guards it.
- **Unsettled premises and what would settle them:** Whether the generator crate accepts
  another proc-macro dependency for about 18 substitutions. A local "all keys substituted,
  no markers left" assertion is the zero-dependency comparator.
- **Evidence:** Interface-checked (crate page); application Proposed.
