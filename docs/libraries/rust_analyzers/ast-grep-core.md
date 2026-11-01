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

# Addendum: `ast-grep-core` 0.51.0 — advanced capabilities and implementation semantics

Research date: November 1, 2026 · Supplements the October 30, 2026 capability reference · Target: Rust crate `ast-grep-core` 0.51.0

This addendum covers functionality and consequential implementation details absent or insufficiently developed in the original guide. It concentrates on programmatic pattern construction, low-level matcher execution, capture semantics, replacement strategies, traversal control, incremental editing, and specialized integration interfaces.

Findings are based primarily on the tagged 0.51.0 upstream Rust implementation. Where an API's observed behavior is narrower than its name or documentation suggests, that distinction is made explicitly. Architectural implications are inferred from the implementations, not presented as benchmarked performance guarantees.

## A. Compiled patterns as programmable syntax representations

### A.1 `PatternNode`: direct access to the compiled pattern representation

In addition to compiling source-shaped strings, `ast-grep-core` exposes the internal representation through the public `PatternNode` enum and `Pattern::node` field.

`PatternNode` has three variants:

| Variant    | Representation                | Significance                                            |
| ---------- | ----------------------------- | ------------------------------------------------------- |
| `MetaVar`  | `MetaVariable` specification  | Captures, dropped metavariables, and sequence wildcards |
| `Terminal` | `text`, `is_named`, `kind_id` | Leaf syntax with optional text-sensitive comparison     |
| `Internal` | `kind_id`, `Vec<PatternNode>` | Recursive syntax structure and ordered child patterns   |

The variants enable inspection and programmatic manipulation of pattern structure without working exclusively through source strings.

The crate also implements `From<Node<'_, D>>` for both `PatternNode` and `Pattern`. This permits an already parsed syntax subtree to be converted into a reusable matcher representation.

Consequently, source-pattern compilation is not the only path:

- `Pattern::try_new()` processes a language-specific pattern string and parses it.
- `Pattern::contextual()` parses a complete enclosing fragment and selects a subtree.
- `Pattern::from(node)` converts an existing syntax node directly into a structural pattern.
- Public `PatternNode` variants permit construction or modification of pattern structure at a lower level.

This makes it possible to derive structural queries from previously inspected source code, reuse syntax trees as pattern inputs, or build query representations programmatically.

Important distinction: conversion from a `Node` is not a semantic AST clone. It recursively constructs matcher nodes, excludes parser-marked missing children, and interprets syntax nodes recognized by `Language::extract_meta_var()` as metavariables. The resulting pattern is therefore governed by ast-grep's matching semantics, not necessarily literal source equivalence.

Direct `PatternNode` manipulation also bypasses some validation and interpretation performed by `PatternBuilder`. For example, contextual patterns maintain a private `root_kind` constraint that ordinary public-field manipulation does not independently reconstruct.

### A.2 Pattern inspection and validation boundaries

`Pattern::dump()` returns a `DumpPattern` tree using a caller-provided kind-ID-to-name function. Its representation contains:

- Whether the node is a metavariable.
- An optional grammar-kind name.
- Pattern text.
- Recursively represented children.

The dump is strictness-aware, not simply a verbatim serialization of the compiled tree. For example, `Signature` suppresses terminal text, AST-oriented strictness levels omit some unnamed terminals, and `Template` suppresses normal grammar-kind distinctions.

This allows inspection of the effective matching representation under the selected strictness policy, a different question from inspecting the original syntax-tree parse.

`PatternError` distinguishes several construction failures:

| Error                 | Meaning                                                       |
| --------------------- | ------------------------------------------------------------- |
| `Parse`               | The underlying document parser failed to produce a tree       |
| `NoContent`           | The ordinary pattern did not contain a selectable syntax node |
| `MultipleNode`        | The source resolved to multiple top-level pattern nodes       |
| `RootMultiMetaVar`    | A standalone `$$$NAME` or `$$$` was used as the root pattern  |
| `InvalidKind`         | A contextual selector named an invalid grammar kind           |
| `NoSelectorInContext` | No node of the requested kind was found in the context        |

A contextual selector chooses the first matching node of the requested kind within its parsed context. It does not establish uniqueness when several such nodes exist.

`Pattern::has_error()` also has a narrower meaning than comprehensive pattern validation: it checks the effective root node's kind against the Tree-sitter error kind. It does not recursively verify that no descendant contains recovered parser errors.

Thus, successful construction, root-kind validity, and complete syntactic validity are separate properties.

### A.3 `fixed_string()` and candidate filtering

`PatternNode::fixed_string()` returns the longest terminal text contained in the pattern. It does not concatenate every fixed fragment into a larger string.

This can supply a cheap textual prefilter for patterns whose selected strictness requires a particular terminal to be present.

However, the prefilter must be consistent with the actual matcher:

