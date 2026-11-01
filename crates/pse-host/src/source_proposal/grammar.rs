// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Grammar edits retain editable-tree handles and the existing immutable proposal boundary.
use super::*;
use ra_ap_syntax::{
    AstNode as _, SourceFile, T, ast,
    syntax_editor::{Position, Removable as _, SyntaxAnnotation, SyntaxEditor},
};

/// Supported grammar-only construction; no semantic session or file application is implied.
#[derive(Clone, Debug, clap::Subcommand)]
pub enum GrammarEdit {
    /// Append a written where predicate; necessity and deduplication are not inferred.
    AddWhere {
        /// Complete Rust where predicate, for example `T: Send`.
        #[arg(long)]
        predicate: String,
    },
    /// Remove the selected grouped import branch, settling empty parents and separators.
    RemoveUse,
    /// Remove the selected type bound and its separator.
    RemoveBound,
    /// Append one associated item to a selected impl/trait body.
    InsertItem {
        /// One complete associated Rust item.
        #[arg(long)]
        item: String,
    },
    /// Append one named field to a selected record declaration.
    InsertField {
        /// One field declaration, for example `pub value: usize`.
        #[arg(long)]
        field: String,
    },
    /// Construct a binary expression with parenthesized operands and result.
    Expression {
        /// Rust binary operator, such as `+` or `&&`.
        #[arg(long)]
        operator: String,
        /// Right-hand Rust expression.
        #[arg(long)]
        rhs: String,
    },
}
fn refusal(message: &str) -> ProposalError {
    ProposalError::Grammar(message.into())
}
fn fragment<N: ra_ap_syntax::AstNode>(text: &str) -> Result<N, ProposalError> {
    let parsed = SourceFile::parse(text, ra_ap_syntax::Edition::Edition2024);
    if !parsed.errors().is_empty() {
        return Err(refusal("fragment has native syntax errors"));
    }
    parsed
        .syntax_node()
        .descendants()
        .find_map(N::cast)
        .ok_or_else(|| refusal("fragment has no required native node"))
}
fn add_where<
    N: ra_ap_syntax::AstNode
        + ast::HasGenericParams
        + ra_ap_syntax::syntax_editor::GetOrCreateWhereClause,
