// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Demand-aware native structural rules, with no count-before-production pass.
use super::*;
use std::collections::BTreeMap;
use tree_sitter::StreamingIterator as _;

/// A bounded observation never invents the number of omitted matches.
#[derive(Debug)]
pub struct NativeMatches {
    /// Exact source-bound candidate rows.
    pub matches: Vec<SyntaxMatch>,
    /// False when demand or cancellation stopped native examination.
    pub complete: bool,
    /// Parsing recovered syntax rather than establishing valid Rust.
    pub recovery: bool,
}

/// One compatible immutable native selector, shared only for the owning request.
pub enum CompiledSyntax {
    /// Native AST matcher with branch-local capture environments.
    Rule(ast_grep_config::RuleCore),
    /// Native extraction query; cursors remain worker-local.
    TreeSitter(tree_sitter::Query),
}
impl std::fmt::Debug for CompiledSyntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rule(_) => f.write_str("CompiledSyntax::Rule(native_ast_matcher)"),
            Self::TreeSitter(query) => f
                .debug_struct("CompiledSyntax::TreeSitter")
                .field("patterns", &query.pattern_count())
                .field("captures", &query.capture_names())
                .finish(),
        }
    }
}
/// One worker-local parse, shared by compatible AST and Tree-sitter clauses for a file.
pub struct ParsedSyntax {
    tree: AstGrep<StrDoc<RustGrammar>>,
}
impl std::fmt::Debug for ParsedSyntax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParsedSyntax")
            .field("source_bytes", &self.tree.root().range().len())
            .field("recovery", &self.tree.root().get_inner_node().has_error())
            .finish()
    }
}
impl ParsedSyntax {
    /// Parse once from the exact acquired source; no reparsing is needed for another selector.
    pub fn new(source: &SourceText) -> Result<Self, QueryError> {
        Ok(Self {
            tree: AstGrep::doc(
                StrDoc::try_new(source.text(), RustGrammar).map_err(QueryError::Parse)?,
            ),
        })
    }
}
impl CompiledSyntax {
    /// Compile an ast-grep core without lint suppression or collection semantics.
    pub fn rule(rule: &serde_json::Value) -> Result<Self, QueryError> {
        let serialized = if rule.get("rule").is_some() {
            rule.clone()
        } else {
            serde_json::json!({"rule":rule})
        };
        let rule: ast_grep_config::SerializableRuleCore = serde_json::from_value(serialized)?;
        Ok(Self::Rule(
            rule.get_matcher(ast_grep_config::DeserializeEnv::new(RustGrammar))
                .map_err(|error| QueryError::Rule(Box::new(error)))?,
        ))
    }
    /// Compile and validate native extraction predicates once for the request.
    pub fn tree_sitter(text: &str) -> Result<Self, QueryError> {
        let language: tree_sitter::Language = tree_sitter_rust::LANGUAGE.into();
        let query = tree_sitter::Query::new(&language, text)
            .map_err(|error| QueryError::Parse(error.to_string()))?;
        for pattern in 0..query.pattern_count() {
            if !query.general_predicates(pattern).is_empty()
                || !query.property_predicates(pattern).is_empty()
            {
                return Err(QueryError::Parse("Tree-sitter extraction cannot evaluate custom or host property predicates; select native text predicates or an explicit supported provider".into()));
            }
        }
        if query.capture_names().is_empty() {
            return Err(QueryError::Parse(
                "Tree-sitter extraction requires at least one addressed capture".into(),
            ));
        }
        Ok(Self::TreeSitter(query))
    }
    /// Evaluate over the existing compatible parse; each match retains its own bindings.
    pub fn evaluate(
        &self,
        source: &SourceText,
        parsed: &ParsedSyntax,
        limit: Option<usize>,
        token: &CancellationToken,
    ) -> Result<NativeMatches, QueryError> {
        if token.is_cancelled() {
            return Ok(NativeMatches {
                matches: Vec::new(),
                complete: false,
                recovery: parsed.tree.root().get_inner_node().has_error(),
            });
        }
        match self {
            Self::Rule(matcher) => rule_matches(source, parsed, matcher, limit, token),
            Self::TreeSitter(query) => tree_sitter_matches(source, parsed, query, limit, token),
        }
    }
}

