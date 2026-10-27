# Declarations, registry, Python boundary and diagnostics (G8)

**Group G8 of the [comparative review](README.md), 2026-10-27.** Focused comparative advice;
it authorizes nothing.

| Item | Value |
| --- | --- |
| Reassessment | 2026-10-27 at source `4f52fd239`; consumes the complete original review and updated inputs at `b29fcf802`, base patterns 0.2 and additional patterns revision 3 (refined to 4 here). All named dispositions were reconsidered with representative current source, consumers and inherited evidence; unchanged evidence retains its original conditions. |
| Opportunities | CBO-037 to CBO-045, and the templating side of CBO-046. The SurrealQL side belongs to G3. |
| Original source baseline | `main` at `3e9a2525b`. That is 9 commits past the audit baseline `dbc4c9436`. `git diff --stat dbc4c9436..HEAD` shows no change in `pse-schema`, `pse-codegen`, `pse-py`, `pse-diagnostics`, `pse-authoring`, `pse-ids`, `pse-model/src/lib.rs`, `boundary_document_wire.rs` or `python/pse/codec`. |
| Consumed accounts | Codebase account `c2f8b485a`. Library assessment `34f9f07ee`. Additional patterns revision 2, of which LOP-X003 and LOP-X007 are central. Base patterns edition 0.2. |
| Standard | Core 3.4, process-simulator 1.5 and the pse-arrow binding, applied qualitatively. Architecture owners: §4.1–§4.2, §21 and §21.5, §23.2, D1. Rationale: ADR-0116, ADR-0210 (proposed), ADR-0224 (proposed), ADR-0178 (proposed amendment) and ADR-0229 (proposed). |
| Evidence | Targets are *Proposed*. Mechanisms are *Interface-checked* where stated. Library behaviour is *Tested* only where the msgspec probe covers it. The CBO-040 build coupling and compile cost were *Tested* and *Measured* by the coordinator-run [registry build-coupling probe](../../design_review/evidence/library-leverage-registry-build-coupling-2026-10-27/README.md); no other quantitative benefit is claimed. |

## Group summary

**One joint decision runs through the group.** The typed Rust declarations stay the single
authority. That covers:
- identity bytes;
- diagnostic codes;
- boundary documents;
- the Python surface.

Libraries are used where they render, encode or expose a fact they do not own:
- miette renders diagnostics;
- msgspec decodes typed documents;
- PyO3 publishes classes and stubs;
- postcard is a possible carrier for registry data.

Library routes that would create a second spelling of a typed fact are declined:
- string codes in the miette derive;
- the serde framer used for identity;
- msgspec tags used as the header authority;
- an external schema-to-Python generator.

Several premises turn round when the source is read:
- **CBO-045.** The local forwarding macro is the single authority. The drifted copies are the
  library-derive sites.
- **CBO-041.** The "duplicate check in two languages" is one Rust rule. It is re-run on data
  that crosses no trust boundary.
- **CBO-040.** The "three encodings of the same products" are three sub-products, each encoded
  once.
- **CBO-039.** The real exposure is struct frames that do not destructure their fields. The
  framing mechanism itself is not the problem.

**Reinforcing and competing choices.**
- CBO-039 and CBO-045 could each justify a local proc-macro crate. That would be a crate
  addition, so it needs an ADR and a design review. This review recommends neither. Both are
  met by zero-dependency idioms:
  - exhaustive destructuring for frames;
  - an extended `impl_diagnostic!` for diagnostics.

  If the maintainer still wants derives, one shared crate under one ADR should serve both.
- CBO-041 and CBO-044 reinforce each other. Both cache Python types in generated per-type
  `PyOnceLock` statics. Both pass typed documents directly to native signatures.
- CBO-038 builds on the CBO-045 authority. It renders from the structured facts that the typed
  projection already produces.

**Interactions with other groups.**
- **G3, CBO-046 SurrealQL side.** If G3 adopts a template engine, the two Python `@X@` templates
  could share it. The Python recommendation here stands on its own either way.
- **G2, CBO-017.** The signal-aware waits live in the same `#[pymethods]` that CBO-044 would
  make public. Sequence the work together, so the wait helper is not written twice.
- **G9, CBO-052.** "Several command surfaces without a shared declaration" may cover the
  generated host-operation catalogue. CBO-044's `Runtime` mixins sit on that catalogue
  (`_project_operations_glue.py`).
- **G9, build cost generally.** CBO-040's compile cost belongs to the same developer-loop
  concern.

**Shared prerequisites.** None are hard. Two are soft:
- Extend `impl_diagnostic!` with optional clauses before migrating the derive sites.
- Generate the per-document-type Python handles (CBO-041) before CBO-044 relies on them.

**Priority by consequence.** This is separate from prerequisite order.

| Rank | CBO | Why |
| --- | --- | --- |
| 1 | CBO-045 | A live drift exists between a miette code and a typed code. It is visible in Python exception text against `report.code`. The fix is small and local. |
| 2 | CBO-039 | About 20 struct frames let a new field escape identity silently. Identity is the hashing contract. The fix is small and changes no bytes. |
| 3 | CBO-041 | Every Rust→Python document crossing re-runs untrusted-input checks. The fix is small. |
| 4 | CBO-044 | It can remove forwarding machinery; complete public-family/view coverage is unresolved. It needs an ADR and a design review. |
| 5 | CBO-040 | The probe confirmed that a model change breaks regeneration and that the generated module dominates `pse-schema` compilation. Prefer an embedded product conditionally; its preservation mechanism remains unresolved and amends proposed ADR-0210. |
| 6 | CBO-046 | Local risk reduction in the generator. |
| 7 | CBO-038 | A minor author-facing improvement. |
| — | CBO-037, CBO-042, CBO-043 | Retain current. |

<a id="cbo-037"></a>

## CBO-037 — Grammar facts maintained in several hand-written forms

### Responsibility and target intent

`pse-authoring` turns `.pse` text into declaration rows. Promised behaviour includes:
- exact field spans;
- `ParseBudget` byte and depth bounds;
- deterministic identity assignment during the parse;
- canonical rendering;
- the expression grammar, with first-error `AuthoringError::Syntax {at, offset, expected, found}`
  for authors.

The cursor implementation is incidental.

Corrections confirmed at HEAD:
- winnow already lexes for both parsers.
- `dsl/arbitrary.rs` is hand-written proptest strategies.
- `dsl/walk.rs` is almost entirely exhaustive matches. The exceptions are two wildcard arms, in
  the predicate mutation and free-path helpers.
- The round-trip property test is the declared consistency mechanism for the parse/render
  inverse pair (DP-23 names round trips across representations).

### Credible complete options

1. **Retain current.**
2. **winnow structural parsing on the installed 1.0.4.**
   - APIs: `stream::TokenSlice`, `Stateful<I, S>`, `combinator::expression` (Pratt), `with_span`,
     and `dispatch!` with `cut_err` (see `winnow/token-stream-combinators`).
   - This covers the parse form only.
3. **chumsky 0.13.0** with `pratt` and recovery combinators (`chumsky/token-input-with-recovery`).
   This is a new multi-error capability.
