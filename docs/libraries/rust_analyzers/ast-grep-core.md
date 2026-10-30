# ast-grep-core 0.51.0 — Rust Library Capability Reference

Research date: October 30, 2026 · Target: ast-grep-core 0.51.0, released October 15, 2026 · License: MIT

## 1. Overview and architectural role

`ast-grep-core` is a programmable structural code-matching and rewriting engine built around concrete syntax trees produced by Tree-sitter. It lets Rust applications parse source code, match code-shaped patterns against syntax trees, capture structural variables, navigate syntactic relationships, and construct or apply source edits.

Its defining feature is that patterns look like ordinary code rather than AST constructors or textual regular expressions. A pattern such as:

```
$OBJ.$METHOD($$$ARGS)
```

can describe an entire family of method calls, capture their component syntax nodes, and support transformations that preserve the matched source text.

Unlike a conventional AST query engine, `ast-grep-core` also provides a source-oriented rewrite model: matched nodes retain source ranges and text, while replacement templates can reuse captured fragments and their original formatting. The result is a useful middle ground between text substitution and fully semantic compiler-based transformations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### Capability map

| Capability family            | Principal Rust interfaces                           | Architectural significance                                                          |
| ---------------------------- | --------------------------------------------------- | ----------------------------------------------------------------------------------- |
| Parsing and source ownership | `AstGrep<D>`, `StrDoc<L>`, `LanguageExt`            | Parse source into a navigable tree while retaining the original text                |
| Structural matching          | `Pattern`, `MatchStrictness`, `Matcher`             | Match syntax shapes without constructing AST visitors for each search               |
| Tree inspection              | `Node`, `Position`, tree navigation methods         | Inspect node kinds, fields, hierarchy, siblings, ranges, and parse errors           |
| Capture and composition      | `NodeMatch`, `MetaVarEnv`, `Op`, `And`, `Or`, `Not` | Combine structural conditions and extract matched syntactic components              |
| Rewriting                    | `Replacer`, `TemplateFix`, `Edit`, `Root::edit`     | Generate source changes, preserve captured text, and reparse edited documents       |
| Extensibility and execution  | `Language`, `Doc`, `SgNode`, custom `Matcher`       | Integrate grammars, custom predicates, document representations, and host workflows |

The essential pipeline is:

```
Source text + language grammar
             ↓
     Tree-sitter parser
             ↓
    AstGrep / syntax tree
             ↓
   Compiled Matcher / Pattern
             ↓
   NodeMatch + MetaVarEnv
             ↓
  Diagnostics / extraction / Edit
             ↓
  Updated source and reparsed tree
```

`ast-grep-core` is especially relevant to structural code search, syntax-aware indexing, lightweight static analysis, custom lint engines, codemods, migration tools, source transformation services, and programmable code-intelligence systems.

Its central boundary is syntactic, not semantic. It recognizes language grammar structures and relationships in parsed source code. It does not, by itself, resolve names across modules, infer types, construct a project-wide symbol graph, or perform compiler-equivalent refactoring validation. Those capabilities require additional analysis layers.

Primary references:

