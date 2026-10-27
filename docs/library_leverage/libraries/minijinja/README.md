# MiniJinja

Realizations considered: not a dependency. Upgrade route `minijinja` 3.0.4 (released
2026-10-26, MSRV 1.70). Note that this release is one day old at research time, and a
3.0.x line that has been out longer may be preferred for a pin.
Research date 2026-10-27. Sources: Context7 `/websites/rs_minijinja` (docs.rs
`Environment::{set_undefined_behavior, set_auto_escape_callback, set_formatter}`);
crates.io API.

## Types

### minijinja/runtime-text-templates-strict
- **Need it serves:** Render text artifacts (code, DDL, configuration) from named templates
  and a context, with explicit substitution points instead of chained string replacement.
- **Recognizable code shapes:**
  - `TEMPLATE.replace("__X__", ..).replace("__Y__", ..)` chains over raw-string constants.
  - `@X@` markers in `include_str!` templates.
  - Large `push_str` and `format!` emitters with control flow interleaved with text.
- **Mechanisms and APIs:**
  - `Environment::new()` with `add_template(name, source)`, `get_template(name)` and
    `render(context!{..})`.
  - `set_undefined_behavior(UndefinedBehavior::Strict)`: an undefined variable is an error.
    The default is `Lenient`, which renders an empty string.
  - `set_auto_escape_callback(|name| AutoEscape::None | ..)` picks escaping by template name.
  - `set_formatter(..)` installs a custom value formatter. A target-language literal escaper
    can be written here.
  - Custom filters and functions, loops, conditionals and macros.
  - Templates can be embedded with `include_str!`.
- **Obligations discharged:**
  - Placeholder parsing and lookup.
  - Detection of undefined references (strict mode).
  - Structured control flow in templates.
  - Value formatting hooks.
- **Residual caller obligations:**
  - Target-language escaping, through a custom formatter or filters. There is no built-in
    escaper for SQL dialects or Python literals.
  - Detecting context values the template never uses. Strict mode catches the reverse case
    only.
  - Whitespace and newline discipline for byte-exact output.
- **Semantic conditions:**
  - Output is deterministic for a deterministic context. Map iteration order follows the
    context value's order, so use ordered maps for anything iterated.
  - Errors are runtime errors, at codegen time for a generator. They are not compile errors.
- **Version / build conditions:** 3.x is a major line. Examples written for 1.x and 2.x may
  differ.
- **Limits and negative results:**
  - No compile-time checking of template variables.
  - No knowledge of the target language's syntax.
- **Evidence:** Interface-checked: Context7 docs.rs `Environment` methods. Determinism and
  map-order statements are documentation knowledge and were not verified in source.

## Current applicability

### [CBO-046](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-046) — SurrealQL and Python emitted through string templates (templating side)
- **Types:** `minijinja/runtime-text-templates-strict`.
- **Realization:** Upgrade route `minijinja =3.0.x`, a new dependency of the code generator
  only.
- **Conclusion:** Fits with conditions, but it is weak. Strict-undefined detection fixes
  one of the two placeholder failure modes, a placeholder spelled in the template with no
  value supplied. It does not escape SurrealQL or Python values. A codegen-time helper that
  asserts every key was substituted and that no `__[A-Z_]+__` marker remains would cover the
  same risk with no dependency.
- **What it would discharge / what remains local:**
  - Discharged: placeholder substitution in the 18 `replace(..)` sites (5 SurrealQL
    generators) and the `@X@` markers in the 2 Python templates.
  - Remains local:
    - Value escaping.
    - The textual rewrites over generated Python (`.replace("v.", "")` and
      `.replace("s.{name}", name)`). These are code-rewriting, not placeholders, and a
      template engine does not address them.
- **Prerequisites, configuration or extension:**
  - Strict undefined behavior.
  - `AutoEscape::None`, or a custom formatter that quotes SurrealQL string literals.
  - Embedded templates.
- **Semantic gaps against the CBO's preservation constraints:**
  - Byte-deterministic output: preserved if whitespace control (`{%-`) and trailing-newline
    settings are fixed. `codegen-check` drift detection would catch regressions.
- **Unsettled premises and what would settle them:**
  - Whether any substituted value is a string that needs escaping. Of the 18 sites, all but
    one are integer constants. The exception is one single-quoted interpretation literal
    plus generated guard blocks. Integer-only substitution needs no escaper.
- **Evidence:** Interface-checked (docs); application Proposed.

**Reviewer correction (2026-10-27).** The Python rewrites the templates would address are two `.replace` rewrites of emitted Python (`local.replace(&format!("s.{name}"), name)` and `.replace("v.", "")` in `codegen/python/mod.rs`) plus the `from typing import …` line surgery in `codegen/documents.rs`. The comparative review removes them by local generator parameterization instead: emitters receive their qualifier context, the document emitter tracks the typing names it uses, and a check asserts that no `@X@` marker remains. It adopts no template engine; on the SurrealQL side it rejects `DEFINE PARAM` and keeps the placeholders, which are already parse-validated. See the [comparative review](../../reviews/declarations-registry-and-python-boundary.md#cbo-046).
