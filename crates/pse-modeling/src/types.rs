// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical and declaration-reference types; scientific names are admitted data.
use crate::{Result, invalid};
use pse_ids::SemanticId;
use pse_quantity::{QuantityRegistry, QuantityTypeId, Ratio, scheme::Scheme};
use std::collections::{BTreeMap, BTreeSet};

/// A checked language type. Kind and subject references never use scientific enums.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    /// Predicate value.
    Boolean,
    /// Exact integer value.
    Integer,
    /// Text or label.
    Text,
    /// Complete physical type scheme.
    Quantity(Scheme),
    /// Member of a declared entity kind.
    Entity(SemanticId),
    /// Member of a package enumeration.
    Enum(SemanticId),
    /// Finite ordered membership.
    Set(Box<Type>),
    /// A continuous coordinate axis, realized before finite expansion.
    Continuous(SemanticId, Box<Type>),
    /// Ordered typed product coordinate.
    Tuple(Vec<Type>),
    /// Typed table declaration.
    Table(SemanticId),
    /// A row in a typed table.
    Row(SemanticId),
    /// Definition reference.
    Definition(SemanticId),
    /// Interface-typed slot.
    Interface(SemanticId),
    /// Explicit pure function reference with named arguments and result.
    Function {
        arguments: Vec<(String, Type)>,
        result: Box<Type>,
    },
    /// Explicit optional value.
    Optional(Box<Type>),
    /// Indexed values retain coordinate kind identities; membership is a structural binding.
    Indexed {
        element: Box<Type>,
        axes: Vec<SemanticId>,
    },
}
/// Physical context supplied by admission; no registry is inferred from source literals.
pub struct TypeContext<'a> {
    /// Fully admitted reference physical registry.
    pub quantities: &'a QuantityRegistry,
    /// Admitted physical prerequisites; each use checks its actual operand contracts.
    pub preconditions: &'a pse_quantity::PhysicalPreconditions,
    /// Package names bound to complete physical types.
    pub names: &'a BTreeMap<String, QuantityTypeId>,
}
impl TypeContext<'_> {
    /// Resolve a type against the current lexical declaration and quantity environment.
    /// # Errors
    /// Unknown types, malformed schemes or invalid axes.
    pub fn resolve(
        &self,
        text: &str,
        variables: &BTreeSet<String>,
        names: &BTreeMap<String, Type>,
        at: SemanticId,
    ) -> Result<Type> {
        let text = text.trim();
        if let Some(signature) = text.strip_prefix("Fn(") {
            let mut depth = 1;
            let mut close = None;
            for (i, ch) in signature.char_indices() {
                match ch {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            close = Some(i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let close = close.ok_or_else(|| invalid(at, "unterminated function type"))?;
            let result = signature[close + 1..]
                .trim()
                .strip_prefix("->")
                .ok_or_else(|| invalid(at, "function type needs a result"))?;
            let raw = &signature[..close];
            let mut parts = Vec::new();
            let mut begin = 0;
            let mut depth = 0;
            for (i, ch) in raw.char_indices() {
                match ch {
                    '<' | '[' | '(' => depth += 1,
                    '>' | ']' | ')' => depth -= 1,
                    ',' if depth == 0 => {
                        parts.push(&raw[begin..i]);
                        begin = i + 1;
                    }
                    _ => {}
                }
            }
            if !raw.trim().is_empty() {
                parts.push(&raw[begin..]);
            }
            let mut seen = BTreeSet::new();
            let arguments = parts
                .into_iter()
                .map(|part| {
                    let (name, ty) = part.split_once(':').ok_or_else(|| {
                        invalid(at, "function type argument requires name and type")
                    })?;
                    let name = name.trim();
                    if name.is_empty() || !seen.insert(name) {
                        return Err(invalid(at, "duplicate or empty function argument"));
                    }
                    Ok((name.into(), self.resolve(ty, variables, names, at)?))
                })
                .collect::<Result<_>>()?;
            return Ok(Type::Function {
                arguments,
                result: Box::new(self.resolve(result, variables, names, at)?),
            });
        }
        if let Some(inner) = text.strip_suffix('?') {
            return Ok(Type::Optional(Box::new(
                self.resolve(inner, variables, names, at)?,
            )));
        }
        for (prefix, kind) in [("Set<", 0), ("Row<", 1), ("Table<", 2)] {
            if let Some(inner) = text.strip_prefix(prefix).and_then(|s| s.strip_suffix('>')) {
                let value = self.resolve(inner, variables, names, at)?;
                return match (kind, value) {
                    (0, value) => Ok(Type::Set(Box::new(value))),
                    (1, Type::Table(id)) => Ok(Type::Row(id)),
                    (2, Type::Table(id)) => Ok(Type::Table(id)),
                    _ => Err(invalid(at, "table reference required")),
                };
            }
        }
        if let Some(inner) = text
            .strip_prefix("Tuple<")
            .and_then(|s| s.strip_suffix('>'))
        {
            let mut parts = Vec::new();
            let mut depth = 0;
            let mut begin = 0;
            for (i, ch) in inner.char_indices() {
                match ch {
                    '<' | '[' | '(' => depth += 1,
                    '>' | ']' | ')' => depth -= 1,
                    ',' if depth == 0 => {
                        parts.push(&inner[begin..i]);
                        begin = i + 1;
                    }
                    _ => {}
                }
            }
            parts.push(&inner[begin..]);
            return Ok(Type::Tuple(
                parts
                    .into_iter()
                    .map(|p| self.resolve(p, variables, names, at))
                    .collect::<Result<_>>()?,
            ));
        }
        if let Some(open) = text.find('[') {
            if !text.ends_with(']') {
                return Err(invalid(at, "unterminated indexed type"));
            }
            let element = self.resolve(&text[..open], variables, names, at)?;
            let axes = text[open + 1..text.len() - 1]
                .split(',')
                .map(|name| match names.get(name.trim()) {
                    Some(Type::Entity(id) | Type::Enum(id) | Type::Definition(id)) => Ok(*id),
                    _ => Err(invalid(at, format!("unknown index domain {}", name.trim()))),
                })
                .collect::<Result<_>>()?;
            return Ok(Type::Indexed {
                element: Box::new(element),
                axes,
            });
        }
        match text {
            "Boolean" => return Ok(Type::Boolean),
            // `Count` names the physical count kind of an integer decision (ADR-0103).
            "Integer" => return Ok(Type::Integer),
            "Text" => return Ok(Type::Text),
            _ => {}
        }
        if let Some(value) = names.get(text) {
            return Ok(value.clone());
        }
        let scheme = self.scheme(text, variables, names, at)?;
        fn closed(scheme: &Scheme) -> bool {
            match scheme {
                Scheme::Concrete(_) => true,
                Scheme::Variable(_) => false,
                Scheme::Delta(value) | Scheme::Power(value, _) => closed(value),
                Scheme::Product(a, b) | Scheme::Quotient(a, b) => closed(a) && closed(b),
            }
        }
        Ok(Type::Quantity(if closed(&scheme) {
            Scheme::Concrete(
                scheme
                    .resolve_with_evidence(self.quantities, &BTreeMap::new(), self.preconditions)
                    .map_err(|e| invalid(at, e.to_string()))?,
            )
        } else {
            scheme
        }))
    }
    fn scheme(
        &self,
        text: &str,
        variables: &BTreeSet<String>,
        names: &BTreeMap<String, Type>,
        at: SemanticId,
    ) -> Result<Scheme> {
        let text = text.trim();
        let mut depth = 0;
        for (i, c) in text.char_indices().rev() {
            match c {
                '>' | ')' => depth += 1,
                '<' | '(' => depth -= 1,
                '*' | '/' if depth == 0 => {
                    let a = Box::new(self.scheme(&text[..i], variables, names, at)?);
                    let b = Box::new(self.scheme(&text[i + 1..], variables, names, at)?);
                    return Ok(if c == '*' {
                        Scheme::Product(a, b)
                    } else {
                        Scheme::Quotient(a, b)
                    });
                }
                _ => {}
            }
        }
        if let Some((base, exponent)) = text.rsplit_once('^') {
            let exponent = exponent.trim().trim_matches(['(', ')']);
            let (num, den) = exponent.split_once('/').unwrap_or((exponent, "1"));
            let exponent = Ratio::new(
                num.parse().map_err(|_| invalid(at, "rational exponent"))?,
                den.parse().map_err(|_| invalid(at, "rational exponent"))?,
            )
            .map_err(|e| invalid(at, e.to_string()))?;
            return Ok(Scheme::Power(
                Box::new(self.scheme(base, variables, names, at)?),
                exponent,
            ));
        }
        if let Some(inner) = text
            .strip_prefix("Delta<")
            .or_else(|| text.strip_prefix("Δ<"))
            .and_then(|s| s.strip_suffix('>'))
        {
            return Ok(Scheme::Delta(Box::new(
                self.scheme(inner, variables, names, at)?,
            )));
        }
        if variables.contains(text) {
            return Ok(Scheme::Variable(text.into()));
        }
        if let Some(Type::Quantity(scheme)) = names.get(text) {
            return Ok(scheme.clone());
        }
        self.names
            .get(text)
            .filter(|_| !text.contains('.'))
            .copied()
            .map(Scheme::Concrete)
            .ok_or_else(|| invalid(at, format!("unknown type {text}")))
    }
}