4. **Parser generators** (`grammar-generators/declarative-grammar-with-printing`).
5. **Derives for the other forms.**
   - derive-visitor 0.4.0 for the walker.
   - proptest-derive 0.9.0 for the generator.
6. **Local hardening.** No library:
   - Remove the two wildcard arms.
   - Add a compile-time variant witness in the generator: an exhaustive `match` that must be
     extended when a variant is added.

### Comparative argument

- **Where the cost actually lies.** Most of the 3.7k-line declaration parser is context-sensitive
  domain logic: named-identity derivation, kind binding, type-variable scope and the field-span
  map. Under winnow all of it would move unchanged into `Stateful` state. The cursor primitives
  are about 80 lines per parser. A combinator rewrite therefore buys little locality.
- **What the rewrite costs.** It must reproduce author-facing error text and positions through a
  `ContextError`→`AuthoringError` adapter. This includes the end-of-input position
  (`current_token_start` falls back at end of input). That makes it lateral: behaviour parity
  work for a small reduction (H1, H28).
- **No investigated candidate removes the required renderer.** The bounded search did not
  find a grammar-derived printer meeting this contract; it does not establish ecosystem-wide absence, so
  parsers and renderers stay two definitions whatever tool is chosen. Rust-peg or pest would turn
  identity derivation into side-effecting actions, which is harder to audit (DP-08).
- **The walker is already guarded.** Its exhaustive matches turn a new variant into a compile
  error. derive-visitor is a single-maintainer release from 2024 with undocumented fallible
  traversal, and adds nothing material.
- **The generator is the real gap.** A new variant is silently never generated. proptest-derive
  would move the domain-validity strategies into attributes rather than remove them, so a local
  variant witness closes the gap more directly.
- **chumsky answers a different question.** Its distinctive value is multi-error recovery, a
  product capability for editor or LSP consumers. Its partial-AST risk sits right at identity
  assignment.

### Revealing case

An author's file ends mid-declaration. Today the error reads `found: "end of input"` at the text
length. An unadapted winnow `TokenSlice` may use the previous token's end. Retaining source
length in the adapter can preserve current diagnostics, including trailing whitespace. This is
migration work, not an inherent inability of winnow to preserve behavior.

### Disposition

**Retain current.**
- Optional local hardening, outside library leverage: make the two walker arms exhaustive, and
  add the generator variant witness.
- If multi-error authoring diagnostics are wanted, route chumsky through a functional-extension
  review (LOP-12). It is not a leverage replacement.

### Rule impacts

None.

### Evidence

- winnow and chumsky APIs are *Interface-checked* per their accounts (winnow from installed 1.0.4
  source; chumsky from documentation only).
- Walker and generator observations come from static reading at HEAD.

<a id="cbo-038"></a>

## CBO-038 — Author-facing diagnostics do not render source context

### Responsibility and target intent

Show an author where a failure occurred.

Promised:
- §23.2 typed codes and structured facts;
- `AuthoringError` already projects `DiagnosticLocation {document, start, end}` (UTF-8 bytes)
  through `impl_diagnostic!` `facts(...)`;
- no source text in wire or Arrow forms;
- ADR-0224: on the agent interface, "original source-bearing messages remain private", and
  rendered text supplies no classification.

Today the Python exception text reads `[authoring::parse::syntax] X expected at byte N, found Y`.
No excerpt renderer exists in `crates/`, `xtask/` or `python/pse` (account search, confirmed).

### Credible complete options

1. **Retain current.**
2. **Per-error labels.** `labels()` on each span-carrying error, through a new macro clause, plus
   `Report::with_source_code` at the driver. This is the account's route.
3. **Render from the structured facts at the presentation boundary that owns the source.** This
   route uses `miette` 7.6.0:
   - Build `MietteDiagnostic::new(message).with_code(..).with_help(..)` and
     `.and_labels(LabeledSpan::new_primary_with_span(..))` from the `BoundaryDiagnostic`
     locations.
   - Attach `NamedSource::new(path, text)` through `Report::with_source_code`.
   - Render with `NarratableReportHandler::render_report`, which needs no feature. Alternatively
     use `GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor())`, which needs
     `fancy-base`.
   - A location set that spans several documents renders one block per document.
4. **ariadne or codespan-reporting.** Each is a second diagnostic model, and ariadne defaults to
   char offsets. Both are ruled out by their accounts.

### Comparative argument

**Option 3 serves every error type at once.** It renders from data that already crosses the
boundary in one uniform shape, so the 50-odd error types stay untouched (H4). It keeps a rendered
diagnostic a projection of the finding (§23.2 invariants).

**Option 2 is weaker.** It works only while the Rust error object is still alive next to its
source. That is no longer true once a report has crossed into Python.

**Two placements cover the real consumers.**
- **Load or edit failure.** The boundary that read the documents holds path, text and identity at
  the moment of failure: the admission driver's load path, called from `pse-py`. It can attach the
  rendering to the raised exception as a PEP 678 note (`BaseException.add_note`, CPython 3.14 here).
- **Retained packages.** An explicit render call on the native package object. This placement
  depends on whether the retained package keeps a map from document id to (path, text) after
  admission. Settle this by reading the code, not with a probe.

**Feature choice.** The plain-text narratable handler needs no feature change. The graphical form
needs `fancy-base`, which feature unification turns on workspace-wide. That only adds
`owo-colors` and `textwrap` to builds and changes no behaviour. It is still a revisit of the
recorded pin premise ("without a rendering dependency").

### Revealing case

The agent and MCP interface. If the excerpt went into `DiagnosticReport.message` or the envelope,
authored source would reach the agent projection, contrary to ADR-0224. The target therefore:
- leaves the report and envelope unchanged;
- puts rendering only on caller-local presentation (an exception note or an explicit render call);
- keeps the agent interface consuming reports, never exception text, as ADR-0224 already
  requires.

A second case: a location whose byte offsets do not fall on UTF-8 boundaries, such as a span
reported mid-codepoint by a budget refusal. The renderer must clamp or refuse such a span rather
than slice it.

### Disposition

**Recommend.** This is a minor functional improvement.

Bounded target:
- one presentation helper in Rust that renders a `BoundaryDiagnostic` against a caller-owned
  source map;
- the load-failure path adds the rendering as an exception note;
- optionally, a render method on retained packages;
- narratable by default, graphical if the maintainer accepts `fancy-base`.

Residual work:
- the id→source map at the retained-package boundary;
- span clamping.

Nothing disappears; the addition is small.

### Rule impacts

- None while the rendering stays presentation-only.
- Enabling `fancy-base` is a dependency-feature change, which needs no ADR.
- Putting rendered text into the report or envelope would change the Python boundary contract
  (ADR and design review) and conflict with ADR-0224. It is not recommended.

### Evidence

*Interface-checked* in installed miette 7.6.0 source:
- `src/miette_diagnostic.rs`: `MietteDiagnostic::{new, with_code, with_help, and_labels}`;
- `src/eyreish/error.rs`: `Report::with_source_code`;
- `src/handlers/narratable.rs`: `render_report`;
- `src/lib.rs`: `handler` and `highlighters` gated on `fancy-base`, while `miette_diagnostic`,
  `named_source` and `handlers` are ungated.

