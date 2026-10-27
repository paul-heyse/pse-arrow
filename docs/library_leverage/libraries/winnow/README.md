# winnow

Realizations considered: installed `winnow` 1.0.4 (released 2026-07-13; the workspace enables
`std`, and the consuming crate adds `ascii` and `parser`). No upgrade is needed: 1.0.4 was the
newest release on crates.io on 2026-10-27.
Research date 2026-10-27. Sources: installed source `winnow-1.0.4/src/stream/token.rs`,
`src/stream/stateful.rs`, `src/combinator/expression.rs`, `src/parser.rs` (`with_span`),
`src/error.rs` (`StrContext`), `src/macros/dispatch.rs`, `src/_topic/lexing.rs`, and the
`Cargo.toml` feature table, all at 1.0.4.

## Types

### winnow/token-stream-combinators
- **Need it serves:** Parse an already-lexed token vector with combinators instead of a
  hand-written cursor (`peek`/`eat`/`expect`/`error`/position bookkeeping). Byte spans must
  still come from the original text.
- **Recognizable code shapes:**
  - A `struct Cursor { tokens: Vec<Token>, pos }` with `peek`, `eat(&str)`, `expect(&str)`
    and `error(expected)`.
  - Spans recovered by reading `tokens[pos-1].span.end`.
  - The same cursor primitives copied into more than one parser that consumes one lexer.
- **Mechanisms and APIs:**
  - `winnow::stream::TokenSlice<'t, T>` is a `Stream` over `&[T]`.
    - It implements `Compare<O>` when `T: PartialEq<O>`, so a `&str` literal or a token-kind
      value works directly as a parser: `"(".parse_next(i)`, `literal(Kind::Ident)`.
    - `ContainsToken` gives token-set matching with `one_of`.
    - If `T: Location`, the slice implements `Location` from the tokens' own offsets
      (`current_token_start`, `previous_token_end`). `Parser::with_span()` and `.span()` then
      return byte ranges in the original text.
  - `Stateful<I, S>` threads mutable parser state through every combinator. Examples are a
    depth counter, identities seen so far, a type-variable scope stack and a field-location
    map.
  - `combinator::expression(operand).prefix(..).infix(..).postfix(..)` is a Pratt parser with
    binding powers and `Left`/`Right` associativity. Fold closures receive the input and can
    fail.
  - `dispatch!` provides committed keyword branches. `cut_err` commits an alternative.
  - `ContextError` with `.context(StrContext::Expected(StrContextValue::Description(..)))`
    collects expectation context.
  - `TokenSlice::previous_tokens()` helps build error context.
- **Obligations discharged:**
  - Cursor movement, backtracking and checkpoints.
  - Alternative and repetition combinators.
  - Precedence climbing.
  - Span computation from token locations.
  - Expectation accumulation for errors.
- **Residual caller obligations:**
  - The lexer remains the caller's. It can itself be winnow over `LocatingSlice<&str>`.
  - Mapping `ContextError` into the domain error, including the exact "expected / found /
    offset" shape.
  - Budgets and depth limits: winnow has no built-in recursion budget, so a counter lives in
    `Stateful` state.
  - Context-sensitive decisions such as identity derivation and scope lookups, written as
    ordinary Rust inside parsers.
  - Printing. winnow is parse-only.
- **Semantic conditions:**
  - Offsets are whatever the token's `Location` impl returns. Byte-exactness depends on the
    lexer.
  - At end of input, `current_token_start` falls back to the previous token's end and then
    to 0. A caller that reports "end of input at text length" must special-case it.
  - `ErrMode::Backtrack` and `ErrMode::Cut` decide whether alternatives are tried. A
    committed keyword branch must use `cut_err`, or a later alternative can mask its error.
  - `ContextError` reports one position, the farthest or committed one. There is no
    multi-error recovery on `TokenSlice`: `Recover` returns `Err` and
    `is_recovery_supported()` is false, and recovery is behind the `unstable-recover` feature.
- **Version / build conditions:**
  - `TokenSlice`, `expression` and `Stateful` are present in 1.0.4 under the `parser` feature.
  - `unstable-recover` is an unstable feature.
  - The `debug` feature (`trace` output) pulls in `anstream`, `anstyle` and `terminal_size`.
