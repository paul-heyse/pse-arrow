// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit typed hard-domain guards. Evidence regions and permissions belong to applicability.
use crate::{CheckedPackage, DeclarationId, Function, Result, Type, TypeContext, invalid};
use pse_authoring::dsl::{
    CompareOp, Expr, ExprKind, Path, PathSegment, Predicate, PredicateKind, Span,
};
use pse_authoring::language::ModelingEnvelopeGuard;
use pse_ids::SemanticId;
use pse_model::generated::enums::ModelingEnvelopeExtent as Extent;
use std::collections::{BTreeMap, BTreeSet};

/// A validity envelope a relation or an entity kind declares as data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    /// The declaring table, or the entity kind's envelope declaration.
    pub owner: DeclarationId,
    /// The axis name guards refer to.
    pub axis: String,
    /// The axis quantity type; both bounds have it.
    pub ty: Type,
    /// The column or attribute holding the lower bound.
    pub lower: String,
    /// The column or attribute holding the upper bound.
    pub upper: String,
}

/// What one validity predicate of a specialized form reads (Plan 23 H5): the parameter
/// sets whose values bound it and the form's declared arguments it constrains. A rejection
/// names them as its lineage.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reads {
    /// Identities of the table rows or entities read, in argument order.
    pub sets: Vec<SemanticId>,
    /// Positions of the constrained arguments among the form's declared arguments.
    pub variables: Vec<u32>,
}
impl Reads {
    /// Frame the sets by identity and the variables by position.
    pub(crate) fn frame(&self, h: &mut pse_ids::FramedHasher) {
        h.str("reads").u64(self.sets.len() as u64);
        for set in &self.sets {
            h.id(set);
        }
        h.u64(self.variables.len() as u64);
        for variable in &self.variables {
            h.u64(u64::from(*variable));
        }
    }
    /// What `predicate` of `function` reads: the static row or entity arguments it names,
    /// by their identities in `statics`, and the physical arguments it names. Its local
    /// bindings are not arguments.
    pub(crate) fn of(
        p: &CheckedPackage,
        function: &Function,
        predicate: &Predicate,
        statics: &crate::specialize::Environment,
    ) -> Self {
        let named = predicate
            .paths()
            .into_iter()
            .filter_map(|path| path.segments.first().map(|s| s.name.as_str()))
            .collect::<BTreeSet<_>>();
        let mut reads = Self::default();
        for (position, (name, ty)) in function.arguments.iter().enumerate() {
            if !named.contains(name.as_str()) {
                continue;
            }
            match ty {
                Type::Quantity(_) => reads.variables.push(position as u32),
                _ => reads
                    .sets
                    .extend(statics.get(name).and_then(|value| p.set_identity(value))),
            }
        }
        reads
    }
}

/// A data-layer guard of a function: an envelope of a row or entity argument, and the
/// arguments or the integration interval it guards.
#[derive(Clone, Debug, PartialEq)]
pub struct Guard {
    /// The guarded envelope.
    pub envelope: Envelope,
    /// The argument carrying the row or entity whose bounds apply.
    pub carrier: String,
    /// One argument, or the interval between two.
    pub extent: Extent,
    /// The guarded arguments: one point, or the two endpoints of the interval.
    pub arguments: Vec<String>,
    /// Every guarded argument lies within the carrier's bounds: for an interval, both of its
    /// endpoints, which bounds the whole interval because an envelope is an interval.
    pub predicate: Predicate,
    /// The carrier's row or entity and the guarded arguments, once specialized.
    pub reads: Reads,
}

/// A quantity type as a refusal names it: its physical-document name when it has one.
pub(crate) fn type_name(c: &TypeContext<'_>, ty: &Type) -> String {
    if let Type::Quantity(scheme) = ty
        && let Ok(id) =
            scheme.resolve_with_evidence(c.quantities, &BTreeMap::new(), c.preconditions)
        && let Ok(quantity) = c.quantities.quantity_type(id)
        && let Some(name) = &quantity.name
    {
        return name.clone();
    }
    format!("{ty:?}")
}