Placement is *Proposed*.

<a id="cbo-039"></a>

## CBO-039 — Hand-written identity framing beside the serde-derived framer

### Responsibility and target intent

Content identity for non-registry typed values: resolved records, schemes, units, selections,
envelopes and derived items.

Promised:
- exact bytes, held by golden vectors and published identities (DP-04);
- curated tag spellings that differ from Rust names: `"named"` for `Scheme::Concrete`,
  `"difference"` for `Delta`, `"declared-boundary"`;
- canonical forms: `render_expr` text, ordered collections, first-use ordinals;
- domain separation (`pse_ids::Frame::*V1`);
- per-domain leaf layouts. `pse-modeling/src/derived.rs` keeps its own `FrameValue` leaf trait,
  whose `u64` framing differs from `SemanticFrame for u64`.

**Correction to the codebase account.** There are 62 non-generated `fn frame(` sites across about
32 files.
- 12 are primitive and collection impls in `pse-model/src/lib.rs`.
- About 6 are generator emitters in `pse-codegen`.
- `derived.rs` already derives its relationship framing from one declaration through its local
  `relationships!` macro.

**The real exposure.** About 20 struct frames read `self.field` without destructuring. Examples:
- `pse-modeling/src/contextual.rs`, the impl near `render_expr(&self.temperature)`;
- `pse-quantity/src/resolved.rs`;
- `pse-model/src/applicability.rs`;
- `pse-operations/src/canonical_artifacts.rs`.

Adding a field to one of these structs compiles cleanly and silently leaves identity unchanged.

### Credible complete options

1. **Retain current.**
2. **The serde `Framer` (`pse-ids/src/document.rs`).** It frames type tags and field names, so
   identities would change. serde attributes are shared with the wire, and there is no
   identity-only namespace (`serde/selective-ordered-identity-framing`). Ruled out.
3. **A local `#[derive(SemanticFrame)]` proc-macro** with `#[frame(tag)]`, `#[frame(skip)]` and
   `#[frame(with = path)]`. It can reproduce today's bytes for full-field frames. No library
   supplies it, and it is a new crate.
4. **A reflection library such as `facet`.** Not researched. It carries the same byte-emulation
   and attribute-spelling burden on a young 0.x surface.
5. **Exhaustive destructuring in the struct frames.** Write `let Self { a, b, c: _ } = self;` and
   frame from the bindings. This changes no bytes and needs no dependency. A new field becomes a
   compile error at the frame site.

### Comparative argument

**An identity change must be a decision** (DP-04, DP-24). A derive's natural default adds a new
field to the frame. That silently changes bytes for existing values wherever no golden vector
covers the type, and it splits caches.

**Destructuring forces the decision without choosing it.** The author must write either the
binding or `field: _` beside the curated tags and canonical forms, which stay where the next
reader looks (DP-13 §F: the bespoke reason is stated in place). The derive's remaining benefit is
concision, and it costs a crate, an ADR and a design review. The serde route fails the frozen-byte
constraint outright.

### Revealing case

