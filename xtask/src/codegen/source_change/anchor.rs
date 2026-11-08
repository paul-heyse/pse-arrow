// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Locate direct declaration members in the original authored UTF-8.

use super::{Case, SourceSpan};
use anyhow::{Context, Result, ensure};
use miette::SourceCode as _;
use std::ops::Range;
use tree_sitter::{Node, Parser, Query, QueryCursor, StreamingIterator as _};

fn tokens(node: Node<'_>) -> Vec<Node<'_>> {
    node.children(&mut node.walk())
        .filter(|node| !node.is_extra())
        .collect()
}

fn text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}

fn group<'tree>(node: Node<'tree>, delimiter: &str) -> Option<Vec<Node<'tree>>> {
    if node.kind() != "token_tree" {
        return None;
    }
    let nodes = tokens(node);
    let (open, inner) = nodes.split_first()?;
    let (_, inner) = inner.split_last()?;
    (open.kind() == delimiter).then(|| inner.to_vec())
}

fn unique<T>(mut candidates: impl Iterator<Item = T>, target: &str) -> Result<T> {
    let candidate = candidates
        .next()
        .with_context(|| format!("{target} was not found in the selected declaration"))?;
    ensure!(candidates.next().is_none(), "{target} is ambiguous");
    Ok(candidate)
}

fn optional_column(tuple: Node<'_>, source: &str) -> Result<Range<usize>> {
    let named: Vec<_> = tuple
        .named_children(&mut tuple.walk())
        .filter(|node| !node.is_extra())
        .collect();
    let [_, fields, _] = named.as_slice() else {
        anyhow::bail!("canonical_projects must be a registry tuple");
    };
    ensure!(
        fields.kind() == "macro_invocation"
            && fields
                .child_by_field_name("macro")
                .is_some_and(|node| text(node, source) == "vec"),
        "canonical_projects fields must be a vec! declaration"
    );
    let vec = fields
        .named_children(&mut fields.walk())
        .find_map(|node| group(node, "["))
        .context("canonical_projects field list is malformed")?;
    let description = unique(
        vec.split(|node| node.kind() == ",").filter(|item| {
            let [callee, args, ..] = item else {
                return false;
            };
            text(*callee, source) == "column"
                && group(*args, "(").is_some_and(|args| {
                    args.first()
                        .is_some_and(|arg| text(*arg, source) == "\"description\"")
                })
        }),
        "direct description column",
    )?;
    let [callee, args, dot, optional, empty] = description else {
        anyhow::bail!("description must be a direct optional column");
    };
    let args = group(*args, "(").context("description column arguments are malformed")?;
    ensure!(
        matches!(args.as_slice(), [_, comma, _, ..] if comma.kind() == ",")
            && dot.kind() == "."
            && text(*optional, source) == "optional"
            && group(*empty, "(").is_some_and(|args| args.is_empty()),
        "description must be a direct optional column"
    );
    let range = callee.start_byte()..empty.end_byte();
    let expression: syn::Expr = syn::parse_str(&source[range.clone()])
        .context("optional description expression is malformed")?;
    let syn::Expr::MethodCall(expression) = expression else {
        anyhow::bail!("description must be a direct optional column");
    };
    ensure!(
        matches!(*expression.receiver, syn::Expr::Call(call) if call.args.len() == 2),
        "description column requires exactly two arguments"
    );
    Ok(range)
}