- A `Signature` pattern may ignore terminal text.
- A `Template` pattern relaxes grammar-kind comparisons.
- A pure metavariable may offer no useful fixed string.
- The longest terminal alone does not prove that a candidate satisfies the complete pattern.

`Pattern::potential_kinds()` separately provides possible root-kind IDs for candidate filtering. It returns no restriction when the effective pattern cannot be safely constrained by a single kind, including `Template` matching and certain error or metavariable patterns.

These are complementary optimization signals: one is based on literal text, the other on grammar kinds. Neither is a semantic index or a sufficient matching test.

Primary source:\
[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/matcher/pattern.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/matcher/pattern.rs?utm_source=chatgpt.com)

## B. Low-level matcher execution and filtering

### B.1 `MatcherExt`: direct evaluation and subtree discovery

The `ast_grep_core::matcher::MatcherExt` trait supplies two methods to every `Matcher` implementation:

| Method             | Behavior                                                                                   |
| ------------------ | ------------------------------------------------------------------------------------------ |
| `match_node(node)` | Evaluates the supplied node directly, initializing a new capture environment               |
| `find_node(node)`  | Performs a depth-first search from the supplied node, returning the first successful match |

Both return `Option<NodeMatch<'tree, D>>`.

`match_node()` allocates a fresh `MetaVarEnv`, passes it through `match_node_with_env()`, and returns the matched node together with the completed environment. The node returned by a custom matcher can differ from the input candidate, so matching and match-result identity should not be assumed identical in every implementation.

`find_node()` calls `match_node()` on nodes visited by `dfs()`. It does not independently use `potential_kinds()` to reject impossible kinds before evaluating the matcher.

This distinguishes three superficially similar entry points:

- `MatcherExt::match_node()` — evaluate one candidate.
- `Node::find()` / `MatcherExt::find_node()` — stop at the first match during DFS.
- `Node::find_all()` — lazily enumerate matches while applying the available `potential_kinds()` filter.

The third interface can avoid expensive matcher execution on nodes with impossible kinds. The first two are useful for direct tests and first-match discovery, but do not implement that same optimization.

`MatcherExt` is blanket-implemented for all `Matcher` types. Developers implement `Matcher`, not a separate custom extension trait.

### B.2 `KindMatcher` and explicit parser-error matching

`KindMatcher` is more than a string wrapper around `Node::kind()`.

Its constructors and utilities include:

| API                                | Behavior                                                                               |
| ---------------------------------- | -------------------------------------------------------------------------------------- |
| `KindMatcher::new(kind, lang)`     | Constructs the matcher from the language's kind ID; does not return a validation error |
| `KindMatcher::try_new(kind, lang)` | Rejects an invalid kind name                                                           |
| `KindMatcher::from_id(id)`         | Constructs directly from a grammar-kind ID                                             |
| `is_invalid()`                     | Tests whether the resolved kind ID represents an unknown kind                          |
| `KindMatcher::error_matcher()`     | Constructs a matcher for the Tree-sitter `ERROR` kind                                  |

`KindMatcher::error_matcher()` performs an exact kind-ID comparison. It does not automatically match all missing nodes, nor is it a general-purpose predicate for every recovered syntax condition.

A subtle but important distinction exists between `KindMatcher` and structural `Pattern` matching. Internally, pattern comparison treats a goal node with the special `ERROR` kind differently from an ordinary terminal kind. It can function as a wildcard in that matching path. The standalone `KindMatcher` does not inherit this wildcard behavior.

Consequently, an error-bearing compiled pattern should not be treated as interchangeable with an explicit error-node matcher.

### B.3 Raw string matchers versus compiled patterns

The convenience implementation `Matcher for str` compiles a `Pattern` within `match_node_with_env()`:

```
let pattern = Pattern::new(self, node.lang().clone());
pattern.match_node_with_env(node, env)
```

This happens when evaluating each candidate, rather than once per complete search.

The same implementation constructs a pattern again when `get_match_len()` is invoked. It also inherits the default `Matcher::potential_kinds()` implementation, which returns `None`.

For repeated searching, this creates two differences compared with a precompiled `Pattern`:

1. Pattern parsing and compilation can recur for each candidate instead of being amortized across many nodes or documents.
2. `Node::find_all()` cannot exploit the root-kind filter that a compiled `Pattern` can expose.

The source establishes additional work and lost filtering opportunities, not a particular speedup factor. Actual performance depends on tree size, pattern complexity, grammar, and the number of candidates.

It also creates a reliability distinction: the string matcher calls `Pattern::new()`, which unwraps pattern construction and may panic on an invalid pattern, whereas `Pattern::try_new()` provides an explicit error path.

A compiled `Pattern` is therefore both a reusable query artifact and a way to separate query construction failures from execution.