/// Compile a native serialized rule core and traverse one parsed tree exactly once.
pub fn native_rule(
    source: &SourceText,
    rule: &serde_json::Value,
    limit: Option<usize>,
    token: &CancellationToken,
) -> Result<NativeMatches, QueryError> {
    CompiledSyntax::rule(rule)?.evaluate(source, &ParsedSyntax::new(source)?, limit, token)
}
fn rule_matches(
    source: &SourceText,
    parsed: &ParsedSyntax,
    matcher: &ast_grep_config::RuleCore,
    limit: Option<usize>,
    token: &CancellationToken,
) -> Result<NativeMatches, QueryError> {
    let root = parsed.tree.root();
    let mut recovery = root.get_inner_node().has_error();
    let mut matches = Vec::new();
    let mut complete = true;
    for node in root.dfs() {
        if token.is_cancelled() || limit.is_some_and(|limit| matches.len() >= limit) {
            complete = false;
            break;
        }
        recovery |= node.is_error() || node.is_missing();
        let Some(matched) = matcher.match_node(node) else {
            continue;
        };
        let mut captures = Vec::new();
        for variable in matched.get_env().get_matched_variables() {
            let (name, sequence) = match variable {
                MetaVariable::Capture(name, _) => (name, false),
                MetaVariable::MultiCapture(name) => (name, true),
                _ => continue,
            };
            let nodes = if sequence {
                matched.get_env().get_multiple_matches(&name)
            } else {
                matched
                    .get_env()
                    .get_match(&name)
                    .cloned()
                    .into_iter()
                    .collect()
            };
            captures.push(Capture {
                name,
                sequence,
                nodes: nodes
                    .into_iter()
                    .map(|node| CapturedNode {
                        locator: locator(source, node.range()),
                        text: node.text().into_owned(),
                    })
                    .collect(),
            });
        }
        captures.sort_by(|a, b| a.name.cmp(&b.name));
        matches.push(SyntaxMatch {
            index: matches.len(),
            locator: locator(source, matched.range()),
            kind: matched.kind().into_owned(),
            text: matched.text().into_owned(),
            captures,
        });
    }
    if token.is_cancelled() {
        complete = false;
    }
    Ok(NativeMatches {
        matches,
        complete,
        recovery,
    })
}

