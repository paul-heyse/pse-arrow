# Parser generators and bidirectional grammar tools (rust-peg, pest, LALRPOP, tree-sitter)

Realizations considered:
- None of these is a direct dependency.
- `pest` 2.9.2 is in the lock only transitively, through a vendored `pest_derive` used by
  `py_literal`.
- Upgrade routes: `peg` 0.8.6 (released 2026-05-04) and `lalrpop` 0.23.2 (released
  2026-10-19).
- tree-sitter was not version-resolved.

Research date 2026-10-27. Sources:
- crates.io API (versions and dates).
- A web search for a Rust invertible or bidirectional parse-and-print crate. It found only
  the Haskell `invertible-syntax` package (Rendel & Ostermann 2010), `syn`'s separate parse
  and print paths, and print-only crates.
- General knowledge of each tool's model, not re-verified at these versions.

This file is a grouped negative and conditional record. It holds one type, which describes
the need and the scope of the negative result.

## Types

### grammar-generators/declarative-grammar-with-printing
- **Need it serves:** Derive a parser from a declarative grammar and, ideally, derive the
  matching printer from the same definition, so that parse and render cannot drift.
- **Recognizable code shapes:**
  - A hand-written parser and a hand-written renderer for one language, kept in sync only by
    a round-trip property test.
  - Keyword spellings repeated across the parser, the renderer and validators.
- **Mechanisms and APIs:**
  - rust-peg: the `peg::parser!{ grammar ... }` macro generates a PEG parser in-crate.
    - Rules can call arbitrary Rust, and it can parse custom token slices through
      `Parse`/`ParseElem`.
    - `position!()` captures spans.
  - pest: a `.pest` grammar plus `#[derive(Parser)]` produces a dynamic `Pairs` tree with
    spans.
    - The AST is built by a separate walk.
    - It parses text only, not token vectors.
  - LALRPOP: an LR(1) generator run at build time.
    - It accepts an external lexer that yields `(start, Tok, end)` triples.
    - Actions are arbitrary Rust, and errors carry spans.
  - tree-sitter: an incremental GLR parser for editors.
    - The grammar is written in JavaScript and compiled to C.
    - It produces a concrete syntax tree with error nodes.
- **Obligations discharged:** Grammar-to-parser generation, and with it the cursor,
  backtracking and precedence code, which is declared rather than coded.
- **Residual caller obligations:**
  - AST construction, which is the main cost for pest and tree-sitter.
  - Context-sensitive checks.
  - Budgets.
  - Printing, in every case.
- **Semantic conditions:**
  - PEG ordered choice changes ambiguity resolution relative to hand recursive descent.
  - LR grammars reject constructs that need context.
  - tree-sitter's error-tolerant trees always succeed and need a separate validity pass.
- **Version / build conditions:**
  - LALRPOP and tree-sitter add build-time generation steps.
  - tree-sitter adds a C toolchain dependency.
- **Limits and negative results:**
  - No maintained Rust crate was found that derives both parser and printer from one grammar
    (an invertible syntax description).
    - Search scope: a web search for Rust bidirectional or invertible parse-and-print
      crates; crates.io knowledge of `pretty` 0.12.5 (Wadler-style printer only) and
      `genco` (code emission only).
    - An empty search does not prove absence.
  - Parse and print remain two definitions with every tool here.
- **Evidence:** Interface-checked for versions only (crates.io API). Tool models are from
  documentation knowledge and are not re-verified at the listed versions. The negative is
  bounded by the search scope above.

## Current applicability

### [CBO-037](../../../library_leverage/codebase/authoring-and-identity.md#cbo-037) — Grammar facts maintained in several hand-written forms
- **Types:** `grammar-generators/declarative-grammar-with-printing`.
- **Realization:** Upgrade routes only, each a new dependency. pest is present transitively,
  which gives no direct-use right or advantage.
- **Conclusion:** Ruled out here for consolidating the five forms, because no candidate
  derives the renderer. Unsettled as a parse-form-only replacement, where rust-peg over the
  existing token slice is the closest fit.
- **What it would discharge / what remains local:**
  - A grammar declaration would replace the cursor code.
  - Renderer, walker, generator, keyword tables, identity derivation, budgets and the
    raw-text expression capture all remain local.
- **Prerequisites, configuration or extension:**
  - rust-peg: implement `peg::Parse` and `ParseElem` for the token slice, and keep the
    `Stateful`-like context in grammar arguments.
  - pest: re-lex from text. It cannot consume the existing token vector.
- **Semantic gaps against the CBO's preservation constraints:**
  - Context-sensitive identity derivation and scopes happen during parse. In PEG and LR
    actions they become side-effecting actions, which is harder to audit than the current
    code.
  - pest's text-only input and separate AST walk would add a translation layer (LOP-06).
- **Unsettled premises and what would settle them:**
  - Whether one keyword table, a local `const` set shared by lexer, parser and renderer,
    removes most of the drift without any grammar tool. A local count of the keyword
    occurrences would settle it.
- **Evidence:** Interface-checked (versions only); application Proposed.
