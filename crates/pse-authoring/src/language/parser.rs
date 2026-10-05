// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::cells::Cell;
use super::types::{TypeExponent, TypeNode, TypeNodeKind, node, push};
use super::*;
use crate::dsl::lexer::{Kind, Token, tokenize};
use crate::{AuthoringError, ParseBudget, SourceSpan};
use pse_ids::SemanticId;
use pse_model::generated::identities::DeclarationId;
use std::collections::{BTreeMap, BTreeSet};
use winnow::stream::LocatingSlice;

type Parameter = (String, Vec<TypeNode>, Option<String>);
type SpannedDeclarations = (
    Vec<Declaration>,
    BTreeMap<DeclarationId, BTreeMap<String, SourceSpan>>,
);

type Result<T> = std::result::Result<T, AuthoringError>;
/// The internal keyword of a kind-level binding `name = cell;`; no source spells it.
const BIND: &str = "=bind";
/// Identity policy is selected by admitted package metadata, never inferred from a path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityPolicy {
    /// Every declaration carries an explicit `@id("...")`.
    Explicit,
    /// Deterministic creation identities; rename changes identity until IDs are assigned.
    Named,
}

/// Parse a complete modeling source without allocating identities or performing I/O.
/// # Errors
/// Syntax, malformed identities, duplicate identities or exceeded source/nesting limits.
pub fn parse(
    text: &str,
    document: SemanticId,
    policy: IdentityPolicy,
    budget: ParseBudget,
) -> Result<Vec<Declaration>> {
    parse_with_spans(text, document, policy, budget).map(|(rows, _)| rows)
}
/// Parse declarations and exact payload field locations in one source pass.
/// # Errors
/// Source grammar, budget or field-location capture fails.
pub fn parse_with_spans(
    text: &str,
    document: SemanticId,
    policy: IdentityPolicy,
    budget: ParseBudget,
) -> Result<SpannedDeclarations> {
    if text.len() as u64 > budget.max_bytes || text.len() > u32::MAX as usize {
        return Err(AuthoringError::Budget {
            limit: "bytes",
            allowed: budget.max_bytes.min(u64::from(u32::MAX)),
            needed: text.len() as u64,
        });
    }
    let tokens = tokenize(text).map_err(|e| syntax_error(document, e))?;
    let mut parser = Cursor {
        tokens,
        text,
        pos: 0,
        document,
        policy,
        budget,
        rows: Vec::new(),
        ids: BTreeSet::new(),
        type_variables: Vec::new(),
        field_locations: BTreeMap::new(),
    };
    parser.block(None, 0, false)?;
    let mut spans = BTreeMap::new();
    for row in &parser.rows {
        let fields = field_spans::collect(&row.value, &parser.field_locations)
            .map_err(|_| parser.error("source field locations"))?;
        spans.insert(row.declaration_id, fields);
    }
    Ok((parser.rows, spans))
}

fn syntax_error(document: SemanticId, error: crate::dsl::DslError) -> AuthoringError {
    match error {
        crate::dsl::DslError::Syntax {
            offset,
            span,
            expected,
            found,
        } => AuthoringError::Syntax {
            at: SourceSpan::new(document, span.start, span.end),
            offset,
            expected,
            found,
        },
        crate::dsl::DslError::Budget {
            limit,
            allowed,
            needed,
        } => AuthoringError::Budget {
            limit,
            allowed,
            needed,
        },
        other => AuthoringError::Contract {
            at: Some(SourceSpan::head(document)),
            reason: other.to_string(),
        },
    }
}

/// One standalone type expression, with `variables` the type parameters in scope.
pub(super) fn parse_type(text: &str, variables: &[&str]) -> Result<Vec<TypeNode>> {
    let tokens = tokenize(text).map_err(|e| syntax_error(SemanticId::NIL, e))?;
    let mut parser = Cursor {
        tokens,
        text,
        pos: 0,
        document: SemanticId::NIL,
        policy: IdentityPolicy::Named,
        budget: ParseBudget::default(),
        rows: Vec::new(),
        ids: BTreeSet::new(),
        type_variables: variables.iter().map(|v| (*v).to_owned()).collect(),
        field_locations: BTreeMap::new(),
    };
    let nodes = parser.type_expr()?;
    if parser.pos != parser.tokens.len() {
        return Err(parser.error("end of type"));
    }
    Ok(nodes)
}

/// One standalone data cell (ADR-0123 Outcome 1).
pub(super) fn parse_cell(text: &str) -> Result<Cell> {
    let tokens = tokenize(text).map_err(|e| syntax_error(SemanticId::NIL, e))?;
    let mut parser = Cursor {
        tokens,
        text,
        pos: 0,
        document: SemanticId::NIL,
        policy: IdentityPolicy::Named,
        budget: ParseBudget::default(),
        rows: Vec::new(),
        ids: BTreeSet::new(),
        type_variables: Vec::new(),
        field_locations: BTreeMap::new(),
    };
    let cell = parser.cell()?;
    if parser.pos != parser.tokens.len() {
        return Err(parser.error("end of cell"));
    }
    Ok(cell)
}

/// Explicit source-creation operation. Existing IDs are retained; missing ones receive UUIDv7.
/// # Errors
/// The source is malformed or cannot be rendered.
pub fn assign_ids(text: &str, document: SemanticId, budget: ParseBudget) -> Result<String> {
    assign_ids_with(text, document, budget, &mut || {
        SemanticId::from_bytes(*uuid::Uuid::now_v7().as_bytes())
    })
}
/// Explicit creation action using a caller-owned identity supplier.
/// # Errors
/// Invalid source or a nil/duplicate supplied identity.
pub fn assign_ids_with(
    text: &str,
    document: SemanticId,
    budget: ParseBudget,
    next: &mut dyn FnMut() -> SemanticId,
) -> Result<String> {
    let mut rows = parse(text, document, IdentityPolicy::Named, budget)?;
    let mut remap = BTreeMap::new();
    let mut used = rows
        .iter()
        .filter(|row| text[row.source_start as usize..].starts_with("@id"))
        .map(|r| r.declaration_id)
        .collect::<BTreeSet<_>>();
    for row in &rows {
        let start = row.source_start as usize;
        if !text[start..].starts_with("@id") {
            let id = DeclarationId::from(next());
            if id.as_id() == SemanticId::NIL || !used.insert(id) {
                return Err(AuthoringError::Contract {
                    at: Some(SourceSpan::new(
                        document,
                        row.source_start as u32,
                        row.source_end as u32,
                    )),
                    reason: "ID supplier returned a nil or duplicate identity".into(),
                });
            }
            remap.insert(row.declaration_id, id);
        }
    }
    // An enumeration member written without `@id` carries its named derivation; it
    // receives a supplied identity as a declaration does (ADR-0123 Outcome 2).
    let mut members = BTreeSet::new();
    for row in &mut rows {
        let owner = row.declaration_id.as_id();
        let span = SourceSpan::new(document, row.source_start as u32, row.source_end as u32);
        for member in row
            .value
            .enumeration
            .as_mut()
            .into_iter()
            .flat_map(|e| e.members.iter_mut())
        {
            if member.member_id == pse_ids::named_id(owner, &format!("member:{}", member.name)) {
                let id = next();
                if id == SemanticId::NIL || !members.insert(id) {
                    return Err(AuthoringError::Contract {
                        at: Some(span),
                        reason: "ID supplier returned a nil or duplicate member identity".into(),
                    });
                }
                member.member_id = id;
            }
        }
    }
    for row in &mut rows {
        row.declaration_id = remap
            .get(&row.declaration_id)
            .copied()
            .unwrap_or(row.declaration_id);
        row.parent_id = row
            .parent_id
            .map(|id| remap.get(&id).copied().unwrap_or(id));
    }
    render(&rows)
}