>(
    node: &N,
    predicate: ast::WherePred,
    editor: &SyntaxEditor,
) -> Result<(), ProposalError> {
    if let Some(clause) = node.where_clause() {
        let last = clause
            .predicates()
            .last()
            .ok_or_else(|| refusal("existing where clause has no predicate"))?;
        let comma = last
            .syntax()
            .next_sibling_or_token()
            .filter(|token| token.kind() == T![,]);
        let make = editor.make();
        if let Some(comma) = comma {
            editor.insert_all(
                Position::after(comma),
                vec![
                    make.whitespace(" ").into(),
                    predicate.syntax().clone().into(),
                    make.token(T![,]).into(),
                ],
            );
        } else {
            editor.insert_all(
                Position::after(last.syntax()),
                vec![
                    make.token(T![,]).into(),
                    make.whitespace(" ").into(),
                    predicate.syntax().clone().into(),
                ],
            );
        }
    } else {
        node.get_or_create_where_clause(editor, std::iter::once(predicate));
    }
    Ok(())
}
fn remove_bound(bound: ast::TypeBound, editor: &SyntaxEditor) -> Result<(), ProposalError> {
    let list = bound
        .syntax()
        .parent()
        .and_then(ast::TypeBoundList::cast)
        .ok_or_else(|| refusal("bound has no native bound list"))?;
    let bounds = list.bounds().collect::<Vec<_>>();
    if bounds.len() == 1 {
        list.remove(editor);
        return Ok(());
    }
    let next = bound.syntax().next_sibling_or_token();
    let separator = bound
        .syntax()
        .siblings_with_tokens(if bounds.last() == Some(&bound) {
            ra_ap_syntax::Direction::Prev
        } else {
            ra_ap_syntax::Direction::Next
        })
        .skip(1)
        .find(|element| element.kind() == T![+])
        .ok_or_else(|| refusal("bound has no native separator"))?;
    editor.delete(separator);
    if let Some(token) = next.filter(|token| token.kind() == ra_ap_syntax::SyntaxKind::WHITESPACE) {
        editor.delete(token);
    }
    editor.delete(bound.syntax());
    Ok(())
}
/// Produce a source-guarded preview for an exact native node range, retaining unrelated bytes.
/// # Errors
/// Selection is stale/malformed, a fragment is invalid, or the node does not support the operation.
pub fn preview_grammar(
    source: SourceText,
    selected: Range<usize>,
    operation: GrammarEdit,
) -> Result<ChangeProposal, ProposalError> {
    if source.text().get(selected.clone()).is_none() {
        return Err(ProposalError::InvalidRange(0));
    }
    let parsed = SourceFile::parse(source.text(), ra_ap_syntax::Edition::Edition2024);
    let (editor, root) = SyntaxEditor::new(parsed.syntax_node());
    let node = root
        .descendants()
        .filter(|node| {
            usize::from(node.text_range().start()) == selected.start
                && usize::from(node.text_range().end()) == selected.end
                && match &operation {
                    GrammarEdit::AddWhere { .. } => matches!(
                        node.kind(),
                        ra_ap_syntax::SyntaxKind::FN
                            | ra_ap_syntax::SyntaxKind::IMPL
                            | ra_ap_syntax::SyntaxKind::STRUCT
                            | ra_ap_syntax::SyntaxKind::TRAIT
                            | ra_ap_syntax::SyntaxKind::ENUM
                    ),
                    GrammarEdit::RemoveUse => ast::UseTree::can_cast(node.kind()),
                    GrammarEdit::RemoveBound => ast::TypeBound::can_cast(node.kind()),
                    GrammarEdit::InsertItem { .. } => matches!(
                        node.kind(),
                        ra_ap_syntax::SyntaxKind::IMPL
                            | ra_ap_syntax::SyntaxKind::TRAIT
                            | ra_ap_syntax::SyntaxKind::ASSOC_ITEM_LIST
                    ),
                    GrammarEdit::InsertField { .. } => matches!(
                        node.kind(),
                        ra_ap_syntax::SyntaxKind::STRUCT
                            | ra_ap_syntax::SyntaxKind::RECORD_FIELD_LIST
                    ),
                    GrammarEdit::Expression { .. } => ast::Expr::can_cast(node.kind()),
                }
        })
        .last()
        .ok_or_else(|| refusal("selection is not an exact supported native syntax node"))?;
    let annotation = SyntaxAnnotation::default();
    editor.add_annotation(&node, annotation);
    match &operation {
        GrammarEdit::AddWhere { predicate } => {
            let clause =
                fragment::<ast::WhereClause>(&format!("fn __<T>() where {predicate} {{}}"))?;
            if clause.predicates().count() != 1 {
                return Err(refusal("supply exactly one where predicate"));
            }
            let predicate = clause
                .predicates()
                .next()
                .ok_or_else(|| refusal("predicate is unavailable"))?;
            if let Some(node) = ast::Fn::cast(node.clone()) {
                add_where(&node, predicate, &editor)?;
            } else if let Some(node) = ast::Impl::cast(node.clone()) {
                add_where(&node, predicate, &editor)?;
            } else if let Some(node) = ast::Struct::cast(node.clone()) {
                add_where(&node, predicate, &editor)?;
            } else if let Some(node) = ast::Trait::cast(node.clone()) {
                add_where(&node, predicate, &editor)?;
            } else if let Some(node) = ast::Enum::cast(node.clone()) {
                add_where(&node, predicate, &editor)?;
            } else {
                return Err(refusal(
                    "where predicates require a supported generic declaration",
                ));
            }
        }
        GrammarEdit::RemoveUse => {
            ast::UseTree::cast(node)
                .ok_or_else(|| refusal("select the exact import branch"))?
                .remove_recursive(&editor);
        }
        GrammarEdit::RemoveBound => remove_bound(
            ast::TypeBound::cast(node).ok_or_else(|| refusal("select the exact type bound"))?,
            &editor,
        )?,
        GrammarEdit::InsertItem { item } => {
            let items = fragment::<ast::AssocItemList>(&format!("impl __ {{ {item} }}"))?;
            if items.assoc_items().count() != 1 {
                return Err(refusal("supply exactly one associated item"));
            }
            let item = items
                .assoc_items()
                .next()
                .ok_or_else(|| refusal("item is unavailable"))?;
            let list = ast::AssocItemList::cast(node.clone())
                .or_else(|| node.children().find_map(ast::AssocItemList::cast))
                .ok_or_else(|| refusal("select an impl/trait or its associated-item list"))?;
            list.add_items(&editor, vec![item]);
        }
        GrammarEdit::InsertField { field } => {
            let fields = fragment::<ast::RecordFieldList>(&format!("struct __ {{ {field} }}"))?;
            if fields.fields().count() != 1 {
                return Err(refusal("supply exactly one record field"));
            }
            let field = fields
                .fields()
                .next()
                .ok_or_else(|| refusal("field is unavailable"))?;
            let list = ast::RecordFieldList::cast(node.clone())
                .or_else(|| node.children().find_map(ast::RecordFieldList::cast))
                .ok_or_else(|| refusal("select a record declaration or field list"))?;
            let close = list
                .r_curly_token()
                .ok_or_else(|| refusal("record has no closing brace"))?;
            let make = editor.make();
            let mut inserted = Vec::new();
            if let Some(last) = list.fields().last() {
                let has_comma = last
                    .syntax()
                    .siblings_with_tokens(ra_ap_syntax::Direction::Next)
                    .skip(1)
                    .take_while(|element| element.kind() != T!['}'])
                    .any(|element| element.kind() == T![,]);
                if !has_comma {
                    editor.insert(Position::after(last.syntax()), make.token(T![,]));
                }
            }
            inserted.extend([
                make.whitespace("\n    ").into(),
                field.syntax().clone().into(),
                make.token(T![,]).into(),
                make.whitespace("\n").into(),
            ]);
            editor.insert_all(Position::before(close), inserted);
        }
        GrammarEdit::Expression { operator, rhs } => {
            let expression = ast::Expr::cast(node.clone())
                .ok_or_else(|| refusal("select an exact Rust expression"))?;
            let rhs = fragment::<ast::LetStmt>(&format!("fn __() {{ let __ = {rhs}; }}"))?
                .initializer()
                .ok_or_else(|| refusal("right operand is unavailable"))?;
            let binary = fragment::<ast::BinExpr>(&format!("fn __() {{ a {operator} b; }}"))?;
            if binary
                .op_token()
                .is_none_or(|token| token.text() != operator.trim())
            {
                return Err(refusal("supply exactly one native binary operator"));
            }
            let operator = binary
                .op_kind()
                .ok_or_else(|| refusal("operator is not a native binary operator"))?;
            let lhs = ast::make::expr_paren(expression).into();
            let rhs = ast::make::expr_paren(rhs).into();
            let replacement = ast::make::expr_paren(ast::make::expr_bin_op(lhs, operator, rhs));
            editor.replace(&node, replacement.syntax());
        }
    }
    let edit = editor.finish();
    let after = edit.new_root().text().to_string();
    if parsed.errors().is_empty()
        && !SourceFile::parse(&after, ra_ap_syntax::Edition::Edition2024)
            .errors()
            .is_empty()
    {
        return Err(refusal("edited source has new native syntax errors"));
    }
    // An annotation identifies the edited handle across the tree edit. Removed handles may vanish.
    let _mapped = edit.find_annotation(annotation);
    let before = source.text();
    let mut prefix = before
        .bytes()
        .zip(after.bytes())
        .take_while(|(left, right)| left == right)
        .count();
    while !before.is_char_boundary(prefix) || !after.is_char_boundary(prefix) {
        prefix -= 1;
    }
    let mut suffix = before[prefix..]
        .bytes()
        .rev()
        .zip(after[prefix..].bytes().rev())
        .take_while(|(left, right)| left == right)
        .count();
    while !before.is_char_boundary(before.len() - suffix)
        || !after.is_char_boundary(after.len() - suffix)
    {
        suffix -= 1;
    }
    let candidate = EditCandidate::new(
        source.identity().clone(),
        prefix..before.len() - suffix,
        after[prefix..after.len() - suffix].into(),
    );
    ChangeProposal::new(ProducerContext::Syntax { backend: "ra_ap_syntax/0.0.360;SyntaxEditor".into(), configuration: format!("edition=2024;exact-node;{operation:?}") }, vec![source], vec![candidate], vec![0], vec!["Grammar-only preview; no deduplication, necessity, semantic equivalence, compilation or source application established".into()], vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    fn edited<N: ra_ap_syntax::AstNode>(
        source: &str,
        contains: &str,
        operation: GrammarEdit,
    ) -> String {
        let parsed = SourceFile::parse(source, ra_ap_syntax::Edition::Edition2024);
        let node = parsed
            .syntax_node()
            .descendants()
            .filter_map(N::cast)
            .filter(|node| node.syntax().text().to_string().contains(contains))
            .last()
            .unwrap();
        let range = node.syntax().text_range();
        let proposal = preview_grammar(
            SourceText::new("/grammar.rs".into(), source.into()).unwrap(),
            usize::from(range.start())..usize::from(range.end()),
            operation,
        )
        .unwrap();
        let edit = proposal.inspect(0).unwrap();
        let mut result = source.to_string();
        result.replace_range(edit.range.clone(), edit.replacement);
        let reparsed = SourceFile::parse(&result, ra_ap_syntax::Edition::Edition2024);
        assert!(
            reparsed.errors().is_empty(),
            "{result}: {:?}",
            reparsed.errors()
        );
        result
    }
    #[test]
    fn where_edits_preserve_comments_in_existing_and_new_clauses() {
        let result = edited::<ast::Fn>(
            "// untouched\nfn f<T>() where T: Clone /* retained */ { }",
            "f",
            GrammarEdit::AddWhere {
                predicate: "T: Send".into(),
            },
        );
        assert!(result.contains("/* retained */"));
        assert!(result.starts_with("// untouched\n"));
        assert!(result.contains("T: Send"));
        assert!(
            edited::<ast::Fn>(
                "fn f<T>() {}",
                "f",
                GrammarEdit::AddWhere {
                    predicate: "T: Send".into()
                }
            )
            .contains("where T: Send")
        );
    }
    #[test]
    fn grouped_use_bound_and_insertions_follow_native_separators() {
        let result = edited::<ast::UseTree>("use a::{b, c, d};", "c", GrammarEdit::RemoveUse);
        assert!(!result.contains("c,"));
        assert!(result.contains("b") && result.contains("d"));
        let result = edited::<ast::TypeBound>(
            "fn f<T: Clone + Send + Sync>() {}",
            "Send",
            GrammarEdit::RemoveBound,
        );
        assert!(result.contains("Clone") && result.contains("Sync"));
        let result = edited::<ast::Struct>(
            "struct S { /* retained */ a: u8 }",
            "S",
            GrammarEdit::InsertField {
                field: "pub b: u16".into(),
            },
        );
        assert!(result.contains("/* retained */") && result.contains("pub b: u16"));
        let result = edited::<ast::Impl>(
            "impl S { fn a() {} }",
            "S",
            GrammarEdit::InsertItem {
                item: "fn b() {}".into(),
            },
        );
        assert!(result.contains("fn a()") && result.contains("fn b()"));
    }
    #[test]
    fn expression_preview_preserves_parent_precedence_and_source_guards() {
        let result = edited::<ast::BinExpr>(
            "fn f(){ let x = (a + b) * c; }",
            "a + b",
            GrammarEdit::Expression {
                operator: "*".into(),
                rhs: "d + e".into(),
            },
        );
        assert!(result.contains("(("));
        assert!(result.contains("(d + e)"));
        let source = SourceText::new("/grammar.rs".into(), "fn f() {}".into()).unwrap();
        assert!(preview_grammar(source, 1..4, GrammarEdit::RemoveUse).is_err());
    }
}