/// An envelope of `what`: its axis has a quantity type and is bounded by two distinct
/// columns or attributes of exactly that type, whose types `bound` answers.
#[expect(
    clippy::too_many_arguments,
    reason = "one envelope's declared parts and the owner's bound types, checked together"
)]
pub(crate) fn resolve(
    c: &TypeContext<'_>,
    owner: DeclarationId,
    what: &str,
    axis: &str,
    ty: Type,
    lower: &str,
    upper: &str,
    bound: impl Fn(&str) -> Option<Type>,
) -> Result<Envelope> {
    if !matches!(ty, Type::Quantity(_)) {
        return Err(invalid(
            owner,
            format!(
                "envelope {axis} of {what} bounds a quantity; {} is not one",
                type_name(c, &ty)
            ),
        ));
    }
    if lower == upper {
        return Err(invalid(
            owner,
            format!("envelope {axis} of {what} is bounded by two distinct columns or attributes"),
        ));
    }
    for (side, name) in [("lower", lower), ("upper", upper)] {
        let actual = bound(name).ok_or_else(|| {
            invalid(
                owner,
                format!("envelope {axis} of {what} names {side} bound {name}, which {what} does not declare"),
            )
        })?;
        if actual != ty {
            return Err(invalid(
                owner,
                format!(
                    "envelope {axis} of {what} bounds a {}; its {side} bound {name} is a {}",
                    type_name(c, &ty),
                    type_name(c, &actual)
                ),
            ));
        }
    }
    Ok(Envelope {
        owner,
        axis: axis.to_owned(),
        ty,
        lower: lower.to_owned(),
        upper: upper.to_owned(),
    })
}

/// A row's or an entity's bounds of `envelope` are ordered: the envelope is an interval. A
/// refusal is attributed to `at`, the declaration supplying the data.
pub(crate) fn ordered(
    envelope: &Envelope,
    lower: &crate::specialize::Value,
    upper: &crate::specialize::Value,
    at: DeclarationId,
    holder: impl FnOnce() -> String,
) -> Result<()> {
    use crate::specialize::Value;
    match (lower, upper) {
        (Value::Number { bits: lo, .. }, Value::Number { bits: hi, .. })
            if f64::from_bits(*lo) <= f64::from_bits(*hi) =>
        {
            Ok(())
        }
        _ => Err(invalid(
            at,
            format!(
                "{} has envelope {} reversed: its lower bound {} exceeds its upper bound {}",
                holder(),
                envelope.axis,
                envelope.lower,
                envelope.upper
            ),
        )),
    }
}

/// Resolve every function's guards against the envelopes its row and entity arguments
/// carry, and generate each guard's domain predicate. Runs once tables and kinds are
/// admitted.
pub(crate) fn admit(p: &mut CheckedPackage, c: &TypeContext<'_>) -> Result<()> {
    let mut resolved = BTreeMap::new();
    for (id, function) in &p.functions {
        let Some(declared) = p
            .declarations
            .get(id)
            .and_then(|row| row.value.function.as_ref())
        else {
            continue;
        };
        let mut seen = BTreeSet::new();
        let mut guards = Vec::new();
        for declared in &declared.guards {
            let guard = guard(p, c, *id, function, declared)?;
            if !seen.insert((
                guard.envelope.owner,
                guard.carrier.clone(),
                guard.arguments.clone(),
            )) {
                return Err(invalid(
                    *id,
                    format!(
                        "{} guards envelope {}.{} of {} twice",
                        p.declarations[id].name,
                        declared.carrier,
                        declared.envelope,
                        guard.arguments.join(", ")
                    ),
                ));
            }
            guards.push(guard);
        }
        if !guards.is_empty() {
            resolved.insert(*id, guards);
        }
    }
    for (id, guards) in resolved {
        if let Some(function) = p.functions.get_mut(&id) {
            function.envelopes = guards;
        }
    }
    Ok(())
}