struct Cursor<'a> {
    tokens: Vec<Token<'a>>,
    text: &'a str,
    pos: usize,
    document: SemanticId,
    policy: IdentityPolicy,
    budget: ParseBudget,
    rows: Vec<Declaration>,
    ids: BTreeSet<DeclarationId>,
    /// The type parameters of the enclosing scopes and function, innermost last: a
    /// type expression names them as `variable` nodes.
    type_variables: Vec<String>,
    field_locations: BTreeMap<usize, (String, SourceSpan)>,
}
impl Cursor<'_> {
    fn peek(&self) -> &str {
        self.tokens.get(self.pos).map_or("", |t| t.text)
    }
    fn eat(&mut self, value: &str) -> bool {
        if self.peek() == value {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn at(&self) -> u32 {
        self.tokens
            .get(self.pos)
            .map_or(self.text.len() as u32, |t| t.span.start)
    }
    fn error(&self, expected: &str) -> AuthoringError {
        let token = self.tokens.get(self.pos);
        AuthoringError::Syntax {
            at: SourceSpan::new(
                self.document,
                self.at(),
                token.map_or(self.at(), |t| t.span.end),
            ),
            offset: self.at(),
            expected: expected.into(),
            found: self.peek().into(),
        }
    }
    fn expect(&mut self, value: &str) -> Result<()> {
        if self.eat(value) {
            Ok(())
        } else {
            Err(self.error(value))
        }
    }
    fn word(&mut self) -> Result<String> {
        let token = self
            .tokens
            .get(self.pos)
            .ok_or_else(|| self.error("name"))?;
        if !matches!(token.kind, Kind::Identifier | Kind::Quoted) {
            return Err(self.error("name"));
        }
        self.pos += 1;
        let span = token.span;
        let value: String = if token.kind == Kind::Quoted {
            crate::grammar::quoted(&mut LocatingSlice::new(token.text))
                .map_err(|_| self.error("quoted string"))?
        } else {
            token.text.into()
        };
        self.field_locations.insert(
            value.as_ptr() as usize,
            (
                value.clone(),
                SourceSpan::new(self.document, span.start, span.end),
            ),
        );
        Ok(value)
    }

    fn path(&mut self) -> Result<String> {
        let start = self.at();
        let mut name = self.word()?;
        while self.eat(".") {
            name.push('.');
            name.push_str(&self.word()?);
        }
        let end = self.tokens[self.pos - 1].span.end;
        self.field_locations.insert(
            name.as_ptr() as usize,
            (name.clone(), SourceSpan::new(self.document, start, end)),
        );
        Ok(name)
    }
    fn until(&mut self, stops: &[&str]) -> Result<String> {
        let begin = self.pos;
        let mut stack = Vec::new();
        while self.pos < self.tokens.len() {
            let t = self.peek();
            if stack.is_empty() && stops.contains(&t) {
                break;
            }
            match t {
                "(" => stack.push(")"),
                "[" => stack.push("]"),
                "{" => stack.push("}"),
                ")" | "]" | "}" if stack.pop() != Some(t) => {
                    return Err(self.error("balanced expression"));
                }
                _ => {}
            }
            if stack.len() > self.budget.max_depth as usize {
                return Err(self.error("bounded expression"));
            }
            self.pos += 1;
        }
        if !stack.is_empty() {
            return Err(self.error("closed expression"));
        }
        if begin == self.pos {
            return Err(self.error("nonempty expression"));
        }
        let start = self.tokens[begin].span.start as usize;
        let end = self.tokens[self.pos - 1].span.end as usize;
        let value: String = self.text[start..end].trim().into();
        self.field_locations.insert(
            value.as_ptr() as usize,
            (
                value.clone(),
                SourceSpan::new(self.document, start as u32, end as u32),
            ),
        );
        Ok(value)
    }
    fn peek_at(&self, ahead: usize) -> &str {
        self.tokens.get(self.pos + ahead).map_or("", |t| t.text)
    }
    /// Whether `parent` is an entity kind whose block is being read.
    fn in_kind(&self, parent: Option<DeclarationId>) -> bool {
        parent
            .and_then(|parent| self.rows.iter().rev().find(|r| r.declaration_id == parent))
            .is_some_and(|row| {
                row.value.kind == pse_model::generated::enums::ModelingDeclarationKind::EntityKind
            })
    }
    /// Path segments of a name, kept apart so a quoted segment may contain a dot.
    fn segments(&mut self) -> Result<Vec<String>> {
        let mut segments = vec![self.word()?];
        while self.eat(".") {
            segments.push(self.word()?);
        }
        Ok(segments)
    }
    /// ADR-0123 Outcome 5: `provenance(source, role[, lineage(kind path, …)])`, each a
    /// path; a lineage entry names a dataset or a source.
    fn provenance(&mut self) -> Result<ModelingProvenance> {
        self.expect("provenance")?;
        self.expect("(")?;
        let source = self.segments()?;
        self.expect(",")?;
        let role = self.segments()?;
        let mut lineage = Vec::new();
        if self.eat(",") {
            self.expect("lineage")?;
            self.expect("(")?;
            loop {
                let kind = self.vocabulary::<pse_model::generated::enums::ModelingLineageKind>(
                    "lineage kind dataset, source or fit",
                )?;
                lineage.push(ModelingLineageEntry {
                    kind,
                    path: self.segments()?,
                });
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }
        self.expect(")")?;
        Ok(ModelingProvenance {
            source,
            role,
            lineage,
        })
    }
    /// ADR-0123 Outcome 1: one type expression as a post-order arena.
    ///
    /// ```text
    /// type    := 'Fn' '(' [name ':' type {',' name ':' type}] ')' '->' type
    ///          | indexed ['?']
    /// indexed := product ['[' path {',' path} ']']
    /// product := power {('*' | '/') power}
    /// power   := primary ['^' (['-'] integer | '(' ['-'] integer ['/' integer] ')')]
    /// primary := 'Boolean' | 'Integer' | 'Text' | '(' type ')'
    ///          | ('Set' | 'Row' | 'Table' | 'Delta' | 'Δ') '<' type '>'
    ///          | 'Tuple' '<' type {',' type} '>' | 'Id' '<' path '>'
    ///          | 'QuantityType' | 'ReferenceState' | path
    /// ```
    ///
    /// A single-segment path naming a type parameter in scope is a `variable` node.
    fn type_expr(&mut self) -> Result<Vec<TypeNode>> {
        let mut nodes = Vec::new();
        self.type_node(&mut nodes, 0)?;
        Ok(nodes)
    }
    fn type_node(&mut self, nodes: &mut Vec<TypeNode>, depth: u32) -> Result<u32> {
        if depth > self.budget.max_depth {
            return Err(self.error("bounded type"));
        }
        if self.peek() == "Fn" && self.peek_at(1) == "(" {
            self.pos += 2;
            let mut children = Vec::new();
            let mut names = BTreeSet::new();
            if !self.eat(")") {
                loop {
                    let at = self.pos;
                    let name = self.word()?;
                    if !names.insert(name.clone()) {
                        self.pos = at;
                        return Err(self.error("distinct function argument name"));
                    }
                    self.expect(":")?;
                    let ty = self.type_node(nodes, depth + 1)?;
                    children.push(push(
                        nodes,
                        TypeNode {
                            name: Some(name),
                            ..node(TypeNodeKind::Argument, vec![ty])
                        },
                    ));
                    if self.eat(")") {
                        break;
                    }
                    self.expect(",")?;
                }
            }
            self.expect("->")?;
            children.push(self.type_node(nodes, depth + 1)?);
            return Ok(push(nodes, node(TypeNodeKind::Function, children)));
        }
        let ty = self.type_indexed(nodes, depth)?;
        Ok(if self.eat("?") {
            push(nodes, node(TypeNodeKind::Optional, vec![ty]))
        } else {
            ty
        })
    }
    fn type_indexed(&mut self, nodes: &mut Vec<TypeNode>, depth: u32) -> Result<u32> {
        let element = self.type_product(nodes, depth)?;
        if !self.eat("[") {
            return Ok(element);
        }
        let mut children = vec![element];
        loop {
            let path = self.segments()?;
            children.push(push(
                nodes,
                TypeNode {
                    path: Some(path),
                    ..node(TypeNodeKind::Named, vec![])
                },
            ));
            if self.eat("]") {
                break;
            }
            self.expect(",")?;
        }
        Ok(push(nodes, node(TypeNodeKind::Indexed, children)))
    }
    fn type_product(&mut self, nodes: &mut Vec<TypeNode>, depth: u32) -> Result<u32> {
        let mut left = self.type_power(nodes, depth)?;
        loop {
            let kind = match self.peek() {
                "*" => TypeNodeKind::Product,
                "/" => TypeNodeKind::Quotient,
                _ => return Ok(left),
            };
            self.pos += 1;
            let right = self.type_power(nodes, depth)?;
            left = push(nodes, node(kind, vec![left, right]));
        }
    }
    fn type_integer(&mut self) -> Result<i32> {
        let at = self.pos;
        let negative = self.eat("-");
        let value = self
            .tokens
            .get(self.pos)
            .filter(|t| t.kind == Kind::Number)
            .and_then(|t| t.text.parse::<i32>().ok());
        let Some(value) = value else {
            self.pos = at;
            return Err(self.error("integer exponent"));
        };
        self.pos += 1;
        Ok(if negative { -value } else { value })
    }
    fn type_power(&mut self, nodes: &mut Vec<TypeNode>, depth: u32) -> Result<u32> {
        let base = self.type_primary(nodes, depth)?;
        if !self.eat("^") {
            return Ok(base);
        }
        let at = self.pos;
        let (num, den) = if self.eat("(") {
            let num = self.type_integer()?;
            let den = if self.eat("/") {
                self.type_integer()?
            } else {
                1
            };
            self.expect(")")?;
            (num, den)
        } else {
            (self.type_integer()?, 1)
        };
        let exponent = pse_quantity::Ratio::new(num, den).map_err(|_| {
            self.pos = at;
            self.error("rational exponent")
        })?;
        Ok(push(
            nodes,
            TypeNode {
                exponent: Some(TypeExponent {
                    num: exponent.num(),
                    den: exponent.den(),
                }),
                ..node(TypeNodeKind::Power, vec![base])
            },
        ))
    }
    /// Close a generic; in a type, `Set<T>=value` closes it before the assignment.
    fn close_generic(&mut self) -> Result<()> {
        if self.peek() == ">=" {
            let token = self.tokens[self.pos];
            let split = token.span.start + 1;
            self.tokens[self.pos] = Token {
                text: &self.text[token.span.start as usize..split as usize],
                span: crate::dsl::Span {
                    start: token.span.start,
                    end: split,
                },
                kind: token.kind,
            };
            self.tokens.insert(
                self.pos + 1,
                Token {
                    text: &self.text[split as usize..token.span.end as usize],
                    span: crate::dsl::Span {
                        start: split,
                        end: token.span.end,
                    },
                    kind: token.kind,
                },
            );
        }
        self.expect(">")
    }
    fn type_primary(&mut self, nodes: &mut Vec<TypeNode>, depth: u32) -> Result<u32> {
        use TypeNodeKind as K;
        if self.eat("(") {
            let ty = self.type_node(nodes, depth + 1)?;
            self.expect(")")?;
            return Ok(ty);
        }
        if self.peek_at(1) == "<" {
            let kind = match self.peek() {
                "Set" => Some(K::Set),
                "Row" => Some(K::Row),
                "Table" => Some(K::Table),
                "Delta" | "Δ" => Some(K::Delta),
                "Tuple" => Some(K::Tuple),
                "Id" => Some(K::Identifier),
                "Coordinate" => Some(K::Coordinate),
                "Reduced" => Some(K::ReducedLaw),
                "Transfer" => Some(K::Transfer),
                _ => None,
            };
            if let Some(kind) = kind {
                self.pos += 2;
                if matches!(kind, K::Identifier | K::Coordinate | K::ReducedLaw) {
                    let path = self.segments()?;
                    self.close_generic()?;
                    return Ok(push(
                        nodes,
                        TypeNode {
                            path: Some(path),
                            ..node(kind, vec![])
                        },
                    ));
                }
                let mut children = vec![self.type_node(nodes, depth + 1)?];
                while matches!(kind, K::Tuple | K::Transfer) && self.eat(",") {
                    children.push(self.type_node(nodes, depth + 1)?);
                }
                if kind == K::Transfer && children.len() != 3 {
                    return Err(self.error("Transfer<physical type, boundary, Into|OutOf>"));
                }
                self.close_generic()?;
                return Ok(push(nodes, node(kind, children)));
            }
        }
        if !matches!(
            self.tokens.get(self.pos).map(|t| t.kind),
            Some(Kind::Identifier | Kind::Quoted)
        ) {
            return Err(self.error("type"));
        }
        let path = self.segments()?;
        if let [single] = path.as_slice() {
            let leaf = match single.as_str() {
                "Boolean" => Some(K::Boolean),
                "Integer" => Some(K::Integer),
                "Text" => Some(K::Text),
                "QuantityType" => Some(K::QuantityType),
                "ReferenceState" => Some(K::ReferenceState),
                "Applicability" => Some(K::Applicability),
                _ => None,
            };
            if let Some(kind) = leaf {
                return Ok(push(nodes, node(kind, vec![])));
            }
            if self.type_variables.iter().any(|v| v == single) {
                return Ok(push(
                    nodes,
                    TypeNode {
                        name: Some(single.clone()),
                        ..node(K::Variable, vec![])
                    },
                ));
            }
        }
        Ok(push(
            nodes,
            TypeNode {
                path: Some(path),
                ..node(K::Named, vec![])
            },
        ))
    }
    /// A cell path: a plain identifier, then further segments that may be quoted.
    fn cell_path(&mut self) -> Result<Vec<String>> {
        if self.tokens.get(self.pos).map(|t| t.kind) != Some(Kind::Identifier) {
            return Err(self.error("cell path"));
        }
        self.segments()
    }
    /// A finite number token with an optional leading minus sign.
    fn cell_number(&mut self, expected: &str) -> Result<(f64, bool)> {
        let at = self.pos;
        let negative = self.eat("-");
        let Some(token) = self.tokens.get(self.pos).filter(|t| t.kind == Kind::Number) else {
            self.pos = at;
            return Err(self.error(expected));
        };
        let value = token
            .text
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| self.error(expected))?;
        let integral = token.text.bytes().all(|b| b.is_ascii_digit());
        self.pos += 1;
        Ok((if negative { -value } else { value }, integral))
    }
    /// ADR-0123 Outcome 1: one data cell, parsed once where it is written; see
    /// [`super::cells`] for the grammar.
    fn cell(&mut self) -> Result<Cell> {
        use super::cells::*;
        let start = self.pos;
        let value = match self.peek() {
            "missing" | "true" | "false" if self.peek_at(1) != "." => {
                let word = self.word()?;
                match word.as_str() {
                    "missing" => CellValue::from_missing(),
                    other => CellValue::from_boolean(CellBoolean {
                        value: other == "true",
                    }),
                }
            }
            "Id" if self.peek_at(1) == "<" => {
                self.pos += 2;
                let scheme = self.segments()?;
                self.close_generic()?;
                self.expect("(")?;
                let at = self.pos;
                if self.tokens.get(self.pos).map(|t| t.kind) != Some(Kind::Quoted) {
                    return Err(self.error("quoted identifier value"));
                }
                let value = self.word().inspect_err(|_| self.pos = at)?;
                self.expect(")")?;
                CellValue::from_identifier(CellIdentifier { scheme, value })
            }
            "{" => {
                self.pos += 1;
                let mut paths = Vec::new();
                if !self.eat("}") {
                    loop {
                        let member = self.cell()?;
                        if member.uncertainty.is_some() {
                            return Err(self.error("a reference set member carries no uncertainty"));
                        }
                        paths.push(
                            match member
                                .value
                                .selected()
                                .map_err(|e| self.error(&e.to_string()))?
                            {
                                CellSelected::Reference(reference) => CellPath {
                                    path: reference.path.clone(),
                                    keys: None,
                                },
                                CellSelected::Row(row) => CellPath {
                                    path: row.target.clone(),
                                    keys: Some(row.keys.clone()),
                                },
                                _ => {
                                    return Err(
                                        self.error("plain or keyed reference in a reference set")
                                    );
                                }
                            },
                        );
                        if self.eat("}") {
                            break;
                        }
                        self.expect(",")?;
                    }
                }
                CellValue::from_references(CellReferences { paths })
            }
            _ if self.tokens.get(self.pos).map(|t| t.kind) == Some(Kind::Quoted) => {
                CellValue::from_text(CellText {
                    value: self.word()?,
                })
            }
            _ if self.peek() == "-"
                || self.tokens.get(self.pos).map(|t| t.kind) == Some(Kind::Number) =>
            {
                let (magnitude, integral) = self.cell_number("number")?;
                if self.peek() == "{" {
                    // The unit literal is the expression language's own, flattened into
                    // one canonical product (ADR-0124).
                    let first = self.tokens[start].span.start as usize;
                    self.pos += 1;
                    self.until(&["}"])?;
                    self.expect("}")?;
                    let end = self.tokens[self.pos - 1].span.end as usize;
                    use crate::dsl::ExprKind;
                    let unit = crate::dsl::parse_expr(&self.text[first..end])
                        .ok()
                        .and_then(|expression| {
                            let literal = match expression.kind {
                                ExprKind::Neg(inner) => inner.kind,
                                other => other,
                            };
                            match literal {
                                ExprKind::Number(number) => number.unit,
                                _ => None,
                            }
                        })
                        .ok_or_else(|| {
                            self.pos = start;
                            self.error("number with a unit literal")
                        })?;
                    CellValue::from_quantity(CellQuantity {
                        magnitude,
                        unit: Some(unit_factors(&unit)),
                    })
                } else if integral {
                    let value = self.tokens[self.pos - 1]
                        .text
                        .parse::<i64>()
                        .ok()
                        .and_then(|v| {
                            if magnitude < 0. {
                                v.checked_neg()
                            } else {
                                Some(v)
                            }
                        })
                        .ok_or_else(|| {
                            self.pos = start;
                            self.error("64-bit integer")
                        })?;
                    CellValue::from_integer(CellInteger { value })
                } else {
                    CellValue::from_quantity(CellQuantity {
                        magnitude,
                        unit: None,
                    })
                }
            }
            _ => {
                let path = self.cell_path()?;
                if self.eat("[") {
                    // Plan 23 KR5: `target[keys]` names a keyed row or a table row by its
                    // key cells, each a scalar cell.
                    let mut keys = Vec::new();
                    loop {
                        let at = self.pos;
                        let key = self.cell()?;
                        keys.push(key_cell(&key).map_err(|_| {
                            self.pos = at;
                            self.error("key cell")
                        })?);
                        if self.eat("]") {
                            break;
                        }
                        self.expect(",")?;
                    }
                    CellValue::from_row(CellRow { target: path, keys })
                } else {
                    CellValue::from_reference(CellReference { path })
                }
            }
        };
        let uncertainty = if self.eat("±") {
            let kind = self.vocabulary::<ModelingUncertaintyKind>("standard, relative or bound")?;
            self.expect("(")?;
            let (magnitude, _) = self.cell_number("uncertainty magnitude")?;
            self.expect(")")?;
            Some(CellUncertainty { kind, magnitude })
        } else {
            None
        };
        Ok(Cell { value, uncertainty })
    }
    /// ADR-0125: how a data document stores the key, column or table value `name`:
    /// `[storage {unit}] [by scheme]`. The storage unit is a unit literal, the only
    /// statement of the unit a document's magnitudes are in; `by` names the identifier
    /// scheme by whose values a document names a referenced entity. `None` when neither is
    /// written.
    fn storage(&mut self, name: &str) -> Result<Option<ModelingColumnStorage>> {
        let unit = if self.eat("storage") {
            let start = self.pos;
            if self.peek() != "{" {
                return Err(self.error("storage unit literal"));
            }
            let first = self.tokens[self.pos].span.start as usize;
            self.pos += 1;
            self.until(&["}"])?;
            self.expect("}")?;
            let end = self.tokens[self.pos - 1].span.end as usize;
            let unit = crate::dsl::parse_expr(&format!("1{}", &self.text[first..end]))
                .ok()
                .and_then(|expression| match expression.kind {
                    crate::dsl::ExprKind::Number(number) => number.unit,
                    _ => None,
                })
                .ok_or_else(|| {
                    self.pos = start;
                    self.error("storage unit literal")
                })?;
            Some(unit_factors(&unit))
        } else {
            None
        };
        let scheme = if self.eat("by") {
            Some(self.segments()?)
        } else {
            None
        };
        Ok(
            (unit.is_some() || scheme.is_some()).then(|| ModelingColumnStorage {
                name: name.to_owned(),
                storage_unit: unit,
                scheme,
            }),
        )
    }
    /// A bracketed list of cells, `[` and `]` included; `[]` is empty.
    fn cells(&mut self) -> Result<Vec<Cell>> {
        self.expect("[")?;
        let mut cells = Vec::new();
        if self.eat("]") {
            return Ok(cells);
        }
        loop {
            cells.push(self.cell()?);
            if self.eat("]") {
                return Ok(cells);
            }
            self.expect(",")?;
        }
    }
    /// Whether an inclusive integer range `lo..hi` starts here.
    fn at_range(&self) -> bool {
        let number = |ahead: usize| {
            self.tokens
                .get(self.pos + ahead)
                .is_some_and(|t| t.kind == Kind::Number)
        };
        if self.peek() == "-" {
            number(1) && self.peek_at(2) == ".."
        } else {
            number(0) && self.peek_at(1) == ".."
        }
    }
    /// An inclusive integer range `lo..hi` with `lo <= hi` (Plan 23 KR5).
    fn integer_range(&mut self) -> Result<ModelingIntegerRange> {
        let at = self.pos;
        let lower = i64::from(self.type_integer()?);
        self.expect("..")?;
        let upper = i64::from(self.type_integer()?);
        if lower > upper {
            self.pos = at;
            return Err(self.error("an integer range whose lower bound is at most its upper"));
        }
        Ok(ModelingIntegerRange { lower, upper })
    }
    /// ADR-0123 Outcome 3: `complete_over(key [in set | in lo..hi], ...)`, one entry per
    /// named key; a key named alone is open for its datasets to claim.
    fn completeness(&mut self) -> Result<Vec<ModelingCompleteness>> {
        self.expect("complete_over")?;
        self.expect("(")?;
        let mut entries = Vec::new();
        loop {
            let key = self.word()?;
            let (set, range) = if self.eat("in") {
                if self.at_range() {
                    (None, Some(self.integer_range()?))
                } else {
                    (Some(self.segments()?), None)
                }
            } else {
                (None, None)
            };
            entries.push(ModelingCompleteness { key, set, range });
            if self.eat(")") {
                return Ok(entries);
            }
            self.expect(",")?;
        }
    }
    /// ADR-0123 Outcome 4: `: Type in lower..upper` after an envelope's axis name, the axis
    /// quantity type and the two typed columns or attributes bounding it.
    fn envelope_bounds(&mut self) -> Result<(Vec<TypeNode>, String, String)> {
        self.expect(":")?;
        let r#type = self.type_expr()?;
        self.expect("in")?;
        let lower = self.word()?;
        self.expect("..")?;
        let upper = self.word()?;
        Ok((r#type, lower, upper))
    }
    /// ADR-0123 Outcome 4: `guards(carrier.axis: argument, carrier.axis: [start, end], …)`,
    /// the envelopes of row or entity arguments a function guards: at one argument, or
    /// over the integration interval between two.
    fn guards(&mut self) -> Result<Vec<ModelingEnvelopeGuard>> {
        use pse_model::generated::enums::ModelingEnvelopeExtent as Extent;
        self.expect("(")?;
        let mut guards = Vec::new();
        loop {
            let carrier = self.word()?;
            self.expect(".")?;
            let envelope = self.word()?;
            self.expect(":")?;
            let (extent, arguments) = if self.eat("[") {
                let start = self.word()?;
                self.expect(",")?;
                let end = self.word()?;
                self.expect("]")?;
                (Extent::Interval, vec![start, end])
            } else {
                (Extent::Point, vec![self.word()?])
            };
            guards.push(ModelingEnvelopeGuard {
                carrier,
                envelope,
                extent,
                arguments,
            });
            if self.eat(")") {
                return Ok(guards);
            }
            self.expect(",")?;
        }
    }
    fn fixture_literal<T: std::str::FromStr>(&mut self) -> Result<T> {
        self.expect("(")?;
        let value = self.until(&[")"])?;
        self.expect(")")?;
        value
            .parse()
            .map_err(|_| self.error("fixture policy literal"))
    }
    fn fixture_option<T: std::str::FromStr>(&mut self, name: &str) -> Result<T> {
        self.expect(name)?;
        self.fixture_literal()
    }
    /// Resolve the authored alias into the existing numeric policy slot; an
    /// explicit number continues to be an independent per-slot override.
    fn fixture_accuracy(&mut self, name: &str) -> Result<f64> {
        self.expect("(")?;
        let value = self.until(&[")"])?;
        let accuracy = if value == "global" {
            pse_model::numerics::DEFAULT_ENGINEERING_ACCURACY
        } else {
            value
                .parse()
                .map_err(|_| self.error(&format!("{name} tolerance number or global")))?
        };
        self.expect(")")?;
        Ok(accuracy)
    }
    /// A registry vocabulary word; a word outside it is refused at its own position.
    fn vocabulary<T: std::str::FromStr>(&mut self, expected: &str) -> Result<T> {
        let at = self.pos;
        let word = self.word()?;
        word.parse().map_err(|_| {
            self.pos = at;
            self.error(expected)
        })
    }
    /// `(<literal>)` holding a number or, with `expected` "nonnegative integer", a count
    /// in the registry's nonnegative domain; a malformed literal is refused where it is
    /// written. The policy's consumer owns the admissible values.
    fn policy_literal<T: std::str::FromStr>(
        &mut self,
        expected: &str,
        admitted: impl Fn(&T) -> bool,
    ) -> Result<T> {
        self.expect("(")?;
        let at = self.pos;
        let value = self.until(&[")"])?.parse::<T>().ok().filter(admitted);
        let value = value.ok_or_else(|| {
            self.pos = at;
            self.error(expected)
        })?;
        self.expect(")")?;
        Ok(value)
    }
    fn policy_number(&mut self) -> Result<f64> {
        self.policy_literal("number", |_| true)
    }
    fn policy_count(&mut self) -> Result<i64> {
        self.policy_literal("nonnegative integer", |v: &i64| *v >= 0)
    }
    /// The named options of one policy setting, each at most once and at least one, in any
    /// order: `<name>(<value>) ...`; `read` takes each value by its index in `names`.
    fn policy_options<const N: usize>(
        &mut self,
        names: [&str; N],
        mut read: impl FnMut(&mut Self, usize) -> Result<()>,
    ) -> Result<()> {
        let mut seen = [false; N];
        while let Some(index) = names.iter().position(|n| *n == self.peek()) {
            if seen[index] {
                return Err(self.error(&format!("one {} option", names[index])));
            }
            seen[index] = true;
            self.pos += 1;
            read(self, index)?;
        }
        if !seen.contains(&true) {
            return Err(self.error(&names.join(", ")));
        }
        Ok(())
    }
    /// ADR-0119: `policy { backend <b>; presolve <auto|off>; derivatives [step(x)]
    /// [tolerance(x)] [cells(n)]; limits [items(n)] [body_occurrences(n)] [body_slots(n)]
    /// [foreign_bytes(n)]; }` states the fixture's execution policy over the run's. Every
    /// setting is optional and at most once; an unknown or repeated setting is refused where
    /// it is written.
    fn fixture_policy(
        &mut self,
    ) -> Result<AuthoredModelingDeclarationsFieldValueScopeFixturePolicy> {
        use pse_model::generated::enums::{NativeBackend, PresolvePolicyKind};
        self.expect("{")?;
        let mut policy = AuthoredModelingDeclarationsFieldValueScopeFixturePolicy {
            backend: None,
            presolve: None,
            native_options: Vec::new(),
            derivative_step: None,
            derivative_tolerance: None,
            derivative_cells: None,
            items: None,
            body_occurrences: None,
            body_slots: None,
            foreign_bytes: None,
        };
        let mut seen = BTreeSet::new();
        loop {
            if self.peek() == "}" {
                if seen.is_empty() {
                    return Err(self.error("a fixture policy setting"));
                }
                self.pos += 1;
                return Ok(policy);
            }
            let setting = self.peek().to_owned();
            if !matches!(
                setting.as_str(),
                "backend" | "presolve" | "derivatives" | "limits" | "options"
            ) {
                return Err(self.error("backend, presolve, derivatives, limits or options"));
            }
            if !seen.insert(setting.clone()) {
                return Err(self.error(&format!("one {setting} setting")));
            }
            self.pos += 1;
            match setting.as_str() {
                "options" => {
                    self.expect("{")?;
                    let mut names = BTreeSet::new();
                    while !self.eat("}") {
                        let name = self.word()?;
                        if !names.insert(name.clone()) {
                            return Err(self.error("one value per native option"));
                        }
                        self.expect("=")?;
                        let value = self.cell()?;
                        self.expect(";")?;
                        policy.native_options.push(AuthoredModelingDeclarationsFieldValueScopeFixturePolicyNativeOptionsItem {name, value});
                    }
                    if names.is_empty() {
                        return Err(self.error("a native option"));
                    }
                }
                "backend" => {
                    policy.backend = Some(self.vocabulary::<NativeBackend>("native backend")?);
                }
                "presolve" => {
                    let at = self.pos;
                    let kind = self.vocabulary::<PresolvePolicyKind>("auto or off")?;
                    // Explicit presolve needs complete library controls, not a fixture word.
                    if kind == PresolvePolicyKind::Explicit {
                        self.pos = at;
                        return Err(self.error("auto or off"));
                    }
                    policy.presolve = Some(kind);
                }
                "derivatives" => {
                    self.policy_options(["step", "tolerance", "cells"], |cursor, index| {
                        match index {
                            0 => policy.derivative_step = Some(cursor.policy_number()?),
                            1 => policy.derivative_tolerance = Some(cursor.policy_number()?),
                            _ => policy.derivative_cells = Some(cursor.policy_count()?),
                        }
                        Ok(())
                    })?;
                }
                _ => {
                    self.policy_options(
                        ["items", "body_occurrences", "body_slots", "foreign_bytes"],
                        |cursor, index| {
                            let value = Some(cursor.policy_count()?);
                            match index {
                                0 => policy.items = value,
                                1 => policy.body_occurrences = value,
                                2 => policy.body_slots = value,
                                _ => policy.foreign_bytes = value,
                            }
                            Ok(())
                        },
                    )?;
                }
            }
            self.expect(";")?;
        }
    }
    /// A parenthesized, comma-separated list of expressions, kept as source text.
    fn expressions(&mut self) -> Result<Vec<String>> {
        self.expect("(")?;
        let mut values = Vec::new();
        while !self.eat(")") {
            values.push(self.until(&[",", ")"])?);
            if self.eat(")") {
                break;
            }
            self.expect(",")?;
        }
        Ok(values)
    }
    fn literal_list<T: std::str::FromStr>(&mut self, name: &str) -> Result<Vec<T>> {
        self.expect(name)?;
        self.expect("(")?;
        let mut values = Vec::new();
        if !self.eat(")") {
            loop {
                let value = self.until(&[",", ")"])?;
                values.push(
                    value
                        .parse()
                        .map_err(|_| self.error("numeric scheme coefficient"))?,
                );
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }
        Ok(values)
    }
    fn names(&mut self, open: &str, close: &str) -> Result<Vec<String>> {
        if !self.eat(open) {
            return Ok(Vec::new());
        }
        let mut names = Vec::new();
        if !self.eat(close) {
            loop {
                names.push(self.path()?);
                if self.eat(close) {
                    break;
                }
                self.expect(",")?;
            }
        }
        Ok(names)
    }
    fn parameters(&mut self) -> Result<Vec<Parameter>> {
        if !self.eat("(") {
            return Ok(Vec::new());
        }
        let mut result = Vec::new();
        if !self.eat(")") {
            loop {
                let name = self.word()?;
                self.expect(":")?;
                let ty = self.type_expr()?;
                let default = if self.eat("=") {
                    Some(self.until(&[",", ")"])?)
                } else {
                    None
                };
                result.push((name, ty, default));
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }
        Ok(result)
    }
    fn indices(&mut self) -> Result<Vec<(String, String)>> {
        if !self.eat("[") {
            return Ok(Vec::new());
        }
        let mut result = Vec::new();
        loop {
            let name = self.word()?;
            self.expect("in")?;
            let domain = self.until(&[",", "]"])?;
            result.push((name, domain));
            if self.eat("]") {
                break;
            }
            self.expect(",")?;
        }
        Ok(result)
    }
    /// The members of an objective annotation after its opening parenthesis, through the
    /// closing one: the sense, then each named member at most once, in any order.
    fn objective_members(
        &mut self,
    ) -> Result<AuthoredModelingDeclarationsFieldValueAnnotationObjective> {
        let sense = self
            .word()?
            .parse()
            .map_err(|_| self.error("objective sense minimize or maximize"))?;
        let mut objective = AuthoredModelingDeclarationsFieldValueAnnotationObjective {
            sense,
            priority: None,
            weight: None,
            normalization: None,
            absolute_tolerance: None,
            relative_tolerance: None,
        };
        while self.eat(",") {
            let member = self.word()?;
            self.expect("=")?;
            let value = self.until(&[",", ")"])?;
            let slot = match member.as_str() {
                "priority" => {
                    if objective.priority.is_some() {
                        return Err(self.error("one objective priority"));
                    }
                    objective.priority = Some(
                        value
                            .replace(' ', "")
                            .parse()
                            .map_err(|_| self.error("integer objective priority"))?,
                    );
                    continue;
                }
                "weight" => &mut objective.weight,
                "normalization" => &mut objective.normalization,
                "absolute_tolerance" => &mut objective.absolute_tolerance,
                "relative_tolerance" => &mut objective.relative_tolerance,
                _ => {
                    return Err(self.error(
                        "objective member priority, weight, normalization, absolute_tolerance or relative_tolerance",
                    ));
                }
            };
            if slot.replace(value).is_some() {
                return Err(self.error("each objective member at most once"));
            }
        }
        self.expect(")")?;
        Ok(objective)
    }
    /// Typed Plan 27 goal members. Subject and observation are required; the remaining
    /// members use their shared policy defaults unless explicitly stated.
    fn accuracy_goal_members(
        &mut self,
    ) -> Result<AuthoredModelingDeclarationsFieldValueAnnotationAccuracyGoal> {
        use pse_model::generated::enums::{
            AccuracyGoalSubject as Subject, AccuracyGoalUse as Use,
            AccuracyObservation as Observation, NumericalAccuracyClass as Class,
        };
        let subject = self.vocabulary::<Subject>("accuracy goal subject")?;
        self.expect(",")?;
        let observation = self.vocabulary::<Observation>("accuracy observation")?;
        let mut goal = AuthoredModelingDeclarationsFieldValueAnnotationAccuracyGoal {
            subject,
            observation,
            time: None,
            resolution: None,
            criterion_lower: None,
            criterion_upper: None,
            required_class: Class::Estimated,
            use_policy: Use::Assess,
            refine: true,
        };
        let mut seen = BTreeSet::new();
        while self.eat(",") {
            let member = self.word()?;
            if !seen.insert(member.clone()) {
                return Err(self.error("each accuracy goal member at most once"));
            }
            self.expect("=")?;
            let value = self.until(&[",", ")"])?;
            match member.as_str() {
                "time" => {
                    goal.time = Some(value);
                }
                "resolution" => {
                    goal.resolution = Some(value);
                }
                "criterion_lower" => {
                    goal.criterion_lower = Some(value);
                }
                "criterion_upper" => {
                    goal.criterion_upper = Some(value);
                }
                "required_class" => {
                    goal.required_class = value
                        .parse::<Class>()
                        .map_err(|_| self.error("accuracy class estimated or certified"))?;
                    if goal.required_class == Class::Unresolved {
                        return Err(self.error("accuracy class estimated or certified"));
                    }
                }
                "use_policy" => {
                    goal.use_policy = value
                        .parse::<Use>()
                        .map_err(|_| self.error("accuracy goal use assess or require_satisfied"))?;
                }
                "refine" => {
                    if value != "true" && value != "false" {
                        return Err(self.error("accuracy goal refine true or false"));
                    }
                    goal.refine = value
                        .parse()
                        .map_err(|_| self.error("accuracy goal refine true or false"))?;
                }
                _ => return Err(self.error("accuracy goal member")),
            }
        }
        self.expect(")")?;
        Ok(goal)
    }
    /// A physical scale has one tagged interpretation and one authored expression.
    fn engineering_scale_members(
        &mut self,
    ) -> Result<AuthoredModelingDeclarationsFieldValueAnnotationEngineeringScale> {
        use pse_model::generated::enums::EngineeringScaleKind as Kind;
        let mut kind = None;
        let mut value = None;
        while !self.eat(")") {
            let member = self.word()?;
            self.expect("=")?;
            let raw = self.until(&[",", ")"])?;
            match member.as_str() {
                "kind" if kind.is_none() => {
                    kind = Some(
                        raw.parse::<Kind>()
                            .map_err(|_| self.error("engineering scale kind"))?,
                    );
                }
                "value" if value.is_none() => value = Some(raw),
                "kind" | "value" => {
                    return Err(self.error("each engineering scale member at most once"));
                }
                _ => return Err(self.error("engineering scale member kind or value")),
            }
            if self.peek() != ")" {
                self.expect(",")?;
            }
        }
        Ok(
            AuthoredModelingDeclarationsFieldValueAnnotationEngineeringScale {
                kind: kind.ok_or_else(|| self.error("engineering scale kind"))?,
                value: value.ok_or_else(|| self.error("engineering scale value"))?,
            },
        )
    }
    /// `>= 0` closing one member of a complementarity pair; the zero may carry a unit.
    fn complement_zero(&mut self) -> Result<()> {
        self.expect(">=")?;
        let zero = self.until(&[",", ")"])?;
        match crate::dsl::parse_expr(&zero).map(|e| e.kind) {
            Ok(crate::dsl::ExprKind::Number(n)) if n.value == 0.0 => Ok(()),
            _ => Err(self.error("complementarity member >= 0")),
        }
    }
    /// A connection maximum: a nonnegative count, or `many` for none.
    fn connection_maximum(&mut self) -> Result<Option<i64>> {
        if self.eat("many") {
            return Ok(None);
        }
        let value = self
            .tokens
            .get(self.pos)
            .filter(|t| t.kind == Kind::Number)
            .and_then(|t| t.text.parse::<u32>().ok());
        let value = value.ok_or_else(|| self.error("nonnegative connection maximum or many"))?;
        self.pos += 1;
        Ok(Some(i64::from(value)))
    }
    fn block(&mut self, parent: Option<DeclarationId>, depth: u32, braced: bool) -> Result<()> {
        if depth > self.budget.max_depth {
            return Err(AuthoringError::Budget {
                limit: "depth",
                allowed: u64::from(self.budget.max_depth),
                needed: u64::from(depth),
            });
        }
        let mut ordinal = 0;
        while !self.peek().is_empty() && !(braced && self.peek() == "}") {
            self.item(parent, depth, ordinal)?;
            ordinal += 1;
        }
        if braced {
            self.expect("}")?;
            self.eat(";");
        }
        Ok(())
    }
    fn item(&mut self, parent: Option<DeclarationId>, depth: u32, ordinal: i64) -> Result<()> {
        let start = self.at();
        let variables = self.type_variables.len();
        let explicit = if self.eat("@") {
            self.expect("id")?;
            self.expect("(")?;
            let id = SemanticId::parse_hex(&self.word()?).map_err(|_| self.error("semantic ID"))?;
            self.expect(")")?;
            Some(DeclarationId::from(id))
        } else {
            None
        };
        let is_override = self.eat("override");
        let mut keyword = self.word()?;
        // ADR-0123 Outcome 2: in an entity kind, `name = cell;` binds an inherited attribute.
        let bound = (self.peek() == "=" && self.in_kind(parent))
            .then(|| std::mem::replace(&mut keyword, BIND.into()));
        if keyword == "entity" && self.eat("kind") {
            keyword = "entity_kind".into();
        }
        if keyword == "identifier" && self.eat("scheme") {
            keyword = "identifier_scheme".into();
        }
        if keyword == "coordinate" && self.eat("map") {
            keyword = "coordinate_map".into();
        }
        if keyword == "reference" && self.eat("translation") {
            keyword = "reference_translation".into();
        }
        if keyword == "material" {
            self.expect("port")?;
            keyword = "state_port".into();
        }
        let named_connection = keyword == "connect" && self.peek_at(1) == ":";
        let anonymous = !named_connection
            && matches!(
                keyword.as_str(),
                "when" | "require" | "connect" | "contribute" | "expect" | "annotation"
            );
        let entity_kind = if keyword == "entity" {
            Some(self.path()?)
        } else {
            None
        };
        let name = if named_connection {
            let name = self.word()?;
            self.expect(":")?;
            name
        } else if anonymous {
            format!("{keyword}#{ordinal}")
        } else if let Some(name) = bound {
            name
        } else {
            self.path()?
        };
        // A key, an attribute, a derived attribute and a binding are one declaration kind.
        let role = match keyword.as_str() {
            "key" | "derived" | BIND => "attribute",
            other => other,
        };
        let id = match explicit {
            Some(id) => id,
            None if self.policy == IdentityPolicy::Explicit => {
                return Err(AuthoringError::MissingId {
                    at: SourceSpan::new(self.document, start, self.at()),
                    kind: role.to_owned(),
                    name,
                });
            }
            None => DeclarationId::from(pse_ids::named_id(
                parent.map_or(self.document, DeclarationId::as_id),
                &format!("{role}:{name}"),
            )),
        };
        if !self.ids.insert(id) {
            return Err(self.error("unique declaration identity"));
        }
        // Anonymous declarations compose by identity, not by their position in
        // an unrelated scope. Ordinals only seed IDs in the named test policy.
        let name = if anonymous {
            format!("{keyword}#{id}")
        } else {
            name
        };
        let mut child_block = false;
        let value = match keyword.as_str() {
            "evolve" => {
                self.expect("on")?;
                let target = self.path()?;
                self.expect("using")?;
                let axis = self.path()?;
                self.expect("bind")?;
                let argument = self.path()?;
                self.expect(";")?;
                Value::from_temporal(AuthoredModelingDeclarationsFieldValueTemporal {
                    target,
                    axis,
                    argument,
                })
            }
            "relax" => {
                self.expect("on")?;
                let target = self.path()?;
                self.expect("nominal")?;
                let nominal = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_relaxation(AuthoredModelingDeclarationsFieldValueRelaxation {
                    target,
                    nominal,
                })
            }
            "continue" => {
                self.expect("on")?;
                let target = self.path()?;
                self.expect("from")?;
                let start = self.until(&["to"])?;
                self.expect("to")?;
                let end = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_continuation(AuthoredModelingDeclarationsFieldValueContinuation {
                    target,
                    start,
                    end,
                })
            }
            "domain" => {
                self.expect(":")?;
                let r#type = self.type_expr()?;
                self.expect("from")?;
                let lower = self.until(&["to"])?;
                self.expect("to")?;
                let upper = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_continuous(AuthoredModelingDeclarationsFieldValueContinuous {
                    r#type,
                    lower,
                    upper,
                })
            }
            "difference" => {
                let order = self.fixture_option("order")?;
                let offsets = self.literal_list("offsets")?;
                let weights = self.literal_list("weights")?;
                let quadrature = self.literal_list("quadrature")?;
                self.expect(";")?;
                Value::from_difference_scheme(
                    AuthoredModelingDeclarationsFieldValueDifferenceScheme {
                        order,
                        offsets,
                        weights,
                        quadrature,
                    },
                )
            }
            "collocation" => {
                let alpha = self.fixture_option("alpha")?;
                let beta = self.fixture_option("beta")?;
                let right_endpoint = self.fixture_option("right")?;
                self.expect(";")?;
                Value::from_collocation_scheme(
                    AuthoredModelingDeclarationsFieldValueCollocationScheme {
                        alpha,
                        beta,
                        right_endpoint,
                    },
                )
            }
            "discretize" => {
                self.expect("on")?;
                let target = self.path()?;
                self.expect("using")?;
                let scheme = self.path()?;
                self.expect("(")?;
                self.expect("elements")?;
                self.expect("=")?;
                let elements = self.until(&[","])?;
                self.expect(",")?;
                self.expect("order")?;
                self.expect("=")?;
                let order = self.until(&[")"])?;
                self.expect(")")?;
                self.expect(";")?;
                Value::from_discretization(AuthoredModelingDeclarationsFieldValueDiscretization {
                    target,
                    scheme,
                    elements,
                    order,
                })
            }
            "realize" => {
                use pse_model::generated::enums::ModelingRealizationPolicy as Policy;
                self.expect("on")?;
                let target = self.path()?;
                self.expect("using")?;
                let spelling = self.word()?;
                let mut argument = None;
                let mut function = None;
                let policy = match spelling.as_str() {
                    // ADR-0104: `bigm(M)` asserts an authored M; `bigm(derived[, margin])`
                    // derives it from the case box.
                    "bigm" => {
                        self.expect("(")?;
                        let policy = if self.eat("derived") {
                            if self.eat(",") {
                                argument = Some(self.until(&[")"])?);
                            }
                            Policy::DerivedBigM
                        } else {
                            argument = Some(self.until(&[")"])?);
                            Policy::BigM
                        };
                        self.expect(")")?;
                        policy
                    }
                    "hull" => {
                        if self.eat("(") {
                            argument = Some(self.until(&[")"])?);
                            self.expect(")")?;
                        }
                        Policy::Hull
                    }
                    // ADR-0104 §5: `smooth(f, width)` equates the authored smoothing
                    // function f(first, second, width) to zero; `penalty(l1)` selects the
                    // l1 exact-penalty route.
                    "smooth" => {
                        self.expect("(")?;
                        function = Some(self.path()?);
                        self.expect(",")?;
                        argument = Some(self.until(&[")"])?);
                        self.expect(")")?;
                        Policy::Smooth
                    }
                    "penalty" => {
                        self.expect("(")?;
                        self.expect("l1")?;
                        self.expect(")")?;
                        Policy::PenaltyL1
                    }
                    "big_m" | "derived_big_m" => {
                        return Err(self.error("bigm(M) or bigm(derived)"));
                    }
                    "penalty_l1" => return Err(self.error("penalty(l1)")),
                    other => other
                        .parse()
                        .map_err(|_| self.error("realization policy"))?,
                };
                let accelerator = if policy == Policy::Accelerated {
                    self.expect("(")?;
                    let reference = self.word()?;
                    self.expect(")")?;
                    Some(reference)
                } else {
                    None
                };
                self.expect(";")?;
                Value::from_realization(AuthoredModelingDeclarationsFieldValueRealization {
                    target,
                    policy,
                    accelerator,
                    argument,
                    function,
                })
            }
            "sos1" | "sos2" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueOrderedSetIndicesItem { name, domain }
                    })
                    .collect();
                self.expect(":")?;
                let member = self.until(&["weight"])?;
                self.expect("weight")?;
                let weight = self.until(&[";"])?;
                self.expect(";")?;
                let set = AuthoredModelingDeclarationsFieldValueOrderedSet {
                    indices,
                    member,
                    weight,
                };
                if keyword == "sos1" {
                    Value::from_sos1(set)
                } else {
                    Value::from_sos2(set)
                }
            }
            "atmost" | "atleast" | "exactly" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueCardinalityIndicesItem {
                            name,
                            domain,
                        }
                    })
                    .collect();
                self.expect(":")?;
                let count = self.until(&["of"])?;
                self.expect("of")?;
                let member = self.until(&[";"])?;
                self.expect(";")?;
                let value = AuthoredModelingDeclarationsFieldValueCardinality {
                    indices,
                    count,
                    member,
                };
                match keyword.as_str() {
                    "atmost" => Value::from_atmost(value),
                    "atleast" => Value::from_atleast(value),
                    _ => Value::from_exactly(value),
                }
            }
            "piecewise" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValuePiecewiseIndicesItem { name, domain }
                    })
                    .collect();
                self.expect(":")?;
                let output = self.until(&["=="])?;
                self.expect("==")?;
                let input = self.until(&["at"])?;
                self.expect("at")?;
                self.expect("(")?;
                let abscissa = self.until(&[","])?;
                self.expect(",")?;
                let ordinate = self.until(&[")"])?;
                self.expect(")")?;
                self.expect(";")?;
                Value::from_piecewise(AuthoredModelingDeclarationsFieldValuePiecewise {
                    indices,
                    output,
                    input,
                    abscissa,
                    ordinate,
                })
            }
            "complements" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueComplementarityIndicesItem {
                            name,
                            domain,
                        }
                    })
                    .collect();
                self.expect(":")?;
                self.expect("(")?;
                let first = self.until(&[">="])?;
                self.complement_zero()?;
                self.expect(",")?;
                let second = self.until(&[">="])?;
                self.complement_zero()?;
                self.expect(")")?;
                self.expect(";")?;
                Value::from_complementarity(AuthoredModelingDeclarationsFieldValueComplementarity {
                    indices,
                    first,
                    second,
                })
            }
            "logic" => {
                let indices =
                    self.indices()?
                        .into_iter()
                        .map(|(name, domain)| {
                            AuthoredModelingDeclarationsFieldValueLogicIndicesItem { name, domain }
                        })
                        .collect();
                self.expect(":")?;
                let proposition = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_logic(AuthoredModelingDeclarationsFieldValueLogic {
                    indices,
                    proposition,
                })
            }
            "package" | "entity_kind" | "interface" | "def" | "case" | "test" | "stage"
            | "implicit" | "regime" | "disjunction" | "alternative" => {
                let type_parameters = self.names("<", ">")?;
                // A scope's type parameters are in scope for its parameters and members.
                self.type_variables.extend(type_parameters.iter().cloned());
                let parameters = self
                    .parameters()?
                    .into_iter()
                    .map(|(name, r#type, default_value)| {
                        AuthoredModelingDeclarationsFieldValueScopeParametersItem {
                            name,
                            r#type,
                            default_value,
                        }
                    })
                    .collect();
                let mut bases = Vec::new();
                if self.eat(":") || self.eat("extends") {
                    loop {
                        bases.push(self.path()?);
                        if !self.eat(",") {
                            break;
                        }
                    }
                }
                let mut branch = None;
                let mut operational = None;
                let selection = if self.eat("select") {
                    if self.eat("branch") {
                        self.expect("(")?;
                        branch = Some(self.until(&[")"])?);
                        self.expect(")")?;
                        None
                    } else if self.eat("operational") {
                        self.expect("(")?;
                        let mut anchors = Vec::new();
                        loop {
                            let target = self.word()?;
                            self.expect("=")?;
                            let expression = self.until(&[",", ")"])?;
                            anchors.push(
                                AuthoredModelingDeclarationsFieldValueScopeOperationalAnchorsItem {
                                    target,
                                    expression,
                                },
                            );
                            if !self.eat(",") {
                                break;
                            }
                        }
                        self.expect(")")?;
                        self.expect("settings")?;
                        self.expect("(")?;
                        let settings = self.word()?;
                        self.expect(")")?;
                        let neighborhood = if self.eat("neighborhood") {
                            self.expect("(")?;
                            let predicate = self.until(&[")"])?;
                            self.expect(")")?;
                            Some(predicate)
                        } else {
                            None
                        };
                        operational =
                            Some(AuthoredModelingDeclarationsFieldValueScopeOperational {
                                anchors,
                                settings,
                                neighborhood,
                            });
                        None
                    } else {
                        self.expect("minimum")?;
                        self.expect("(")?;
                        let criterion = self.until(&[","])?;
                        self.expect(",")?;
                        let tolerance = self.until(&[")"])?;
                        self.expect(")")?;
                        Some(AuthoredModelingDeclarationsFieldValueScopeSelection {
                            criterion,
                            tolerance,
                        })
                    }
                } else {
                    None
                };
                let eligibility = if self.eat("eligible") {
                    self.expect("(")?;
                    let value = self.until(&[")"])?;
                    self.expect(")")?;
                    Some(value)
                } else {
                    None
                };
                if (selection.is_some() || branch.is_some() || operational.is_some())
                    && keyword != "implicit"
                    || eligibility.is_some() && keyword != "regime"
                {
                    return Err(self
                        .error("selection belongs to implicit blocks and eligibility to regimes"));
                }
                // ADR-0123 Outcome 5: an entity kind's facets, then a test's oracle source.
                let mut facets = Vec::new();
                while keyword == "entity_kind"
                    && let Ok(facet) = self
                        .peek()
                        .parse::<pse_model::generated::enums::ModelingKindFacet>()
                {
                    self.pos += 1;
                    if facets.contains(&facet) {
                        return Err(self.error("distinct kind facets"));
                    }
                    facets.push(facet);
                }
                let oracle = if self.eat("oracle") {
                    Some(self.segments()?)
                } else {
                    None
                };
                let fixture = if self.eat("fixture") {
                    self.expect("{")?;
                    self.expect("dof")?;
                    let degrees_of_freedom = self
                        .until(&[";"])?
                        .parse()
                        .map_err(|_| self.error("signed integer degrees of freedom"))?;
                    self.expect(";")?;
                    let mut specifications = Vec::new();
                    let mut route = None;
                    let mut procedure = None;
                    let mut endpoint = None;
                    let mut intent = None;
                    let mut policy = None;
                    let mut stages = Vec::new();
                    let mut integration = None;
                    let mut schedules = Vec::new();
                    let mut shooting = None;
                    let mut modes =
                        Vec::<AuthoredModelingDeclarationsFieldValueScopeFixtureModesItem>::new();
                    let mut initialization = None;
                    let mut expected_failure = None;
                    let mut diagnostics = Vec::new();
                    while !self.eat("}") {
                        // ADR-0119 Outcome 3: `mode <name> [facts(<fact> = <bool>, ...)];`
                        // declares a same-layout mode; the events after it belong to it.
                        if self.eat("mode") {
                            let name = self.word()?;
                            let mut facts = Vec::new();
                            if self.eat("facts") {
                                self.expect("(")?;
                                while !self.eat(")") {
                                    // `<namespace>.<name> = <bool>`: a fact in its registry
                                    // namespace (ADR-0123 Outcome 1).
                                    let namespace = self.vocabulary::<pse_model::generated::enums::ModelingFactNamespace>("fact namespace analysis, objective or stage")?;
                                    self.expect(".")?;
                                    let name = self.segments()?.join(".");
                                    self.expect("=")?;
                                    let value = match self.word()?.as_str() {
                                        "true" => true,
                                        "false" => false,
                                        _ => return Err(self.error("true or false")),
                                    };
                                    facts.push(
                                        AuthoredModelingDeclarationsFieldValueScopeFixtureModesItemFactsItem {
                                            namespace,
                                            name,
                                            value,
                                        },
                                    );
                                    if self.eat(")") {
                                        break;
                                    }
                                    self.expect(",")?;
                                }
                            }
                            self.expect(";")?;
                            modes.push(
                                AuthoredModelingDeclarationsFieldValueScopeFixtureModesItem {
                                    name,
                                    facts,
                                    events: Vec::new(),
                                },
                            );
                            continue;
                        }
                        // `event <guard> direction(<d>) tolerance(<x>) [reset(<state> = <member>, ...)]
                        // (next(<mode>) | terminal);`
                        if self.eat("event") {
                            let guard = self.until(&["direction"])?;
                            self.expect("direction")?;
                            self.expect("(")?;
                            let direction = self
                                .word()?
                                .parse()
                                .map_err(|_| self.error("either, rising or falling"))?;
                            self.expect(")")?;
                            self.expect("tolerance")?;
                            self.expect("(")?;
                            let tolerance = self.until(&[")"])?;
                            self.expect(")")?;
                            let mut reset = Vec::new();
                            if self.eat("reset") {
                                self.expect("(")?;
                                while !self.eat(")") {
                                    let target = self.until(&["="])?;
                                    self.expect("=")?;
                                    let expression = self.until(&[",", ")"])?;
                                    reset.push(
                                        AuthoredModelingDeclarationsFieldValueScopeFixtureModesItemEventsItemResetItem {
                                            target,
                                            expression,
                                        },
                                    );
                                    if self.eat(")") {
                                        break;
                                    }
                                    self.expect(",")?;
                                }
                            }
                            let next = if self.eat("terminal") {
                                None
                            } else {
                                self.expect("next")?;
                                self.expect("(")?;
                                let next = self.word()?;
                                self.expect(")")?;
                                Some(next)
                            };
                            self.expect(";")?;
                            let event =
                                AuthoredModelingDeclarationsFieldValueScopeFixtureModesItemEventsItem {
                                    guard,
                                    direction,
                                    tolerance,
                                    reset,
                                    next,
                                };
                            modes
                                .last_mut()
                                .ok_or_else(|| self.error("a mode clause before its events"))?
                                .events
                                .push(event);
                            continue;
                        }
                        // ADR-0119 Outcome 2: `schedule u at(t1, ...) values(v0, v1, ...);`
                        // holds `u` piecewise constant with one value per interval; with
                        // `free [lower(<x>)] [upper(<x>)]` its values are shooting controls.
                        if self.eat("schedule") {
                            let target = self.until(&["at"])?;
                            self.expect("at")?;
                            let times = self.expressions()?;
                            self.expect("values")?;
                            let values = self.expressions()?;
                            let free = self.eat("free");
                            let mut bound = |name: &str| -> Result<Option<String>> {
                                if !free || !self.eat(name) {
                                    return Ok(None);
                                }
                                self.expect("(")?;
                                let value = self.until(&[")"])?;
                                self.expect(")")?;
                                Ok(Some(value))
                            };
                            let lower = bound("lower")?;
                            let upper = bound("upper")?;
                            self.expect(";")?;
                            schedules.push(
                                AuthoredModelingDeclarationsFieldValueScopeFixtureIntegrationSchedulesItem {
                                    target,
                                    times,
                                    values,
                                    free,
                                    lower,
                                    upper,
                                },
                            );
                            continue;
                        }
                        // ADR-0110 Outcome 5: `shoot single;` or `shoot multiple nodes(...);`.
                        if self.eat("shoot") {
                            if shooting.is_some() {
                                return Err(self.error("one shooting clause"));
                            }
                            let method = self
                                .word()?
                                .parse()
                                .map_err(|_| self.error("single or multiple"))?;
                            let nodes = if self.eat("nodes") {
                                self.expressions()?
                            } else {
                                Vec::new()
                            };
                            self.expect(";")?;
                            shooting =
                                Some(AuthoredModelingDeclarationsFieldValueScopeFixtureShooting {
                                    method,
                                    nodes,
                                });
                            continue;
                        }
                        if self.eat("route") {
                            if route.is_some() {
                                return Err(self.error("one fixture temporal route"));
                            }
                            route = Some(
                                self.word()?
                                    .parse()
                                    .map_err(|_| self.error("fixture temporal route"))?,
                            );
                            self.expect(";")?;
                            continue;
                        }
                        if self.eat("procedure") {
                            if procedure.is_some() {
                                return Err(self.error("one fixture procedure"));
                            }
                            procedure = Some(
                                self.word()?
                                    .parse()
                                    .map_err(|_| self.error("fixture procedure"))?,
                            );
                            self.expect(";")?;
                            continue;
                        }
                        if self.eat("endpoint") {
                            if endpoint.is_some() {
                                return Err(self.error("one endpoint requirement"));
                            }
                            let kind = self
                                .word()?
                                .parse::<pse_model::generated::enums::EndpointPolicy>()
                                .map_err(|_| self.error("endpoint policy"))?;
                            let event = if kind == pse_model::generated::enums::EndpointPolicy::DeclaredTerminalEvent {
                                self.expect("(")?; let event = self.until(&[")"])?; self.expect(")")?; Some(event)
                            } else { None };
                            self.expect(";")?;
                            endpoint =
                                Some(AuthoredModelingDeclarationsFieldValueScopeFixtureEndpoint {
                                    kind,
                                    event,
                                });
                            continue;
                        }
                        if self.eat("intent") {
                            if intent.is_some() {
                                return Err(self.error("one fixture solve intent"));
                            }
                            intent = Some(
                                self.word()?
                                    .parse()
                                    .map_err(|_| self.error("fixture solve intent"))?,
                            );
                            self.expect(";")?;
                            continue;
                        }
                        if self.peek() == "policy" {
                            if policy.is_some() {
                                return Err(self.error("one fixture policy"));
                            }
                            self.pos += 1;
                            policy = Some(self.fixture_policy()?);
                            continue;
                        }
                        if self.eat("stages") {
                            if !stages.is_empty() {
                                return Err(self.error("one initialization stage sequence"));
                            }
                            stages = self.names("(", ")")?;
                            self.expect(";")?;
                            continue;
                        }
                        if self.eat("initialize") {
                            if initialization.is_some() {
                                return Err(self.error("one initialization policy"));
                            }
                            initialization = Some(
                                AuthoredModelingDeclarationsFieldValueScopeFixtureInitialization {
                                    homotopy: self.fixture_option("homotopy")?,
                                    initial_step: self.fixture_option("step")?,
                                    minimum_step: self.fixture_option("minimum")?,
                                    growth: self.fixture_option("growth")?,
                                    maximum_attempts: self.fixture_option("attempts")?,
                                    time_limit_seconds: self.fixture_option("seconds")?,
                                },
                            );
                            self.expect(";")?;
                            continue;
                        }
                        if self.eat("integrate") {
                            if integration.is_some() {
                                return Err(self.error("one integration fixture"));
                            }
                            self.expect("samples")?;
                            let samples = self.expressions()?;
                            self.expect("relative")?;
                            let relative_tolerance = self.fixture_accuracy("relative")?;
                            self.expect("normalized_absolute")?;
                            let normalized_absolute_tolerance =
                                self.fixture_accuracy("normalized absolute")?;
                            self.expect("step")?;
                            self.expect("(")?;
                            let initial_step = self.until(&[")"])?;
                            self.expect(")")?;
                            let quadrature_relative_tolerance = if self.eat("quadrature_relative") {
                                Some(self.fixture_accuracy("quadrature relative")?)
                            } else {
                                None
                            };
                            let mut quadratures = Vec::new();
                            if self.eat("quadrature_absolute") {
                                self.expect("(")?;
                                while !self.eat(")") {
                                    let target = self.until(&["="])?;
                                    self.expect("=")?;
                                    let absolute_tolerance = self.until(&[",", ")"])?;
                                    quadratures.push(AuthoredModelingDeclarationsFieldValueScopeFixtureIntegrationQuadraturesItem { target, absolute_tolerance });
                                    if self.eat(")") {
                                        break;
                                    }
                                    self.expect(",")?;
                                }
                            }
                            self.expect(";")?;
                            integration = Some(
                                AuthoredModelingDeclarationsFieldValueScopeFixtureIntegration {
                                    samples,
                                    relative_tolerance,
                                    normalized_absolute_tolerance,
                                    initial_step,
                                    quadrature_relative_tolerance,
                                    quadratures,
                                    schedules: Vec::new(),
                                },
                            );
                            continue;
                        }
                        // Plan 23 CT-S13: `diagnose "<rule>" at(<member>, ...);` expects a
                        // numerical diagnostic finding naming each member at the solved point.
                        if self.eat("diagnose") {
                            let rule = self
                                .word()?
                                .parse::<pse_diagnostics::DiagnosticRule>()
                                .map_err(|error| self.error(&error.to_string()))?;
                            self.expect("at")?;
                            let members = self.expressions()?;
                            if members.is_empty() {
                                return Err(self.error("a diagnosed member"));
                            }
                            self.expect(";")?;
                            diagnostics.push(
                                AuthoredModelingDeclarationsFieldValueScopeFixtureDiagnosticsItem {
                                    rule,
                                    members,
                                },
                            );
                            continue;
                        }
                        // Plan 23 H5: `failure <class> validity(<layer>) [form(<path>)]
                        // [set(<expression>, ...)] variable(<argument or member>, ...);` or
                        // `failure <class> members(<member>, ...);` names the failure's typed
                        // lineage.
                        if self.eat("failure") {
                            if expected_failure.is_some() {
                                return Err(self.error("one expected failure"));
                            }
                            let class = self
                                .word()?
                                .parse()
                                .map_err(|_| self.error("failure class"))?;
                            let mut applicability = None;
                            let (validity, members) = if self.eat("validity") {
                                self.expect("(")?;
                                let layer = self.vocabulary::<pse_model::generated::enums::ModelingValidityLayer>("validity layer form or data")?;
                                self.expect(")")?;
                                let form = if self.eat("form") {
                                    self.expect("(")?;
                                    let form = self.path()?;
                                    self.expect(")")?;
                                    Some(form)
                                } else {
                                    None
                                };
                                let sets = if self.eat("set") {
                                    self.expressions()?
                                } else {
                                    Vec::new()
                                };
                                self.expect("variable")?;
                                let variables = self.expressions()?;
                                (
                                    Some(AuthoredModelingDeclarationsFieldValueScopeFixtureExpectedFailureValidity {
                                        layer,
                                        form,
                                        sets,
                                        variables,
                                    }),
                                    Vec::new(),
                                )
                            } else if self.eat("applicability") {
                                self.expect("(")?;
                                let layer = self.vocabulary("applicability layer")?;
                                self.expect(",")?;
                                let outcome = self
                                    .vocabulary("applicable, outside_region or unknown_evidence")?;
                                self.expect(")")?;
                                self.expect("claim")?;
                                self.expect("(")?;
                                let claim = self.path()?;
                                self.expect(")")?;
                                self.expect("form")?;
                                self.expect("(")?;
                                let form = self.path()?;
                                self.expect(")")?;
                                let sets = if self.eat("set") {
                                    self.expressions()?
                                } else {
                                    Vec::new()
                                };
                                let variables = if self.eat("variable") {
                                    self.expressions()?
                                } else {
                                    Vec::new()
                                };
                                applicability=Some(AuthoredModelingDeclarationsFieldValueScopeFixtureExpectedFailureApplicability {layer,outcome,claim,form,sets,variables});
                                (None, Vec::new())
                            } else if self.eat("members") {
                                (None, self.expressions()?)
                            } else {
                                return Err(
                                    self.error("failure lineage validity(...) or members(...)")
                                );
                            };
                            self.expect(";")?;
                            expected_failure = Some(
                                AuthoredModelingDeclarationsFieldValueScopeFixtureExpectedFailure {
                                    class,
                                    validity,
                                    applicability,
                                    members,
                                },
                            );
                            continue;
                        }
                        let kind = self
                            .word()?
                            .parse::<pse_model::generated::enums::ModelingFixtureBinding>()
                            .map_err(|_| self.error("fixture value, fix, free, lower or upper"))?;
                        let target = self.until(&["=", ";"])?;
                        let expression = if self.eat("=") {
                            Some(self.until(&[";"])?)
                        } else {
                            None
                        };
                        self.expect(";")?;
                        specifications.push(
                            AuthoredModelingDeclarationsFieldValueScopeFixtureSpecificationsItem {
                                target,
                                kind,
                                expression,
                            },
                        );
                    }
                    if !schedules.is_empty() {
                        integration
                            .as_mut()
                            .ok_or_else(|| {
                                self.error("an integrate clause for the scheduled inputs")
                            })?
                            .schedules = schedules;
                    }
                    Some(AuthoredModelingDeclarationsFieldValueScopeFixture {
                        degrees_of_freedom,
                        route,
                        procedure,
                        endpoint,
                        intent,
                        policy,
                        stages,
                        initialization,
                        integration,
                        modes,
                        shooting,
                        expected_failure,
                        diagnostics,
                        specifications,
                    })
                } else {
                    None
                };
                if (oracle.is_some() || fixture.is_some())
                    && !matches!(keyword.as_str(), "test" | "case")
                {
                    return Err(self.error("fixture/oracle metadata belongs to a test or case"));
                }
                let scope = AuthoredModelingDeclarationsFieldValueScope {
                    parameters,
                    bases,
                    type_parameters,
                    selection,
                    branch,
                    operational,
                    eligibility,
                    oracle,
                    facets,
                    fixture,
                };
                self.expect("{")?;
                child_block = true;
                match keyword.as_str() {
                    "package" => Value::from_package(scope),
                    "entity_kind" => Value::from_entity_kind(scope),
                    "interface" => Value::from_interface(scope),
                    "def" => Value::from_definition(scope),
                    "case" => Value::from_case(scope),
                    "test" => Value::from_test(scope),
                    "stage" => Value::from_stage(scope),
                    "regime" => Value::from_regime(scope),
                    "disjunction" => Value::from_disjunction(scope),
                    "alternative" => Value::from_alternative(scope),
                    _ => Value::from_implicit(scope),
                }
            }
            "use" => {
                self.expect("@")?;
                // ADR-0123 Outcome 7: an exact requirement, written `"1.2.3"` or `"=1.2.3"`.
                let at = self.pos;
                let written = self.word()?;
                let version = exact_requirement(&written).ok_or_else(|| {
                    self.pos = at;
                    self.error("exact package version")
                })?;
                let alias = if self.eat("as") {
                    Some(self.word()?)
                } else {
                    None
                };
                self.expect(";")?;
                Value::from_import(AuthoredModelingDeclarationsFieldValueImport { version, alias })
            }
            // ADR-0123 Outcome 2: `enum E { [@id("…")] member [facets(f, …)], … }`. A member's
            // identity is explicit under the explicit policy and derived from the enumeration
            // and its name under the named one; renaming an explicit member keeps its
            // identity. A member declares the data facets of a role (Outcome 5).
            "enum" => {
                self.expect("{")?;
                let mut members = Vec::new();
                if !self.eat("}") {
                    loop {
                        let member_start = self.at();
                        let explicit = if self.eat("@") {
                            self.expect("id")?;
                            self.expect("(")?;
                            let id = SemanticId::parse_hex(&self.word()?)
                                .map_err(|_| self.error("semantic ID"))?;
                            self.expect(")")?;
                            Some(id)
                        } else {
                            None
                        };
                        let member = self.word()?;
                        let member_id = match explicit {
                            Some(id) => id,
                            None if self.policy == IdentityPolicy::Explicit => {
                                return Err(AuthoringError::MissingId {
                                    at: SourceSpan::new(self.document, member_start, self.at()),
                                    kind: "enum member".into(),
                                    name: format!("{name}.{member}"),
                                });
                            }
                            None => pse_ids::named_id(id.as_id(), &format!("member:{member}")),
                        };
                        let mut facets = Vec::new();
                        if self.eat("facets") {
                            self.expect("(")?;
                            if !self.eat(")") {
                                loop {
                                    let facet = self.vocabulary::<pse_model::generated::enums::ModelingDataFacet>(
                                        "declared data role facet",
                                    )?;
                                    if facets.contains(&facet) {
                                        return Err(self.error("distinct data facets"));
                                    }
                                    facets.push(facet);
                                    if self.eat(")") {
                                        break;
                                    }
                                    self.expect(",")?;
                                }
                            }
                        }
                        members.push(
                            AuthoredModelingDeclarationsFieldValueEnumerationMembersItem {
                                member_id,
                                name: member,
                                facets,
                            },
                        );
                        if self.eat("}") {
                            break;
                        }
                        self.expect(",")?;
                    }
                }
                self.eat(";");
                Value::from_enum(AuthoredModelingDeclarationsFieldValueEnumeration { members })
            }
            // ADR-0123 Outcome 2: attribute values are cells, typed at admission.
            "entity" => {
                let kind_name = entity_kind.ok_or_else(|| self.error("entity kind"))?;
                let provenance = if self.peek() == "provenance" {
                    Some(self.provenance()?)
                } else {
                    None
                };
                self.expect("{")?;
                let mut attributes = Vec::new();
                while !self.eat("}") {
                    let name = self.word()?;
                    self.expect("=")?;
                    let value = self.cell()?;
                    let provenance = if self.peek() == "provenance" {
                        Some(self.provenance()?)
                    } else {
                        None
                    };
                    attributes.push(AuthoredModelingDeclarationsFieldValueEntityAttributesItem {
                        name,
                        value,
                        provenance,
                    });
                    if self.eat("}") {
                        break;
                    }
                    if !self.eat(",") {
                        self.expect(";")?;
                    }
                }
                self.eat(";");
                Value::from_entity(AuthoredModelingDeclarationsFieldValueEntity {
                    kind_name,
                    provenance,
                    attributes,
                })
            }
            // `attribute name: T [unique] [= cell];` or `key name: T [unique] [= cell];`
            // declares an attribute of an entity kind; `name = cell;` binds an inherited one
            // (ADR-0123 Outcome 2). `derived name: T [unique] = expression;` declares one that
            // admission evaluates for each entity (Plan 23 D0).
            "attribute" | "key" | "derived" => {
                self.expect(":")?;
                let r#type = Some(self.type_expr()?);
                let storage = self.storage(&name)?.into_iter().collect();
                let unique = self.eat("unique");
                let (value, derived) = if keyword == "derived" {
                    self.expect("=")?;
                    (None, Some(self.until(&[";"])?))
                } else if self.eat("=") {
                    (Some(self.cell()?), None)
                } else {
                    (None, None)
                };
                self.expect(";")?;
                Value::from_attribute(AuthoredModelingDeclarationsFieldValueAttribute {
                    r#type,
                    key: keyword == "key",
                    value,
                    unique,
                    derived,
                    storage,
                })
            }
            BIND => {
                self.expect("=")?;
                let value = Some(self.cell()?);
                self.expect(";")?;
                Value::from_attribute(AuthoredModelingDeclarationsFieldValueAttribute {
                    r#type: None,
                    key: false,
                    value,
                    unique: false,
                    derived: None,
                    storage: Vec::new(),
                })
            }
            // `identifier scheme name;`: values of the scheme are opaque (ADR-0123 Outcome 2).
            "identifier_scheme" => {
                self.expect(";")?;
                Value::from_identifier_scheme()
            }
            // `constant name: T = cell provenance(source, role[, lineage(…)]);`: a typed
            // declaration, not a zero-argument function, with its provenance (ADR-0123
            // Outcome 5).
            "constant" => {
                self.expect(":")?;
                let r#type = self.type_expr()?;
                self.expect("=")?;
                let value = self.cell()?;
                let provenance = self.provenance()?;
                self.expect(";")?;
                Value::from_constant(AuthoredModelingDeclarationsFieldValueConstant {
                    r#type,
                    value,
                    provenance,
                })
            }
            "coordinate_map" => {
                let arguments = self
                    .parameters()?
                    .into_iter()
                    .map(|(name, r#type, default_value)| {
                        AuthoredModelingDeclarationsFieldValueCoordinateMapArgumentsItem {
                            name,
                            r#type,
                            default_value,
                        }
                    })
                    .collect();
                let validity = if self.eat("valid") {
                    self.expect("(")?;
                    let predicate = self.until(&[")"])?;
                    self.expect(")")?;
                    Some(predicate)
                } else {
                    None
                };
                self.expect("{")?;
                child_block = true;
                Value::from_coordinate_map(AuthoredModelingDeclarationsFieldValueCoordinateMap {
                    arguments,
                    validity,
                })
            }
            "slot" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueCoordinateSlotIndicesItem {
                            name,
                            domain,
                        }
                    })
                    .collect();
                self.expect("=")?;
                let expression = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_coordinate_slot(AuthoredModelingDeclarationsFieldValueCoordinateSlot {
                    indices,
                    expression,
                })
            }
            "reconstruction" => {
                self.expect("for")?;
                let map = self.path()?;
                let arguments = self
                    .parameters()?
                    .into_iter()
                    .map(|(name, r#type, default_value)| {
                        AuthoredModelingDeclarationsFieldValueReconstructionArgumentsItem {
                            name,
                            r#type,
                            default_value,
                        }
                    })
                    .collect();
                self.expect("->")?;
                let return_type = self.type_expr()?;
                self.expect("reference")?;
                let reference = self.path()?;
                self.expect("=")?;
                let normalization = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_reconstruction(AuthoredModelingDeclarationsFieldValueReconstruction {
                    map,
                    arguments,
                    return_type,
                    reference,
                    normalization,
                })
            }
            "response" => {
                self.expect("from")?;
                let witness = self.word()?;
                let arguments = self
                    .parameters()?
                    .into_iter()
                    .map(|(name, r#type, default_value)| {
                        AuthoredModelingDeclarationsFieldValueResponseArgumentsItem {
                            name,
                            r#type,
                            default_value,
                        }
                    })
                    .collect();
                self.expect("->")?;
                let return_type = self.type_expr()?;
                self.expect("=")?;
                let body = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_response(AuthoredModelingDeclarationsFieldValueResponse {
                    witness,
                    arguments,
                    return_type,
                    body,
                })
            }
            "reference_translation" => {
                let arguments = self
                    .parameters()?
                    .into_iter()
                    .map(|(name, r#type, default_value)| {
                        AuthoredModelingDeclarationsFieldValueReferenceTranslationArgumentsItem {
                            name,
                            r#type,
                            default_value,
                        }
                    })
                    .collect();
                self.expect("->")?;
                let return_type = self.type_expr()?;
                self.expect("anchors")?;
                self.expect("(")?;
                self.expect("source")?;
                self.expect("=")?;
                let source_anchor = self.path()?;
                self.expect(",")?;
                self.expect("target")?;
                self.expect("=")?;
                let target_anchor = self.path()?;
                self.expect(")")?;
                self.expect("at")?;
                self.expect("(")?;
                self.expect("temperature")?;
                self.expect("=")?;
                let temperature = self.until(&[","])?;
                self.expect(",")?;
                self.expect("pressure")?;
                self.expect("=")?;
                let pressure = self.until(&[")"])?;
                self.expect(")")?;
                let provenance = self.provenance()?;
                self.expect(";")?;
                Value::from_reference_translation(
                    AuthoredModelingDeclarationsFieldValueReferenceTranslation {
                        arguments,
                        return_type,
                        source_anchor,
                        target_anchor,
                        temperature,
                        pressure,
                        provenance,
                    },
                )
            }
            "boundary" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueBoundaryIndicesItem { name, domain }
                    })
                    .collect();
                self.expect(";")?;
                Value::from_boundary(AuthoredModelingDeclarationsFieldValueBoundary { indices })
            }
            "exchange" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueExchangeIndicesItem { name, domain }
                    })
                    .collect();
                self.expect("between")?;
                let from = self.until(&["and"])?;
                self.expect("and")?;
                let to = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_exchange(AuthoredModelingDeclarationsFieldValueExchange {
                    indices,
                    from,
                    to,
                })
            }
            "fn" => {
                let type_parameters = self.names("<", ">")?;
                self.type_variables.extend(type_parameters.iter().cloned());
                let arguments = self
                    .parameters()?
                    .into_iter()
                    .map(|(name, r#type, default_value)| {
                        AuthoredModelingDeclarationsFieldValueFunctionArgumentsItem {
                            name,
                            r#type,
                            default_value,
                        }
                    })
                    .collect();
                self.expect("->")?;
                let return_type = self.type_expr()?;
                let guards = if self.eat("guards") {
                    self.guards()?
                } else {
                    Vec::new()
                };
                let applicability = if self.eat("applicability") {
                    self.expressions()?
                } else {
                    Vec::new()
                };
                let validity = if self.eat("valid") {
                    self.expect("(")?;
                    let predicate = self.until(&[")"])?;
                    self.expect(")")?;
                    Some(predicate)
                } else {
                    None
                };
                let continuity = if self.eat("piecewise") {
                    let value = self.until(&["="])?;
                    Some(
                        value
                            .parse::<i64>()
                            .ok()
                            .filter(|v| (0..=2).contains(v))
                            .ok_or_else(|| self.error("piecewise order 0, 1 or 2"))?,
                    )
                } else {
                    None
                };
                let external = if self.eat("external") {
                    let implementation = self.word()?;
                    self.expect("revision")?;
                    let revision = self.word()?;
                    self.expect("data")?;
                    let data = self.word()?;
                    self.expect("output")?;
                    let output = self.until(&["derivatives"])?;
                    self.expect("derivatives")?;
                    let derivatives = self
                        .until(&["source"])?
                        .parse()
                        .map_err(|_| self.error("derivative order"))?;
                    self.expect("source")?;
                    let derivative_source = self
                        .word()?
                        .parse()
                        .map_err(|_| self.error("derivative source"))?;
                    self.expect("smoothness")?;
                    let smoothness = self
                        .until(&[";"])?
                        .parse()
                        .map_err(|_| self.error("smoothness order"))?;
                    Some(AuthoredModelingDeclarationsFieldValueFunctionExternal {
                        implementation,
                        revision,
                        data,
                        output,
                        derivatives,
                        derivative_source,
                        smoothness,
                    })
                } else {
                    None
                };
                let body = if self.eat("=") {
                    Some(self.until(&[";"])?)
                } else {
                    None
                };
                self.expect(";")?;
                Value::from_function(AuthoredModelingDeclarationsFieldValueFunction {
                    external,
                    validity,
                    guards,
                    applicability,
                    continuity,
                    type_parameters,
                    arguments,
                    return_type,
                    body,
                })
            }
            "param" | "var" | "let" | "alias" | "set" | "child" | "port" | "preset" | "scope" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueBindingIndicesItem { name, domain }
                    })
                    .collect();
                let r#type = if self.eat(":") {
                    Some(self.type_expr()?)
                } else {
                    None
                };
                // ADR-0103: a variable always carries its declared domain; continuous is the
                // default spelling. No other binding declares one.
                let domain = if keyword == "var" {
                    Some(if self.eat("in") {
                        self.word()?
                            .parse::<pse_model::generated::enums::ModelingVariableDomain>()
                            .map_err(|_| {
                                self.error(
                                    "continuous, integer, binary, semicontinuous or semiinteger",
                                )
                            })?
                    } else {
                        pse_model::generated::enums::ModelingVariableDomain::Continuous
                    })
                } else if self.peek() == "in" {
                    return Err(self.error("a domain facet belongs to a var declaration"));
                } else {
                    None
                };
                let expression = if self.eat("=") {
                    Some(self.until(&[";", "defined"])?)
                } else {
                    None
                };
                let defined_by = if self.eat("defined") {
                    self.expect("by")?;
                    Some(self.until(&[";"])?)
                } else {
                    None
                };
                self.expect(";")?;
                let b = AuthoredModelingDeclarationsFieldValueBinding {
                    r#type,
                    indices,
                    expression,
                    defined_by,
                    domain,
                };
                match keyword.as_str() {
                    "param" => Value::from_parameter(b),
                    "var" => Value::from_variable(b),
                    "let" => Value::from_let(b),
                    "alias" => Value::from_alias(b),
                    "set" => Value::from_set(b),
                    "child" => Value::from_child(b),
                    "port" => Value::from_port(b),
                    "preset" => Value::from_preset(b),
                    _ => Value::from_scope_value(b),
                }
            }
            "eq" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueEquationIndicesItem { name, domain }
                    })
                    .collect();
                // ADR-0104: `when y` or `when not y` makes an indicator constraint.
                let condition = if self.eat("when") {
                    let active = !self.eat("not");
                    let variable = self.until(&[":"])?;
                    Some(AuthoredModelingDeclarationsFieldValueEquationCondition {
                        variable,
                        active,
                    })
                } else {
                    None
                };
                self.expect(":")?;
                let expression = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_equation(AuthoredModelingDeclarationsFieldValueEquation {
                    indices,
                    expression,
                    condition,
                })
            }
            "when" => {
                let predicate = self.until(&["{"])?;
                self.expect("{")?;
                child_block = true;
                Value::from_when(AuthoredModelingDeclarationsFieldValueGuard { predicate })
            }
            "require" => {
                let predicate = self.until(&[":"])?;
                self.expect(":")?;
                let message = self.word()?;
                self.expect(";")?;
                Value::from_requirement(AuthoredModelingDeclarationsFieldValueRequirement {
                    predicate,
                    message,
                })
            }
            "connect" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueConnectionIndicesItem { name, domain }
                    })
                    .collect();
                let from = self.until(&["->"])?;
                self.expect("->")?;
                let to = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_connection(AuthoredModelingDeclarationsFieldValueConnection {
                    indices,
                    from,
                    to,
                })
            }
            "state" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueStateSpecificationIndicesItem {
                            name,
                            domain,
                        }
                    })
                    .collect();
                let extends = if self.eat("extends") {
                    Some(self.until(&["supplied"])?)
                } else {
                    None
                };
                self.expect("supplied")?;
                self.expect("(")?;
                let supplied = self.until(&[")"])?;
                self.expect(")")?;
                self.expect("{")?;
                let mut coordinates = Vec::new();
                let mut reconstructions = Vec::new();
                let mut transports = Vec::new();
                while !self.eat("}") {
                    let role = self.word()?;
                    let name = self.word()?;
                    let indices = self.indices()?;
                    match role.as_str() {
                        "coordinate" => {
                            self.expect("=")?;
                            let target = self.until(&[";"])?;
                            coordinates.push(AuthoredModelingDeclarationsFieldValueStateSpecificationCoordinatesItem {
                                name, target, indices: indices.into_iter().map(|(name, domain)|
                                    AuthoredModelingDeclarationsFieldValueStateSpecificationCoordinatesItemIndicesItem {name,domain}).collect(),
                            });
                        }
                        "reconstruct" => {
                            self.expect(":")?;
                            let equation = self.until(&["tolerance"])?;
                            self.expect("tolerance")?;
                            let tolerance = self.until(&[";"])?;
                            reconstructions.push(AuthoredModelingDeclarationsFieldValueStateSpecificationReconstructionsItem {
                                name, equation, tolerance, indices: indices.into_iter().map(|(name, domain)|
                                    AuthoredModelingDeclarationsFieldValueStateSpecificationReconstructionsItemIndicesItem {name,domain}).collect(),
                            });
                        }
                        "transport" => {
                            self.expect("=")?;
                            let expression = self.until(&["tolerance"])?;
                            self.expect("tolerance")?;
                            let tolerance = self.until(&[";"])?;
                            transports.push(AuthoredModelingDeclarationsFieldValueStateSpecificationTransportsItem {
                                name, expression, tolerance, indices: indices.into_iter().map(|(name, domain)|
                                    AuthoredModelingDeclarationsFieldValueStateSpecificationTransportsItemIndicesItem {name,domain}).collect(),
                            });
                        }
                        _ => return Err(self.error("state coordinate, reconstruct or transport")),
                    }
                    self.expect(";")?;
                }
                self.eat(";");
                Value::from_state_specification(
                    AuthoredModelingDeclarationsFieldValueStateSpecification {
                        indices,
                        extends,
                        supplied,
                        coordinates,
                        reconstructions,
                        transports,
                    },
                )
            }
            "state_port" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueStatePortIndicesItem { name, domain }
                    })
                    .collect();
                self.expect("=")?;
                let specification = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_state_port(AuthoredModelingDeclarationsFieldValueStatePort {
                    indices,
                    specification,
                })
            }
            "conserve" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueInventoryBalanceIndicesItem {
                            name,
                            domain,
                        }
                    })
                    .collect();
                self.expect(":")?;
                let r#type = self.type_expr()?;
                self.expect("on")?;
                let axis = self.path()?;
                self.expect("inventory")?;
                let inventory = self.until(&["flux"])?;
                self.expect("flux")?;
                let flux = self.until(&["tolerance"])?;
                self.expect("tolerance")?;
                let tolerance = self.until(&["transfers", ";"])?;
                let mut transfers = Vec::new();
                if self.eat("transfers") {
                    self.expect("(")?;
                    while !self.eat(")") {
                        // Match fixture event guard paths, including authored indices.
                        let event = self.until(&["="])?;
                        self.expect("=")?;
                        let expression = self.until(&[",", ")"])?;
                        transfers.push(
                            AuthoredModelingDeclarationsFieldValueInventoryBalanceTransfersItem {
                                event,
                                expression,
                            },
                        );
                        if self.eat(")") {
                            break;
                        }
                        self.expect(",")?;
                    }
                }
                self.expect(";")?;
                Value::from_inventory_balance(
                    AuthoredModelingDeclarationsFieldValueInventoryBalance {
                        indices,
                        r#type,
                        axis,
                        inventory,
                        flux,
                        tolerance,
                        transfers,
                    },
                )
            }
            "accumulate" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueAccumulatorIndicesItem {
                            name,
                            domain,
                        }
                    })
                    .collect();
                self.expect(":")?;
                let r#type = self.type_expr()?;
                let boundary = if self.eat("boundary") {
                    Some(self.until(&["conservation", "accounting", "observation"])?)
                } else {
                    None
                };
                let mode = self
                    .word()?
                    .parse()
                    .map_err(|_| self.error("conservation, accounting or observation"))?;
                self.expect("tolerance")?;
                let tolerance = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_accumulator(AuthoredModelingDeclarationsFieldValueAccumulator {
                    indices,
                    r#type,
                    boundary,
                    mode,
                    tolerance,
                })
            }
            "contribute" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueContributionIndicesItem {
                            name,
                            domain,
                        }
                    })
                    .collect();
                let target = self.until(&["role"])?;
                self.expect("role")?;
                let role = self
                    .word()?
                    .parse()
                    .map_err(|_| self.error("contribution role"))?;
                self.expect("=")?;
                let expression = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_contribution(AuthoredModelingDeclarationsFieldValueContribution {
                    indices,
                    target,
                    expression,
                    role,
                })
            }
            // ADR-0123 Outcome 3: `table name[key: T storage, k: 0..2]: T storage | {col: T
            // storage, derived d: T = e}, where each storage is `[storage {unit}] [by scheme]`
            // (ADR-0125),
            // [envelope axis: Q in lower..upper]... [symmetric(i, j) [diagonal
            // allowed|excluded]] [unique(names)]... [complete_over(key [in set | in lo..hi],
            // ...)] [missing required|optional|default cell] [require predicate]... ;`,
            // clauses in this order; an envelope is bounded by two columns (Outcome 4).
            "table" => {
                let mut keys = Vec::new();
                let mut storage = Vec::new();
                if self.eat("[") {
                    loop {
                        let name = self.word()?;
                        self.expect(":")?;
                        let (r#type, range) = if self.at_range() {
                            let range = self.integer_range()?;
                            (vec![node(TypeNodeKind::Integer, vec![])], Some(range))
                        } else {
                            (self.type_expr()?, None)
                        };
                        if let Some(entry) = self.storage(&name)? {
                            storage.push(entry);
                        }
                        keys.push(AuthoredModelingDeclarationsFieldValueTableKeysItem {
                            name,
                            r#type,
                            range,
                        });
                        if self.eat("]") {
                            break;
                        }
                        self.expect(",")?;
                    }
                }
                self.expect(":")?;
                let mut columns = Vec::new();
                let value_type = if self.eat("{") {
                    while !self.eat("}") {
                        let derived = self.eat("derived");
                        let name = self.word()?;
                        self.expect(":")?;
                        let r#type = self.type_expr()?;
                        let derived = if derived {
                            self.expect("=")?;
                            Some(self.until(&[",", "}"])?)
                        } else {
                            if let Some(entry) = self.storage(&name)? {
                                storage.push(entry);
                            }
                            None
                        };
                        columns.push(AuthoredModelingDeclarationsFieldValueTableColumnsItem {
                            name,
                            r#type,
                            derived,
                        });
                        if self.eat("}") {
                            break;
                        }
                        self.expect(",")?;
                    }
                    None
                } else {
                    Some(self.type_expr()?)
                };
                if value_type.is_some()
                    && let Some(entry) = self.storage("value")?
                {
                    storage.push(entry);
                }
                let mut envelopes = Vec::new();
                while self.eat("envelope") {
                    let name = self.word()?;
                    let (r#type, lower, upper) = self.envelope_bounds()?;
                    envelopes.push(ModelingEnvelope {
                        name,
                        r#type,
                        lower,
                        upper,
                    });
                }
                let symmetry = if matches!(self.peek(), "symmetric" | "ordered") {
                    let ordered = self.word()? == "ordered";
                    self.expect("(")?;
                    let first = self.word()?;
                    self.expect(",")?;
                    let second = self.word()?;
                    self.expect(")")?;
                    let diagonal = if self.eat("diagonal") {
                        self.vocabulary("allowed or excluded")?
                    } else {
                        pse_model::generated::enums::ModelingDiagonalPolicy::Allowed
                    };
                    Some(AuthoredModelingDeclarationsFieldValueTableSymmetry {
                        ordered,
                        first,
                        second,
                        diagonal,
                    })
                } else {
                    None
                };
                let mut unique = Vec::new();
                while self.eat("unique") {
                    unique.push(AuthoredModelingDeclarationsFieldValueTableUniqueItem {
                        names: self.names("(", ")")?,
                    });
                }
                let complete_over = if self.peek() == "complete_over" {
                    self.completeness()?
                } else {
                    Vec::new()
                };
                use pse_model::generated::enums::ModelingMissingPolicy as Missing;
                let missing_policy = if self.eat("missing") {
                    self.vocabulary::<Missing>("required, optional or default")?
                } else {
                    Missing::Required
                };
                let default_value = if missing_policy == Missing::Default {
                    Some(self.cell()?)
                } else {
                    None
                };
                let mut requirements = Vec::new();
                while self.eat("require") {
                    requirements.push(self.until(&["require", ";"])?);
                }
                self.expect(";")?;
                Value::from_table(AuthoredModelingDeclarationsFieldValueTable {
                    keys,
                    columns,
                    value_type,
                    storage,
                    missing_policy,
                    default_value,
                    complete_over,
                    symmetry,
                    unique,
                    requirements,
                    envelopes,
                })
            }
            "applicability" => {
                use pse_model::generated::enums::ModelingApplicabilityKind as Claim;
                let arguments = self
                    .parameters()?
                    .into_iter()
                    .map(|(name, r#type, default_value)| {
                        AuthoredModelingDeclarationsFieldValueApplicabilityArgumentsItem {
                            name,
                            r#type,
                            default_value,
                        }
                    })
                    .collect();
                self.expect("owner")?;
                let owner = self.path()?;
                self.expect("scope")?;
                let scope = self.vocabulary("form, data or closure scope")?;
                self.expect("evidence")?;
                let evidence = self.path()?;
                let claim_kind: Claim =
                    self.vocabulary("region, interval, unrestricted, unknown or union")?;
                let mut basis = None;
                let mut predicate = None;
                let mut reason = None;
                let mut axis = None;
                let mut lower = None;
                let mut upper = None;
                let mut alternatives = Vec::new();
                match claim_kind {
                    Claim::Region => {
                        basis = Some(self.vocabulary("fitted, recommended or validated basis")?);
                        self.expect("(")?;
                        predicate = Some(self.until(&[")"])?);
                        self.expect(")")?;
                    }
                    Claim::Interval => {
                        basis = Some(self.vocabulary("fitted, recommended or validated basis")?);
                        axis = Some(self.until(&["in"])?);
                        self.expect("in")?;
                        lower = Some(self.until(&[".."])?);
                        self.expect("..")?;
                        upper = Some(self.until(&["dependencies", ";"])?);
                    }
                    Claim::Unknown => {
                        self.expect("(")?;
                        reason = Some(self.word()?);
                        self.expect(")")?;
                    }
                    Claim::Union => alternatives = self.expressions()?,
                    Claim::Unrestricted => {}
                }
                let dependencies = if self.eat("dependencies") {
                    self.expressions()?
                } else {
                    Vec::new()
                };
                self.expect(";")?;
                Value::from_applicability(AuthoredModelingDeclarationsFieldValueApplicability {
                    arguments,
                    owner,
                    scope,
                    evidence,
                    claim_kind,
                    basis,
                    predicate,
                    reason,
                    axis,
                    lower,
                    upper,
                    alternatives,
                    dependencies,
                })
            }
            "permission" => {
                let target_kind = self.vocabulary("records or families")?;
                let targets = self.expressions()?;
                let mut allow_unknown = false;
                let mut allow_extrapolation = false;
                let mut seen = BTreeSet::new();
                while matches!(self.peek(), "allow_unknown" | "allow_extrapolation") {
                    let option = self.word()?;
                    if !seen.insert(option.clone()) {
                        return Err(self.error("one permission flag of each kind"));
                    }
                    let flag = self
                        .word()?
                        .parse::<bool>()
                        .map_err(|_| self.error("true or false"))?;
                    match option.as_str() {
                        "allow_unknown" => allow_unknown = flag,
                        _ => allow_extrapolation = flag,
                    }
                }
                self.expect(";")?;
                Value::from_permission(AuthoredModelingDeclarationsFieldValuePermission {
                    target_kind,
                    targets,
                    allow_unknown,
                    allow_extrapolation,
                })
            }
            // ADR-0123 Outcome 4: `envelope axis: Q in lower..upper;` in an entity kind, the
            // axis bounded by two of the kind's attributes.
            "envelope" => {
                let (r#type, lower, upper) = self.envelope_bounds()?;
                self.expect(";")?;
                Value::from_envelope(AuthoredModelingDeclarationsFieldValueEnvelope {
                    r#type,
                    lower,
                    upper,
                })
            }
            // `dataset name: target [bind(key = cell, …)] [complete_over(…)]
            // provenance(source, role[, lineage(…)]) ({ [keys] = [values]; … } | from "path";)`:
            // positional cells for a table or a keyed kind, or the package data document
            // supplying a table's rows (ADR-0125); a key the dataset supplies for every row
            // is a declared binding (ADR-0123 Outcome 2); the provenance is typed (Outcome 5).
            "dataset" => {
                self.expect(":")?;
                let target = self.path()?;
                let mut bindings = Vec::new();
                if self.eat("bind") {
                    self.expect("(")?;
                    loop {
                        let name = self.word()?;
                        self.expect("=")?;
                        let value = self.cell()?;
                        bindings.push(AuthoredModelingDeclarationsFieldValueDatasetBindingsItem {
                            name,
                            value,
                        });
                        if self.eat(")") {
                            break;
                        }
                        self.expect(",")?;
                    }
                }
                let complete_over = if self.peek() == "complete_over" {
                    self.completeness()?
                } else {
                    Vec::new()
                };
                let provenance = self.provenance()?;
                let mut rows = Vec::new();
                // ADR-0125: `from "data/…parquet";` names the package data document
                // supplying the rows, in place of inline rows.
                let document = if self.eat("from") {
                    let at = self.pos;
                    if self.tokens.get(self.pos).map(|t| t.kind) != Some(Kind::Quoted) {
                        return Err(self.error("quoted data document path"));
                    }
                    let path = self.word().inspect_err(|_| self.pos = at)?;
                    self.expect(";")?;
                    Some(path)
                } else {
                    self.expect("{")?;
                    None
                };
                while document.is_none() && !self.eat("}") {
                    let keys = self.cells()?;
                    self.expect("=")?;
                    let values = if self.peek() == "[" {
                        self.cells()?
                    } else {
                        vec![self.cell()?]
                    };
                    self.expect(";")?;
                    rows.push(AuthoredModelingDeclarationsFieldValueDatasetRowsItem {
                        keys,
                        values,
                    });
                }
                if document.is_none() {
                    self.eat(";");
                }
                Value::from_dataset(AuthoredModelingDeclarationsFieldValueDataset {
                    target,
                    provenance,
                    bindings,
                    complete_over,
                    rows,
                    document,
                })
            }
            "annotation" => {
                use pse_model::generated::enums::ModelingAnnotationKind as A;
                let kind = self.vocabulary::<A>("annotation kind")?;
                let mut annotation = AuthoredModelingDeclarationsFieldValueAnnotation {
                    kind,
                    target: String::new(),
                    arguments: Vec::new(),
                    scheme: None,
                    connectivity: None,
                    objective: None,
                    accuracy_goal: None,
                    engineering_scale: None,
                    engineering_default: None,
                };
                // A shared engineering rule marks an existing typed constant at package
                // scope. It is deliberately not target-attached: consumers select it by
                // the constant's stable semantic identity.
                if kind == A::EngineeringRule {
                    if depth != 1 {
                        return Err(self.error("engineering_rule belongs at package scope"));
                    }
                    annotation.target = self.until(&[";"])?;
                    self.expect(";")?;
                    Value::from_annotation(annotation)
                } else {
                    annotation.target = self.until(&["("])?;
                    self.expect("(")?;
                    match kind {
                        // ADR-0111: `annotation objective t(sense, member = value, ...)` carries
                        // typed members, not positional arguments.
                        A::Objective => annotation.objective = Some(self.objective_members()?),
                        // A hard physical domain range has exactly two endpoints.
                        A::Valid => {
                            annotation.arguments.push(self.until(&[","])?);
                            self.expect(",")?;
                            annotation.arguments.push(self.until(&[",", ")"])?);
                            self.expect(")")?;
                        }
                        A::Scale => {
                            annotation.scheme = Some(self.vocabulary("constraint scaling scheme")?);
                            self.expect(")")?;
                        }
                        A::AccuracyGoal => {
                            annotation.accuracy_goal = Some(self.accuracy_goal_members()?);
                        }
                        A::EngineeringScale => {
                            annotation.engineering_scale = Some(self.engineering_scale_members()?);
                        }
                        A::EngineeringDefault => {
                            let rule = self.until(&[")"])?;
                            if rule.is_empty() {
                                return Err(self.error("shared engineering rule path"));
                            }
                            self.expect(")")?;
                            annotation.engineering_default = Some(
                            AuthoredModelingDeclarationsFieldValueAnnotationEngineeringDefault {
                                rule,
                            },
                        );
                        }
                        A::EngineeringRule => {
                            return Err(self.error("engineering_rule marker syntax"));
                        }
                        // `annotation connectivity port(incoming, outgoing)`, each a
                        // nonnegative maximum or `many`.
                        A::Connectivity => {
                            let incoming = self.connection_maximum()?;
                            self.expect(",")?;
                            let outgoing = self.connection_maximum()?;
                            self.expect(")")?;
                            annotation.connectivity = Some(
                                AuthoredModelingDeclarationsFieldValueAnnotationConnectivity {
                                    incoming,
                                    outgoing,
                                },
                            );
                        }
                        A::Start | A::Nominal | A::Bounds | A::Report | A::Check => {
                            if !self.eat(")") {
                                loop {
                                    annotation.arguments.push(self.until(&[",", ")"])?);
                                    if self.eat(")") {
                                        break;
                                    }
                                    self.expect(",")?;
                                }
                            }
                        }
                    }
                    self.expect(";")?;
                    Value::from_annotation(annotation)
                }
            }
            "expect" => {
                let actual = self.until(&["=="])?;
                self.expect("==")?;
                let expected = self.until(&["tolerance"])?;
                self.expect("tolerance")?;
                let tolerance = self.until(&["relative", ";"])?;
                let relative_tolerance = if self.eat("relative") {
                    Some(self.until(&[";"])?)
                } else {
                    None
                };
                self.expect(";")?;
                Value::from_expectation(AuthoredModelingDeclarationsFieldValueExpectation {
                    actual,
                    expected,
                    tolerance,
                    relative_tolerance,
                })
            }
            _ => return Err(self.error("modeling declaration")),
        };
        let position = self.rows.len();
        self.rows.push(Declaration {
            declaration_id: id,
            document_id: self.document,
            parent_id: parent,
            ordinal,
            name,
            is_override,
            source_start: i64::from(start),
            source_end: i64::from(self.at()),
            value,
        });
        if child_block {
            self.block(Some(id), depth + 1, true)?;
        }
        self.type_variables.truncate(variables);
        self.rows[position].source_end = i64::from(
            self.tokens
                .get(self.pos.saturating_sub(1))
                .map_or(self.at(), |t| t.span.end),
        );
        Ok(())
    }
}