Primary sources:\
[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/matcher.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/matcher.rs?utm_source=chatgpt.com)

[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/matcher/kind.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/matcher/kind.rs?utm_source=chatgpt.com)

## C. Capture environments: equality, constraints, and source recovery

### C.1 Repeated captures use a specialized equality relation

Metavariable consistency is not always equivalent to checking source text or Rust node identity.

For single captures, the implementation uses `does_node_match_exactly()`, whose behavior includes:

- Treating identical node IDs as equal.
- Comparing text when at least one node has no named children.
- Otherwise requiring the same grammar-kind ID, the same number of children, and recursive child equality.

This is a structural equality policy with a text-based fallback for leaf-like nodes. It is not compiler-level equality or resolved-symbol identity.

For repeated multi-captures, `MetaVarEnv::match_multi_var()` compares the captured sequences after filtering to named nodes. Anonymous punctuation can therefore be excluded from repeated-sequence consistency checks.

The equality behavior should not be confused with the source bytes recovered for replacement.

### C.2 Captured node sequences versus recovered source spans

`get_multiple_matches(name)` returns a vector of captured nodes.

For `MetaVariable::MultiCapture`, `get_var_bytes()` instead returns the source interval extending from the first captured node's start to the last captured node's end.

That interval can include punctuation, whitespace, comments, and other syntax between the captured nodes. It is not constructed by concatenating each captured node's text.

This is useful for source-preserving rewrites: a captured argument sequence can retain its original separators and layout even when the capture environment exposes individual nodes for structured inspection.

It also introduces a consequential distinction for extraction pipelines. The captured node sequence and the source interval are different representations of the same syntactic region; one cannot always be reconstructed from the other without revisiting the source.

For multi-captures matching no nodes, the environment can return an empty vector, while byte extraction returns `None`.

### C.3 Constraint evaluation and binding propagation

`MetaVarEnv::match_constraints()` accepts a map of capture names to matcher conditions.

The 0.51.0 implementation iterates over single-node captured variables. It does not independently iterate over all multi-node captures or transformed values.

The method uses a copy-on-write environment. Successful constraint matchers can establish additional bindings, which are committed to the environment if the constraint evaluation succeeds. Failed constraint evaluation does not commit its speculative changes.

This creates a useful capability: capture constraints can do more than return a Boolean result. They may enrich the successful match environment with additional syntactic evidence.

But it is not a general constraint engine over arbitrary sequences, derived strings, or cross-file facts.

### C.4 Labels and transformations share capture-related infrastructure

`MetaVarEnv::add_label()` stores labeled nodes using the same underlying map employed by multi-node captures. This means labels and multi-capture names are not completely independent namespaces in the implementation.

In particular, labeled nodes can appear through multi-capture-oriented access and variable enumeration. Consumers that distinguish ordinary captures from diagnostic labels need to preserve that distinction themselves rather than assuming storage separation.

`insert_transformation()` stores a caller-supplied transformed value, adjusting its indentation relative to the referenced captured source where applicable. It does not provide a general-purpose text-transformation language.

This is the low-level mechanism that permits replacement generation to consume derived values. The higher-level library configuration layer remains responsible for supplying its richer declarative transformation operations.

Primary sources:\
[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/meta_var.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/meta_var.rs?utm_source=chatgpt.com)

[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/match_tree/mod.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/match_tree/mod.rs?utm_source=chatgpt.com)

## D. Replacement strategies and their distinct semantics

### D.1 `Replacer<D>` is a multi-representation abstraction

The `Replacer<D>` trait provides more replacement strategies than string templates alone.

Its required method is `generate_replacement()`, which produces the source representation to insert for a particular `NodeMatch`. The default `get_replaced_range()` method independently determines the source interval to replace.

Version 0.51.0 implements `Replacer<D>` for several types:

| Replacer             | How replacement content is generated                                                          | Important distinction                                                                 |
| -------------------- | --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| `str`                | Parses the string as a replacement template for each generation                               | Convenient, but template preparation is repeated                                      |
| `TemplateFix`        | Uses a previously prepared textual template with metavariable and transformation substitution | Supports indentation-aware captured-fragment insertion                                |
| `Node<'_, D>`        | Copies the node's original source interval                                                    | No metavariable substitution in the replacement node                                  |
| `Root<D>`            | Walks a parsed replacement document and substitutes recognized metavariable nodes             | Syntax-structured discovery of placeholders, but output remains source bytes          |
| Custom `Replacer<D>` | Supplied by the embedding application                                                         | Can generate arbitrary replacement content and customize the effective replaced range |

The `Replacer` implementation for `&T` also allows the same prepared replacer to be borrowed and reused.

This is an important abstraction boundary: matching determines which source structures qualify, whereas replacement generation determines what to insert. The two can be implemented independently and combined through the generic API.

