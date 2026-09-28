// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use crate::dsl::lexer::{Kind, Token, tokenize};
use crate::{AuthoringError, ParseBudget, SourceSpan};
use pse_ids::SemanticId;
use pse_model::generated::identities::DeclarationId;
use std::collections::{BTreeMap, BTreeSet};
use winnow::stream::LocatingSlice;

type Result<T> = std::result::Result<T, AuthoringError>;
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
    if text.len() as u64 > budget.max_bytes || text.len() > u32::MAX as usize {
        return Err(AuthoringError::Budget {
            limit: "bytes",
            allowed: budget.max_bytes.min(u64::from(u32::MAX)),
            needed: text.len() as u64,
        });
    }
    let tokens = tokenize(text).map_err(|e| match e {
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
    })?;
    let mut parser = Cursor {
        tokens,
        text,
        pos: 0,
        document,
        policy,
        budget,
        rows: Vec::new(),
        ids: BTreeSet::new(),
    };
    parser.block(None, 0, false)?;
    Ok(parser.rows)
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
        if token.kind == Kind::Quoted {
            crate::grammar::quoted(&mut LocatingSlice::new(token.text))
                .map_err(|_| self.error("quoted string"))
        } else {
            Ok(token.text.into())
        }
    }
    fn path(&mut self) -> Result<String> {
        let mut name = self.word()?;
        while self.eat(".") {
            name.push('.');
            name.push_str(&self.word()?);
        }
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
                ")" | "]" | "}" => {
                    if stack.pop() != Some(t) {
                        return Err(self.error("balanced expression"));
                    }
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
        Ok(self.text[start..end].trim().into())
    }
    fn type_name(&mut self, stops: &[&str]) -> Result<String> {
        let begin = self.pos;
        let mut angle = 0;
        let mut square = 0;
        let mut paren = 0;
        while self.pos < self.tokens.len() {
            // In a type, `Set<T>=value` closes the generic before assignment.
            if angle > 0 && self.peek() == ">=" {
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
            let t = self.peek();
            if angle == 0 && square == 0 && paren == 0 && stops.contains(&t) {
                break;
            }
            match t {
                "<" => angle += 1,
                ">" => angle -= 1,
                "[" => square += 1,
                "]" => square -= 1,
                "(" => paren += 1,
                ")" => paren -= 1,
                _ => {}
            }
            if angle < 0 || square < 0 || paren < 0 {
                return Err(self.error("balanced type"));
            }
            self.pos += 1;
        }
        if begin == self.pos || angle != 0 || square != 0 || paren != 0 {
            return Err(self.error("type"));
        }
        Ok(self.text
            [self.tokens[begin].span.start as usize..self.tokens[self.pos - 1].span.end as usize]
            .trim()
            .into())
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
    fn parameters(&mut self) -> Result<Vec<(String, String, Option<String>)>> {
        if !self.eat("(") {
            return Ok(Vec::new());
        }
        let mut result = Vec::new();
        if !self.eat(")") {
            loop {
                let name = self.word()?;
                self.expect(":")?;
                let ty = self.type_name(&["=", ",", ")"])?;
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
        if keyword == "entity" && self.eat("kind") {
            keyword = "entity_kind".into();
        }
        let anonymous = matches!(
            keyword.as_str(),
            "when" | "require" | "connect" | "contribute" | "expect" | "annotation"
        );
        let entity_kind = if keyword == "entity" {
            Some(self.path()?)
        } else {
            None
        };
        let name = if anonymous {
            format!("{keyword}#{ordinal}")
        } else {
            self.path()?
        };
        let id = match explicit {
            Some(id) => id,
            None if self.policy == IdentityPolicy::Explicit => {
                return Err(AuthoringError::MissingId {
                    at: SourceSpan::new(self.document, start, self.at()),
                    kind: keyword,
                    name,
                });
            }
            None => DeclarationId::from(pse_ids::named_id(
                parent.map_or(self.document, DeclarationId::as_id),
                &format!("{keyword}:{name}"),
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
                let type_name = self.type_name(&["from"])?;
                self.expect("from")?;
                let lower = self.until(&["to"])?;
                self.expect("to")?;
                let upper = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_continuous(AuthoredModelingDeclarationsFieldValueContinuous {
                    type_name,
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
                    "big_m" | "derived_big_m" => {
                        return Err(self.error("bigm(M) or bigm(derived)"));
                    }
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
                let parameters = self
                    .parameters()?
                    .into_iter()
                    .map(|(name, type_name, default_value)| {
                        AuthoredModelingDeclarationsFieldValueScopeParametersItem {
                            name,
                            type_name,
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
                let selection = if self.eat("select") {
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
                if selection.is_some() && keyword != "implicit"
                    || eligibility.is_some() && keyword != "regime"
                {
                    return Err(self
                        .error("selection belongs to implicit blocks and eligibility to regimes"));
                }
                let oracle = if self.eat("source") {
                    let reference = self.word()?;
                    self.expect("revision")?;
                    let revision = self.word()?;
                    Some(AuthoredModelingDeclarationsFieldValueScopeOracle {
                        reference,
                        revision,
                    })
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
                    let mut execution = None;
                    let mut stages = Vec::new();
                    let mut integration = None;
                    let mut initialization = None;
                    let mut expected_failure = None;
                    while !self.eat("}") {
                        if self.eat("run") {
                            if execution.is_some() {
                                return Err(self.error("one fixture execution mode"));
                            }
                            execution = Some(
                                self.word()?
                                    .parse()
                                    .map_err(|_| self.error("fixture execution mode"))?,
                            );
                            self.expect(";")?;
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
                            self.expect("(")?;
                            let mut samples = Vec::new();
                            while !self.eat(")") {
                                samples.push(self.until(&[",", ")"])?);
                                if self.eat(")") {
                                    break;
                                }
                                self.expect(",")?;
                            }
                            self.expect("relative")?;
                            self.expect("(")?;
                            let relative_tolerance = self
                                .until(&[")"])?
                                .parse()
                                .map_err(|_| self.error("relative tolerance number"))?;
                            self.expect(")")?;
                            self.expect("normalized_absolute")?;
                            self.expect("(")?;
                            let normalized_absolute_tolerance = self
                                .until(&[")"])?
                                .parse()
                                .map_err(|_| self.error("normalized absolute tolerance number"))?;
                            self.expect(")")?;
                            self.expect("step")?;
                            self.expect("(")?;
                            let initial_step = self.until(&[")"])?;
                            self.expect(")")?;
                            let quadrature_relative_tolerance = if self.eat("quadrature_relative") {
                                Some(self.fixture_literal()?)
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
                                },
                            );
                            continue;
                        }
                        if self.eat("failure") {
                            if expected_failure.is_some() {
                                return Err(self.error("one expected failure"));
                            }
                            let class = self
                                .word()?
                                .parse()
                                .map_err(|_| self.error("failure class"))?;
                            let rule = self.word()?;
                            self.expect(";")?;
                            expected_failure = Some(
                                AuthoredModelingDeclarationsFieldValueScopeFixtureExpectedFailure {
                                    class,
                                    rule,
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
                    Some(AuthoredModelingDeclarationsFieldValueScopeFixture {
                        degrees_of_freedom,
                        execution,
                        stages,
                        initialization,
                        integration,
                        expected_failure,
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
                    eligibility,
                    oracle,
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
                let version = self.word()?;
                semver::Version::parse(version.trim_start_matches('='))
                    .map_err(|_| self.error("exact package version"))?;
                let alias = if self.eat("as") {
                    Some(self.word()?)
                } else {
                    None
                };
                self.expect(";")?;
                Value::from_import(AuthoredModelingDeclarationsFieldValueImport { version, alias })
            }
            "enum" => {
                let members = self.names("{", "}")?;
                self.eat(";");
                Value::from_enum(AuthoredModelingDeclarationsFieldValueEnumeration { members })
            }
            "entity" => {
                let kind_name = entity_kind.ok_or_else(|| self.error("entity kind"))?;
                self.expect("{")?;
                let mut attributes = Vec::new();
                while !self.eat("}") {
                    let name = self.word()?;
                    self.expect("=")?;
                    let expression = self.until(&[",", ";", "}"])?;
                    attributes.push(AuthoredModelingDeclarationsFieldValueEntityAttributesItem {
                        name,
                        expression,
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
                    attributes,
                })
            }
            "fn" => {
                let type_parameters = self.names("<", ">")?;
                let arguments = self
                    .parameters()?
                    .into_iter()
                    .map(|(name, type_name, default_value)| {
                        AuthoredModelingDeclarationsFieldValueFunctionArgumentsItem {
                            name,
                            type_name,
                            default_value,
                        }
                    })
                    .collect();
                self.expect("->")?;
                let return_type = self.type_name(&["=", ";", "valid", "piecewise", "external"])?;
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
                    continuity,
                    type_parameters,
                    arguments,
                    return_type,
                    body,
                })
            }
            "param" | "var" | "let" | "alias" | "attribute" | "set" | "child" | "port"
            | "preset" | "scope" => {
                let indices = self
                    .indices()?
                    .into_iter()
                    .map(|(name, domain)| {
                        AuthoredModelingDeclarationsFieldValueBindingIndicesItem { name, domain }
                    })
                    .collect();
                let type_name = if self.eat(":") {
                    self.type_name(&["=", ";", "defined", "in"])?
                } else {
                    String::new()
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
                    type_name,
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
                    "attribute" => Value::from_attribute(b),
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
                let from = self.until(&["->"])?;
                self.expect("->")?;
                let to = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_connection(AuthoredModelingDeclarationsFieldValueConnection {
                    from,
                    to,
                })
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
                let type_name = self.type_name(&["conservation", "accounting"])?;
                let mode = self
                    .word()?
                    .parse()
                    .map_err(|_| self.error("conservation or accounting"))?;
                self.expect("tolerance")?;
                let tolerance = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_accumulator(AuthoredModelingDeclarationsFieldValueAccumulator {
                    indices,
                    type_name,
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
                let (transfer_id, transfer_side) = if self.eat("transfer") {
                    let id = self.word()?;
                    let side = self.word()?;
                    (Some(id), Some(side))
                } else {
                    (None, None)
                };
                self.expect("=")?;
                let expression = self.until(&[";"])?;
                self.expect(";")?;
                Value::from_contribution(AuthoredModelingDeclarationsFieldValueContribution {
                    indices,
                    target,
                    expression,
                    role,
                    transfer_id,
                    transfer_side,
                })
            }
            "table" => {
                let mut keys = Vec::new();
                if self.eat("[") {
                    loop {
                        let name = self.word()?;
                        self.expect(":")?;
                        let type_name = self.type_name(&[",", "]"])?;
                        keys.push(AuthoredModelingDeclarationsFieldValueTableKeysItem {
                            name,
                            type_name,
                            default_value: None,
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
                        let name = self.word()?;
                        self.expect(":")?;
                        let type_name = self.type_name(&[",", "}"])?;
                        columns.push(AuthoredModelingDeclarationsFieldValueTableColumnsItem {
                            name,
                            type_name,
                            default_value: None,
                        });
                        if self.eat("}") {
                            break;
                        }
                        self.expect(",")?;
                    }
                    String::new()
                } else {
                    self.type_name(&["missing", ";"])?
                };
                let missing_policy = if self.eat("missing") {
                    self.word()?
                } else {
                    "required".into()
                };
                let default_value = if missing_policy == "default" {
                    Some(self.until(&[";"])?)
                } else {
                    None
                };
                self.expect(";")?;
                Value::from_table(AuthoredModelingDeclarationsFieldValueTable {
                    keys,
                    columns,
                    value_type,
                    missing_policy,
                    default_value,
                })
            }
            "dataset" => {
                self.expect(":")?;
                let table = self.path()?;
                self.expect("source")?;
                let source = self.word()?;
                self.expect("{")?;
                let mut rows = Vec::new();
                while !self.eat("}") {
                    self.expect("[")?;
                    let mut keys = Vec::new();
                    loop {
                        keys.push(self.until(&[",", "]"])?);
                        if self.eat("]") {
                            break;
                        }
                        self.expect(",")?;
                    }
                    self.expect("=")?;
                    let values = if self.eat("[") {
                        let mut values = Vec::new();
                        loop {
                            values.push(self.until(&[",", "]"])?);
                            if self.eat("]") {
                                break;
                            }
                            self.expect(",")?;
                        }
                        values
                    } else {
                        vec![self.until(&[";"])?]
                    };
                    self.expect(";")?;
                    rows.push(AuthoredModelingDeclarationsFieldValueDatasetRowsItem {
                        keys,
                        values,
                    });
                }
                self.eat(";");
                Value::from_dataset(AuthoredModelingDeclarationsFieldValueDataset {
                    table,
                    source,
                    rows,
                })
            }
            "annotation" => {
                let annotation_type = self.word()?;
                let target = self.until(&["("])?;
                self.expect("(")?;
                let mut arguments = Vec::new();
                if !self.eat(")") {
                    loop {
                        arguments.push(self.until(&[",", ")"])?);
                        if self.eat(")") {
                            break;
                        }
                        self.expect(",")?;
                    }
                }
                self.expect(";")?;
                Value::from_annotation(AuthoredModelingDeclarationsFieldValueAnnotation {
                    annotation_type,
                    target,
                    arguments,
                })
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
        self.rows[position].source_end = i64::from(
            self.tokens
                .get(self.pos.saturating_sub(1))
                .map_or(self.at(), |t| t.span.end),
        );
        Ok(())
    }
}
