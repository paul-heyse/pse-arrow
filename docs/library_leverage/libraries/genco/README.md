# genco

Realizations considered: not a dependency. Upgrade route `genco` 0.20.0 (released
2026-10-07, MSRV 1.88).
Research date 2026-10-27. Sources: docs.rs `genco` 0.20.0 crate page (fetched 2026-10-27);
crates.io API.

## Types

### genco/language-aware-quasi-quoting
- **Need it serves:** Emit source code for a non-Rust target language (Python among them)
  from Rust. Indentation, import collection and string quoting are handled by a
  language-aware token stream instead of `push_str` and `format!`.
- **Recognizable code shapes:**
  - Rust generators that build Python or other source with `push_str`, `writeln!` and
    manual indentation counters.
  - Hand-maintained import headers.
  - Textual post-processing of generated code, such as stripping prefixes with
    `.replace(..)`.
- **Mechanisms and APIs:**
  - `quote!` and `quote_in!` produce language-typed `Tokens<L>`. `$var` interpolates.
  - Whitespace detection drives structural indentation. Python is explicitly supported for
    indentation-significant output.
  - Language import items are collected and emitted as grouped import statements.
  - Per-language string quoting.
  - New languages can be added with `impl_lang!`.
  - Output goes through a formatter configuration.
- **Obligations discharged:**
  - Indentation and spacing.
  - Import tracking and header generation.
  - String literal quoting for the target language.
- **Residual caller obligations:**
  - All semantics of what is generated.
  - Verifying that import ordering and grouping are deterministic.
  - Escaping a literal `$` in quoted code.
- **Semantic conditions:** The page does not state import ordering. Determinism must be
  verified before relying on byte-exact regeneration.
- **Version / build conditions:** 0.x API. Proc-macro.
- **Limits and negative results:**
  - Not a template engine, and not a typed AST of the target language.
  - Syntax errors in the output are not detected.
- **Evidence:** Interface-checked: crate page only (language list, whitespace detection,
  import handling, quoting claim).

## Current applicability

### [CBO-046](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-046) — SurrealQL and Python emitted through string templates (Python side)
- **Types:** `genco/language-aware-quasi-quoting`.
- **Realization:** Upgrade route `genco =0.20.0`, a generator-only dependency.
- **Conclusion:** Unsettled. It is the only candidate found that targets the Python emitter
  (about 117 `push_str`/`format!`/`writeln!` sites across the Python generator modules)
  rather than just placeholders. Byte-deterministic import ordering is unverified, and the
  generated Python is already checked by regeneration drift and type checks downstream.
- **What it would discharge / what remains local:**
  - Discharged: indentation, import headers, string quoting and the textual rewrites that
    strip qualifiers from generated code.
  - Remains local: the registry-to-Python mapping itself.
- **Prerequisites, configuration or extension:** Rewriting the Python generator modules in
  `quote!` form.
- **Semantic gaps against the CBO's preservation constraints:** Byte-deterministic output
  is unverified.
- **Unsettled premises and what would settle them:** Import ordering determinism and
  `$`-escaping rules. Reading `genco` 0.20.0's Python `Lang` implementation would settle
  both.
- **Evidence:** Interface-checked (crate page); application Proposed.

**Reviewer correction (2026-10-27).** The Python rewrites the templates would address are two `.replace` rewrites of emitted Python (`local.replace(&format!("s.{name}"), name)` and `.replace("v.", "")` in `codegen/python/mod.rs`) plus the `from typing import …` line surgery in `codegen/documents.rs`. The comparative review removes them by local generator parameterization instead: emitters receive their qualifier context, the document emitter tracks the typing names it uses, and a check asserts that no `@X@` marker remains. It adopts no template engine; on the SurrealQL side it rejects `DEFINE PARAM` and keeps the placeholders, which are already parse-validated. See the [comparative review](../../reviews/declarations-registry-and-python-boundary.md#cbo-046).
