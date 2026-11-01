// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Written assertion forms and local derivations, without assurance or macro-identity claims.
use super::*;
use ra_ap_syntax::{
    AstNode as _, ast,
    ast::{HasArgList as _, HasName as _},
};

/// One positional operand; left/right syntax does not designate actual/expected semantics.
#[derive(Debug, Serialize, JsonSchema)]
pub struct AssertionObservation {
    /// Entire original macro call.
    pub invocation: CapturedNode,
    /// Original written macro path.
    pub path: String,
    /// Supported source form, opaque custom macro, local shadowing or malformed input.
    pub interpretation: String,
    /// Positional source operands, excluding optional diagnostic message arguments.
    pub operands: Vec<CapturedNode>,
    /// Macro binding identity remains separate from recognizing its written grammar.
    pub identity: String,
}
/// Written local initializer, not a resolved data-flow or oracle-independence proof.
#[derive(Debug, Serialize, JsonSchema)]
pub struct BindingObservation {
    /// Original pattern.
    pub pattern: CapturedNode,
    /// Original initializer, when present.
    pub initializer: Option<CapturedNode>,
}
/// One written call with authored positional arguments.
#[derive(Debug, Serialize, JsonSchema)]
pub struct CallObservation {
    /// Original entire call.
    pub call: CapturedNode,
    /// Original callee/receiver spelling, without resolved identity.
    pub target: String,
    /// Original ordered arguments.
    pub arguments: Vec<CapturedNode>,
}
/// Source observations useful for inspecting what a selected test actually checks.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ValidationStructure {
    /// Exact written call sites; a call does not establish an assertion.
    pub calls: Vec<CallObservation>,
    /// Supported and opaque macro calls, independently qualified.
    pub assertions: Vec<AssertionObservation>,
    /// Written local expected/actual-value derivation candidates.
    pub bindings: Vec<BindingObservation>,
    /// Original parser recovery facts.
    pub parse_issues: Vec<crate::source_read::ParseIssue>,
    /// Whole observations omitted by output demand.
    pub omitted: usize,
}

