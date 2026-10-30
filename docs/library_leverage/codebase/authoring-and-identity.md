# Authoring language, physical semantics and identity

This page covers `pse-authoring`, the front half of `pse-modeling`, `pse-quantity`, `pse-vocabulary`, `pse-ids` and the package-loading route in `pse-admission`. Contracts are owned by
[§9](../../authoritative_design/sections/physical-semantics.md#section-9),
[§10–§12 and §22](../../authoritative_design/sections/models-and-composition.md#section-11), and
[§5](../../authoritative_design/sections/identity-and-publication.md#section-5). The baseline is on
the [entry page](README.md). Mechanisms were read statically and are *Interface-checked*; every value claim is *Proposed* and nothing is *Measured*. Counts come from `rg` and `wc` and are shape indicators.

**System picture.** Authored `.pse` sources become declaration rows, then checked, physically typed, specialized mathematics. Consumers are `pse-admission` (load, edit, reparse), `pse-compiler`, `pse-runtime` and the host commands.

- **Two grammars on one lexical base.** The expression grammar (`dsl/`) is a token-slice recursive-descent parser with precedence climbing. The declaration language (`language/`) is a second token cursor with a roughly 1k-line renderer and a parse, render, parse round-trip test.
- **winnow.** It tokenizes (`dsl/lexer.rs`, `grammar.rs`, `language/static_value.rs`), with `LocatingSlice` offsets. Both parsers consume its tokens.
- **`ParseBudget`.** It bounds bytes and nesting depth before expansion.
- **Units and dimensions.** A registry-driven domain algebra with rational exponents, offsets, datums and physical-type contracts (ADR-0124). Registry-owned semantics make a generic units or unification library a poor fit, and that is legitimate bespoke work.
- **Identity.** blake3 derive-keys with length-prefixed framing. `pse-ids/src/document.rs` frames any serde value through a canonical `Framer` without a hand-written field list. The `vocabulary!` macro derives spellings, serde, `FromStr` and `ALL` from one table.
- **Already library-native.** Graph work in this area uses petgraph and rustworkx. `pse-structural` delegates matching, DM and BTF. Salsa is used with LRU, durability and interning. Specialization, rewrite and fold operate on authored declarations before `pse-math` and do not overlap Symbolica.

Opportunities cluster in three places: the hand-written language front end ([CBO-113](#cbo-113), [CBO-037](#cbo-037), [CBO-114](#cbo-114), [CBO-038](#cbo-038)), per-type identity framing ([CBO-039](#cbo-039)), and the package loader ([CBO-115](#cbo-115)). The resolved-type wire hierarchies and the second IR belong to [CBO-110](mathematics-and-compilation.md#cbo-110) and [CBO-109](mathematics-and-compilation.md#cbo-109).

**Coverage on this page.** Read closely: `language/parser.rs` (entry points and about 260 lines of `item`), the whole of `dsl/lexer.rs`, `dsl/walk.rs` by function list and counts, `pse-admission` `document/edit.rs` and the load route (`load.rs`, `value.rs`, `decode.rs`, `hydrate.rs`, `spans.rs`, `allocation.rs`), `pse-quantity` `premises`, and `pse-ids` heads. Read at heads or signatures only: `language/render.rs`, `dsl/parser.rs` beyond its prologue and unit grammar, `pse-modeling` `check*`, `expression*` and `resolved*`, `pse-quantity` registry and unit bodies, `pse-admission` `owned.rs`, `physical/*`. Not read: the remaining 1,500 lines of `Cursor::item`, `specialize.rs::instantiate_body`, `check.rs` `inventory_package` and `finish_package`, and the field-by-field bodies of the hand `frame` methods. About 15k lines of domain-dense modeling code were judged from module documents and signatures, which is a claim from structure, not from reading.

<a id="cbo-039"></a>

## CBO-039 — Identity framing written by hand, with silent omission possible

- **Operation and consumers.** Content identity for resolved records, schemes, units, selections, envelopes, derived items and preparation inputs. Consumers are publication, replay, caches and same-point reuse decisions.
- **Current mechanism.**
  - `pse-ids/src/document.rs` provides a serde `Serializer` (`Framer`) so that a field added to a settings type changes identity with no field list. It has 13 workspace call sites, for example `consistent::profile_identity`, `math/solves.rs` and `fitting/preparation.rs`.
  - Elsewhere, hand-written `frame` methods implement `pse_model::SemanticFrame`, whose byte layout differs from the serde `Framer`. `rg` finds 65 `fn frame(` sites outside `generated/`, in `pse-model/src/lib.rs` (12), `pse-modeling/src/derived.rs` (10), `contextual.rs`, `pse-quantity/src/{resolved,unit,scheme,premises}.rs`, `resolved/support.rs` and `pse-codegen` emitters.
  - A regex scan of 60 non-codegen bodies found 34 that read `self.field` with no destructuring or `match self`. That scan does not judge selectivity and includes small newtype frames with nothing to omit.
  - Preparation identities are enumerated by hand at the use site. `prepare_simulation_mode` walks `ModelingDynamicV3` in about 100 lines, and `artifact_request` repeats field lists for two hashes. Some frames hash rendered text (`applicability.rs::Use::frame`, `contextual.rs` through `dsl::render_expr`), so the renderer's spelling is identity there. Other identities hash JSON text (`controls_identity`, canonical object versions through `serde_json::to_vec`).
- **Why it matters.** A field added to a hand-framed type can silently fail to affect identity, which is the failure the serde framer exists to prevent. Reuse then widens without notice. This is the nearest identity-soundness exposure in the area (Proposed).
- **Patterns.** LOP-02, LOP-X007, LOP-09.
- **Evidence.** `pse-ids/src/document.rs` (`Framer`), `pse-model/src/lib.rs` (`SemanticFrame`), `pse-quantity/src/scheme.rs::frame`, `resolved/framing.rs` head, `dynamics.rs` hash block in `pse-runtime`, `artifact_request`, `project_comparison.rs::commitment`. The 65 count and the 34-of-60 scan are `rg`-level.
- **Footprint.** Inspected: the sites named above and the framer. Suspected but unread: full bodies of most hand frames. No actual omission was found, and none was audited for.
- **Preservation constraints.**
  - Frame tags, field order and frame versions (`V10`, `V11` historical meaning) are identity; any change re-identifies.
  - `pse-ids` golden vectors, `f64` as bits, deliberate exclusions such as spans, canonical first-use ordinals.
  - Graph canonicalization in `resolved/framing.rs` is legitimately bespoke.
  - Exhaustive framing must reach manually selected nested records as well as the receiver: `Node::frame` selects `Claim` fields, so an exhaustive `Node` match alone does not force a decision about a new `Claim` field (reassessment, 2026-10-27).
- **Counterevidence and uncertainty.**
  - Switching to the serde framer changes bytes, so it is not a realistic route. A byte-preserving guard, such as mandatory destructuring or a derive over the existing `FramedHasher` order, would make a new field fail to compile without touching any identity.
  - Many frames select a subset deliberately, and independently written frames are a check against accidental coupling. A derive can over-include and churn identities.
  - `derived.rs` is partly declarative (`relationships!` with its own leaf layout, read 2026-10-27), yet the scan also counts field-reading frames there. Whether those are the expansion of the declarative form was not established.
- **Status.** Current mechanism. A compile-time completeness guard is Proposed.
- **Research questions.**
  - Which hand frames are full-field, which selective and which canonicalized?
  - Can a derive with skip and order attributes reproduce the existing bytes for the full-field class?
  - Can the preparation walks move into the types they identify?

<a id="cbo-113"></a>

## CBO-113 — The modeling language has a semantic parser but no language toolchain

- **Operation and consumers.** Package authors and agents write and revise `.pse` sources. Admission and edit routes need exact field spans, and `assign_ids` rewrites source.
- **Current mechanism.**
  - Two token-cursor parsers sit on the winnow tokenizer: `language/parser.rs::Cursor` (about 3.7k lines) and `dsl/parser.rs::Cursor` (about 780 lines, with near-duplicate `eat`, `expect` and `error`).
  - A declaration keeps expression fields as raw text (`Cursor::until`). Those fields are tokenized and parsed again later (`expression/occurrences.rs`, `ParsedFields::parse`).
  - The tree is not lossless. The lexer drops `//` comments, and `assign_ids_with` ends in `render(&rows)`. By reading, `pse-admission` `document/edit.rs::assign_ids` takes the whole rendered output as `DocumentEdit.after`, so assigning ids to a `.pse` file strips comments and reformats the document. YAML documents in the same function use byte insertions at parser ranges. No test asserts either behavior.
  - Every syntax error returns at the first `?`.
  - Searches of `Cargo.toml` files, source and `git ls-files` found no CST, LSP server, formatter or editor grammar. No plan or functional review addresses `.pse` tooling, by topic search.
- **Why it matters.** Authoring is the main write path, and scientific knowledge is meant to live in authored packages (blueprint §22.3). Possible gains: comment-preserving rewrites, a formatter, all syntax errors in one pass, hover with physical type and unit, go-to-definition across revisions, and exact spans as a by-product of a tree. The incremental semantic owner (Salsa queries in `pse-compiler`) already exists (Proposed).
- **Patterns.** LOP-12, LOP-06, LOP-02.
- **Evidence.** `language/parser.rs` entry points and `Cursor`, `dsl/lexer.rs` in full, `dsl/parser.rs` head, `grammar.rs`, `render.rs` head, `occurrences.rs` head, `pse-admission/.../document/edit.rs` in full.
- **Footprint.** Inspected as above. Admission-side rendering and the CLI product `Query` route for authoring operations were not examined for tooling.
- **Preservation constraints.** Parse budgets and depth limits that refuse before expansion; named and explicit identity policy; exact field byte ranges with text kept verbatim; the generated `Declaration` row as the semantic product; Unicode symbols; grammar version facets in the registry; one parser authority, since a hand-maintained editor grammar would be a second spelling.
- **Counterevidence and uncertainty.**
  - Committed-keyword parsing gives precise errors and is small relative to the language. Raw expression text is partly deliberate because rows carry exact authored bytes.
  - Demand for editor features is unestablished. The main authors are agents, who may value batch diagnostics ([CBO-114](#cbo-114)) more. Draft save and admit and `project_draft_delta` already give feedback at save granularity.
  - Whether the comment loss is acceptable policy has not been decided.
- **Status.** Parsers and the comment-dropping rewrite are the current mechanism. Toolchain capabilities are Proposed.
- **Research questions.**
  - Can a lossless syntax tree sit beneath the current cursors without changing rows or spans?
  - Does error recovery fit the committed-keyword style?
  - What minimal semantic query surface could an editor adapter reuse?

<a id="cbo-037"></a>

## CBO-037 — Grammar facts in several hand-written forms

- **Operation and consumers.** Adding or changing a declaration kind, field or expression form. Consumers are `pse-admission` load and edit, workflow code and tests.
- **Current mechanism.** Three distinct obligations are maintained by hand.
  - **Concrete syntax.** The registry (`pse-schema/src/catalog/modeling.rs`) generates payload structs, `SemanticEq`, schema and a visitor that follows each field's declared grammar (`SourceGrammar`). Surface syntax is separate: `Cursor::item` has a hand arm per keyword, `render.rs` (17 functions) mirrors them in reverse, and the round-trip tests (`kernel_language.rs`) pin the pair. Literal spellings such as `"big_m"` are refused by hand beside generated enums.
  - **Traversal.** `dsl/walk.rs` (703 lines, 63 `ExprKind::` arms) holds `walk_expr`, `strip_expr`, `paths_expr`, `mutate_expr` and `free_expr`. `expression/inventory.rs`, `occurrences.rs::static_parts`, `extent.rs` and `dsl/arbitrary.rs` also enumerate variants, and `structural_eq` clones both trees to strip spans. Field-span capture (`field_spans.rs`) is a further form.
  - **Binders.** `selected_source.rs::lexical_bindings` hand-chains the six declaration families that carry binders, and `requirements` is similar.
  - Keyword spellings are also matched as strings, for example `intrinsic_callee` in `pse-modeling/src/expression.rs` and the `"tuple" | "set_of"` match in `specialize/value.rs`.
- **Why it matters.** A new construct touches more than the five forms first estimated, and a missed traversal silently under-reports dependencies for selected-source tracking. Declaring concrete syntax beside the field, deriving traversal from the AST, or holding binders in the declaration schema would reduce drift and diff size (Proposed). The round-trip test covers concrete syntax only.
- **Patterns.** LOP-02, LOP-X003, LOP-X007.
- **Evidence.** Both parser prologues, `unit_product`, `unit_power`, `unit_exponent`, `dsl/walk.rs` head and counts, `dsl/arbitrary.rs` head, `language/field_spans.rs`, `selected_source.rs`.
- **Footprint.** Inspected: the files named. Suspected: about 8k lines across parser, renderer, walker and generator code, and the remaining arms of `item`. The second IR (`resolved::Node`, `framing.rs`, `transform_graph.rs`) is [CBO-109](mathematics-and-compilation.md#cbo-109).
- **Preservation constraints.** Existing spellings and alternative forms (`bigm(derived, m)`, `connect name:`); exact byte spans per field; parse, render, parse preserving structure and ids (§7.7); first-error messages; renderer independent of parser state; `ParseBudget`.
- **Counterevidence and uncertainty.**
  - Syntax is irregular and English-like by design, and the grammar is context-sensitive (kind-level binding, type-variable scope and named identity derivation occur in the parse). A declarative description could flatten or complicate it.
  - The expression grammar has about eleven main variants and changes rarely. Semantic walkers (`infer`, `Evaluator`, `rewrite`) legitimately stay hand-written.
  - Replacing the cursors with winnow combinators looks lateral and removes none of the parallel forms. The hand AST generator bounds shapes deliberately, and an independent generator is assurance.
- **Status.** The parsers are a current promise. Consolidation is Proposed.
- **Research questions.**
  - Can a bidirectional or invertible syntax description carry the irregular arms?
  - Can the registry's existing `SourceGrammar` declaration also drive parsing and printing?
  - Can walker and generator be derived from the AST type?

<a id="cbo-114"></a>

## CBO-114 — Checking reports one failure per cycle

- **Operation and consumers.** An author or agent fixing a package; the admission driver and host commands that report check results.
- **Current mechanism.**
  - Checking and specialization flow as `Result<_, ModelingError>` through `?`. `check_declarations` in `pse-modeling/src/check.rs` stops at its first failure.
  - The carrier for several failures exists: `TypedDiagnostic::diagnostic_children` and `pse_model::diagnostic::project_typed` retain aggregates and locations recursively, and `ModelingError::located` attaches a span and declaration name.
  - By `rg`, no modeling or compiler error returns multiple children, and `AuthoringError` and `CompileError` return `related` as `None`.
- **Why it matters.** Each fix-and-run cycle reveals one problem, so a package with several independent errors needs several full cycles. Declaration-granular Salsa queries are a natural unit for collecting independent failures, and spans are already held (Proposed).
- **Patterns.** LOP-03, LOP-12.
- **Evidence.** `rg` over the three crates and the error enum heads, the `CompileError` macro block, the `AuthoringError` head, `check_declarations`. The bodies of the later check phases were not read.
- **Footprint.** Suspected but unread: `phases.rs::finish`, `check.rs` `inventory_package` and `finish_package`, and whether `pse-runtime` or `BoundaryDiagnostic` aggregates at a higher layer.
- **Preservation constraints.** Later phases must not run on invalid prerequisites or emit cascades; deterministic ordering; §23.2 codes; transient refusals stay separate from semantic failures.
- **Counterevidence and uncertainty.** First-failure reporting keeps cascades out and simplifies determinism. The check phases may order dependencies so that few failures are independent. How collection would compose with the driver that feeds results back into Salsa ([CBO-107](mathematics-and-compilation.md#cbo-107)) is unsettled.
- **Status.** Proposed.
- **Research questions.**
  - Which declaration-level failures are independent of each other?
  - Can Salsa accumulators, or an aggregating error variant over the existing carrier, collect them without cascades?

<a id="cbo-038"></a>

## CBO-038 — Held spans do not reach human diagnostics

- **Operation and consumers.** Show an author where a parse or check error occurred. Consumers are the CLI and the agent interface.
- **Current mechanism.**
  - `AuthoringError::Syntax` carries a `SourceSpan` and byte offset keyed by document identity (`SemanticId`). Its `facts` project a `DiagnosticLocation {document, start, end}`. `ModelingError::Located` carries span and name into `BoundaryDiagnostic.locations`, at the whole declaration range unless a field owner supplied a narrower one (`expression/occurrences.rs`, `forms.rs`).
  - `impl_diagnostic!` forwards `source_code` and `labels` to a child and ends at `None`. A search for `LabeledSpan`, `NamedSource`, `#[label` and `with_source_code` found only the forwarders in `pse-columnar/src/engine.rs` and `pse-host/src/envelope/diagnostic.rs`.
  - The host renders. `envelope/diagnostic.rs::from_diagnostic` captures labels, resolving line and column when source exists and otherwise recording "source context unavailable". `pse-host/src/cli.rs` renders the native error with `GraphicalReportHandler` or `NarratableReportHandler`.
  - So machine consumers receive structured locations. What is missing is the attachment of document text to authoring and modeling errors at the point where the host renders them.
- **Why it matters.** The renderer and the span are both in place, so attaching source may be a small change that turns offsets into annotated excerpts for authors (Proposed).
- **Patterns.** LOP-03, LOP-12.
- **Evidence.** The searches above; `pse-diagnostics/src/lib.rs` macro; `pse-host/src/envelope/diagnostic.rs`; `pse-host/src/cli.rs`; `error.rs` in `pse-authoring`. The host renderer was added in `27a827495` (2026-10-30).
- **Footprint.** Whether host commands hold document text at failure time (the driver holding `CapturedPackage.files` and `pse_ids::named_id(package, path)` identities) was not traced. The MCP route was not checked.
- **Preservation constraints.** Wire and Arrow diagnostic forms must not carry source text. ADR-0224 keeps original source-bearing messages private on the agent interface, so rendering belongs at the source-owning presentation boundary and never in the report or envelope. Spans name documents by `SemanticId`, not path; codes follow §23.2.
- **Counterevidence and uncertainty.** Many failures supply only the whole declaration range, so excerpts would be coarse until narrower spans are supplied. Excerpts of the first error do not address [CBO-114](#cbo-114).
- **Status.** Host rendering is a current mechanism. Source attachment is Proposed.
- **Research questions.**
  - Where does a `SemanticId`-to-text map attach at failure time?
  - Can `LabeledSpan`s be derived from `SourceSpan` and `DiagnosticLocation` at the host boundary?

<a id="cbo-115"></a>

## CBO-115 — Authoring load takes a long route with parallel span and accounting forms

- **Operation and consumers.** Package loading for all model runs (`load_package`, `load_reusing`) in `pse-admission`.
- **Current mechanism.**
  - YAML passes through `serde_saphyr` into a local `Spanned<Value>` tree (`value.rs`, a custom `Visitor` for duplicate keys). TOML passes through `toml::Spanned<toml::Value>` into the same `Value`.
  - `spans.rs` builds a `BTreeMap<String, SourceSpan>` of stringified paths. `hydrate.rs` injects context and identity by mutating `Value` in a fixed-point loop. `decode.rs` implements a `serde::Deserializer` over `&Value` into generated strict DTOs, and `generated/document_adapters.rs` pushes those into Arrow builders. P0 and physical admission decode views back to rows.
  - `allocation.rs` lexes the source to estimate node counts before parsing, duplicating a budget that saphyr also enforces (`max_nodes`, `max_events`). `allocation.rs` and `load.rs::fresh_payload_extent` keep further shape-specific sizing.
- **Why it matters.** Source attribution travels in a parallel path-keyed map, not in the values, so every consumer of a location re-derives a path string. Parsing and accounting grow with each pass, and error mapping spans three error families. Fewer hops would simplify the loader (Proposed).
- **Patterns.** LOP-06, LOP-03, LOP-X001.
- **Evidence.** `load.rs` heads and function list, the head of `value.rs`, `decode.rs`, `hydrate.rs` head, `spans.rs`, `allocation.rs`, `document_adapters.rs` head, `read.rs` head, the head of `data.rs`.
- **Footprint.** `owned.rs`, `edit.rs` and the middle of `load.rs` were skimmed. The question of how many times one registry record is translated between YAML, structs, Arrow and generated Rust was not traced end to end.
- **Preservation constraints.** Exact retained bytes and edit preimages; deny-unknown-field strictness; stable identity injection order; duplicate-key refusal; alias budgets; span fidelity including alias reference ranges.
- **Counterevidence and uncertainty.** Generated strict DTOs cannot carry spans without a second type system, and `Spanned<T>` per field is a heavy projection. Hydration needs a mutable tree. Documents are small and loading is infrequent, so simplicity, not speed, is the value.
- **Status.** Current mechanism.
- **Research questions.**
  - Can the saphyr deserializer report spans by path so that a span index is an output of one parse pass?
  - Can the pre-parse node estimate reuse the parser's own limits?

## Examined and not retained

- **Physical-type inference and schemes.** `pse-quantity` `infer`, `scheme`, `resolved`, `unit` and `conversion` implement registered operation rules, delta types, datums and contracts. No generic unifier or type-level units library expresses authored registries, and exact canonical bits with datum refusal are contract. The solver loop was not read.
- **`Ratio` against a rational crate.** The checked `i16` canonical form, refusal of unreduced stored bytes and `const` constructors are contract, and the code is small.
- **`p0.rs` package resolution.** Versions are exact over a complete inventory, so there is nothing for a version solver to solve.
- **`pse-model` `study_policy`, `strategy`, `numerics`.** Pure typed policy with petgraph for cycle checks. `NumericalPolicy::validate` is hand range-checking that a validation crate could partly type, but the checks are few and cross-field.
- **Generated enum volume and `vocabulary!`.** About 29.5k generated lines of enums share a mechanical derive set. Compile cost was not measured and no burden beyond size was found.
