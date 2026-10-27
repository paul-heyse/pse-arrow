# chumsky

Realizations considered: not a dependency. Upgrade route `chumsky` 0.13.0 (released
2026-05-06, MSRV 1.65; the `pratt` feature is needed for operator precedence). It would be a
new direct dependency with no pinned-family interaction.
Research date 2026-10-27. Sources: Context7 `/websites/rs_chumsky_chumsky` (docs.rs latest:
`Parser::map_with`, `MapExtra`, `Input::split_token_span`/`MappedSpan`, `Pratt`,
`parse_with_state`); crates.io API for version and date. Installed source was not inspected
because the crate is not in the lock.

## Types

### chumsky/token-input-with-recovery
- **Need it serves:**
  - Parse a token stream with combinators that carry spans and parser state.
  - Recover from syntax errors and report several diagnostics in one pass, instead of
    stopping at the first.
- **Recognizable code shapes:**
  - Parsers that return on the first error where an editor or author would benefit from all
    errors.
  - Token vectors of `(Token, Span)` pairs.
- **Mechanisms and APIs:**
  - `Input` over slices of `(T, S)`, split with `split_token_span`, so spans are tracked
    separately from tokens.
  - `map_with(|out, e| e.span() / e.state())` attaches spans and touches state.
  - `extra::Full<Error, State, Context>` with `parse_with_state(input, &mut state)` provides
    mutable state.
  - `Parser::pratt(ops)` handles precedence (feature `pratt`).
  - `recover_with(via_parser(..))`, `skip_then_retry_until` and `nested_delimiters` provide
    recovery.
  - `ParseResult` carries both output and errors.
  - The `Rich` error type collects expected and found sets.
  - Context-sensitive parsing is available through `configure`, `with_ctx` and
    `ignore_with_ctx`.
- **Obligations discharged:**
  - Combinator plumbing, spans and precedence.
  - Multi-error recovery and partial output.
- **Residual caller obligations:**
  - Lexing (chumsky can also lex).
  - Budgets: there is no built-in recursion limit, so it lives in state.
  - Domain context decisions, error mapping and printing.
- **Semantic conditions:**
  - Recovery produces partial ASTs. Consumers must treat the output as invalid whenever
    errors are non-empty.
  - Deterministic error ordering should be verified.
  - Span type is generic, and byte or char semantics come from the input.
- **Version / build conditions:**
  - The API changed substantially from 0.9 to 0.10 (zero-copy rewrite). Examples older than
    0.10 do not apply.
  - 0.13.0 is the current stable release.
- **Limits and negative results:**
  - Parse-only, with no printing.
  - No AST visitor or generator derivation.
  - Compile-time and type-complexity cost of large combinator graphs is a known ergonomic
    concern (documentation guidance; not measured here).
- **Evidence:** Interface-checked (Context7 docs.rs pages named above). Recovery combinator
  names come from chumsky's documented API and were not re-read at 0.13.0. The specific
  names `skip_then_retry_until` and `nested_delimiters` are unverified at this exact version.

## Current applicability

### [CBO-037](../../../library_leverage/codebase/authoring-and-identity.md#cbo-037) — Grammar facts maintained in several hand-written forms
- **Types:** `chumsky/token-input-with-recovery`.
- **Realization:** Upgrade route: a new dependency `chumsky =0.13.0` with feature `pratt`.
- **Conclusion:** Fits with conditions. Its distinctive value over the installed combinator
  library is multi-error recovery, which is a new authoring capability (LOP-12), not a
  reduction of the five-form upkeep.
- **What it would discharge / what remains local:**
  - Discharged: the same cursor and precedence work that the installed combinator library
    would discharge, plus recovery.
  - Remains local: everything else listed for the installed combinator library, including
    renderers, walker, generator, budgets and identity derivation.
- **Prerequisites, configuration or extension:**
  - A second parser-combinator library beside the existing lexer dependency, or a lexer
    rewrite.
  - State type for identities and scopes.
- **Semantic gaps against the CBO's preservation constraints:**
  - Partial output after recovery must never reach identity assignment or row emission.
  - `ParseBudget` remains caller-owned.
  - First-error offsets that callers or tests rely on may change ordering.
- **Unsettled premises and what would settle them:**
  - Whether authors need more than the first syntax error. That is a product question for
    CLI, MCP and editor consumers.
  - If not, the installed combinator library covers the same parse-form ground with no new
    dependency.
- **Evidence:** Interface-checked (docs); application Proposed.

**Reviewer correction (2026-10-27).** The comparative review retains the current parsers: most of the parser is context-sensitive domain logic, a combinator library only moves the cursor, and no tool derives the printer. It suggests two optional local hardenings instead (exhaustive walker arms and a generator variant witness). Multi-error recovery is a new authoring capability that belongs to a functional-extension review, not to leverage. See the [comparative review](../../reviews/declarations-registry-and-python-boundary.md#cbo-037).