/// Advanced native field/capture extraction; query limits and cancellation stay visible.
pub fn tree_sitter_query(
    source: &SourceText,
    query: &str,
    limit: Option<usize>,
    token: &CancellationToken,
) -> Result<NativeMatches, QueryError> {
    CompiledSyntax::tree_sitter(query)?.evaluate(source, &ParsedSyntax::new(source)?, limit, token)
}
fn tree_sitter_matches(
    source: &SourceText,
    parsed: &ParsedSyntax,
    query: &tree_sitter::Query,
    limit: Option<usize>,
    token: &CancellationToken,
) -> Result<NativeMatches, QueryError> {
    use std::ops::ControlFlow;
    let root = parsed.tree.root().get_inner_node();
    let mut cursor = tree_sitter::QueryCursor::new();
    let mut progress = |_: &tree_sitter::QueryCursorState| {
        if token.is_cancelled() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    };
    let mut matches = Vec::new();
    let mut complete = true;
    {
        let options = tree_sitter::QueryCursorOptions::new().progress_callback(&mut progress);
        let mut native =
            cursor.matches_with_options(query, root, source.text().as_bytes(), options);
        while let Some(found) = native.next() {
            if token.is_cancelled() || limit.is_some_and(|limit| matches.len() >= limit) {
                complete = false;
                break;
            }
            let Some(first) = found.captures().first() else {
                continue;
            };
            let start = found
                .captures()
                .iter()
                .map(|capture| capture.node.start_byte())
                .min()
                .unwrap_or(first.node.start_byte());
            let end = found
                .captures()
                .iter()
                .map(|capture| capture.node.end_byte())
                .max()
                .unwrap_or(first.node.end_byte());
            let mut captured = BTreeMap::<String, Vec<CapturedNode>>::new();
            for capture in found.captures() {
                let name = query.capture_names()[capture.index as usize].to_string();
                captured.entry(name).or_default().push(CapturedNode {
                    locator: locator(source, capture.node.byte_range()),
                    text: source.text()[capture.node.byte_range()].to_string(),
                });
            }
            matches.push(SyntaxMatch {
                index: matches.len(),
                locator: locator(source, start..end),
                kind: first.node.kind().into(),
                text: source.text()[start..end].into(),
                captures: captured
                    .into_iter()
                    .map(|(name, nodes)| Capture {
                        name,
                        sequence: nodes.len() > 1,
                        nodes,
                    })
                    .collect(),
            });
        }
    }
    if cursor.did_exceed_match_limit() {
        return Err(QueryError::Parse(
            "native Tree-sitter match limit exceeded; answer is incomplete".into(),
        ));
    }
    if token.is_cancelled() {
        complete = false;
    }
    Ok(NativeMatches {
        matches,
        complete,
        recovery: root.has_error(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_rules_scope_bindings_and_first_demand() {
        let source = SourceText::new(
            "/test.rs".into(),
            "fn a(){ foo(x,x); foo(x,y); bar(x); }".into(),
        )
        .unwrap();
        let token = CancellationToken::new();
        let rule =
            serde_json::json!({"all":[{"pattern":"foo($A,$A)"},{"not":{"pattern":"bar($A)"}}]});
        let result = native_rule(&source, &rule, None, &token).unwrap();
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].captures[0].nodes[0].text, "x");
        let alternative = serde_json::json!({"any":[{"all":[{"pattern":"foo($A,$A)"},{"has":{"pattern":"never()"}}]},{"pattern":"foo($B,$B)"}]});
        let alternative = native_rule(&source, &alternative, None, &token).unwrap();
        assert_eq!(alternative.matches.len(), 1);
        assert!(
            alternative.matches[0]
                .captures
                .iter()
                .any(|capture| capture.name == "B")
        );
        assert!(
            !alternative.matches[0]
                .captures
                .iter()
                .any(|capture| capture.name == "A"),
            "failed native alternatives must not leak their capture environment"
        );
        let first = native_rule(
            &source,
            &serde_json::json!({"kind":"call_expression"}),
            Some(1),
            &token,
        )
        .unwrap();
        assert_eq!(first.matches.len(), 1);
        assert!(!first.complete);
    }
    #[test]
    fn native_tree_sitter_fields_predicates_and_stale_free_coordinates() {
        let source =
            SourceText::new("/test.rs".into(), "fn café(){ foo(); bar(); }".into()).unwrap();
        let result = tree_sitter_query(
            &source,
            "(call_expression function: (identifier) @name (#eq? @name \"foo\"))",
            None,
            &CancellationToken::new(),
        )
        .unwrap();
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].captures[0].nodes[0].text, "foo");
        assert_eq!(result.matches[0].locator.select(&source).unwrap(), "foo");
        assert!(
            tree_sitter_query(&source, "(not_a_node) @x", None, &CancellationToken::new()).is_err()
        );
        assert!(
            tree_sitter_query(
                &source,
                "((identifier) @x (#custom? @x))",
                None,
                &CancellationToken::new()
            )
            .is_err()
        );
    }
    #[test]
    fn native_relations_preserve_enclosing_fields_and_ignore_no_ad_hoc_candidates() {
        let source = SourceText::new(
            "/relations.rs".into(),
            "fn first(){\n// ast-grep-ignore\nfoo(x,x);bar(y);\n}\nfn other(){foo(z,z);}".into(),
        )
        .unwrap();
        let rule = serde_json::json!({"all":[{"pattern":"foo($A,$A)"},{"inside":{"kind":"function_item","has":{"kind":"identifier","regex":"^first$","field":"name"},"stopBy":"end"}}]});
        let result = native_rule(&source, &rule, None, &CancellationToken::new()).unwrap();
        assert_eq!(
            result.matches.len(),
            1,
            "native lint suppression comments must not suppress ad hoc search witnesses"
        );
        assert_eq!(result.matches[0].text, "foo(x,x)");
        assert!(result.complete && !result.recovery);
        let sibling = serde_json::json!({"all":[{"kind":"expression_statement"},{"has":{"pattern":"foo($A,$A)"}},{"precedes":{"kind":"expression_statement","has":{"pattern":"bar($B)"}}}]});
        assert_eq!(
            native_rule(&source, &sibling, None, &CancellationToken::new())
                .unwrap()
                .matches
                .len(),
            1
        );
    }
    #[test]
    fn tree_sitter_anchors_quantifiers_and_recovered_positive_witnesses_remain_truthful() {
        let source = SourceText::new(
            "/captures.rs".into(),
            "fn complete(){foo(a,b);bar(c);}\nfn incomplete(".into(),
        )
        .unwrap();
        let result = tree_sitter_query(
            &source,
            "(arguments . (identifier) @first . (identifier) @second .)",
            None,
            &CancellationToken::new(),
        )
        .unwrap();
        assert_eq!(result.matches.len(), 1);
        assert!(result.recovery);
        assert_eq!(
            result.matches[0]
                .captures
                .iter()
                .flat_map(|capture| &capture.nodes)
                .map(|node| node.text.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b"]
        );
        let quantified_source =
            SourceText::new("/quantified.rs".into(), "fn complete(){foo(a,b,c);}".into()).unwrap();
        // Quantify the separator with its following node. Anonymous commas
        // break an identifier-only repeated sequence in this native grammar.
        let quantified = tree_sitter_query(
            &quantified_source,
            r#"(arguments (identifier) @arguments ("," (identifier) @arguments)+)"#,
            None,
            &CancellationToken::new(),
        )
        .unwrap();
        assert_eq!(quantified.matches.len(), 1);
        assert_eq!(
            quantified.matches[0]
                .captures
                .iter()
                .find(|capture| capture.name == "arguments")
                .unwrap()
                .nodes
                .iter()
                .map(|node| node.text.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b", "c"],
            "every native repeated-group capture survives extraction"
        );
        assert!(quantified.complete && !quantified.recovery);
        let canceled = CancellationToken::new();
        canceled.cancel();
        let stopped = native_rule(
            &source,
            &serde_json::json!({"kind":"call_expression"}),
            None,
            &canceled,
        )
        .unwrap();
        assert!(!stopped.complete && stopped.matches.is_empty());
    }
}