- **Limits and negative results:**
  - No bidirectional (print) support.
  - No derive for AST visitors or generators.
  - No error recovery over token slices in the stable feature set.
- **Evidence:** Interface-checked against installed `winnow-1.0.4` source: `TokenSlice` impls
  for `Location`, `Compare` and `Recover`; the `expression` signature and doc example;
  `with_span` doc example; feature table.

### winnow/located-text-lexing
- **Need it serves:** Byte-offset-accurate lexing directly over `&str`.
- **Recognizable code shapes:** Manual `char_indices` loops with offset arithmetic, and
  `str::find` slicing.
- **Mechanisms and APIs:**
  - `LocatingSlice<&str>`.
  - `ascii::{multispace0, float, digit1}`.
  - `token::{one_of, take_while, any}`.
  - `.take()` returns the consumed slice.
  - `Location::current_token_start` gives the offset.
- **Obligations discharged:** Character classification, number recognition and offset
  tracking.
- **Residual caller obligations:**
  - Domain token rules such as range-vs-decimal disambiguation and non-finite rejection.
  - Byte budgets.
- **Semantic conditions:** `ascii::float` accepts forms that `str::parse::<f64>` may treat
  differently, for example `inf` and `nan` spellings. Callers that forbid non-finite values
  must check after recognition.
- **Version / build conditions:** Requires the `ascii` feature.
- **Limits and negative results:** None beyond those of the token-stream type.
- **Evidence:** Interface-checked against installed source `src/ascii`, `src/stream/locating.rs`.

## Current applicability

### [CBO-037](../../../library_leverage/codebase/authoring-and-identity.md#cbo-037) — Grammar facts maintained in several hand-written forms
- **Types:** `winnow/token-stream-combinators`, `winnow/located-text-lexing`.
- **Realization:** Installed pin 1.0.4. No dependency change. `TokenSlice` and `expression`
  need only the `parser` feature the consuming crate already enables.
- **Conclusion:** Fits with conditions. It can replace both token cursors and the
  hand-written precedence climber, but it discharges only the parse form. Renderer, walker,
  generator and keyword-table drift are untouched.
- **What it would discharge / what remains local:**
  - Discharged:
    - The duplicated cursor primitives (about 80 lines in each parser).
    - `binary()` precedence climbing, through `expression` with `Right` associativity
      for `^`.
    - Manual span bookkeeping, through `with_span` once the `Token` type implements
      `Location`.
    - Keyword commitment, through `dispatch!` and `cut_err`.
  - Remains local:
    - The bracket-balanced `until(stops)` capture of raw expression text, unless expressions
      are parsed inline.
    - Identity derivation (`named_id`), kind-level binding (`=bind`), type-variable scope and
      the field-location map. These all become `Stateful` state.
    - `ParseBudget` depth and byte limits.
    - Error mapping to `AuthoringError::Syntax{at, offset, expected, found}`.
    - All renderers, the walker and the generator.
- **Prerequisites, configuration or extension:**
  - Implement `winnow::stream::Location` for the lexer's `Token`.
  - Define a `Stateful<TokenSlice<'_, Token>, ParseState>` input type.
  - Write an error adapter from `ContextError` with `StrContext::Expected` to the domain error.
  - The lexer already uses `LocatingSlice`.
- **Semantic gaps against the CBO's preservation constraints:**
  - *Exact byte spans per field:* preserved if spans come from token `Location`. The current
    `field_spans` pointer-keyed capture could instead be fed by `with_span` at the `word`,
    `path` and `until` sites. The serde-walk join from field path to span stays local.
  - *ParseBudget / nesting:* not provided. It must be a state counter, as today.
  - *Deterministic identity:* unaffected, because it is caller code.
  - *Canonical render text:* out of scope for winnow.
  - *`SourceSpan` errors keyed by document identity:* via the adapter.
  - Error wording and position at end of input must be checked against current tests:
    `found: "end of input"` and the span at text length.
- **Unsettled premises and what would settle them:**
  - Whether the rewrite is lateral. The context-sensitive parse means most of the 3.7k-line
    declaration parser is domain logic, not cursor logic. A spike converting `dsl/parser.rs`
    (783 lines, mostly expression grammar) and measuring diff and error parity would settle
    it.
  - Whether `ContextError` reproduces current "expected X" messages exactly.
- **Evidence:** Interface-checked (installed source as above); application Proposed.