### D.2 Parsed-document replacement with `Root<D>`

The `Root<D>` implementation provides a form of structural template expansion that is not developed in the existing guide.

A replacement can itself be parsed as a document. The implementation then walks that replacement tree and examines named-leaf nodes for recognizable metavariables.

For each recognized metavariable with an available binding, it substitutes the captured source representation. Other portions of the replacement document retain their original text.

This supports replacements with repeated metavariables, nested code structures, and sequence captures without manually assembling every fragment in the caller.

However, parsed replacement does not mean mutable-AST regeneration. The implementation still produces source content by collecting text edits against the replacement document and merging them into an output buffer.

Several boundaries follow:

- Placeholder recognition depends on the replacement grammar and on which syntax nodes are classified as named leaves.
- Replacement parsing helps locate syntactic placeholders but does not establish that the final substituted output is valid code.
- Unlike `TemplateFix`, the structural replacer does not perform the same indentation-normalization process.
- It should not be assumed to protect string-literal contents from substitution. The upstream tests explicitly demonstrate replacement of a metavariable-like fragment within a quoted JavaScript string when the grammar exposes it appropriately.

The most useful distinction is therefore syntax-aware placeholder discovery versus textual placeholder scanning, not an absolute guarantee that syntactic contexts such as strings are left unchanged.

A `Node` replacer represents a third model: it transfers the original source representation of a selected syntax node without interpreting metavariables in that node.

Together, these implementations allow replacement content to originate from a textual template, parsed template, captured source subtree, or custom generator while using the same edit-generation interface.

Primary sources:\
[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/replacer.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/replacer.rs?utm_source=chatgpt.com)

[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/replacer/structural.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/replacer/structural.rs?utm_source=chatgpt.com)

### D.3 `TemplateFix` inspection and validation limitations

`TemplateFix` exposes two additional inspection methods:

- `used_vars()` returns a set of metavariable names referenced by the template.
- `exact_node_var()` returns a single non-transformed metavariable name when its internal parsed variable sequence contains exactly one such reference.

These methods make replacement templates more inspectable. In combination with `Pattern::defined_vars()`, they can support rule validation before a transformation is executed.

But the validation guarantees are limited.

`TemplateFix::try_new()` is effectively infallible in 0.51.0. Its implementation always returns `Ok(create_template(...))`, and `TemplateFixError` is an empty enum. A successful result does not mean that all referenced variables are available in the eventual match environment.

When a recognized replacement variable has no available binding, the template generator can omit its substitution instead of reporting an error. For transformation pipelines, a well-formed template is therefore not the same as a fully resolvable template.

`used_vars()` supports preflight checks, but set membership alone is insufficient to establish correctness. A replacement may refer to single captures, multi-captures, or transformation outputs that require different forms of stored data.

`exact_node_var()` has another noteworthy limitation. Despite its name, the 0.51.0 implementation checks the number and kind of parsed metavariable references, not whether surrounding literal fragments are empty.

The upstream tests explicitly confirm:

| Template                   | `exact_node_var()` |
| -------------------------- | ------------------ |
| `$A`                       | `Some("A")`        |
| `$$$A`                     | `Some("A")`        |
| `prefix $A`                | `Some("A")`        |
| `$A$B`                     | `None`             |
| `$A` marked as transformed | `None`             |

Thus, the result is useful as an indicator that one node-backed variable appears, but it is not proof that the complete replacement consists solely of that node's source text.

`TemplateFix::with_transform()` provides the connection to already computed transformation values. It identifies names that should be read from `MetaVarEnv` as transformed values; it does not independently perform substring, regex, case conversion, or recursive rewriting operations. Those higher-level operations belong to the configuration ecosystem.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://ast-grep.github.io\&sz=32)

ast-grep+1



Primary sources:\
[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/replacer/template.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/replacer/template.rs?utm_source=chatgpt.com)