- Core Rust API: [https://docs.rs/ast-grep-core/latest/ast_grep_core/](https://docs.rs/ast-grep-core/latest/ast_grep_core/?utm_source=chatgpt.com)
- Upstream repository: [https://github.com/ast-grep/ast-grep](https://github.com/ast-grep/ast-grep?utm_source=chatgpt.com)
- Pattern matching internals: [https://ast-grep.github.io/advanced/match-algorithm](https://ast-grep.github.io/advanced/match-algorithm?utm_source=chatgpt.com)
- Pattern parsing internals: [https://ast-grep.github.io/advanced/pattern-parse](https://ast-grep.github.io/advanced/pattern-parse?utm_source=chatgpt.com)

## 2. Scope and package boundaries

The distinction between `ast-grep-core` and the broader ast-grep project is important. The project exposes capabilities across multiple Rust crates, CLI tools, and language bindings, but they do not all belong to the core crate.

| Component                                      | Primary responsibility                                                                                                               |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `ast-grep-core`                                | Syntax-tree abstraction, structural patterns, matching, metavariables, traversal, and source edits                                   |
| `ast-grep-language`                            | Packaged Tree-sitter grammars, supported-language enumeration, file-type mapping, language-specific pattern preprocessing            |
| `ast-grep-config`                              | Declarative rule configuration, richer relational and composite rules, constraints, utility rules, and transformation configuration  |
| `ast-grep-dynamic`                             | Dynamic integration of external Tree-sitter grammars                                                                                 |
| `ast-grep-outline`                             | Higher-level structural outline extraction                                                                                           |
| `ast-grep-lsp`                                 | Language-server functionality                                                                                                        |
| `ast-grep` CLI                                 | File discovery, parallel project scanning, YAML-based lint workflows, interactive rewrites, diagnostics, and command-line interfaces |
| `@ast-grep/napi`, `ast-grep-py`, WASM bindings | Language-specific API surfaces built using the underlying matching machinery                                                         |

These components can be combined, but their APIs and responsibilities should not be treated as interchangeable. In particular, the full YAML rule language is not the same as the low-level `Matcher` interface, and the CLI's recursive file scanning is not an intrinsic operation of `AstGrep`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+3



### Version and build characteristics

For release 0.51.0:

- The workspace specifies Rust 1.90.0 as its minimum supported Rust version and uses the Rust 2024 edition.
- The core crate's default `tree-sitter` feature enables its Tree-sitter integration.
- The workspace uses Tree-sitter 0.27.
- The core crate does not bundle the general catalog of language grammars; those are supplied by companion crates or custom language implementations.
- The `tree-sitter` feature is optional, allowing the matching abstractions to be separated from their default parser adapter.

The released package is still in the `0.x` series; APIs should be pinned and checked for compatibility rather than assumed to have a stable 1.0 contract.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+2



Primary references:

- Core crate manifest: [https://docs.rs/crate/ast-grep-core/0.51.0/source/Cargo.toml.orig](https://docs.rs/crate/ast-grep-core/0.51.0/source/Cargo.toml.orig?utm_source=chatgpt.com)
- Release-specific workspace manifest: [https://github.com/ast-grep/ast-grep/blob/0.51.0/Cargo.toml](https://github.com/ast-grep/ast-grep/blob/0.51.0/Cargo.toml?utm_source=chatgpt.com)
- Companion language crate: [https://docs.rs/ast-grep-language/latest/ast_grep_language/](https://docs.rs/ast-grep-language/latest/ast_grep_language/?utm_source=chatgpt.com)

## 3. Parsing, document representation, and syntax-tree access

### 3.1 `AstGrep`, `Root`, and `StrDoc`

The central parsed-document abstraction is `AstGrep<D>`, an alias for `Root<D>`, where `D` implements the `Doc` trait.

For ordinary Tree-sitter-backed source code, the concrete document type is `StrDoc<L>`, parameterized by a language implementing `LanguageExt`.

Conceptually:

```
AstGrep<StrDoc<L>>
    ├── source text
    ├── language definition
    └── Tree-sitter tree
             └── Node<'r, StrDoc<L>>
```

The principal operations are:

| Interface                           | Behavior                                                    |
| ----------------------------------- | ----------------------------------------------------------- |
| `LanguageExt::ast_grep(source)`     | Construct a parsed document using a language implementation |
| `AstGrep::new(source, language)`    | Construct a Tree-sitter-backed root                         |
| `Root::try_new(source, language)`   | Fallible construction                                       |
| `root.root()`                       | Obtain the root syntax node                                 |
| `root.lang()`                       | Access the language definition                              |
| `root.source()` / `root.get_text()` | Access original or updated source text                      |
| `root.edit(edit)`                   | Apply a source edit and update the parse tree               |
| `root.generate()`                   | Consume the root and return its source string               |

`Root` owns the document. Its `Node` objects are lightweight views associated with that root, with Rust lifetimes preventing ordinary references to nodes from outliving the underlying document.

Parsing does not imply syntactic validity. Tree-sitter can produce a tree containing recovered errors or missing nodes. A successful `Root::try_new` establishes that parsing produced a tree, not that a compiler would accept the source.

The concrete Tree-sitter adapter exposes this distinction through `Node::is_error()` and `Node::is_missing()`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



Source:

[https://docs.rs/ast-grep-core/latest/src/ast_grep_core/tree_sitter/mod.rs.html](https://docs.rs/ast-grep-core/latest/src/ast_grep_core/tree_sitter/mod.rs.html?utm_source=chatgpt.com)

### 3.2 Navigating syntax trees

`Node<'r, D>` exposes both structural and source-oriented information.

| Family                          | Important methods                                                                |
| ------------------------------- | -------------------------------------------------------------------------------- |
| Identity and classification     | `kind()`, `kind_id()`, `node_id()`, `is_named()`, `is_leaf()`, `is_named_leaf()` |
| Recovery and grammar properties | `is_error()`, `is_missing()`, `is_extra()`                                       |
| Text and location               | `text()`, `range()`, `start_pos()`, `end_pos()`                                  |
| Hierarchy                       | `parent()`, `ancestors()`, `children()`, `named_children()`, `child()`           |
| Grammar fields                  | `field()`, `field_children()`, `child_by_field_id()`                             |
| Siblings                        | `prev()`, `next()`, `prev_all()`, `next_all()`                                   |
| Traversal                       | `dfs()`, `find()`, `find_all()`                                                  |
| Interoperability                | `get_inner_node()`, `get_doc()`, `root()`                                        |

The distinction between named and unnamed nodes matters. Tree-sitter typically represents constructs such as expressions and declarations with named nodes, while punctuation and fixed keywords may appear as anonymous nodes.

Consequently, `named_children()` is not interchangeable with `children()`. A transformation that needs commas, delimiters, keywords, or operators may require the complete child sequence.

`is_named_leaf()` also has a subtle meaning: it identifies a node with no named children, not necessarily a named node with no children at all.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



#### Grammar fields

A particularly useful capability is access to grammar-defined fields rather than relying on positional child indices.

For example, a language grammar may expose a function's `name`, `parameters`, or `body` as fields.

This allows a syntax-aware operation to distinguish structurally meaningful children even if punctuation or optional syntax changes their positions.

However, field names and node kinds come from the particular Tree-sitter grammar. They are not a language-independent semantic schema.

### 3.3 Source ranges, characters, and bytes

For the standard `String` document representation, `Node::range()` returns UTF-8 byte offsets.

`Position` also distinguishes:

- `line()` — zero-based line.
- `byte_point()` — zero-based line and byte column.
- `column(&node)` — character-based column computed from the source.

The character-column conversion is documented as a linear operation in the relevant source span. Repeated conversion of many positions can therefore add work to diagnostic or indexing pipelines.

Most importantly, offsets must not be silently transferred between coordinate systems. UTF-8 byte offsets, Unicode character positions, and UTF-16 editor positions are different representations.

The core `Content` abstraction exists partly to accommodate source encodings across Rust and the project's bindings; it does not eliminate the need for deliberate coordinate conversion at integration boundaries.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



Source:

[https://docs.rs/ast-grep-core/latest/src/ast_grep_core/node.rs.html](https://docs.rs/ast-grep-core/latest/src/ast_grep_core/node.rs.html?utm_source=chatgpt.com)

## 4. Structural pattern matching

### 4.1 Patterns are compiled syntax trees

`Pattern` represents a syntax-tree pattern constructed from source-like code.

Its principal constructors are:

```
Pattern::try_new(source, language)
Pattern::new(source, language)
Pattern::contextual(context, selector, language)
```

`try_new` returns a `Result<Pattern, PatternError>`. The convenience `new` constructor unwraps that result and can panic on pattern-construction failure.

A compiled `Pattern` implements `Matcher`; it can therefore be passed directly into traversal methods and composed with other matchers.

The important difference from a regular expression is the unit of matching. A pattern matches syntax-tree nodes and their arrangement, not arbitrary character spans.

For example:

```
console.log($ARG)
```

matches the structural form of a call, independently of source layout such as:

```
console
  .log(value)
```

It does not match the same characters inside a string or comment merely because the text looks similar.

Compiled patterns also expose `defined_vars()`, `fixed_string()`, `has_error()`, `dump()`, and `with_strictness()`. These support inspection, rule validation, selective prefiltering, and controlled matching behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source:

[https://docs.rs/ast-grep-core/latest/ast_grep_core/matcher/struct.Pattern.html](https://docs.rs/ast-grep-core/latest/ast_grep_core/matcher/struct.Pattern.html?utm_source=chatgpt.com)

### 4.2 Metavariables and structural capture

The pattern language supports several forms of metavariable.

| Form             | Meaning                                                    |
| ---------------- | ---------------------------------------------------------- |
| `$NAME`          | Match and capture one named syntax node                    |
| `$$NAME`         | Match and capture an unnamed syntax node where appropriate |
| `$$$NAME`        | Capture a sequence of zero or more syntax nodes            |
| `$$$`            | Match a sequence without retaining a named capture         |
| `$_`             | Match a node without retaining its binding                 |
| `$NAME` repeated | Require occurrences to match structurally consistently     |

For instance:

```
$OBJECT.$METHOD($$$ARGS)
```

captures the object, method, and argument sequence separately.

A sequence metavariable is more expressive than a single-node wildcard. It can match variable-length argument lists, parameter lists, or statement sequences.

One important detail is that multiple metavariables are matched lazily: they stop at the first subsequent pattern node that satisfies the following portion of the pattern. This is not an unrestricted backtracking regular-expression engine.

The same metavariable name appearing more than once introduces a structural equality requirement:

```
$X == $X
```

This can recognize syntactically equivalent operands, but it does not establish that the operands refer to the same resolved symbol. Conversely, two different spellings can denote the same semantic symbol without satisfying this pattern.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://ast-grep.github.io\&sz=32)

ast-grep+2



A frequently overlooked limitation is that metavariables must correspond to syntax nodes that the parser recognizes as the metavariable. They are not arbitrary substrings.

Thus `obj.$METHOD` is meaningful, while something resembling `obj.on$EVENT` generally does not capture the suffix of an identifier. Similarly, a metavariable written inside a string literal is ordinarily string content rather than an AST wildcard.

Source:

[https://ast-grep.github.io/guide/pattern-syntax](https://ast-grep.github.io/guide/pattern-syntax?utm_source=chatgpt.com)

[https://ast-grep.github.io/advanced/pattern-parse](https://ast-grep.github.io/advanced/pattern-parse?utm_source=chatgpt.com)

### 4.3 Contextual patterns

`Pattern::contextual(context, selector, language)` addresses cases in which a desired syntax fragment does not parse correctly or unambiguously in isolation.

Consider a JavaScript object property:

```
{ "enabled": true }
```

The fragment `"enabled": true` may be interpreted differently outside an object expression. A contextual pattern supplies the complete enclosing snippet, then selects the specific grammar node kind that should become the actual matcher.

This is important for transformations involving:

- Expression fragments that require statement or declaration context.
- Object members, class members, and method definitions.
- Language constructs whose grammar depends on an enclosing structure.
- Ambiguous fragments that Tree-sitter otherwise interprets using a different production.

It is not equivalent to requiring the surrounding context to exist in the target. The context primarily makes the pattern itself parse into the intended syntax-tree shape; the selected node defines the matcher.

The language-specific `pre_process_pattern()` hook can also alter pattern interpretation before compilation.

### 4.4 Matching strictness

`Pattern::with_strictness()` changes which syntax-tree differences matter.

| `MatchStrictness` | Matching behavior                                                | Consequence                                                           |
| ----------------- | ---------------------------------------------------------------- | --------------------------------------------------------------------- |
| `Cst`             | Match named and unnamed nodes on both sides                      | Strong structural and punctuation sensitivity                         |
| `Smart`           | Require all pattern nodes; ignore extra unnamed target nodes     | Default; less syntax in the pattern usually broadens matches          |
| `Ast`             | Compare named nodes, ignoring unnamed nodes on both sides        | Less sensitive to punctuation and delimiter details                   |
| `Relaxed`         | Similar to AST matching, also ignoring comments                  | Useful where comments are irrelevant to the target structure          |
| `Signature`       | Compare named-node structure and kinds while ignoring leaf text  | Broader structural-shape matching                                     |
| `Template`        | Specialized text-oriented matching with node-kind checks relaxed | Useful for template-like comparison, but requires separate evaluation |

The official matching-algorithm guide focuses on the first five levels. The Rust enum additionally includes `Template`, which is also exposed by the CLI's strictness documentation.

A significant example of default `Smart` behavior:

```
function $NAME() {}
```

can match:

```
async function work() {}
```

because the extra `async` keyword is an unnamed node in the relevant grammar. If `async` appears explicitly in the pattern, it becomes required.

This means the default is not exact concrete-syntax equivalence. More relaxed levels may be valuable for broad detection, but they increase the risk of matching structures that should not be transformed identically.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://ast-grep.github.io\&sz=32)

ast-grep+2



Sources:

[https://ast-grep.github.io/advanced/match-algorithm](https://ast-grep.github.io/advanced/match-algorithm?utm_source=chatgpt.com)

[https://docs.rs/ast-grep-core/latest/ast_grep_core/enum.MatchStrictness.html](https://docs.rs/ast-grep-core/latest/ast_grep_core/enum.MatchStrictness.html?utm_source=chatgpt.com)

## 5. The matcher abstraction and rule composition

### 5.1 `Matcher` is the extensibility interface

The core `Matcher` trait determines whether a candidate node satisfies a condition while updating its metavariable environment.

Its principal method has the conceptual signature:

```
fn match_node_with_env<'tree, D: Doc>(
    &self,
    node: Node<'tree, D>,
    env: &mut Cow<'_, MetaVarEnv<'tree, D>>,
) -> Option<Node<'tree, D>>;
```

The returned node may be different from the candidate, which matters for matchers that inspect a descendant or other related node.

The trait also exposes two valuable optimization and editing hooks:

- `potential_kinds()` — a possible set of node-kind IDs against which the matcher can succeed.
- `get_match_len()` — additional control over the effective matched span, including cases involving trailing anonymous punctuation.

Important built-in matcher implementations include:

| Matcher                                | Purpose                                           |
| -------------------------------------- | ------------------------------------------------- |
| `Pattern`                              | Structural syntax-tree matching                   |
| `KindMatcher`                          | Match a grammar node kind                         |
| `RegexMatcher`                         | Apply a regex to node text                        |
| `MatchAll` / `MatchNone`               | Unconditional matching primitives                 |
| `Op`, `And`, `Or`, `Not`, `All`, `Any` | Compose matcher logic                             |
| `str`                                  | Convenient direct use of source-like pattern text |

The trait is generic over the document type and is not object-safe (`dyn`-compatible). That is consequential when building heterogeneous rule registries: ordinary `Box<dyn Matcher>` is not a direct substitute for the supplied generic combinators.

A custom matcher can exploit syntax inspection, source text, and externally supplied criteria. However, its implementation must respect the generic document interface and manage metavariable bindings correctly.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source:

[https://docs.rs/ast-grep-core/latest/ast_grep_core/matcher/trait.Matcher.html](https://docs.rs/ast-grep-core/latest/ast_grep_core/matcher/trait.Matcher.html?utm_source=chatgpt.com)

### 5.2 Boolean composition and binding consistency

`ast_grep_core::ops` supplies typed combinators including:

```
Op::every(matcher).and(other)
Op::either(matcher).or(other)
Op::not(matcher)
Op::all(matchers)
Op::any(matchers)
```

These offer native alternatives to implementing many ad hoc AST predicate trees.

The interesting detail is the treatment of metavariable state during composition.

The source implementation uses copy-on-write environments to isolate speculative bindings. For conjunction, bindings from the candidate match are committed only when all conditions succeed. Disjunction can discard unsuccessful branches, and negation deliberately avoids leaking captures produced during a failed negative test.

That matters because a matcher is not merely a Boolean function. It can also produce a mapping from metavariable names to source nodes. A logically correct composite matcher must preserve both the Boolean result and the consistency of that mapping.

Order can matter when earlier successful components establish bindings that subsequent components must satisfy.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



Source:

[https://docs.rs/ast-grep-core/latest/src/ast_grep_core/ops.rs.html](https://docs.rs/ast-grep-core/latest/src/ast_grep_core/ops.rs.html?utm_source=chatgpt.com)

### 5.3 Structural relationships

`Node` exposes direct predicates for checking surrounding syntax:

| Method              | What it examines   |
| ------------------- | ------------------ |
| `matches(matcher)`  | The node itself    |
| `inside(matcher)`   | Ancestors          |
| `has(matcher)`      | Descendants        |
| `precedes(matcher)` | Following siblings |
| `follows(matcher)`  | Preceding siblings |

These permit checks such as whether a call occurs inside a function, whether a declaration contains a particular subtree, or whether one statement precedes another.

Important distinction: these Rust `Node` convenience methods traverse their respective axes. Their semantics should not be confused with the YAML relational rules in `ast-grep-config`, where `stopBy` controls search depth and defaults to the neighboring node. The declarative layer also offers options such as `field`, `nthChild`, and richer relational rule composition.

The core crate gives developers the primitives to implement structural relationships; the configuration crate provides a higher-level rule vocabulary.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



Sources:

[https://docs.rs/ast-grep-core/latest/src/ast_grep_core/node.rs.html](https://docs.rs/ast-grep-core/latest/src/ast_grep_core/node.rs.html?utm_source=chatgpt.com)

[https://ast-grep.github.io/guide/rule-config/relational-rule.html](https://ast-grep.github.io/guide/rule-config/relational-rule.html?utm_source=chatgpt.com)

## 6. Captures, match environments, and structured extraction

### 6.1 `NodeMatch`

A successful `find()` or `find_all()` operation returns `NodeMatch<'tree, D>`, which combines:

1. The matched syntax node.
2. Its populated `MetaVarEnv`.

`NodeMatch` dereferences to `Node`, so ordinary tree inspection remains available directly on the result.

Relevant methods include:

```
match_result.get_node()
match_result.get_env()
match_result.get_env_mut()
match_result.replace_by(replacer)
```

This makes the capture result a useful structural extraction record, rather than merely a source-range hit. A consumer can retain the node kind, location, captured components, and related syntactic evidence.

Source:

[https://docs.rs/ast-grep-core/latest/ast_grep_core/matcher/struct.NodeMatch.html](https://docs.rs/ast-grep-core/latest/ast_grep_core/matcher/struct.NodeMatch.html?utm_source=chatgpt.com)

### 6.2 `MetaVarEnv`

`MetaVarEnv` stores captures and associated transformation data.

Its public API includes:

| Method                                          | Role                                                      |
| ----------------------------------------------- | --------------------------------------------------------- |
| `get_match(name)`                               | Retrieve a single captured node                           |
| `get_multiple_matches(name)`                    | Retrieve a captured node sequence                         |
| `get_matched_variables()`                       | Enumerate variable identifiers in the environment         |
| `insert()` / `insert_multi()`                   | Establish captures programmatically                       |
| `match_constraints()`                           | Evaluate supplied matchers against captured variables     |
| `insert_transformation()` / `get_transformed()` | Store and retrieve transformed values                     |
| `get_var_bytes()`                               | Obtain the underlying source representation of a variable |
| `add_label()` / `get_labels()`                  | Associate additional node groups with labels              |

The last several methods are less obvious but architecturally important.

Constraints allow structural captures to be validated after matching without embedding every criterion in the initial pattern.

Transformations allow replacement generation to consume derived values rather than only the original captured text.

Labels allow a match to carry additional source nodes for downstream reporting or highlighting.

These capabilities make it possible to treat one structural match as a small, evidence-bearing record with multiple associated syntax spans. They do not, by themselves, constitute a cross-file semantic fact model.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source:

[https://docs.rs/ast-grep-core/latest/ast_grep_core/meta_var/struct.MetaVarEnv.html](https://docs.rs/ast-grep-core/latest/ast_grep_core/meta_var/struct.MetaVarEnv.html?utm_source=chatgpt.com)

## 7. Rewriting and source-edit generation

### 7.1 Editing is fundamentally source-range based

`ast-grep-core` does not operate as a mutable object graph in which setting a node field automatically regenerates source code.

Its primary rewrite operation produces an `Edit`, carrying:

```
position
deleted_length
inserted_text
```

For `StrDoc`, these describe an edit against the source's byte-oriented representation.

An edit can be constructed from a matched node, then applied to a root. Once applied, the source text and parse tree are updated together.

This separation is useful because applications can inspect, validate, combine, preview, or reject proposed edits before modifying a document.

### 7.2 Replacers and `TemplateFix`

The `Replacer<D>` abstraction controls replacement generation. The primary text-template implementation is `TemplateFix`.

For example:

```
// Match
console.log($ARG)

// Replacement
logger.info($ARG)
```

The captured `$ARG` is substituted using the matched source fragment rather than regenerated from a newly constructed semantic AST.

The API includes:

```
TemplateFix::try_new(template, &language)
TemplateFix::with_transform(template, &language, transformed_names)
```

`TemplateFix` also implements indentation handling when constructing replacements, particularly for captured multiline source fragments.

This is different from running a full language formatter. It aims to preserve and adjust source text, but does not establish canonical formatting or syntactic correctness for arbitrary replacements.

The `with_transform` interface consumes already defined transformed variables. The higher-level catalog of declarative transformations—such as substring or regex-based replacements—is primarily provided through configuration-layer functionality, not automatically by `TemplateFix` alone.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



Sources:

[https://docs.rs/ast-grep-core/latest/src/ast_grep_core/replacer/template.rs.html](https://docs.rs/ast-grep-core/latest/src/ast_grep_core/replacer/template.rs.html?utm_source=chatgpt.com)

[https://ast-grep.github.io/guide/rewrite-code](https://ast-grep.github.io/guide/rewrite-code?utm_source=chatgpt.com)

[https://ast-grep.github.io/guide/rewrite/transform](https://ast-grep.github.io/guide/rewrite/transform?utm_source=chatgpt.com)

### 7.3 Single-match and multi-match rewriting

There are several distinct editing interfaces:

| Operation                              | Behavior                                                                |
| -------------------------------------- | ----------------------------------------------------------------------- |
| `NodeMatch::replace_by(replacer)`      | Construct an edit replacing the matched node                            |
| `Node::replace(matcher, replacer)`     | Find a match within the subtree and return an edit without applying it  |
| `Root::replace(matcher, replacer)`     | Find the first match, apply its edit, and update the tree               |
| `Node::replace_all(matcher, replacer)` | Construct edits for multiple matches; does not directly mutate the root |
| `Node::remove()`                       | Construct an edit deleting the node                                     |
| `Node::empty()`                        | Construct an edit removing the node's children when present             |
| `Root::edit(edit)`                     | Apply an edit and update the parsed document                            |

The distinction between generating and applying edits is consequential.

For example, `Root::replace()` performs only a first-match replacement. It is not synonymous with replacing all matching occurrences.

Conversely, `Node::replace_all()` returns proposed edits. The caller must decide how those edits are committed to the source.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



### 7.4 Nested matches and edit conflicts

In version 0.51.0, `Node::replace_all()` uses a visitor configured with `reentrant(false)`.

For an expression such as:

```
Some(Some(1))
```

and a pattern:

```
Some($VALUE)
```

both the outer and inner expressions may satisfy the pattern. But the built-in `replace_all()` behavior deliberately avoids generating nested overlapping replacements.

The implementation includes an explicit TODO for supporting nested replacements.

This should not be mistaken for a guarantee that every logically matching node is rewritten in one pass.

For more complex editing, a caller may need to distinguish:

- All structural matches.
- A non-overlapping subset of matches.
- The desired order of nested transformations.
- Edits generated against the original document versus after earlier edits.
- Source ranges invalidated by intervening modifications.

A practical property of byte-offset edits is that applying multiple independent replacements in ascending order without adjusting subsequent offsets can misplace edits. A batch edit strategy should reconcile conflicts and coordinate offsets explicitly.

Source:

[https://docs.rs/ast-grep-core/latest/src/ast_grep_core/tree_sitter/mod.rs.html](https://docs.rs/ast-grep-core/latest/src/ast_grep_core/tree_sitter/mod.rs.html?utm_source=chatgpt.com)

### 7.5 Incomplete edit helpers

There is an unusually important release-specific caveat.

Although the `Node` API exposes:

```
after()
before()
append()
prepend()
```

the inspected 0.51.0 source still implements these methods using `todo!()`.

These are not usable insertion primitives in the verified release; calling them would panic.

This is an example of why the presence of a public Rust method should not automatically be interpreted as evidence of a completed feature.

Reliable insertion behavior can instead be constructed using explicit `Edit` values and the relevant byte boundaries, with appropriate care for punctuation, whitespace, and grammar correctness.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



## 8. Advanced traversal and incremental operation

### 8.1 Lazy structural discovery

`Node::find_all()` returns an iterator rather than eagerly allocating every match.

Its implementation traverses the syntax tree and applies the matcher to candidate nodes. Before invoking the more expensive matcher, it can consult `Matcher::potential_kinds()` and skip candidates whose node kinds are impossible.

This matters for repeated queries over large trees. A selective kind filter can reduce matching work, although the traversal itself still visits candidate tree positions.

`Node::find()` is appropriate when only the first match is needed; the iterator form supports short-circuiting, filtering, and constructing streaming extraction pipelines.

A separate `Visitor` abstraction in the Tree-sitter traversal module offers greater control, including reentrancy and traversal algorithms.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



### 8.2 Traversal strategies and subtree selection

The Tree-sitter adapter includes visitor and traversal support such as `TsPre` and `Visitor`, together with traversal algorithms for preorder, postorder, and level-order operations.

These choices are not merely cosmetic.

Preorder traversal is useful when selecting outer structures before their contents. Postorder can be useful for bottom-up analysis. Level-order traversal supports breadth-oriented inspection.

Reentrancy determines whether traversal continues into a subtree after finding a match, directly affecting nested-match discovery and edit planning.

For code-intelligence applications, selective traversal can reduce unnecessary work compared with repeatedly constructing independent AST walks for each syntactic operation.

However, the library should not be mistaken for an automatically maintained query index over many files. Its ordinary node traversal remains tree-based, and project-level indexing is a separate architectural responsibility.

Source:

[https://docs.rs/ast-grep-core/latest/ast_grep_core/tree_sitter/traversal/index.html](https://docs.rs/ast-grep-core/latest/ast_grep_core/tree_sitter/traversal/index.html?utm_source=chatgpt.com)

### 8.3 Incremental reparsing

One of the more consequential capabilities is the integration with Tree-sitter's incremental parsing model.

When an edit is applied to a standard `StrDoc`, the implementation:

1. Applies the source modification.
2. Updates the old Tree-sitter tree with the edit coordinates.
3. Parses the changed source using the edited old tree.

Tree-sitter can reuse unchanged portions of the previous parse tree, making this approach useful for repeated localized edits.

This is substantially different from treating every edit as a request to discard and reconstruct all parser state.

There are still costs: the source must be modified, the new parse must be computed, and old node views cannot simply be treated as valid handles into the updated tree. Incremental parsing also does not provide incremental semantic name resolution or a persistent multi-file dependency graph.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



### 8.4 Thread-local parser caching

The 0.51.0 Tree-sitter adapter uses a thread-local parser cache keyed by language.

Each worker thread can reuse a parser for repeated documents in the same language without sharing a single mutable parser across threads.

The implementation also resets the parser before parsing a complete document, avoiding reuse of incomplete parser state from an earlier operation.

This eliminates some parser construction overhead and avoids cross-thread parser locking.

It does not mean that a call to `find_all()` automatically executes in parallel. Parallel file scheduling and worker coordination are responsibilities of the caller or higher-level tools.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



Source:

[https://docs.rs/ast-grep-core/latest/src/ast_grep_core/tree_sitter/mod.rs.html](https://docs.rs/ast-grep-core/latest/src/ast_grep_core/tree_sitter/mod.rs.html?utm_source=chatgpt.com)

## 9. Language extensibility and multilingual documents

### 9.1 `Language` and `LanguageExt`

The library separates the structural matching logic from the details of individual programming languages.

`Language` defines the language-facing behavior required by the pattern system, including grammar-kind lookup, field lookup, metavariable conventions, and pattern construction.

Important extension points include:

- `kind_to_id()` and `field_to_id()` for grammar identifiers.
- `meta_var_char()` for the metavariable prefix.
- `expando_char()` for grammars that cannot parse the ordinary metavariable prefix directly.
- `extract_meta_var()` for recognizing metavariable syntax.
- `pre_process_pattern()` for language-specific pattern normalization.
- `build_pattern()` for constructing compiled patterns.
- `from_path()` for source-file language recognition.

`LanguageExt` adds Tree-sitter-specific integration, including `get_ts_language()` and construction of an `AstGrep` document.

This explains why support for additional languages is not simply a matter of adding a new enum variant. Language-specific grammar behavior can affect how patterns parse, which node kinds exist, and whether metavariable syntax must be rewritten before parsing.

Source:

[https://docs.rs/ast-grep-core/latest/ast_grep_core/tree_sitter/trait.LanguageExt.html](https://docs.rs/ast-grep-core/latest/ast_grep_core/tree_sitter/trait.LanguageExt.html?utm_source=chatgpt.com)

### 9.2 Packaged grammars

The `ast-grep-language` companion crate provides `SupportLang`, typed language implementations, language aliases, and file-type mapping.

Its published enum includes 29 built-in language variants in version 0.51.0, spanning conventional languages such as Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, and C#, as well as formats and other languages such as JSON, YAML, HTML, CSS, HCL, Markdown, Solidity, and Zig.

The companion crate also supplies language-specific preprocessing where required.

The core engine is language-generic, but the effective syntax vocabulary is grammar-specific. Two languages may support the same abstract pattern mechanism without exposing equivalent AST node shapes or field names.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source:

[https://docs.rs/ast-grep-language/latest/ast_grep_language/](https://docs.rs/ast-grep-language/latest/ast_grep_language/?utm_source=chatgpt.com)

### 9.3 Injected and embedded languages

A particularly advanced capability is support for language injection: parsing regions of one source document using another grammar.

Examples include JavaScript inside HTML or other embedded-language combinations.

The core `LanguageExt` trait exposes:

```
injectable_languages()
extract_injections()
```

And `Root<StrDoc<L>>` provides:

```
get_injections(get_lang)
```

The inspected implementation extracts included Tree-sitter ranges and builds independent parse trees for embedded-language regions.

An important implementation detail is that the injected tree retains the full source string and parses with included ranges. This helps preserve source-coordinate alignment, rather than requiring every extracted fragment to be treated as an unrelated zero-based document.

There are costs and limits:

- Each injected parse tree is independent.
- The implementation clones the source string into each returned injected document.
- The caller supplies language-name resolution through `get_lang`.
- Cross-language symbol resolution is not automatically established.

This makes language injection powerful for syntax-aware extraction and search inside mixed-language files, but it is not itself an embedded-language semantic integration system.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



Sources:

[https://docs.rs/ast-grep-core/latest/src/ast_grep_core/tree_sitter/mod.rs.html](https://docs.rs/ast-grep-core/latest/src/ast_grep_core/tree_sitter/mod.rs.html?utm_source=chatgpt.com)

[https://ast-grep.github.io/advanced/language-injection](https://ast-grep.github.io/advanced/language-injection?utm_source=chatgpt.com)

### 9.4 Beyond Tree-sitter: `Doc` and `Content`

Although Tree-sitter is the normal parser backend, `ast-grep-core` abstracts over document representation.

The `Doc`, `Content`, and `SgNode` traits describe the underlying source, syntax nodes, and text representation used by the matcher.

This abstraction helps the project support multiple language bindings and source-encoding strategies without duplicating the entire matching engine.

It also offers a more general extension boundary than merely registering another Tree-sitter grammar.

Nevertheless, a custom `Doc` backend is a substantially deeper integration than selecting a grammar from `ast-grep-language`: the application must provide node traversal, document access, text representation, and the other behaviors expected by the core engine.

Source:

[https://docs.rs/ast-grep-core/latest/ast_grep_core/source/index.html](https://docs.rs/ast-grep-core/latest/ast_grep_core/source/index.html?utm_source=chatgpt.com)

## 10. Rust integration example

The following illustration shows the normal separation of parsing, matching, capture extraction, and rewriting. It uses the `ast-grep-language` companion crate to supply the grammar.

Illustrative API usage, reviewed against public interfaces; not executed in this research.

```
use ast_grep_core::Pattern;
use ast_grep_core::replacer::TemplateFix;
use ast_grep_language::{LanguageExt, SupportLang};

fn inspect_and_rewrite() -> Result<(), Box<dyn std::error::Error>> {
    let lang = SupportLang::JavaScript;

    let mut document = lang.ast_grep(
        "console.log(foo);\nconsole.log(bar);"
    );

    // Compile the structural pattern once.
    let pattern = Pattern::try_new(
        "console.log($VALUE)",
        lang.clone(),
    )?;

    // Inspect every match and extract its captured expression.
    {
        let root = document.root();

        for hit in root.find_all(&pattern) {
            let argument = hit.get_env()
                .get_match("VALUE")
                .expect("pattern binds VALUE");

            println!(
                "match={:?}, argument={}",
                hit.range(),
                argument.text(),
            );
        }
    }

    // Replace the FIRST match and reparse the document.
    let replacement = TemplateFix::try_new(
        "logger.info($VALUE)",
        &lang,
    )?;

    document.replace(&pattern, replacement)?;

    println!("{}", document.source());

    Ok(())
}
```

The resulting source would have the first call rewritten while retaining the second:

```
logger.info(foo);
console.log(bar);
```

Three distinctions are visible:

1. The pattern is compiled separately from the source parse.
2. A match yields captured syntax nodes, not merely captured strings.
3. `Root::replace()` applies a single edit and reparses, whereas multi-match editing requires a separate workflow.

For advanced codemods, the same primitives can support extraction, diagnostics, rewrite previews, rule validation, and selective application without adopting the full ast-grep CLI.

## 11. Performance characteristics and architectural implications

The principal performance mechanisms can be characterized without assuming particular benchmark results.

| Mechanism                    | Potential benefit                                       | Important qualification                                            |
| ---------------------------- | ------------------------------------------------------- | ------------------------------------------------------------------ |
| Compiled patterns            | Avoid repeated pattern parsing                          | Cache or reuse patterns explicitly when querying many documents    |
| `potential_kinds()`          | Reject impossible candidate node kinds early            | Filtering does not eliminate traversal of the syntax tree          |
| Lazy `find_all()`            | Avoid allocating unused match results                   | A complete scan still visits candidate nodes                       |
| Tree-sitter cursor traversal | Traverse existing syntax representations efficiently    | Query selectivity and tree size still determine work               |
| Non-reentrant visitors       | Avoid descending into matched subtrees when unnecessary | May intentionally omit nested matches                              |
| Incremental reparsing        | Reuse unaffected parse structure after edits            | No automatic incremental semantic analysis                         |
| Thread-local parser cache    | Reduce parser initialization and shared-lock overhead   | Parsing across files is not automatically parallelized by the core |
| Source-fragment reuse        | Avoid regenerating captured syntax from a new AST       | Replacement validity and formatting remain separate concerns       |

Two broader design consequences follow.

First, structural matching can replace large amounts of bespoke tree-walking code. Code-shaped patterns, capture environments, tree navigation, and compositional predicates collectively provide a concise substrate for many syntactic analyses.

Second, syntax awareness alone does not provide complete code intelligence. AST matching can identify a call with a particular spelling, but the identity of the called function may depend on imports, aliases, type information, macros, and other program context.

In such cases, a semantic engine and a structural matcher can be complementary: one establishes identities and relationships, while the other recognizes local source structure and constructs source-preserving modifications.

The relative advantage depends on whether the task requires source-form recognition, semantic certainty, or both.

## 12. Relationship to adjacent Rust tooling

| Tool or library                 | Distinct capability                                                          | Relationship to `ast-grep-core`                                    |
| ------------------------------- | ---------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| Tree-sitter                     | Incremental parsing and syntax-tree construction                             | The underlying parser used by the normal adapter                   |
| Tree-sitter queries             | Grammar-node and capture-based querying                                      | Alternative declarative syntax query mechanism                     |
| rust-analyzer / `ra_ap_syntax`  | Rust-specific lossless syntax trees and syntax manipulation                  | A more Rust-specialized syntactic substrate                        |
| `ra_ap_hir`                     | Semantic names, references, type-oriented program understanding              | Complements source-shape matching with semantic knowledge          |
| rustc compiler APIs             | Compiler-level HIR, MIR, types, resolution, and transformations              | Appropriate when compiler semantics are essential                  |
| `syn` / `quote` / `proc_macro2` | Rust syntax parsing, quotation, token transformations                        | Often preferable for Rust macro and token-oriented transformations |
| ast-grep configuration and CLI  | Declarative rule systems, project scanning, reporting, interactive rewriting | Higher-level orchestration built around the core matching engine   |

The key difference from Rust-specific compiler and syntax tools is generality across languages. The key difference from raw Tree-sitter is the higher-level structural pattern and rewrite vocabulary.

### ast-grep-core versus Tree-sitter queries

Both operate on Tree-sitter syntax trees, but their query models differ.

Tree-sitter queries describe patterns using grammar node types, fields, captures, predicates, and query-specific syntax. They are well suited to integrations that already expose Tree-sitter queries, such as highlighting and grammar-focused extraction.

`ast-grep-core` emphasizes familiar source-like snippets, metavariable unification, and source-fragment rewriting.

The choice depends partly on whether queries should be expressed in grammar terminology or in language-like code shapes. Neither approach guarantees symbol-resolution correctness simply because a result matches a syntax tree.

### ast-grep-core versus `ra_ap_syntax`

Both can support syntax-aware Rust source editing, but their scope differs.

`ra_ap_syntax` is tightly integrated with rust-analyzer's Rust syntax representation and related typed AST facilities. `ast-grep-core` exposes a language-generic Tree-sitter-based pattern system with directly usable metavariable capture and template rewrites.

For Rust-only transformations tightly coupled to Rust grammar APIs, the rust-analyzer ecosystem may expose more specialized functionality. For language-independent pattern matching, `ast-grep-core` offers a reusable common abstraction.

These are capability distinctions, not a universal performance ranking.

## 13. Correctness boundaries and notable limitations

The following distinctions are especially consequential for advanced uses.

| Area                     | Limitation or boundary                                                                                       |
| ------------------------ | ------------------------------------------------------------------------------------------------------------ |
| Semantic identity        | Syntax matching does not establish resolved symbols, types, or cross-module relationships                    |
| Parsing validity         | Tree-sitter can return recovered trees containing syntax errors or missing nodes                             |
| Pattern interpretation   | A pattern is only as accurate as its language-specific parse and selected effective node                     |
| Default matching         | `Smart` ignores some unnamed target syntax, so matches are not necessarily exact concrete-syntax equivalents |
| Metavariables            | Captures correspond to syntax nodes, not arbitrary substrings                                                |
| Grammar dependence       | Kind IDs, field names, and some structural behavior vary by language and grammar version                     |
| Replacement validity     | Template output is not automatically compiler-checked or fully formatted                                     |
| Nested rewriting         | `replace_all()` deliberately avoids nested matches in the verified implementation                            |
| Multi-edit coordination  | Overlapping edits and position changes require explicit handling                                             |
| Editing lifecycle        | Existing node views should not be reused as handles into a reparsed tree                                     |
| Insertion helpers        | `after`, `before`, `append`, and `prepend` remain unimplemented in inspected 0.51.0 source                   |
| Project-wide execution   | Repository discovery, distributed indexing, and parallel file orchestration are outside the core matcher     |
| Mutable source internals | Directly modifying `StrDoc` fields risks inconsistency between source and parse tree                         |

A further source-level caveat concerns arbitrary edit construction. The standard `String` adapter applies byte-oriented modifications directly to the underlying UTF-8 byte vector. Callers constructing their own edits must preserve valid UTF-8 and legitimate boundaries; safe-looking integer positions alone are not sufficient.

The intended `NodeMatch` and `Node` operations normally derive ranges from the parser, whereas arbitrary externally supplied offsets introduce additional correctness obligations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source+1



## 14. High-value combinations

Several combinations of native functionality are especially significant from an architectural standpoint.

A. Structural extraction with evidence

`Pattern` + `find_all()` + `MetaVarEnv` + node ranges can produce records containing the enclosing syntax construct, captured components, source locations, and original text. This is substantially richer than regular-expression captures while requiring no semantic compiler database.

B. Constraint-driven syntax analysis

`Pattern` + composed `Matcher` conditions + grammar fields + relational traversal can recognize operations based on both their own form and their surrounding context. This reduces the need for a different handwritten AST visitor for each syntactic policy.

C. Source-preserving codemods

`Pattern` + `TemplateFix` + `Edit` + incremental reparsing provides a compact foundation for controlled rewrites. The original syntax fragments remain available, avoiding wholesale source regeneration for each matched construct.

D. Mixed-language structural inspection

`LanguageExt` + `extract_injections()` + `get_injections()` allows embedded-language syntax trees to participate in structural searching while retaining source-coordinate context.

E. Reusable analysis over many documents

Compiled patterns + node-kind prefiltering + thread-local parser reuse can make repeated analysis more economical, especially when the caller separately manages file scheduling, caching, and downstream result storage.

These are architectural possibilities supported by the primitives, not claims that the core crate supplies a complete indexing, refactoring, or semantic-analysis platform.

## 15. Primary-source reference map

The following are direct source URLs for further technical investigation.

| Topic                                      | Primary source                                                                                                                                                                                  |
| ------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Core crate overview                        | [https://docs.rs/ast-grep-core/latest/ast_grep_core/](https://docs.rs/ast-grep-core/latest/ast_grep_core/?utm_source=chatgpt.com)                                                               |
| `Pattern` API                              | [https://docs.rs/ast-grep-core/latest/ast_grep_core/matcher/struct.Pattern.html](https://docs.rs/ast-grep-core/latest/ast_grep_core/matcher/struct.Pattern.html?utm_source=chatgpt.com)         |
| `Matcher` API                              | [https://docs.rs/ast-grep-core/latest/ast_grep_core/matcher/trait.Matcher.html](https://docs.rs/ast-grep-core/latest/ast_grep_core/matcher/trait.Matcher.html?utm_source=chatgpt.com)           |
| `Node` API                                 | [https://docs.rs/ast-grep-core/latest/ast_grep_core/struct.Node.html](https://docs.rs/ast-grep-core/latest/ast_grep_core/struct.Node.html?utm_source=chatgpt.com)                               |
| `NodeMatch` API                            | [https://docs.rs/ast-grep-core/latest/ast_grep_core/matcher/struct.NodeMatch.html](https://docs.rs/ast-grep-core/latest/ast_grep_core/matcher/struct.NodeMatch.html?utm_source=chatgpt.com)     |
| `MetaVarEnv` API                           | [https://docs.rs/ast-grep-core/latest/ast_grep_core/meta_var/struct.MetaVarEnv.html](https://docs.rs/ast-grep-core/latest/ast_grep_core/meta_var/struct.MetaVarEnv.html?utm_source=chatgpt.com) |
| Pattern syntax                             | [https://ast-grep.github.io/guide/pattern-syntax](https://ast-grep.github.io/guide/pattern-syntax?utm_source=chatgpt.com)                                                                       |
| Pattern parsing and metavariable mechanics | [https://ast-grep.github.io/advanced/pattern-parse](https://ast-grep.github.io/advanced/pattern-parse?utm_source=chatgpt.com)                                                                   |
| Matching algorithm and strictness          | [https://ast-grep.github.io/advanced/match-algorithm](https://ast-grep.github.io/advanced/match-algorithm?utm_source=chatgpt.com)                                                               |
| Rewriting                                  | [https://ast-grep.github.io/guide/rewrite-code](https://ast-grep.github.io/guide/rewrite-code?utm_source=chatgpt.com)                                                                           |
| Core Tree-sitter adapter source            | [https://docs.rs/ast-grep-core/latest/src/ast_grep_core/tree_sitter/mod.rs.html](https://docs.rs/ast-grep-core/latest/src/ast_grep_core/tree_sitter/mod.rs.html?utm_source=chatgpt.com)         |
| Node implementation and edit helpers       | [https://docs.rs/ast-grep-core/latest/src/ast_grep_core/node.rs.html](https://docs.rs/ast-grep-core/latest/src/ast_grep_core/node.rs.html?utm_source=chatgpt.com)                               |
| Boolean matcher composition                | [https://docs.rs/ast-grep-core/latest/src/ast_grep_core/ops.rs.html](https://docs.rs/ast-grep-core/latest/src/ast_grep_core/ops.rs.html?utm_source=chatgpt.com)                                 |
| Language abstractions                      | [https://docs.rs/ast-grep-core/latest/ast_grep_core/language/trait.Language.html](https://docs.rs/ast-grep-core/latest/ast_grep_core/language/trait.Language.html?utm_source=chatgpt.com)       |
| Companion language crate                   | [https://docs.rs/ast-grep-language/latest/ast_grep_language/](https://docs.rs/ast-grep-language/latest/ast_grep_language/?utm_source=chatgpt.com)                                               |
| Embedded-language support                  | [https://ast-grep.github.io/advanced/language-injection](https://ast-grep.github.io/advanced/language-injection?utm_source=chatgpt.com)                                                         |
| Upstream source, release 0.51.0            | [https://github.com/ast-grep/ast-grep/tree/0.51.0](https://github.com/ast-grep/ast-grep/tree/0.51.0?utm_source=chatgpt.com)                                                                     |

## 16. Synthesis

The most valuable way to understand `ast-grep-core` is as a language-generic, source-preserving structural matching engine with five mutually reinforcing capabilities:

1. Source-shaped pattern compilation replaces many explicit grammar-node visitors with readable code patterns.
2. Metavariable capture and compositional matching turn syntax hits into structured, context-sensitive extraction results.
3. Tree traversal and grammar access provide the precision needed when source patterns alone are insufficient.
4. Edit generation and incremental reparsing connect structural recognition directly to controlled source transformations.
5. Generic language and document abstractions allow the same matching machinery to work across grammars and integration environments.

Its strongest architectural advantage is the combination of expressive structural queries, capture-aware matching, and source-oriented rewrites behind one Rust API.

Its defining limitation is equally important: it understands the structure of source code, but not automatically the full meaning of a program. Systems requiring reliable semantic refactoring, type-aware constraints, or cross-file identity should pair these syntactic capabilities with appropriate semantic analysis rather than expecting structural matching to supply those guarantees.