fn selected_member<'tree>(
    bodies: impl Iterator<Item = Node<'tree>>,
    source: &str,
) -> Result<Range<usize>> {
    let declarations = bodies.filter_map(|body| group(body, "{")).flat_map(|body| {
        body.windows(2)
            .enumerate()
            .filter_map(|(i, items)| {
                let [keyword, name] = items else { return None };
                (keyword.kind() == "struct" && text(*name, source) == "PackResultSelection")
                    .then(|| (i.checked_sub(1).map(|i| body[i]), body.get(i + 2).copied()))
            })
            .collect::<Vec<_>>()
    });
    let (vis, fields) = unique(declarations, "PackResultSelection projection")?;
    ensure!(
        vis.is_some_and(|node| node.kind() == "pub"),
        "PackResultSelection declaration is malformed"
    );
    let fields = fields
        .and_then(|fields| group(fields, "{"))
        .context("PackResultSelection has no field body")?;
    let start = unique(
        fields.iter().enumerate().filter_map(|(i, node)| {
            (node.kind() == "identifier"
                && text(*node, source) == "selected_field"
                && (i.checked_sub(1).is_some_and(|i| fields[i].kind() == "pub")
                    || fields.get(i + 1).is_some_and(|node| node.kind() == ":")))
            .then_some(i)
        }),
        "direct selected_field member",
    )?;
    let vis = start
        .checked_sub(1)
        .filter(|i| fields[*i].kind() == "pub")
        .context("selected_field has no public member declaration")?;
    let attributes = fields[..vis]
        .rsplit(|node| node.kind() == ";")
        .next()
        .unwrap_or_default();
    let mut pairs = attributes.chunks_exact(2);
    ensure!(
        pairs.all(|pair| pair[0].kind() == "#" && group(pair[1], "[").is_some())
            && pairs.remainder().is_empty(),
        "selected_field must begin a direct member declaration"
    );
    ensure!(
        fields.get(start + 1).is_some_and(|node| node.kind() == ":"),
        "selected_field has no member type"
    );
    let field = fields[start];
    let tail = &fields[start + 2..];
    let arrow = tail
        .iter()
        .position(|node| node.kind() == "=>")
        .context("selected_field has no projection metadata")?;
    let [metadata, semicolon, ..] = &tail[arrow + 1..] else {
        anyhow::bail!("selected_field projection metadata is malformed");
    };
    ensure!(
        arrow > 0 && group(*metadata, "{").is_some() && semicolon.kind() == ";",
        "selected_field projection metadata is malformed"
    );
    syn::parse_str::<syn::Type>(&source[tail[0].start_byte()..tail[arrow - 1].end_byte()])
        .context("selected_field type is malformed")?;
    Ok(fields[vis].start_byte()..field.end_byte())
}

pub(super) fn locate(case: Case, source: &str) -> Result<SourceSpan> {
    let language = tree_sitter_rust::LANGUAGE.into();
    let mut parser = Parser::new();
    parser.set_language(&language)?;
    let tree = parser
        .parse(source, None)
        .context("Rust declaration parse unavailable")?;
    let root = tree.root_node();
    ensure!(
        !root.has_error(),
        "authored Rust declaration has a syntax error"
    );
    let pattern = match case {
        Case::OptionalProjectDescription => {
            r#"
            ((source_file (function_item name: (identifier) @function body: (block
                (expression_statement (for_expression value: (array_expression
                    (tuple_expression . (string_literal) @table) @anchor))))))
                (#eq? @function "declare")
                (#eq? @table "\"canonical_projects\""))"#
        }
        Case::RecordedResultFieldRename => {
            r#"
            ((source_file (macro_invocation
                macro: (scoped_identifier path: (identifier) @crate name: (identifier) @macro)
                (token_tree) @anchor))
                (#eq? @crate "pse_contract") (#eq? @macro "projection_struct"))"#
        }
    };
    let query = Query::new(&language, pattern)?;
    let capture = query
        .capture_index_for_name("anchor")
        .context("anchor capture is missing")?;
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, root, source.as_bytes());
    let mut candidates = Vec::new();
    while let Some(matched) = matches.next() {
        candidates.extend(matched.nodes_for_capture_index(capture));
    }
    ensure!(
        !cursor.did_exceed_match_limit(),
        "authored declaration search exceeded its match limit"
    );
    let range = match case {
        Case::OptionalProjectDescription => optional_column(
            unique(candidates.into_iter(), "canonical_projects tuple")?,
            source,
        )?,
        Case::RecordedResultFieldRename => selected_member(candidates.into_iter(), source)?,
    };
    let location = source.read_span(&(range.start, range.len()).into(), 0, 0)?;
    Ok(SourceSpan {
        byte_offset: range.start,
        byte_length: range.len(),
        line: location.line(),
        column: location.column(),
        column_encoding: "miette_source_column".into(),
    })
}