/// Parse the selected declaration using existing lossless syntax; no Cargo or native world.
pub fn validation_structure(
    source: &SourceText,
    selected: std::ops::Range<usize>,
    budget: usize,
) -> ValidationStructure {
    let (outline, syntax) = crate::source_read::rust_with_syntax(source.text(), RustEdition::E2024);
    let lines = crate::source_read::line_starts(source.text());
    let capture = |node: ra_ap_syntax::SyntaxNode| {
        let range = node.text_range();
        CapturedNode {
            locator: SourceLocator {
                source: source.identity().clone(),
                range: crate::source_read::location(
                    &lines,
                    usize::from(range.start())..usize::from(range.end()),
                ),
            },
            text: node.text().to_string(),
        }
    };
    let shadowed: std::collections::BTreeSet<_> = syntax
        .descendants()
        .filter_map(ast::MacroRules::cast)
        .filter_map(|item| item.name().map(|name| name.to_string()))
        .collect();
    let mut calls = Vec::new();
    let mut assertions = Vec::new();
    let mut bindings = Vec::new();
    let mut omitted = 0;
    for node in syntax.descendants().filter(|node| {
        let range = node.text_range();
        usize::from(range.start()) >= selected.start && usize::from(range.end()) <= selected.end
    }) {
        if let Some(call) = ast::CallExpr::cast(node.clone()) {
            calls.push(CallObservation {
                call: capture(node.clone()),
                target: call
                    .expr()
                    .map(|expr| expr.syntax().text().to_string())
                    .unwrap_or_default(),
                arguments: call
                    .arg_list()
                    .map(|args| {
                        args.args()
                            .map(|expr| capture(expr.syntax().clone()))
                            .collect()
                    })
                    .unwrap_or_default(),
            });
        } else if let Some(call) = ast::MethodCallExpr::cast(node.clone()) {
            calls.push(CallObservation {
                call: capture(node.clone()),
                target: call
                    .name_ref()
                    .map(|name| name.to_string())
                    .unwrap_or_default(),
                arguments: call
                    .arg_list()
                    .map(|args| {
                        args.args()
                            .map(|expr| capture(expr.syntax().clone()))
                            .collect()
                    })
                    .unwrap_or_default(),
            });
        } else if let Some(binding) = ast::LetStmt::cast(node.clone()) {
            if let Some(pattern) = binding.pat() {
                bindings.push(BindingObservation {
                    pattern: capture(pattern.syntax().clone()),
                    initializer: binding
                        .initializer()
                        .map(|expr| capture(expr.syntax().clone())),
                });
            }
        } else if let Some(mac) = ast::MacroCall::cast(node.clone()) {
            let path = mac
                .path()
                .map(|path| path.syntax().text().to_string())
                .unwrap_or_default();
            let name = path.rsplit("::").next().unwrap_or(&path);
            let count = match name {
                "assert" | "debug_assert" => Some(1),
                "assert_eq" | "assert_ne" | "debug_assert_eq" | "debug_assert_ne" => Some(2),
                _ => None,
            };
            let mut observation = AssertionObservation {
                invocation: capture(node),
                path: path.clone(),
                interpretation:
                    "opaque_custom_macro; inspect original definition before interpreting arguments"
                        .into(),
                operands: Vec::new(),
                identity:
                    "unresolved_written_macro_path; spelling does not establish assertion semantics"
                        .into(),
            };
            if let Some(count) = count {
                if shadowed.contains(name)
                    && !path.starts_with("std::")
                    && !path.starts_with("core::")
                {
                    observation.interpretation =
                        "local_macro_with_same_name; assertion semantics unknown".into();
                } else if let Some(tt) = mac.token_tree() {
                    if let (Some(left), Some(right)) =
                        (tt.left_delimiter_token(), tt.right_delimiter_token())
                    {
                        let start = usize::from(left.text_range().end());
                        let end = usize::from(right.text_range().start());
                        // Parse a known expression-list grammar through the library; preserve offsets
                        // into the original token tree rather than pretty-printing its tokens.
                        let prefix = "__pse(";
                        let parsed = ast::Expr::parse(
                            &format!("{prefix}{})", &source.text()[start..end]),
                            ra_ap_syntax::Edition::Edition2024,
                        );
                        if parsed.errors().is_empty() {
                            if let Some(call) = ast::CallExpr::cast(parsed.syntax_node()) {
                                let args: Vec<_> = call
                                    .arg_list()
                                    .into_iter()
                                    .flat_map(|args| args.args())
                                    .collect();
                                if args.len() >= count {
                                    observation.operands = args
                                        .iter()
                                        .take(count)
                                        .map(|expr| {
                                            let range = expr.syntax().text_range();
                                            let from =
                                                start + usize::from(range.start()) - prefix.len();
                                            let to =
                                                start + usize::from(range.end()) - prefix.len();
                                            CapturedNode {
                                                locator: SourceLocator {
                                                    source: source.identity().clone(),
                                                    range: crate::source_read::location(
                                                        &lines,
                                                        from..to,
                                                    ),
                                                },
                                                text: source.text()[from..to].into(),
                                            }
                                        })
                                        .collect();
                                    observation.interpretation = "supported_written_assertion_form; positional_operands_only".into();
                                } else {
                                    observation.interpretation =
                                        "malformed_supported_assertion_form".into();
                                }
                            }
                        } else {
                            observation.interpretation =
                                "malformed_supported_assertion_form".into();
                        }
                    }
                }
            }
            assertions.push(observation);
        }
    }
    for length in [calls.len(), assertions.len(), bindings.len()] {
        omitted += length.saturating_sub(budget);
    }
    calls.truncate(budget);
    assertions.truncate(budget);
    bindings.truncate(budget);
    ValidationStructure {
        calls,
        assertions,
        bindings,
        parse_issues: outline.parse_issues,
        omitted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operands_keep_source_and_shared_derivation_without_claiming_independence() {
        let text = "fn case(){ let expected = calculation(2); let actual = calculation(2); assert_eq!(actual, expected, \"note, {}\", 3); opaque!(actual, expected); }";
        let source = SourceText::new("/fixture.rs".into(), text.into()).unwrap();
        let result = validation_structure(&source, 0..text.len(), 20);
        assert_eq!(result.bindings.len(), 2);
        assert_eq!(
            result.assertions[0]
                .operands
                .iter()
                .map(|node| node.text.as_str())
                .collect::<Vec<_>>(),
            ["actual", "expected"]
        );
        assert!(
            result.assertions[1]
                .interpretation
                .starts_with("opaque_custom_macro")
        );
        assert!(
            result
                .assertions
                .iter()
                .all(|item| item.identity.starts_with("unresolved"))
        );
    }
    #[test]
    fn shadowing_and_nested_generic_arguments_remain_honest() {
        let text = "macro_rules! assert_eq { ($a:expr,$b:expr)=>{} } fn case(){assert_eq!(a,b);std::assert_eq!(f::<A,B>(x, y), (1, 2));}";
        let source = SourceText::new("/fixture.rs".into(), text.into()).unwrap();
        let result = validation_structure(&source, 0..text.len(), 20);
        assert!(
            result.assertions[0]
                .interpretation
                .starts_with("local_macro")
        );
        assert_eq!(result.assertions[1].operands[0].text, "f::<A,B>(x, y)");
        assert_eq!(result.assertions[1].operands[1].text, "(1, 2)");
    }
}
