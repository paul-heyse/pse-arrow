// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical and declaration-reference types; scientific names are admitted data.
use crate::{Result, invalid};
use pse_authoring::language::{TypeNode, TypeNodeKind, TypeRef, TypeTree};
use pse_model::generated::identities::DeclarationId;
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
    /// A named quantity type of the physical document (ADR-0123 Outcome 6).
    QuantityType,
    /// A named reference state of the physical document, with its typed conditions
    /// (ADR-0123 Outcome 6).
    ReferenceState,
    /// Complete physical type scheme.
    Quantity(Scheme),
    /// Member of a declared entity kind.
    Entity(DeclarationId),
    /// Member of a package enumeration.
    Enum(DeclarationId),
    /// Finite ordered membership.
    Set(Box<Type>),
    /// A continuous coordinate axis, realized before finite expansion.
    Continuous(DeclarationId, Box<Type>),
    /// Ordered typed product coordinate.
    Tuple(Vec<Type>),
    /// Typed table declaration.
    Table(DeclarationId),
    /// A row in a typed table.
    Row(DeclarationId),
    /// Definition reference.
    Definition(DeclarationId),
    /// Interface-typed slot.
    Interface(DeclarationId),
    /// Explicit pure function reference with named arguments and result.
    Function {
        /// Argument names and types in declared order.
        arguments: Vec<(String, Type)>,
        /// Result type.
        result: Box<Type>,
    },
    /// Explicit optional value.
    Optional(Box<Type>),
    /// Indexed values retain coordinate kind identities; membership is a structural binding.
    Indexed {
        /// Type of each indexed element.
        element: Box<Type>,
        /// Coordinate kind of each index, in order.
        axes: Vec<DeclarationId>,
    },
}
/// Physical context supplied by admission; no registry is inferred from source literals.
#[derive(Debug)]
pub struct TypeContext<'a> {
    /// Fully admitted reference physical registry.
    pub quantities: &'a QuantityRegistry,
    /// Admitted physical prerequisites; each use checks its actual operand contracts.
    pub preconditions: &'a pse_quantity::PhysicalPreconditions,
    /// Package names bound to complete physical types.
    pub names: &'a BTreeMap<String, QuantityTypeId>,
}
impl TypeContext<'_> {
    /// Resolve a declared type arena against the current lexical declaration and quantity
    /// environment (ADR-0123 Outcome 1). Every node is walked once; names are joined path
    /// segments looked up in `names` and the physical names.
    /// # Errors
    /// A malformed arena, unknown names, undeclared variables, malformed schemes or axes.
    pub fn resolve(
        &self,
        nodes: &[TypeNode],
        variables: &BTreeSet<String>,
        names: &BTreeMap<String, Type>,
        at: DeclarationId,
    ) -> Result<Type> {
        let tree = TypeTree::new(nodes).map_err(|e| invalid(at, e.to_string()))?;
        self.node(tree.root(), variables, names, at)
    }
    fn node<'a>(
        &self,
        node: TypeRef<'a>,
        variables: &BTreeSet<String>,
        names: &BTreeMap<String, Type>,
        at: DeclarationId,
    ) -> Result<Type> {
        use TypeNodeKind as K;
        let only = |node: TypeRef<'a>| node.child(0).ok_or_else(|| invalid(at, "type child"));
        Ok(match node.kind() {
            K::Boolean => Type::Boolean,
            // `Count` names the physical count kind of an integer decision (ADR-0103).
            K::Integer => Type::Integer,
            K::Text => Type::Text,
            K::QuantityType => Type::QuantityType,
            K::ReferenceState => Type::ReferenceState,
            K::Function => {
                let children = node.children().collect::<Vec<_>>();
                let (result, arguments) = children
                    .split_last()
                    .ok_or_else(|| invalid(at, "function type needs a result"))?;
                Type::Function {
                    arguments: arguments
                        .iter()
                        .map(|argument| {
                            Ok((
                                argument.name().to_owned(),
                                self.node(only(*argument)?, variables, names, at)?,
                            ))
                        })
                        .collect::<Result<_>>()?,
                    result: Box::new(self.node(*result, variables, names, at)?),
                }
            }
            K::Argument => return Err(invalid(at, "an argument is not a type")),
            K::Optional => Type::Optional(Box::new(self.node(only(node)?, variables, names, at)?)),
            K::Set => Type::Set(Box::new(self.node(only(node)?, variables, names, at)?)),
            K::Row | K::Table => match self.node(only(node)?, variables, names, at)? {
                Type::Table(id) if node.kind() == K::Row => Type::Row(id),
                Type::Table(id) => Type::Table(id),
                _ => return Err(invalid(at, "table reference required")),
            },
            K::Tuple => Type::Tuple(
                node.children()
                    .map(|child| self.node(child, variables, names, at))
                    .collect::<Result<_>>()?,
            ),
            K::Indexed => {
                let mut children = node.children();
                let element = children
                    .next()
                    .ok_or_else(|| invalid(at, "indexed element"))?;
                Type::Indexed {
                    element: Box::new(self.node(element, variables, names, at)?),
                    axes: children
                        .map(|axis| {
                            let name = axis.path().join(".");
                            match names.get(&name) {
                                Some(Type::Entity(id) | Type::Enum(id) | Type::Definition(id)) => {
                                    Ok(*id)
                                }
                                _ => Err(invalid(at, format!("unknown index domain {name}"))),
                            }
                        })
                        .collect::<Result<_>>()?,
                }
            }
            K::Identifier => {
                return Err(invalid(
                    at,
                    format!(
                        "identifier scheme {} is not declared",
                        node.path().join(".")
                    ),
                ));
            }
            K::Named => {
                let name = node.path().join(".");
                if let Some(value) = names.get(&name) {
                    return Ok(value.clone());
                }
                self.quantity(self.scheme(node, variables, names, at)?, at)?
            }
            K::Variable | K::Delta | K::Product | K::Quotient | K::Power => {
                self.quantity(self.scheme(node, variables, names, at)?, at)?
            }
        })
    }
    /// A closed scheme resolves to its concrete quantity type; an open one stays a scheme.
    fn quantity(&self, scheme: Scheme, at: DeclarationId) -> Result<Type> {
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
        node: TypeRef<'_>,
        variables: &BTreeSet<String>,
        names: &BTreeMap<String, Type>,
        at: DeclarationId,
    ) -> Result<Scheme> {
        use TypeNodeKind as K;
        let operand = |position: usize| {
            node.child(position)
                .ok_or_else(|| invalid(at, "physical type operand"))
                .and_then(|child| self.scheme(child, variables, names, at))
                .map(Box::new)
        };
        Ok(match node.kind() {
            K::Product => Scheme::Product(operand(0)?, operand(1)?),
            K::Quotient => Scheme::Quotient(operand(0)?, operand(1)?),
            K::Delta => Scheme::Delta(operand(0)?),
            K::Power => {
                let (num, den) = node
                    .exponent()
                    .ok_or_else(|| invalid(at, "rational exponent"))?;
                Scheme::Power(
                    operand(0)?,
                    Ratio::from_parts(num, den).map_err(|e| invalid(at, e.to_string()))?,
                )
            }
            K::Variable => {
                if !variables.contains(node.name()) {
                    return Err(invalid(
                        at,
                        format!("type variable {} is not in scope", node.name()),
                    ));
                }
                Scheme::Variable(node.name().to_owned())
            }
            K::Named => {
                let name = node.path().join(".");
                match names.get(&name) {
                    Some(Type::Quantity(scheme)) => scheme.clone(),
                    Some(_) => return Err(invalid(at, format!("{name} is not a physical type"))),
                    None => self
                        .names
                        .get(&name)
                        .filter(|_| node.path().len() == 1)
                        .copied()
                        .map(Scheme::Concrete)
                        .ok_or_else(|| invalid(at, format!("unknown type {name}")))?,
                }
            }
            _ => {
                return Err(invalid(
                    at,
                    format!("{} is not a physical type operand", node.kind().as_str()),
                ));
            }
        })
    }
}