A new attribution-only field is added to an identity-bearing struct, for example retained
submitted spelling. ADR-0178 retains submitted spelling and units as attribution only, and the
public `ModelingResult.binding_identity` docstring repeats it ("canonical binding identity,
excluding submitted spelling and units").
- A default-including derive would fold that field into identity.
- Today's hand frame silently omits it. The result is correct by accident.
- Destructuring makes the author write `submitted_spelling: _`. The exclusion becomes explicit
  and reviewable.

### Disposition

**Recommend.**
- Retain hand `SemanticFrame` framing as the identity authority.
- Convert field-reading struct frames to exhaustive destructuring, including nested records
  whose fields are selected manually. `applicability::Node::frame` reads `self.claim.*`;
  destructuring only `Node` would not force an include/exclude decision for a new `Claim` field.
  Cover `Claim`, `Permission` and analogous nested records without changing current frame bytes.
- Enum frames are already exhaustive unless they use `..`. Check for `..` in struct-variant
  patterns at the same time.

Nothing new is added, no bytes move, and golden vectors are untouched.

### Rule impacts

- None: no byte or contract change.
- A derive would be a crate addition (ADR and design review).
- The serde route would change the hashing contract (ADR and design review, plus a golden
  migration).

### Evidence

- *Interface-checked*: the `SemanticFrame` trait and impls (`pse-model/src/lib.rs`) and the sample
  frames named above.
- The count comes from `rg 'fn frame\('` over `crates/`, excluding `generated/`, with a
  field-read heuristic.
- The heuristic can over-count `*self` primitive impls. Classify each site during the work.

<a id="cbo-040"></a>

## CBO-040 — Registry products baked as about 87k lines of Rust plus JSONL plus IPC

### Responsibility and target intent

`registry()` and `shared_registry()` hydrate the standard registry from declaration-derived
resolved products. The governing decision is ADR-0210 (proposed; §4.2/§4.6, Plan48 B04). It
requires:
- exact equivalence with independent source assembly;
- explicit failure on unknown variants or corrupt data;
- fresh runtime owners and caches;
- no imported proofs;
- byte-deterministic generation;
- "generated Rust literals retain declaration-owned static strings".

ADR-0210 records *Measured, fixture-bounded*: 304 ms versus 30 ms median construction plus drop.

**Premise correction.** These are not "the same products serialized three ways". Each sub-product
is encoded once, in the form its type supports:
- **Rust literals** construct the hand-written model types (`RelationSpec`, `InvariantSpec` and
  the rest).
- **`standard-data.jsonl`** is a deduplicated pool of canonical JSON for serde-capable leaves:
  Arrow `Field`, references and description values. It has 4,576 lines and 6,548 slice decodes.
  `pse_columnar::native_field::canonical_json` already sorts metadata, so `HashMap` ordering is
  already solved for this route.
- **Fifteen IPC files** carry the self-description `RecordBatch`es.

What remains:
- **(a) A per-type hand token emitter.** `pse-codegen/src/codegen/rust/registry.rs` has
  `relation`, `enumeration`, `invariant`, `migration`, `algorithm`, `document` and `obligations`
  emitters. Each must change with its model type.
- **(b) A self-referential build coupling.**
  - `pse-codegen` depends on `pse-schema` (`pub use pse_schema::*`).
  - `pse-schema/src/lib.rs` compiles `mod generated;` unconditionally.
  - A new model field therefore makes the committed literal fail to compile before the
    generator can be rebuilt to regenerate it.
  - This is static evidence. No bootstrap path was found in `pse-schema`, `xtask/src/codegen*`
    or the `justfile` codegen recipes. How developers get past it in practice is unknown.
- **(c) Compile cost.** The output is a 4.7 MB module of 87,267 lines with 2,399 outlined
  functions. Its own header says array-literal forms "exhausted the compilation memory cap". The
  current cost is unmeasured.

### Credible complete options

1. **Retain current** (ADR-0210 as built).
2. **A serde contract on the products plus one `postcard` 1.1.3 blob.**
   - Mechanics:
     - Serialize with `postcard::to_allocvec` at generation.
     - Embed the blob with `include_bytes!`.
     - Decode with `postcard::take_from_bytes(&'static [u8])` and reject a nonempty remainder:
       `from_bytes` at 1.1.3 ignores trailing bytes. `&'static str` borrows from the static
       blob; `Vec<&str>` and `Option<&str>` fields need `#[serde(borrow)]`.
   - Members postcard cannot carry directly:
     - `FieldContract` becomes a canonical-JSON text member, decoded through serde_json as
       today. The existing pool can stay as an indexed side table.
     - `descriptions` becomes canonical-JSON text, because postcard refuses `deserialize_any`.
   - IPC batches are unchanged.
   - The generator reduces to "serialize the products"; hydration becomes "decode, then the
     existing `ResolvedRegistryProducts::hydrate`".
3. **A self-describing blob** (serde_json or `ciborium` 0.2.2). It is robust to field reordering,
   but it cannot borrow `&'static str`: JSON escapes and CBOR's owned decode prevent it. The model
   would have to move to owned or `Cow` strings, or leak. That conflicts with ADR-0210's static
   strings and ripples through every consumer.
4. **A crate split.** Move standard hydration into a crate the generator does not depend on.
   - It removes (b) and keeps the format.
   - It is a crate addition (ADR and design review), and `registry()` call paths change.
   - A Cargo feature gate is not a substitute, because ADR-0122 unifies features workspace-wide.
5. **Ruled out by the library accounts.** rkyv needs an archived-type API or a full
   deserialization. bincode has ceased. "Arrow IPC for everything" fails because the nested spec
   trees are not tables.
6. **Rust-literal emission through a derive** (a `databake`-style `Bake` trait). This was not
   assessed: Context7 has no entry and no local source exists. It is recorded only as a lead,
   because a derive that emits array literals would likely bring back the compile-memory failure.

### Comparative argument

**What option 2 removes.**
- The per-type emitter (a), which serves DP-16: an ordinary model extension becomes "add the
  field".
- The self-coupling (b): a stale blob is a runtime or `codegen-check` failure, not a generator
  build failure.
- Probably most of (c), though that is unmeasured.

It also keeps what ADR-0210 actually needs: static strings, exact stored fingerprints and closure
identities carried as bytes, fresh owners, and IPC for the batches. Startup parsing already exists
today (6,548 JSON slice decodes), so decoding is not a new class of work. The relative startup
cost is unmeasured.

**What option 2 loses.** Compile-time staleness detection: a named Rust struct literal fails to
compile if a model type changes. Positional postcard instead depends on `codegen-check` and on
ADR-0210's source-versus-hydrated equivalence tests. A compatibility guard needs an expected
description derived from the current compiled types independently of the committed blob.
A digest or constant generated only alongside the blob can remain stale with it.

**Option 4 is narrower.** It fixes only (b), at the cost of a crate decision.

**Retaining stays reasonable until the costs are known.** If (b) has a routine workaround and (c)
is small, ADR-0210's form is the established, measured route. Moving a freshly proposed decision
on unmeasured benefit alone is weak.

### Revealing case

A developer swaps two same-typed fields in a model struct, such as `name` and `version` in
`AlgorithmSpec`, and does not regenerate.
- The Rust literal still constructs correctly, because its fields are named.
- A postcard blob decodes successfully into swapped meanings until `codegen-check` or the
  equivalence tests run.

Shape compatibility, payload integrity and independent source-versus-hydrated equivalence are
different guarantees. A layout digest alone cannot establish all three or detect validly encoded
value corruption. The exact preservation mechanism remains unresolved. ADR-0210's retained
corruption controls exercise corrupt/missing/extra IPC batches; do not silently generalize them
into an already established arbitrary-corruption guarantee.

### Probe outcome

A coordinator-run probe answered this section's question
([evidence](../../design_review/evidence/library-leverage-registry-build-coupling-2026-10-27/README.md); *Tested* and *Measured* at `9cb8e4f4a` on this host, one or two samples per
case).

- **The build coupling (b) is real and has no routine path.** Adding one field to
  `InvariantSpec`, setting it in the hand-written constructor and emitting it from the generator
  makes `just codegen` fail before regenerating anything: `xtask` cannot build because
  `pse-schema` fails with 258 × `E0063` in `generated/standard_registry.rs`. The only workaround
  found is to overwrite the generated module with a seven-line stub and rerun, which is a hand
  edit of a generated path. Five of the 22 regenerating commits of the preceding week also
  changed `pse-schema` model types, so the case is ordinary model extension, not a corner.
- **The compile cost (c) is material.** LLVM code generation of the 87k-line module is about 72%
  of a full `pse-schema` build (25.4 s against 7.3 s with the module stubbed; peak RSS 1.88 GB
  against 1.43 GB). It is repeated whenever regeneration shifts line numbers (about 21 s
  incremental, then about 98% of a downstream `pse-runtime` rebuild's wall time) and mostly
  repeated after hand edits to model sources (about 13 s extra). Type checking alone is cheap.

### Disposition

**Recommend replacing the literal module with an embedded structural product, provisionally
favouring postcard 1.1.3; the exact preservation mechanism remains unresolved.** The probe confirmed both conditions that the
first-pass disposition named as deciding for option 2, and retaining the current form would keep a
generation route that ordinary model extension breaks.

- **Enabling mechanisms.** postcard 1.1.3 with serde derives on `ResolvedRegistryProducts` and its
  members; strings borrowed from the static blob; `FieldContract` and description values carried
  as canonical-JSON text; IPC batches unchanged.
- **Compatibility and integrity.** Reject trailing input with `take_from_bytes`. Derive the
  expected layout from current compiled types independently of the committed payload; a
  codegen-emitted constant beside that payload is insufficient. `serde-reflection` is a research
  lead, not a demonstrated complete guard: enums, constrained deserializers and generic/container
  naming have tracing limitations. Independently choose the payload-integrity protection and
  retain source-versus-hydrated equivalence. No complete serialization/guard closure was tested.
- **Byte determinism.** ADR-0210 requires byte-deterministic generation. Some members carry
  hash-ordered data: `logical_storage: Vec<arrow_schema::DataType>` holds nested `Field` metadata
  in a `HashMap`, which serde writes in iteration order. Every such member must be carried in a
  canonical or sorted form (as `FieldContract` already is, as canonical-JSON text), and
  two fresh generation runs must produce identical bytes. `codegen-check` compares one candidate
  run with committed artifacts; that comparison alone is not a repeat-run test. If some member
  cannot be canonicalized, the recommendation becomes remedy-open for that member.
- **Machinery that disappears.** The per-type Rust-literal emitter and the 87k-line generated
  module, and with them the bootstrap coupling and the repeated LLVM work.
- **Machinery added.** The serde derives, the blob artifact and the guard.
- **Residual work.** Confirm the startup decode cost (unmeasured; startup already performs 6,548
  JSON slice decodes) and keep ADR-0210's source-versus-hydrated equivalence tests as the owner of
  staleness between regenerations.

None of U1, U2 or U3 bears on this choice.

### Rule impacts

- Option 2 amends proposed ADR-0210, specifically its "Rust literals" outcome sentence, through
  the decision route before acceptance (§4.2). The postcard dependency itself needs no ADR.
- Option 4 is a crate addition (ADR and design review).
- No option changes the hashing contract, because fingerprints are carried, not recomputed, and
  D1 is unaffected.

### Evidence

- *Interface-checked*: the generator and generated head and tail at HEAD; the `pse-schema`
  module inclusion and the `pse-codegen` dependency; ADR-0210.
- *Interface-checked by the postcard account*: postcard 1.1.3 borrowing and the `deserialize_any`
  refusal.
- *Tested* and *Measured*: the build coupling and the compile cost, in the
  [registry build-coupling probe](../../design_review/evidence/library-leverage-registry-build-coupling-2026-10-27/README.md).

<a id="cbo-041"></a>

## CBO-041 — Boundary documents transcoded through JSON, with checks in both languages

### Responsibility and target intent

Move typed boundary documents between their Rust owners and the generated msgspec types. The
promises come from ADR-0116 items 6 and 7, §21.5, ADR-0224 decode phases and ADR-0178:
- unknown-field refusal;
- non-finite refusal;
- a version header that is required and unique, and is checked before the body of *untrusted*
  documents;
- byte allowances;
- exact identity wire forms;
- JSON as the encoded document the native entry points accept.

**Observed paths at HEAD.**
- **Python→Rust** (`DocumentInput` and `RecordedInput` in `pse-py/src/documents.rs`):
  1. an `isinstance` check through per-call imports;
  2. a Python `pse.codec.encode_json` call per crossing;
  3. Rust `decode_document`, which applies the allowance and runs `check_required_header`;
  4. `Wire::decode_slice`, which runs `check_header::<N>` again for FlatV1, FlatV2 and ValueV1.
     That is a second identical pre-scan in Rust.
- **Rust→Python** (`DocumentValue::into_pyobject`):
  1. `Wire::to_vec`;
  2. per-call imports of the target class and `pse.codec`;
  3. `decode_json` → `_check_version_headers`, which partially decodes into `msgspec.Raw` slices
     and calls back into Rust `_native.check_document_header` for each versioned struct;
  4. the typed msgspec decode.
- **Import time.** `pse/__init__.py` runs `_check_native_compatibility`. It compares the package
  version, `registry_fingerprint` and `boundary_schema_fingerprint`. Catalogue agreement between
  native and Python is therefore established once per process.
- **Version fields.** Generated version fields are `Literal[N] = N`. msgspec decode already
  refuses a wrong value. The Python header walk adds only required-ness and duplicate refusal,
  which are properties of untrusted input.

**Premise correction.** The header rule has one implementation, Rust
`pse_model::document::check_required_header`. Python owns only the traversal of nested versioned
types and the wording of its messages. The real duplication is that an untrusted-input guarantee
is re-established on in-process producer output (H18, H19).

### Credible complete options

1. **Retain current.**
2. **A trust-aware boundary.**
   - Rust→Python uses a private trusted decode: a cached typed `msgspec.json.Decoder` per type
     with the same `dec_hook`, and no header traversal.
   - Python→Rust keeps its single authority and drops the duplicate pre-scan in `decode_document`
     for the three flat wires. The Native wire keeps its one pre-check.
   - Per-call imports become per-type handles cached in `pyo3::sync::PyOnceLock`. The repository
     already uses it in `enums.rs` and `identities.rs`.
   - The public `decode_json` and `decode_toml` for supplied bytes are unchanged.
3. **msgspec tagged structs as the header mechanism** (`msgspec/tagged-struct-header`). The probe
   shows weaker semantics for untrusted documents:
   - direct decode does not require the tag (A2/S3);
   - the first error in document order wins;
   - equal duplicate tags pass;
   - messages differ.
   It is ruled out as a replacement.
4. **`pythonize` 0.29.0 in-process object transfer.**
   - It removes the text hop.
   - It replaces the encoded-document contract (ADR-0116 item 7).
   - It turns byte allowances into object sizing.
   - It changes the non-finite mechanism: NaN crosses as a float, and `msgspec.convert` accepts
     `nan` and `inf` (probe N10–N13).
   - Its benefit is unmeasured.
5. **The `cattrs` msgspec preconf.** Not applicable here (see CBO-043).

### Comparative argument

**Option 2 matches enforcement to trust transitions.**
- Untrusted bytes keep every check.
- In-process Rust output relies on what is already established: the Rust serializer always writes
  the header, the import-time fingerprint establishes catalogue agreement, and msgspec still
  enforces structure, unknown fields and the `Literal` version value.
- No guarantee weakens. One partial decode, one callback per versioned struct, one Rust pre-scan
  and per-call module lookups disappear.

**Option 4 removes more work but changes a contract.** That is not warranted without
measurement. **Option 3 weakens a guarantee.**

**Implementation subtlety.** A `static` inside a generic function is shared by all its
monomorphizations. Per-type handles must therefore be emitted per document type, for example as
an associated function in the generated `pse-py/src/generated/documents.rs` catalogue. A single
generic static would be wrong.

### Revealing case

A developer rebuilds the Python package but not the extension. The import-time fingerprint check
refuses the import, so the trusted decode never sees a mismatched catalogue. This is the property
the Python header re-check would otherwise be guarding.

A Rust producer defect that omitted a header would be accepted by the trusted decode: msgspec
fills the `Literal` default. That is a producer correctness defect owned by Rust serialization
tests. It is not a trust transition, so the case holds.

### Disposition

**Recommend.**
- Add a private trusted Rust→Python decode.
- Remove the duplicate Rust pre-scan in `decode_document` for the flat wires.
- Generate per-type `PyOnceLock` handles for document classes and codec callables.
- Reuse `_json_decoder`'s existing bounded per-thread decoder cache and preparation path.
  Cached class/callable handles must not silently become shared process-wide decoder instances.
  Restrict trusted decode to immediate native-producer output; public untrusted decode retains
  required, unique, header-first admission.
- Leave the public codec and every untrusted path unchanged.

The repeated header traversal on Rust output and its Rust callback disappear. A small generated
handle per document type is added.

On U1: if msgspec accepted a "required tag on direct decode" option upstream, the Python traversal
for untrusted input could shrink further, though Rust would remain the authority. Contributing it
would be a separate choice.

### Rule impacts

- None. This is a refactor within ADR-0116 and §21.5, since no observable contract changes.
- If the maintainer reads §21.5 or ADR-0224 as requiring Python-side version admission on *every*
  decode, a short ADR clarifying "untrusted ingress only" would apply.
- Option 4 would change the Python boundary contract (ADR and design review).

### Evidence

- *Interface-checked*: `pse-py/src/documents.rs`, `pse-runtime/src/boundary_document_wire.rs`
  (`Wire::decode_slice`), `python/pse/codec/__init__.py`, `python/pse/_build.py`
  (`_check_native_compatibility`), the generated document `Literal` version fields, and
  `PyOnceLock::import` in installed `pyo3-0.29.2/src/sync/once_lock.rs`.
- *Tested*: msgspec semantics within the
  [msgspec probe](../../design_review/evidence/library-leverage-msgspec-tagged-decode-2026-10-27/README.md)
  conditions.

<a id="cbo-042"></a>

## CBO-042 — Document wire envelopes and directional schema surgery

### Responsibility and target intent

Versioned standalone encodings of 250 catalogue entries: 155 FlatV1, 91 Native, 3 FlatV2 and
1 ValueV1. Requirements:
- a separate header, which protects domain `version` fields;
- refusal of unknown body fields through a flattened body;
- published directional schemas that agree with the real deserializer;
- semantic `Output` names for directional differences: 483 distinct classes.

Python reads the header field from the generated `_pse_required_version_field` class variable. It
does not re-derive it.

### Credible complete options

1. **Retain current.** The pieces are:
   - `Versioned<T, N>` with the `BodyAccess` `MapAccess` adapter, which is serde's sanctioned
     `DeserializeSeed` extension point;
   - the hand `JsonSchema` that re-closes the object, injects the header into alternatives and
     fixes examples;
   - `directional_names` and `rename_output` in `pse-codegen/src/codegen/documents.rs`, about 90
     lines.
2. **serde attributes.** `flatten` with `deny_unknown_fields` is documented as unsupported, and
   integer internal tags remain an open request (serde #745). Ruled out.
3. **schemars 1.2.2 `#[schemars(transform = ...)]`** on a derived flatten. This moves the surgery
   into a transform function without deleting it.
4. **schemars contract modes.** Already in use. They do not deduplicate across contracts or name
   `Output` types, which is a TODO in schemars itself.

### Comparative argument

The local code is what neither library supplies:
- strict flatten;
- an integer header;
- cross-contract naming.

The remaining options relocate logic rather than remove it (H4). The four wires are envelope kind
crossed with version, not historical cruft. The cost of the 483 `Output` classes falls on the
size of generated output, not on maintained machinery.

### Revealing case

A body type that has its own `version` field. A serde-tag or msgspec-tag header would collide with
it, and the separate `document_version` header exists precisely for this.

### Disposition

**Retain current.** The one real redundancy, the duplicate header pre-scan, is handled under
CBO-041.

On U1: contributing the schemars opposite-contract dedupe TODO would at most share identical
definitions. Semantic `Output` naming would stay local, so it is not worth pursuing for this
benefit. Bumping schemars to 1.2.3 is neutral here (`sort_all_objects`), though it is still a
regeneration.

### Rule impacts

None.

### Evidence

*Interface-checked* at HEAD: the wire module, the rename pass, and the serde and schemars
accounts.

<a id="cbo-043"></a>

## CBO-043 — Two Python model systems and a custom schema-to-msgspec emitter

### Responsibility and target intent

§21.5 promises frozen attrs classes for each relation row (207 `*Row` classes), validated at
*construction*. Construction-time validation includes:
- `exact_type(bool)`;
- `finite_float`;
- `integer_range`;
- identity-type `instance_of`;
- `_pse_equality_key` (equal signed zeros; content hashes are not equality keys).

It also promises frozen msgspec documents, validated at *decode*, generated by the closed emitter.

**The construction requirement is live.** The public API exports two row families as
user-constructed values: `pse.modeling.Declaration` (`AuthoredModelingDeclarationsRow`) and
`FitDeclaration` (`AuthoredFitCasesRow`). `structure_rows` is public in `pse.codec`.

ADR-0116 recorded the second class system as an accepted cost. §21.5 records why the emitter was
chosen over datamodel-code-generator: the generator emitted `Any`, duplicate enums and unfrozen
structs.

### Credible complete options

1. **Retain current.**
2. **Rows as msgspec Structs.** msgspec `Struct.__init__` validates nothing (probe E3), so every
   construction validator would have to be re-emitted into `__post_init__`. Field-wise equality
   replaces the equality key.
3. **Documents as attrs plus cattrs.** This loses msgspec's typed decode, `forbid_unknown_fields`
   and tagged unions at the wire.
4. **datamodel-code-generator 0.86.0** (`datamodel-code-generator/jsonschema-to-msgspec-structs`).
   The generic mapping exists, but four things are needed and unestablished:
   - reuse of registry enums and identity value types;
   - `_pse_required_version` metadata;
   - `Output` naming;
   - byte determinism across tool versions.
   It would also make `pse-codegen`, which today is a pure path-to-bytes function in Rust (§4.2),
   invoke an external Python tool.
5. **`msgspec.convert(row, Doc, from_attributes=True)`** for the two edit encoders, replacing the
   cattrs unstructure. This is a narrow route.

### Comparative argument

The two systems serve different contracts:
- **constructed, validated values** (attrs with validators and an equality key);
- **wire DTOs validated on decode** (msgspec).

Collapsing either way moves or loses a guarantee, which the no-compromise rule forbids.

Option 4 trades a well-understood closed emitter for tool-version-coupled output. It also adds an
environment dependency to deterministic generation.

Option 5 is narrow and would touch a path ADR-0178's amendment just fixed: private row transport
"cannot bypass … required raw/typed version checks". Its finite-value check must stay anyway,
because `convert` accepts `nan`.

### Revealing case

A user writes `Declaration(..., is_override=1)`. The attrs row refuses it at construction. A
msgspec Struct would accept it, and the error would surface only later during conversion, away
from the user's line.

### Disposition

**Retain current.** The premise is partly rejected: the two systems encode two contracts, not one
meaning twice.

On U1: none needed.

### Rule impacts

None. Revisiting the emitter choice would amend the rationale of ADR-0116 item 7 as recorded in
§21.5. That is the Python boundary contract, so it needs an ADR and a design review.

### Evidence

- *Tested* within the msgspec probe: E3, N10–N13 and C1–C10.
- *Interface-checked*: the generated rows (`python/pse/contracts/authored.py`), `codec/_generated.py`
  and `modeling.py` exports.

<a id="cbo-044"></a>

## CBO-044 — Python wrapper layer forwarding native handles accessor by accessor

### Responsibility and target intent

The public Python API over native handles.

Promised:
- only `pse._build` imports `pse._native`, enforced by the ast-grep rule;
- public names that are separate from native implementation names;
- typed stubs;
- real Python-side behaviour where it exists.

Counts at HEAD:
- 405 `self._handle.` forwards in 13 modules;
- 41 wrapper classes holding `_handle`, of which 6 are Python-constructed (`object.__setattr__` in
  `__init__`, for example `Runtime`);
- 22 sites that unwrap another wrapper's `_handle` to pass it to native code, marked
  `# noqa: SLF001`.

Each accessor exists as a Rust `#[pymethods]` entry, a generated `_native.pyi` signature, and a
hand Python forward with its own docstring.

### Credible complete options

1. **Retain current.**
2. **Make pyclasses the public types.** PyO3 0.29.2, `pyo3/pyclass-as-public-type`:
   - `#[pyclass(frozen, name = "ModelingResult", module = "pse.modeling")]`;
   - re-export through `_build` into the public modules, so the gate is preserved;
   - Rust doc comments become `__doc__`;
   - native methods return public pyclasses directly and accept typed document Structs through
     the existing `DocumentInput`, so the Python `encode_json` disappears;
   - per-index loops become one tuple return (`pyo3/bulk-return-crossing`);
   - Python subclassing of `#[pyclass(subclass)]` native owners is a candidate for retaining
     host-operation mixins. Constructors/MRO, immutability and returned base instances need an
     explicit design before claiming removal of `_host_*` forwarding;
   - genuinely Python behaviour either moves to `#[pymethods]` or stays a small module function.
     An example is `TableStream.extension_report`, which uses pyarrow's `compare_schemas`.
3. **Generated Python forwards** from introspection. This adds another generator hop (LOP-X003).
4. **`__getattr__` delegation.** It hides the surface from type checkers and is rejected under
   the typing rules.

### Comparative argument

For forwarding-only families with complete native realizations, option 2 can make Rust the
accessor authority and remove forwards, duplicated docstrings, unwraps and per-item crossings.
Public naming and gate re-export have interface routes. Indexed views, equality/hash, mutation
exceptions and Python helpers still need the public-family inventory and ADR design; pyclass
`frozen` alone does not establish attrs equivalence or a blanket deletion count.

**Conditions.**
- **Stub docstrings.** The stub contract deliberately strips docstrings
  (`xtask/src/codegen/python_stubs/validate.rs`). It must keep them, or IDE hover loses the
  documentation now held in the `.py` wrappers. `pyo3-introspection` already emits docstrings.
- **Rust-returned objects.** These are always base-class instances. A public type that needs
  Python-only methods therefore cannot be a Python subclass. It needs a Rust implementation or a
  free function.

Option 3 removes hand work but keeps the layer, and adds a generator to maintain.

### Revealing case

`TableStream` is Rust-returned and has the Python method `extension_report(schema)`.
- `__iter__`, `__next__` and the context-manager methods move into `#[pymethods]`.
- `extension_report` becomes either a pyo3-arrow schema comparison in Rust or a module function
  such as `pse.inspection.extension_report(stream, schema)`.

The latter is a public API change that the ADR must list. Full preservation, including indexed
views and helper behavior, remains an inventory and qualification obligation.

### Disposition

**Recommend moving forwarding-only surfaces to native classes where their complete public
contract has a coherent native realization.** Inventory and resolve each public family under an
ADR and design review; complete preservation is not established by class attributes alone.

Target:
- public pyclasses with `name` and `module`;
- re-export through `_build`;
- the stub contract keeps docstrings;
- Python-constructed owners have an explicit construction/MRO and immutability design;
- bulk returns;
- typed documents accepted natively;
- the residual Python-only helpers listed explicitly.

The route can remove much of the forwarding layer, but its final deletion count and residual
wrapper/view set are unestablished. `CompactStudyResult` holds a native study report plus an
index, while `StudyReport` wraps that same native type alone. Renaming one pyclass cannot replace
both: introduce an explicit native indexed view or retain a small Python view.

**Semantics the ADR must list.** Preserve or deliberately revise constructors, properties versus
methods, iteration/context management, repr, mutation exceptions and Python helpers. attrs
currently compares and hashes wrapped Python handle objects, plus other wrapper fields such as
the index. Native report/result types generally derive `Clone, Debug`, not `PartialEq, Hash`;
adding Rust value equality can change semantics. PyO3 `frozen` does not reproduce attrs'
`FrozenInstanceError`, and `subclass` alone supplies no construction/immutability design.
The public-family and stub inventory must cover every module; sampled forwarding counts do not
establish complete preservation.

Coordinate with G2/CBO-017, whose waits sit in the same native methods, and with G9/CBO-052, the
command surfaces.

On U1: if pyo3 0.30's stub changes (`__all__`, submodule re-export) are needed for clean
public-module stubs, it is a version move. 0.30 is a pinned-family major, so it needs an ADR.

### Rule impacts

- **Python boundary contract change: ADR and design review.** Public type ownership moves from
  `python/pse` into the extension, and the §21 text "python/pse is the typed boundary around it"
  needs amendment.
- The stub-contract change is tooling and needs no ADR by itself.

### Evidence

- *Interface-checked*: the pyo3 account (pyclass parameters at 0.29.3 docs; introspection 0.29.2
  `stubs.rs`), and wrapper samples in `_modeling.py`, `_inspection.py`, `_workflow.py` and
  `_project_operations_glue.py`.
- A stub-quality check on generated introspection stubs (not run in this pass) would settle the
  stub question.

<a id="cbo-045"></a>

## CBO-045 — Diagnostics implemented through a forwarding macro beside the miette derive

### Responsibility and target intent

One typed `DiagnosticCode` per failure (§23.2: "nothing else declares a code"). Typed facts and
children, and forwarding through wrappers. miette's `code()` renders the typed code in path form;
`DiagnosticCode`'s `Display` produces `runtime::infrastructure`.

§23.2 permits either a derive or the shared `impl_diagnostic!` projection, and the governance test
`tests/governance/tests/error_taxonomy.rs` checks one or the other.

**Premise inverted.** `impl_diagnostic!` is the single-authority mechanism: miette's `code()` is
projected from `TypedDiagnostic::diagnostic_code`. The drift is at the derive sites. Ten
non-generated files derive `miette::Diagnostic` with a static `code(path)`.

- **A live drift.** `pse-operations/src/canonical.rs` declares the container code
  `runtime::infrastructure`. Its separate `impl TypedDiagnostic` maps `Codec(_)` to
  `ValidationInvariant`.
  - `pse-py/src/inspection/errors.rs` implements `ReportSource for CanonicalError`, so
    `diagnostic()` raises `InspectionError::new_err(message(error))`.
  - `message()` formats miette's `code()` as `[{code}] {error}`.
  - `report.code` uses the typed code (`project_typed` → `from_boundary`).
  - The same failure therefore shows `[runtime::infrastructure]` in its text and
    `validation.invariant` in its report.
- **Codes outside the vocabulary.** Five derive sites have no typed impl. One carries a code that
  is not in the vocabulary: `Withheld` with `pse::native::square_response`.

`disposition()` (`pse-model/src/diagnostic.rs`) is a distinct, exhaustive `const fn` policy
projection (§23.2: class, disposition and retry are separate projections). It is not duplication.

The two `vocabulary!` macros:
- `pse-vocabulary` already depends on `pse-diagnostics`, so one exported macro could serve both.
- They differ in five respects: `description`, the `JsonSchema` impl, the String conversions, the
  type of `ALL`, and the serde route.

### Credible complete options

1. **Retain current.**
2. **Make the macro the only projection** and extend it.
   - Migrate the miette-derive sites that carry codes onto `impl_diagnostic!` (or onto a
     typed-code-emitting form).
   - Make the `forward`, `help`, `related`, `source` and `facts` clauses optional
     (`$( , help(...) )?`), which removes the `{ None }` boilerplate.
   - Optionally add `labels` and `severity` clauses.
3. **miette derive plus a separate `TypedDiagnostic` impl everywhere.** This is the account's
   framing. miette-derive 7.6.0's `code` is a static `LitStr` or `Path` and cannot be an
   expression, so the code is declared twice. This is the current drift source.
4. **A local `#[derive(TypedDiagnostic)]` proc-macro** emitting both traits from per-variant
   `#[typed(code = ...)]`. It gives colocation and format-string help. It is a crate addition.
5. **Vocabulary consolidation.** One exported `vocabulary!` in `pse-diagnostics` with optional
   description and schema arms (strum 0.28.0 is already the mechanism). This is local and low
   priority.

### Comparative argument

Option 2 removes the only route through which a second code spelling can exist, and it uses an
existing facility.

Option 4 adds per-variant colocation. It costs a crate, an ADR and a design review. Its benefit
over the macro is ergonomic, not semantic.

Option 3 is the problem itself.

### Revealing case

A wrapper that forwards everything to a child but overrides `help`. The macro already expresses
this: `forward(...)` plus `help(...)`, with the code taken from the child when the wrapper's own
code is `None`. A miette-derive `forward(field)` cannot carry the typed code. The macro is the
more expressive of the two here, not the less.

### Disposition

**Recommend.**
- Extend `impl_diagnostic!` with optional clauses.
- Migrate the 10 derive sites. Give the 5 sites without a typed impl typed codes, or record that
  they are not public `*Error` types. `Withheld` is not named `*Error`.
- Fix the `CanonicalError` divergence as part of that migration.
- Optionally consolidate `vocabulary!`.

The static miette codes disappear, along with the separate `TypedDiagnostic` impls at the derive
sites. The macro gains optional arms.

On U1: if miette upstream accepted an expression in `code(...)`, the derive sites could project
the typed code. That is low value given option 2.

### Rule impacts

- None. This is a refactor within §23.2, which permits the macro route.
- A proc-macro crate (option 4) would be a crate addition: ADR and design review.

### Evidence

- *Interface-checked*: `pse-diagnostics/src/lib.rs` (macro and `TypedDiagnostic`), the 10 derive
  files, `pse-py/src/inspection/errors.rs` (`record`, `from_boundary`, `message`),
  `pse-diagnostics/src/vocabulary.rs` (`Display for DiagnosticCode`) and §23.2.
- miette-derive `code` is limited to a `LitStr` or `Path` per the miette account (7.6.0 source).

<a id="cbo-046"></a>

## CBO-046 — SurrealQL and Python emitted through string templates (templating side)

### Responsibility and target intent

Byte-deterministic Python contract modules. The textual risks at HEAD:
- **`pse-codegen/src/codegen/python/mod.rs`:**
  - `local.replace(&format!("s.{name}"), name)` rewrites already-generated declarations. It also
    matches any `…s.Name` substring, such as an attribute access on an identifier ending in `s`.
  - `values()` applies `.replace("v.", "")` over generated declarations. It matches any
    identifier ending in `v` followed by `.`.
  - `values.py.template` substitutes `@SEMANTIC_WIDTH@` and `@CONTENT_WIDTH@`.
- **`codegen/documents.rs` `python_module`:** import lines are pruned after generation by string
  surgery on `from typing import …`.

Today, corruption by these rewrites would surface only through downstream type checks or imports.
`codegen-check` compares a candidate run with committed artifacts and checks candidate contracts;
it independently proves neither repeat-run determinism nor semantic correctness.

### Credible complete options

1. **Retain current.**
2. **Local generator parameterization.**
   - Pass the emitting module's qualifier context into `types::storage` and the structure
     emitters, so they write `SemanticId` or `v.SemanticId` and `Name` or `s.Name` directly.
   - Have the document emitter record the typing names it actually uses and write the import
     line once.
   - After `@X@` substitution, assert that no `@[A-Z_]+@` marker remains.
3. **minijinja 3.0.x or askama 0.16.1.** Either handles placeholder substitution only. Neither
   addresses the rewrites, which are code rewriting, not placeholders.
4. **genco 0.20.0.** Language-aware quasi-quoting with import tracking. It means rewriting about
   117 Python emitter sites, and its import-order determinism is unverified.

### Comparative argument

Option 2 removes the risky operation, textual rewriting of emitted code, at its source. It needs
no dependency. A successful `codegen-check` supports unchanged generated bytes for that run;
relevant generator controls must still exercise qualifier/import behavior.

Options 3 and 4 either miss the rewrites or demand a large lateral rewrite. That would only pay
off if the whole Python emitter were being restructured.

### Revealing case

A future structure or relation field whose generated annotation contains `….v.` or `…s.Name`
inside another identifier. Today's rewrite silently alters it. Option 2 cannot produce that
corruption, because it never edits emitted text.

### Disposition

**Recommend** option 2.

Disappear:
- the two `.replace` rewrites;
- the import-line surgery.

Added:
- a qualifier context parameter;
- a used-names set;
- a marker assertion.

The SurrealQL `__X__` placeholders and the guard and relation-id splices belong to G3. If G3
adopts a template engine for SurrealQL, the two Python templates could move onto it for
consistency, but they need nothing beyond the marker assertion.

### Rule impacts

None.

### Evidence

- *Interface-checked*: `codegen/python/mod.rs` (the structure-local rewrite and `values()`) and
  `codegen/documents.rs` (`python_module`).
- Template-engine capabilities are *Interface-checked* per their accounts (documentation only).

## Pattern notes

- **[LOP-X009](../assessment-patterns.md#lop-x009) refined (revision 3).** CBO-041 is the converse of CBO-005: the
  untrusted-input guard is re-run on bytes the trusted producer has just emitted in the same
  process, after an import-time fingerprint has already established agreement. The trust-boundary
  refinement covers both directions.
- **[LOP-X003](../assessment-patterns.md#lop-x003) refined (revision 3):** bootstrap coupling, where a generator's
  crate compiles the generator's own output and that output constructs hand-written types (CBO-040,
  now *Tested*); sub-products split across formats by type capability; runtime decoding versus a
  crate split under workspace feature unification; the independently derived compatibility guard as a candidate replacement for
  named-literal staleness detection.
- **[LOP-X007](../assessment-patterns.md#lop-x007) refined (revision 3):** the local projection can be the authority
  and the library-derive sites the drift (CBO-045); exhaustive destructuring is the zero-dependency
  way to force a per-field decision (CBO-039).

## Limits

- **Not read:**
  - the full 3.7k-line declaration parser;
  - the MCP or capability server's exception handling, which matters for CBO-038 privacy;
  - whether the retained native `ModelingPackage` keeps a map from document id to (path, text)
    after admission;
  - the host-operation generator;
  - the 12 wrapper modules beyond the samples (`_modeling`, `_inspection`, `_workflow`,
    `_project_operations_glue`, plus class-header scans of `_runs`, `diagnostics` and
    `_study_workbench`).
- **Heuristic, unclassified:** the roughly 20 CBO-039 struct frames. It may over-count
  primitive impls.
- **Not researched:** databake and facet.
- **Relied on library accounts:** postcard borrowing, the pyo3 0.29.3 pyclass parameters and
  chumsky.
- **Not done:** no dependency resolution was attempted, and the reviewer ran no builds or probes; the CBO-040 coupling was later *Tested* by the coordinator-run probe.
- **Search scopes for negative claims:**
  - No excerpt renderer: account search, plus `python/pse/diagnostics.py`.
  - No bootstrap path: `pse-schema/src/{lib.rs,generated/mod.rs}`, `xtask/src/codegen*`, the
    `justfile` codegen recipes.
  - No proc-macro crate: `crates/*/Cargo.toml`, `xtask/Cargo.toml`.