fn guard(
    p: &CheckedPackage,
    c: &TypeContext<'_>,
    id: DeclarationId,
    function: &Function,
    declared: &ModelingEnvelopeGuard,
) -> Result<Guard> {
    let name = &p.declarations[&id].name;
    let argument = |argument: &str| {
        function
            .arguments
            .iter()
            .find(|(n, _)| n == argument)
            .map(|(_, ty)| ty)
    };
    let carrier = argument(&declared.carrier).ok_or_else(|| {
        invalid(
            id,
            format!(
                "{name} guards an envelope of {}, which is not one of its arguments",
                declared.carrier
            ),
        )
    })?;
    let (envelopes, what) = match carrier {
        Type::Row(table) => (
            p.tables.get(table).map(|t| t.envelopes.as_slice()),
            format!("table {}", p.declarations[table].name),
        ),
        Type::Entity(kind) => (
            p.kinds.get(kind).map(|k| k.envelopes.as_slice()),
            format!("kind {}", p.declarations[kind].name),
        ),
        _ => {
            return Err(invalid(
                id,
                format!(
                    "{name} guards an envelope of {}, which is neither a row nor an entity argument",
                    declared.carrier
                ),
            ));
        }
    };
    let envelope = envelopes
        .into_iter()
        .flatten()
        .find(|e| e.axis == declared.envelope)
        .cloned()
        .ok_or_else(|| {
            invalid(
                id,
                format!(
                    "{name} guards envelope {} of {what}, which declares none",
                    declared.envelope
                ),
            )
        })?;
    let arity = match declared.extent {
        Extent::Point => 1,
        Extent::Interval => 2,
    };
    if declared.arguments.len() != arity
        || declared.extent == Extent::Interval && declared.arguments[0] == declared.arguments[1]
    {
        return Err(invalid(
            id,
            format!(
                "{name} guards envelope {} at one argument or over the interval between two distinct arguments",
                declared.envelope
            ),
        ));
    }
    for guarded in &declared.arguments {
        let ty = argument(guarded).ok_or_else(|| {
            invalid(
                id,
                format!("{name} guards {guarded}, which is not one of its arguments"),
            )
        })?;
        if *ty != envelope.ty || *guarded == declared.carrier {
            return Err(invalid(
                id,
                format!(
                    "{name} guards {guarded} by envelope {} of {what}, whose axis is a {}; {guarded} is a {}",
                    envelope.axis,
                    type_name(c, &envelope.ty),
                    type_name(c, ty)
                ),
            ));
        }
    }
    let predicate = declared
        .arguments
        .iter()
        .map(|guarded| within(guarded, &declared.carrier, &envelope))
        .reduce(and)
        .ok_or_else(|| invalid(id, "an envelope guard covers at least one argument"))?;
    Ok(Guard {
        envelope,
        carrier: declared.carrier.clone(),
        extent: declared.extent,
        arguments: declared.arguments.clone(),
        predicate,
        reads: Reads::default(),
    })
}

fn path(segments: &[&str]) -> Box<Expr> {
    Box::new(Expr {
        kind: ExprKind::Path(Path {
            segments: segments
                .iter()
                .map(|name| PathSegment {
                    name: (*name).to_owned(),
                    indices: vec![],
                })
                .collect(),
        }),
        span: Span::default(),
    })
}
fn and(a: Predicate, b: Predicate) -> Predicate {
    Predicate {
        kind: PredicateKind::And(Box::new(a), Box::new(b)),
        span: Span::default(),
    }
}
/// `argument >= carrier.lower and argument <= carrier.upper`.
fn within(argument: &str, carrier: &str, envelope: &Envelope) -> Predicate {
    let compare = |op, bound: &str| Predicate {
        kind: PredicateKind::Compare {
            op,
            lhs: path(&[argument]),
            rhs: path(&[carrier, bound]),
        },
        span: Span::default(),
    };
    and(
        compare(CompareOp::Ge, &envelope.lower),
        compare(CompareOp::Le, &envelope.upper),
    )
}