[https://ast-grep.github.io/guide/rewrite/transform](https://ast-grep.github.io/guide/rewrite/transform?utm_source=chatgpt.com)

### D.4 Match extent and edit extent are not identical

`NodeMatch::replace_by()` constructs an edit using the complete range of the matched node.

By contrast, matcher-driven replacement through `Node::replace()`, `Root::replace()`, or `Node::replace_all()` uses `NodeMatch::make_edit()`, which delegates range selection to `Replacer::get_replaced_range()`.

The default implementation takes the matched node's starting byte and, when available, uses the matcher's `get_match_len()` to determine the length of the replaced interval.

`Pattern::get_match_len()` computes an effective end position from the pattern-matching process. This can differ from the complete span of the matched syntax node, particularly where trailing anonymous syntax is not part of the effective match.

This distinction matters in code containing semicolons, commas, other delimiters, or trailing syntax trivia.

A structural match identifies a syntax node. An edit must also decide whether the entire node or only the syntax actually consumed by the matcher is replaced.

`Replacer<D>` allows those decisions to be coupled without hardcoding one replacement-range policy into every matcher or editing operation.

The following operations are therefore not universally equivalent, even when supplied the same replacement content:

| Operation                         | Edit extent                                                      |
| --------------------------------- | ---------------------------------------------------------------- |
| `NodeMatch::replace_by(replacer)` | Entire matched node range                                        |
| Matcher-driven `Node::replace()`  | Replacer-selected range, using the matcher where applicable      |
| `Root::replace()`                 | Same matcher-driven range selection, then applies the first edit |
| `Node::replace_all()`             | Matcher-driven range for each selected non-overlapping match     |

A custom `Replacer` can also override `get_replaced_range()` to define its own extent policy.

For source-preserving codemods, this makes edit-range selection a first-class part of correctness, not just a mechanical consequence of choosing a matched node.

Primary sources:\
[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/matcher/node_match.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/matcher/node_match.rs?utm_source=chatgpt.com)

[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/replacer.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/replacer.rs?utm_source=chatgpt.com)

## E. Advanced traversal: selection, ordering, and pruning

### E.1 Configurable `Visitor`

`tree_sitter::traversal::Visitor` exposes three largely independent traversal controls:

| Control            | Options                                                               | Effect                                                                |
| ------------------ | --------------------------------------------------------------------- | --------------------------------------------------------------------- |
| `algorithm::<A>()` | `PreOrder`, `PostOrder`, and associated level-order traversal support | Changes the order in which candidate nodes are visited                |
| `named_only(bool)` | `true` / `false`                                                      | Determines whether unnamed grammar nodes are considered as candidates |
| `reentrant(bool)`  | `true` / `false`                                                      | Determines whether traversal can continue into nested matches         |

The default visitor uses preorder traversal, permits reentrant matching, and does not restrict candidates to named nodes.

`named_only(true)` is useful when only grammatical constructs represented by named nodes are relevant. But it can exclude operators, keywords, and punctuation that are meaningful for certain concrete-syntax analyses.

Reentrancy controls a different dimension: it determines how the traversal proceeds after a match.

For example, a non-reentrant preorder visitor can select an enclosing matching expression and skip matches nested beneath it. This is the mechanism used by the built-in multi-replacement workflow to avoid generating overlapping nested edits.

Traversal order matters because first-match and non-reentrant selection are order-dependent. Preorder favors enclosing constructs; postorder exposes descendants before their ancestors.

Level-order traversal uses a `VecDeque`-backed frontier. Its breadth-oriented visitation can be useful for some structural analyses, but memory use depends on the width of the tree, unlike depth-first traversal.

### E.2 `Prune`: caller-directed subtree skipping

A more specialized interface is `tree_sitter::traversal::Prune<L>`.

Unlike an ordinary iterator that decides how to advance internally, `Prune` lets the caller decide whether the current syntax node's children should be visited.

Its interface includes:

| Method                     | Behavior                                                            |
| -------------------------- | ------------------------------------------------------------------- |
| `Prune::new(node)`         | Start traversal at a Tree-sitter-backed node                        |
| `current_node()`           | Inspect the currently selected node                                 |
| `descend()`                | Enter the first child; otherwise advance past the completed subtree |
| `skip_subtree()`           | Advance without entering the current node's children                |
| `current_subtree()`        | Capture an opaque marker for the current subtree                    |
| `has_left_subtree(marker)` | Determine whether traversal has moved beyond that marked subtree    |

The distinction from visitor reentrancy is significant.

A non-reentrant `Visitor` prunes based on the outcome of its matcher. `Prune` allows the embedding application to skip children based on an independent decision made while inspecting the current node.

The decision can depend on syntax kind, source-range constraints, information previously extracted from an enclosing construct, or external data available to the caller.

This supports heterogeneous analysis pipelines in which some subtrees receive detailed inspection while others are intentionally skipped.

`PruneSubtree` markers additionally permit tracking when traversal has exited a previously marked enclosing subtree without repeatedly walking parent chains. These are traversal-local markers, not durable syntax identities or source-edit-stable references.

The API is specialized to Tree-sitter-backed `StrDoc<L>` nodes; it is not a generic traversal implementation for arbitrary `Doc` backends.

### E.3 Performance implications

The library supplies several levels of traversal control with different work profiles.

| Mechanism                                              | Work potentially avoided                         |
| ------------------------------------------------------ | ------------------------------------------------ |
| `Matcher::potential_kinds()` inside `Node::find_all()` | Expensive matching against impossible node kinds |
| `Visitor::named_only(true)`                            | Matching unnamed grammar nodes                   |
| `Visitor::reentrant(false)`                            | Visiting descendants beneath selected matches    |
| `Prune::skip_subtree()`                                | Visiting entire caller-rejected subtrees         |
| First-match search                                     | Continuing traversal after the first success     |

Only `Prune` in this group allows arbitrary caller-directed rejection of a complete subtree before entering its children.

The cost tradeoff is that the application owns the traversal decisions and must ensure its pruning policy does not discard syntactic evidence needed later.

Primary source:\
[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/tree_sitter/traversal.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/tree_sitter/traversal.rs?utm_source=chatgpt.com)

Traversal API clarification: In 0.51.0, the built-in `Algorithm` implementations usable through `Visitor::algorithm::<A>()` are `PreOrder` and `PostOrder`. The `Level` breadth-first traversal is available as a separate iterator, not as a predefined `Visitor` algorithm.

## F. Incremental editing: source mutation, reparsing, and cost boundaries

### F.1 `ContentExt` and `perform_edit()`

The Tree-sitter adapter exposes an additional layer beneath the normal `Root::edit()` operation:

- `ContentExt::accept_edit()` updates a source representation and produces a Tree-sitter `InputEdit`.
- `tree_sitter::perform_edit()` invokes `accept_edit()` and applies the resulting `InputEdit` to an existing Tree-sitter tree.
- `StrDoc::do_edit()` uses this mechanism and then reparses with the edited previous tree.

The important distinction is that `perform_edit()` edits the source and informs the existing tree of the change; it does not itself complete the incremental reparse.

This separation provides a lower-level integration boundary for applications that manage their own document and parser lifecycle.

For the standard `String` source representation, the procedure is effectively:

```
Edit<String>
    |
    v
Compute old byte boundaries and points
    |
    v
Modify the source byte vector
    |
    v
Compute new endpoint
    |
    v
Apply InputEdit to old Tree-sitter tree
    |
    v
Reparse using the edited old tree
    |
    v
Updated syntax tree
```

The final reparse is performed by the `StrDoc` editing implementation, rather than by `perform_edit()` itself.

### F.2 Coordinate calculation is additional work

The standard source adapter computes Tree-sitter points by scanning source bytes from the beginning of the document to the relevant offset.

For an edit, this includes the old start and end positions and the new end position after source mutation.

Although incremental parsing can reuse unaffected portions of the previous syntax tree, this coordinate calculation is not itself incremental.

Consequently, edit cost includes at least three distinguishable operations:

1. Source modification and associated allocation or data movement.
2. Computation of edit-coordinate information.
3. Incremental parsing using the edited previous tree.

For repeated localized modifications, especially in large files, the coordinate and source-modification costs can matter independently of how efficiently Tree-sitter reuses parse structure.

This does not establish a quantitative bottleneck without measurement. It does establish that incremental parsing alone does not guarantee the entire edit operation has cost proportional only to the changed subtree.

### F.3 Mutation is not transactional

The 0.51.0 implementation modifies the source and edits the existing Tree-sitter tree before attempting the incremental reparse.

If that reparse returns an error, `Root::edit()` propagates the failure, but the preceding source modification is not automatically rolled back.

This should be distinguished from ordinary syntax errors. Tree-sitter typically produces recovered syntax trees containing `ERROR` or missing nodes for invalid source. A parse failure returning no tree is a different condition.

The relevant internal `TSParseError::TreeUnavailable` path covers cases in which Tree-sitter cannot return a tree, including parser cancellation or timeout-related failures.

The implementation therefore does not provide an atomic edit transaction with a guaranteed unchanged previous document on failure.

For systems that require transactional edit semantics, snapshotting, staged edits, and rollback are separate responsibilities.

The standard `String` adapter also uses direct mutable access to its UTF-8 byte vector. Externally constructed edits must preserve valid UTF-8 boundaries and inserted content. The `Edit` struct's integer fields alone do not enforce those constraints.

Primary sources:\
[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/tree_sitter/mod.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/tree_sitter/mod.rs?utm_source=chatgpt.com)

[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/node.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/node.rs?utm_source=chatgpt.com)

## G. Grammar availability and low-level Tree-sitter interoperability

### G.1 `ast-grep-language` parser feature gating

The companion `ast-grep-language` crate has an important compile-time distinction between a language being represented in its public API and its parser being available in a particular build.

Its default `builtin-parser` feature enables its packaged Tree-sitter grammar dependencies.

It also provides a narrower `napi-lang` feature configuration containing selected grammars used for integration with the Node.js ecosystem.

The parser dispatch implementation uses conditional compilation. When a grammar dependency is disabled, the corresponding parser function contains an `unimplemented!()` fallback.

This means a minimal-feature build can retain language types and `SupportLang` variants without having the underlying Tree-sitter parser available. Calling an unavailable parser path can panic.

The relevant distinction is:

| Layer                            | What availability establishes                          |
| -------------------------------- | ------------------------------------------------------ |
| Language enum/type               | The language is represented in the Rust type system    |
| Compiled grammar dependency      | The Tree-sitter parser is available to that build      |
| `LanguageExt::get_ts_language()` | The actual grammar implementation can be obtained      |
| Successful parsing               | A syntax tree can be produced for the requested source |

A language enum variant should not be treated as an unconditional runtime guarantee that a corresponding parser is installed or compiled.

This is primarily a companion-crate integration issue, not a limitation of the generic `Matcher` abstraction. It becomes relevant when reducing binary size, embedding selected grammars, or building specialized distribution targets.

### G.2 `Root::adopt()` and underlying Tree-sitter nodes

`Root<D>::adopt()` wraps an underlying `D::Node` as an ast-grep `Node` associated with an existing document root.

This complements `Node::get_inner_node()`, which exposes the underlying syntax-node representation.

Together they permit integration between low-level Tree-sitter operations and ast-grep's higher-level node and matcher interfaces.

However, `adopt()` assumes the supplied node belongs to the root's syntax tree. The 0.51.0 implementation checks lineage using `debug_assert!`, so that check is not an unconditional runtime validation in optimized builds.

The method should therefore be understood as an interoperability adapter for nodes with established ownership and lineage, not a mechanism for moving arbitrary syntax nodes between independent trees.

That distinction is particularly important when working with multiple parsed documents, language injections, or reparsed source trees.

Primary sources:\
[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/language/Cargo.toml](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/language/Cargo.toml?utm_source=chatgpt.com)

[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/language/src/parsers.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/language/src/parsers.rs?utm_source=chatgpt.com)

[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/node.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/node.rs?utm_source=chatgpt.com)

## H. Specialized diagnostic and ownership interfaces

### H.1 `display_context()` for contextual source inspection

The Tree-sitter-backed `Node` implementation provides a documentation-hidden `display_context(before, after)` method.

It returns a `DisplayContext` containing:

| Field        | Meaning                                                      |
| ------------ | ------------------------------------------------------------ |
| `matched`    | Source text belonging to the matched node                    |
| `leading`    | Source text preceding the match within the requested context |
| `trailing`   | Source text following the match within the requested context |
| `start_line` | Zero-based starting line of the returned context             |

The `before` and `after` arguments specify how many surrounding lines should be included.

This is useful when a syntax match must be presented with local textual evidence: for example, diagnostic previews, code-search results, or source-context displays.

The method operates on the original source rather than regenerating the node's syntax. It also avoids requiring the caller to implement line-boundary extraction separately.

Its scope is deliberately limited. It does not produce a full diagnostic object, severity, highlighting style, code frame, or editor protocol representation.

Because the method is marked `#[doc(hidden)]`, it should also be distinguished from the crate's more prominent documented interfaces.

### H.2 `PinnedNodeData`: retaining node-bearing data beyond ordinary borrows

The `pinned` module provides a specialized ownership mechanism for cases where normal Rust node lifetimes become difficult to represent.

The motivating problem is that `Node<'r, D>` contains a reference to its owning `Root<D>`. Under conventional borrowing, a node cannot outlive the root reference from which it was obtained.

This is normally desirable, but complicates interfaces that need to retain both the root and node-bearing data together, including some FFI and cross-thread integration patterns.

The module exposes:

| Interface                  | Purpose                                                                                            |
| -------------------------- | -------------------------------------------------------------------------------------------------- |
| `PinnedNodeData<D, T>`     | Stores a root together with node-bearing data                                                      |
| `PinnedNodeData::new()`    | Constructs the retained structure using a closure that receives a lifetime-extended root reference |
| `get_data()`               | Reassociates retained nodes with the stored root and returns the data                              |
| `into_raw()`               | Transfers ownership of the root and retained data separately                                       |
| `unsafe trait NodeData<D>` | Describes how node references within the retained data are accessed and updated                    |

The implementation supplies `NodeData` support for individual nodes, `NodeMatch`, and vectors of `NodeMatch`.

This provides machinery that is particularly useful for building language bindings or retaining structured match results across ownership boundaries.

But its safety model is materially different from ordinary Rust borrowing:

- The constructor internally performs an unsafe lifetime extension.
- Its validity relies on the underlying syntax tree remaining alive while retained node references are used.
- The module is documentation-hidden.
- The source contains unresolved TODOs concerning its unsafe invariants and safety documentation.
- `into_raw()` transfers additional lifetime-management obligations to the caller.

These are consequential qualifications. The module is a specialized integration facility, not a general-purpose durable node store or an automatically safe way to preserve syntax-node references after document edits.

For conventional analysis, retaining source ranges and stable extracted data avoids the specialized lifetime requirements entirely. For FFI applications, the module offers relevant internal machinery, but its safety contract deserves close examination before treating it as a stable extension surface.

Primary sources:\
[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/tree_sitter/mod.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/tree_sitter/mod.rs?utm_source=chatgpt.com)

[https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/pinned.rs](https://github.com/ast-grep/ast-grep/blob/0.51.0/crates/core/src/pinned.rs?utm_source=chatgpt.com)

## I. Cross-capability implications

The additional APIs expose several important architectural possibilities that are less apparent when the crate is understood only as a source-pattern matcher.

### I.1 Reusable structural query representations

`PatternNode`, `Pattern::from(Node)`, `Pattern::dump()`, and the matcher trait together make pattern compilation and execution separable operations.

They enable reuse of compiled queries, inspection of their effective syntax representations, and construction of patterns from existing parsed source structures.

The principal qualification is that programmatically constructed patterns still inherit ast-grep's grammar-specific and strictness-specific matching rules. Constructing a pattern directly does not automatically validate it or attach compiler semantics.

### I.2 Rule validation before source modification

The combination of `Pattern::defined_vars()`, `TemplateFix::used_vars()`, explicit `PatternError` variants, and direct `MatcherExt` evaluation makes several categories of validation possible without custom pattern parsers.

These facilities can establish whether a pattern compiled, which captures it declares, and which variables a replacement references.

However, they do not establish the complete correctness of a rewrite. That additionally depends on capture availability, replacement representation, effective edit range, resulting syntax validity, and the intended semantic transformation.

This separation is valuable because it avoids mistaking successful template construction for a complete rewrite-correctness guarantee.

### I.3 Structural replacement without a mutable AST framework

`Replacer<D>` combines text templates, parsed replacement documents, existing syntax nodes, and custom generation behind one interface.

This can eliminate substantial bespoke string-assembly machinery for source-preserving transformations.

At the same time, the implementation does not offer an immutable structural editing model that regenerates source code from arbitrary modified ASTs. Its outputs remain source-oriented edits with the corresponding range and formatting obligations.

### I.4 Selective syntax extraction with explicit traversal control

`Visitor` and `Prune` provide different levels of control over which nodes are examined.

Their combination with compiled `Pattern` matchers, `MetaVarEnv`, grammar-field access, and source ranges allows richer syntax extraction than an unconditional full-tree scan.

A caller can choose traversal order, ignore unnamed nodes, avoid nested matches, or prune subtrees according to independently computed criteria.

These mechanisms reduce repeated traversal and matching work when their selection assumptions are valid, but they do not supply a persistent source index or automatically infer which subtrees are irrelevant to every query.

### I.5 Clear separation of parser reuse, edit generation, and mutation guarantees

The incremental editing interface exposes three distinct concerns:

- Producing a semantically appropriate replacement.
- Determining and applying a valid byte-range edit.
- Updating and reparsing the underlying syntax tree.

The core crate supplies machinery for all three, but does not collapse them into an atomic, compiler-validated refactoring operation.

This is an important design distinction: the same primitives can support lightweight structural rewrites, diagnostic previews, or more controlled editing workflows, with different correctness guarantees supplied by the surrounding system.

## Addendum conclusion

The additional capabilities strengthen three aspects of the original reference.

First, matching is more programmable than source-string patterns alone suggest. Public compiled-pattern representations, direct matcher execution, explicit grammar-kind matchers, and caller-controlled traversal provide access to substantial lower-level machinery.

Second, rewriting has multiple native representations and important correctness subtleties. Parsed-document replacers, template inspection, effective edit-range selection, and capture-source recovery can reduce custom codemod machinery, but their semantics are not interchangeable.

Third, lifecycle and integration behavior deserve explicit treatment. Compile-time grammar availability, edit-coordinate calculation, non-transactional mutation, Tree-sitter node adoption, and specialized pinned ownership all affect what the library can guarantee when embedded in a larger system.

With these additions, the original guide and addendum collectively describe both the major capabilities of `ast-grep-core` and the less obvious interfaces that determine how those capabilities compose in advanced Rust applications.

Verification note: API behavior and limitations described here were checked against the 0.51.0 tagged upstream implementation, supplemented by official documentation and Context7. No runtime tests or performance benchmarks were performed as part of this addendum.

Primary release source:\
[https://github.com/ast-grep/ast-grep/tree/0.51.0](https://github.com/ast-grep/ast-grep/tree/0.51.0?utm_source=chatgpt.com)

Core crate metadata and features:\
[https://docs.rs/crate/ast-grep-core/0.51.0](https://docs.rs/crate/ast-grep-core/0.51.0?utm_source=chatgpt.com)

Official traversal reference:\
[https://docs.rs/ast-grep-core/0.51.0/ast_grep_core/tree_sitter/traversal/index.html](https://docs.rs/ast-grep-core/0.51.0/ast_grep_core/tree_sitter/traversal/index.html?utm_source=chatgpt.com)