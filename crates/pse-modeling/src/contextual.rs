// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Nominal physical roles and owner-relative transfer admission (Plan 25a).
use crate::{DeclarationId, InstanceId, Result, invalid, specialize::Value};
use std::collections::{BTreeMap, BTreeSet};

/// A scientific reference-change declaration admitted against its actual anchor functions.
/// Both functions receive the same physical temperature, pressure and component request.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceTranslation {
    /// Complete source point quantity.
    pub source: pse_quantity::QuantityTypeId,
    /// Complete target point quantity.
    pub target: pse_quantity::QuantityTypeId,
    /// Source scientific anchor function.
    pub source_anchor: DeclarationId,
    /// Target scientific anchor function.
    pub target_anchor: DeclarationId,
    /// Explicit common temperature expression.
    pub temperature: pse_authoring::dsl::Expr,
    /// Explicit common pressure expression.
    pub pressure: pse_authoring::dsl::Expr,
    /// Kind of the actual composition members.
    pub component_kind: DeclarationId,
    /// Scientific source and declaration identities, retained by specialization.
    pub provenance: Vec<pse_ids::SemanticId>,
    /// Actual common-state anchor calls selected for one finite specialization.
    pub anchors: Vec<ReferenceAnchorPair>,
}

/// Paired scientific function calls, evaluated by the ordinary mathematical compiler.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceAnchorPair {
    /// Actual composition member.
    pub member: pse_ids::SemanticId,
    /// Source and target point expressions, in their respective canonical coordinates.
    pub values: [pse_authoring::dsl::Expr; 2],
    /// Common physical temperature.
    pub temperature: pse_quantity::CanonicalMagnitude,
    /// Common physical pressure.
    pub pressure: pse_quantity::CanonicalMagnitude,
}

/// Admit scientific reference translations after ordinary anchor function signatures.
pub(crate) fn admit_translations(
    package: &mut crate::CheckedPackage,
    context: &crate::TypeContext<'_>,
) -> Result<()> {
    use crate::{PhysicalOperation, Type};
    use pse_quantity::{ScaleKind, scheme::Scheme};
    let rows = package
        .declarations
        .values()
        .filter(|row| row.value.reference_translation.is_some())
        .cloned()
        .collect::<Vec<_>>();
    for row in rows {
        let at = row.declaration_id;
        let declaration = row
            .value
            .reference_translation
            .as_ref()
            .ok_or_else(|| invalid(at, "translation payload"))?;
        let names = package.named_types(at);
        let arguments = declaration
            .arguments
            .iter()
            .map(|argument| {
                Ok((
                    argument.name.clone(),
                    context.resolve(&argument.r#type, &BTreeSet::new(), &names, at)?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        if arguments.len() != 3
            || arguments
                .iter()
                .map(|(name, _)| name)
                .collect::<BTreeSet<_>>()
                .len()
                != 3
            || declaration
                .arguments
                .iter()
                .any(|argument| argument.default_value.is_some())
        {
            return Err(invalid(
                at,
                "reference translation requires three distinct explicit arguments: value, composition and members",
            ));
        }
        let result = context.resolve(&declaration.return_type, &BTreeSet::new(), &names, at)?;
        let named = |ty: &Type| {
            ty.quantity_scheme()
                .ok_or_else(|| invalid(at, "reference translation point quantity required"))?
                .resolve_with_evidence(context.quantities, &BTreeMap::new(), context.preconditions)
                .map_err(|error| invalid(at, error.to_string()))
        };
        let (source, target) = (named(&arguments[0].1)?, named(&result)?);
        let a = context
            .quantities
            .quantity_type(source)
            .map_err(|error| invalid(at, error.to_string()))?;
        let b = context
            .quantities
            .quantity_type(target)
            .map_err(|error| invalid(at, error.to_string()))?;
        let mut expected = a.key.clone();
        expected.reference_state = b.key.reference_state;
        if expected != b.key
            || a.key.scale_kind != ScaleKind::Point
            || a.key.reference_state.is_none()
            || b.key.reference_state.is_none()
        {
            return Err(invalid(
                at,
                "reference translation retains kind, basis, shape and subject between declared datum points",
            ));
        }
        let Type::Set(member) = &arguments[2].1 else {
            return Err(invalid(
                at,
                "reference translation members require an entity set",
            ));
        };
        let Type::Entity(component_kind) = member.as_ref() else {
            return Err(invalid(
                at,
                "reference translation members require an entity kind",
            ));
        };
        let Type::Indexed { element, axes } = &arguments[1].1 else {
            return Err(invalid(
                at,
                "reference translation requires explicit indexed composition",
            ));
        };
        let weight = context
            .quantities
            .quantity_type(named(element)?)
            .map_err(|error| invalid(at, error.to_string()))?;
        if axes != &[*component_kind]
            || weight.key.subject_kind != Some(component_kind.as_id().into())
            || !context
                .quantities
                .kind(weight.key.kind)
                .map_err(|error| invalid(at, error.to_string()))?
                .dimension
                .is_dimensionless()
        {
            return Err(invalid(
                at,
                "composition weights must be dimensionless and identify the actual component axis",
            ));
        }
        let temperature = Type::Quantity(Scheme::Concrete(
            package.reference_attribute_type("temperature", at)?,
        ));
        let pressure = Type::Quantity(Scheme::Concrete(
            package.reference_attribute_type("pressure", at)?,
        ));
        let anchor = |name: &str, result: &Type| -> Result<DeclarationId> {
            let id = package
                .resolve(at, name)
                .ok_or_else(|| invalid(at, "scientific anchor function absent"))?;
            let function = package
                .functions
                .get(&id)
                .ok_or_else(|| invalid(at, "scientific anchor must be a pure function"))?;
            if !function.variables.is_empty()
                || function.external.is_some()
                || function.result != *result
                || function
                    .arguments
                    .iter()
                    .map(|(_, ty)| ty)
                    .collect::<Vec<_>>()
                    != vec![&temperature, &pressure, member.as_ref()]
            {
                return Err(invalid(
                    at,
                    "anchor signature must state common temperature, pressure and actual component, with its point result",
                ));
            }
            Ok(id)
        };
        let source_anchor = anchor(&declaration.source_anchor, &arguments[0].1)?;
        let target_anchor = anchor(&declaration.target_anchor, &result)?;
        let temperature = pse_authoring::dsl::parse_expr(&declaration.temperature)
            .map_err(|error| invalid(at, error.to_string()))?;
        let pressure = pse_authoring::dsl::parse_expr(&declaration.pressure)
            .map_err(|error| invalid(at, error.to_string()))?;
        // Entity facets and provenance are admitted after the function dependency graph.
        // Complete this private signature product from that admitted provenance below.
        let descriptor = ReferenceTranslation {
            source,
            target,
            source_anchor,
            target_anchor,
            temperature,
            pressure,
            component_kind: *component_kind,
            provenance: vec![],
            anchors: vec![],
        };
        package.types.insert(
            at,
            Type::Function {
                arguments: arguments.clone(),
                result: Box::new(result.clone()),
            },
        );
        package.functions.insert(
            at,
            crate::Function {
                applicability: Vec::new(),
                applicability_uses: Vec::new(),
                prerequisites: Vec::new(),
                physical_admissions: BTreeMap::new(),
                physical_operation: Some(PhysicalOperation::ReferenceTranslation(descriptor)),
                reduction: None,
                validity: None,
                envelopes: vec![],
                validity_reads: crate::envelope::Reads::default(),
                external: None,
                continuity: None,
                id: at,
                variables: BTreeSet::new(),
                arguments,
                result,
                body: None,
            },
        );
    }
    Ok(())
}

pub(crate) fn admit_translation_provenance(package: &mut crate::CheckedPackage) -> Result<()> {
    for (at, function) in &mut package.functions {
        if let Some(crate::PhysicalOperation::ReferenceTranslation(translation)) =
            &mut function.physical_operation
        {
            let provenance = package.provenance.get(at).ok_or_else(|| {
                invalid(*at, "reference translation scientific provenance absent")
            })?;
            translation.provenance = vec![
                provenance.source.as_id(),
                provenance.role.enumeration.as_id(),
                provenance.role.member,
                at.as_id(),
            ];
            translation
                .provenance
                .extend(provenance.lineage.iter().map(|(_, id)| id.as_id()));
        }
    }
    Ok(())
}

/// A boundary in a declaration or at one actual instantiated coordinate.
/// Equipment kinds never stand in for an actual boundary owner.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoundaryRef {
    /// Lexical declaration; resolved before contributing a numeric transfer.
    Declared(DeclarationId),
    /// One actual owner and ordered coordinate tuple.
    Bound {
        /// Actual instance.
        instance: InstanceId,
        /// Boundary declaration within that instance.
        declaration: DeclarationId,
        /// Actual coordinates in declaration order.
        coordinates: Vec<Value>,
    },
}
impl BoundaryRef {
    /// Stable explicit framing retains actual owner, declaration and coordinate order.
    pub fn frame(&self, h: &mut pse_ids::FramedHasher) {
        match self {
            Self::Declared(id) => {
                h.str("declared-boundary").id(&id.as_id());
            }
            Self::Bound {
                instance,
                declaration,
                coordinates,
            } => {
                h.str("bound-boundary")
                    .id(&instance.as_id())
                    .id(&declaration.as_id())
                    .u64(coordinates.len() as u64);
                for value in coordinates {
                    value.frame(h);
                }
            }
        }
    }
    /// The declaration, independently of realization.
    pub const fn declaration(&self) -> DeclarationId {
        match self {
            Self::Declared(id)
            | Self::Bound {
                declaration: id, ..
            } => *id,
        }
    }
    /// Whether this owner is instantiated rather than a lexical placeholder.
    pub const fn is_bound(&self) -> bool {
        matches!(self, Self::Bound { .. })
    }
}

/// Positive direction is a convention, not a nonnegativity constraint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransferDirection {
    /// Positive magnitude enters the owner.
    Into,
    /// Positive magnitude leaves the owner.
    OutOf,
}
impl TransferDirection {
    /// Coefficient of a signed magnitude in its owner's energy balance.
    pub const fn coefficient(self) -> i8 {
        match self {
            Self::Into => 1,
            Self::OutOf => -1,
        }
    }
}

/// Semantic roles retained above the physical numeric payload.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhysicalRefinement {
    /// One declared reduced coordinate slot; Scalar cannot substitute for it.
    Coordinate {
        /// Owning map declaration.
        map: DeclarationId,
        /// Ordered slot declaration.
        slot: DeclarationId,
    },
    /// Result of a reduced law awaiting its admitted scientific reconstruction.
    ReducedLaw {
        /// Map used by the reduced implementation.
        map: DeclarationId,
        /// Admitted normalization and physical reconstruction family.
        reconstruction: DeclarationId,
    },
    /// A signed datum-free transfer relative to an actual boundary.
    Transfer {
        /// Owner-relative boundary.
        boundary: BoundaryRef,
        /// Positive-direction convention.
        direction: TransferDirection,
    },
}

/// A declared isolated exchange between two distinct actual boundaries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairedExchange {
    /// Authored pairing identity, retained in preparation evidence.
    pub declaration: DeclarationId,
    /// One boundary.
    pub first: BoundaryRef,
    /// Its opposite boundary.
    pub second: BoundaryRef,
}
impl PairedExchange {
    /// Admit two actual, distinct endpoints; no graph-wide connectivity assumption.
    /// # Errors
    /// A lexical/unbound endpoint or a self-pair.
    pub fn admit(
        declaration: DeclarationId,
        first: BoundaryRef,
        second: BoundaryRef,
    ) -> Result<Self> {
        if !first.is_bound() || !second.is_bound() || first == second {
            return Err(invalid(
                declaration,
                "an exchange requires two distinct actual boundaries",
            ));
        }
        Ok(Self {
            declaration,
            first,
            second,
        })
    }
    /// Change endpoint and convention exactly once.
    /// # Errors
    /// The input does not name one of this exchange's endpoints.
    pub fn reflect(
        &self,
        source: &PhysicalRefinement,
        target: &BoundaryRef,
        direction: TransferDirection,
    ) -> Result<(PhysicalRefinement, i8)> {
        let PhysicalRefinement::Transfer {
            boundary,
            direction: source_direction,
        } = source
        else {
            return Err(invalid(
                self.declaration,
                "reflection requires a directed transfer",
            ));
        };
        if !((boundary == &self.first && target == &self.second)
            || (boundary == &self.second && target == &self.first))
        {
            return Err(invalid(
                self.declaration,
                "reflection must cross the declared opposite endpoints",
            ));
        }
        Ok((
            PhysicalRefinement::Transfer {
                boundary: target.clone(),
                direction,
            },
            -source_direction.coefficient() * direction.coefficient(),
        ))
    }
}
impl PhysicalRefinement {
    /// Versioned callers frame every nominal physical role explicitly, never with Debug.
    pub fn frame(&self, h: &mut pse_ids::FramedHasher) {
        match self {
            Self::Coordinate { map, slot } => {
                h.str("coordinate-slot").id(&map.as_id()).id(&slot.as_id());
            }
            Self::ReducedLaw {
                map,
                reconstruction,
            } => {
                h.str("reduced-law")
                    .id(&map.as_id())
                    .id(&reconstruction.as_id());
            }
            Self::Transfer {
                boundary,
                direction,
            } => {
                h.str("transfer");
                boundary.frame(h);
                h.str(match direction {
                    TransferDirection::Into => "into",
                    TransferDirection::OutOf => "out-of",
                });
            }
        }
    }
    /// Change convention at the same boundary; numeric negation alone never changes it.
    /// # Errors
    /// Not a transfer, or its owner has not been instantiated.
    pub fn reorient(&self, direction: TransferDirection, at: DeclarationId) -> Result<(Self, i8)> {
        let Self::Transfer {
            boundary,
            direction: source,
        } = self
        else {
            return Err(invalid(at, "reorientation requires a directed transfer"));
        };
        if !boundary.is_bound() {
            return Err(invalid(
                at,
                "reorientation requires an actual boundary owner",
            ));
        }
        Ok((
            Self::Transfer {
                boundary: boundary.clone(),
                direction,
            },
            source.coefficient() * direction.coefficient(),
        ))
    }
    /// Admit one energy contribution already expressed positive into this boundary.
    /// # Errors
    /// Wrong owner/convention, or an additional role/side sign would apply a second time.
    pub fn contribution(
        &self,
        target: &BoundaryRef,
        has_sign_modifier: bool,
        at: DeclarationId,
    ) -> Result<()> {
        if has_sign_modifier
            || !target.is_bound()
            || !matches!(self, Self::Transfer { boundary, direction: TransferDirection::Into } if boundary == target)
        {
            return Err(invalid(
                at,
                "a directed contribution requires Into for the actual boundary and no additional sign modifier",
            ));
        }
        Ok(())
    }
}

/// Admit arithmetic on nominal physical payloads before quantity algebra reads them.
/// Coordinate and reduced-law roles require their declaration-owned operation; ordinary
/// numeric sign changes retain a transfer's owner and direction.
/// # Errors
/// An operation would erase a nominal role or combine different boundary conventions.
pub fn arithmetic_refinement(
    request: &pse_quantity::infer::OpRequest<'_>,
    values: &[crate::Type],
    context: &crate::TypeContext<'_>,
    at: DeclarationId,
) -> Result<Option<PhysicalRefinement>> {
    use pse_quantity::infer::OpRequest as Op;
    let refinements = values
        .iter()
        .map(crate::Type::physical_refinement)
        .collect::<Vec<_>>();
    let Some(first) = refinements.iter().flatten().next().copied() else {
        return Ok(None);
    };
    if !matches!(first, PhysicalRefinement::Transfer { .. }) {
        return Err(invalid(
            at,
            "coordinate and reduced-law roles require their admitted physical operation",
        ));
    }
    let preserve = match request {
        Op::Neg | Op::Abs => values.len() == 1,
        Op::Add | Op::Sub => {
            values.len() == 2 && refinements.iter().all(|value| *value == Some(first))
        }
        Op::Mul | Op::Div if values.len() == 2 => {
            let position = usize::from(refinements[0].is_none());
            let other = 1 - position;
            refinements[other].is_none()
                && (!matches!(request, Op::Div) || position == 0)
                && values[other]
                    .quantity_scheme()
                    .and_then(|scheme| {
                        scheme
                            .resolve_contract_with_evidence(
                                context.quantities,
                                &BTreeMap::new(),
                                context.preconditions,
                            )
                            .ok()
                    })
                    .is_some_and(|contract| {
                        contract.named_id() == context.quantities.neutral_dimensionless()
                            && contract.indices().is_empty()
                    })
        }
        _ => false,
    };
    if !preserve {
        return Err(invalid(
            at,
            "arithmetic cannot erase or combine different transfer owners or positive directions",
        ));
    }
    Ok(Some(first.clone()))
}

/// Admit a declaration-owned contextual result without erasing the input's nominal role.
/// # Errors
/// The transfer owner/convention, translation anchors or exact physical contracts differ.
pub(crate) fn admit_operation_result(
    operation: &crate::PhysicalOperation,
    actual: &crate::Type,
    target: &crate::Type,
    context: &crate::TypeContext<'_>,
    at: DeclarationId,
) -> Result<pse_quantity::AdmittedOutputBoundary> {
    use crate::PhysicalOperation as O;
    let required = match operation {
        O::Transfer { source, .. } => source.as_ref(),
        O::TransferMagnitude { source } => Some(source),
        _ => None,
    };
    if actual.physical_refinement() != required {
        return Err(invalid(
            at,
            "contextual operation has the wrong input owner or nominal role",
        ));
    }
    let source = actual
        .quantity_scheme()
        .ok_or_else(|| invalid(at, "contextual operation requires a physical numeric input"))?
        .resolve_contract_with_evidence(context.quantities, &BTreeMap::new(), context.preconditions)
        .map_err(|error| invalid(at, error.to_string()))?;
    admit_operation_numeric_result(
        operation,
        &source,
        target,
        context.quantities,
        context.preconditions,
        at,
    )
}

/// Lower only the exact physical boundary already authorized by its contextual declaration.
/// # Errors
/// A detached cast, incompatible payload or missing scientific anchor evidence.
pub(crate) fn admit_operation_numeric_result(
    operation: &crate::PhysicalOperation,
    source: &pse_quantity::ResolvedPhysicalContract,
    target: &crate::Type,
    registry: &pse_quantity::QuantityRegistry,
    checker: &dyn pse_quantity::infer::InvariantChecker,
    at: DeclarationId,
) -> Result<pse_quantity::AdmittedOutputBoundary> {
    use crate::{PhysicalOperation as O, PhysicalRefinement as R};
    use pse_quantity::scheme::Scheme;
    let destination = target
        .quantity_scheme()
        .ok_or_else(|| invalid(at, "contextual output must be physical"))?
        .resolve_contract_with_evidence(registry, &BTreeMap::new(), checker)
        .map_err(|error| invalid(at, error.to_string()))?;
    match operation {
        O::Transfer {
            source: input,
            result,
            factor,
            exchange,
        } => {
            if target.physical_refinement() != Some(result) || !source.same_meaning(&destination) {
                return Err(invalid(
                    at,
                    "transfer transformation must retain its complete numeric payload",
                ));
            }
            let R::Transfer {
                boundary,
                direction,
            } = result
            else {
                return Err(invalid(at, "transfer result must name a boundary"));
            };
            if !boundary.is_bound() {
                return Err(invalid(at, "transfer result boundary is not instantiated"));
            }
            let expected = match input {
                None if exchange.is_none() => 1,
                Some(R::Transfer {
                    boundary: owner,
                    direction: from,
                }) if exchange.is_none() && owner == boundary => {
                    from.coefficient() * direction.coefficient()
                }
                Some(R::Transfer {
                    boundary: owner,
                    direction: from,
                }) if exchange.is_some() && owner != boundary && owner.is_bound() => {
                    -from.coefficient() * direction.coefficient()
                }
                _ => {
                    return Err(invalid(
                        at,
                        "transfer requires a same-owner reorientation or admitted paired reflection",
                    ));
                }
            };
            if *factor != expected {
                return Err(invalid(
                    at,
                    "transfer numerical orientation factor differs from its convention",
                ));
            }
            let q = registry
                .quantity_type(
                    destination
                        .require_named()
                        .map_err(|error| invalid(at, error.to_string()))?,
                )
                .map_err(|error| invalid(at, error.to_string()))?;
            let kind = registry
                .kind(q.key.kind)
                .map_err(|error| invalid(at, error.to_string()))?;
            if q.key.reference_state.is_some()
                || !kind.extensive
                || kind.addition_kind != pse_quantity::QuantityAdditionKind::Additive
            {
                return Err(invalid(
                    at,
                    "transfer payload must be datum-free extensive and additive",
                ));
            }
        }
        O::TransferMagnitude { source: refinement } => {
            let R::Transfer { boundary, .. } = refinement else {
                return Err(invalid(at, "contribution consumes a directed transfer"));
            };
            refinement.contribution(boundary, false, at)?;
            if target.physical_refinement().is_some() || !source.same_meaning(&destination) {
                return Err(invalid(
                    at,
                    "contribution consumes direction exactly once and preserves its payload",
                ));
            }
        }
        O::ReferenceTranslation(translation) => {
            translation.admitted(registry, at)?;
            let delta = |id| {
                Scheme::Delta(Box::new(Scheme::Concrete(id)))
                    .resolve_contract_with_evidence(registry, &BTreeMap::new(), checker)
                    .map_err(|error| invalid(at, error.to_string()))
            };
            if target.physical_refinement().is_some()
                || !source.same_meaning(&delta(translation.source)?)
                || !destination.same_meaning(&delta(translation.target)?)
            {
                return Err(invalid(
                    at,
                    "scientific translation changes only its selected source/target datum difference",
                ));
            }
        }
        O::ReferenceAnchorMean {
            translation,
            source: is_source,
            coefficient,
        } => {
            translation.admitted(registry, at)?;
            let expected = if *is_source {
                translation.source
            } else {
                translation.target
            };
            if registry.neutral_dimensionless() != Some(*coefficient)
                || target.physical_refinement().is_some()
                || destination.named_id() != Some(expected)
                || !source.same_meaning(&destination)
            {
                return Err(invalid(
                    at,
                    "weighted reference anchor must retain its declared datum point",
                ));
            }
        }
        _ => {
            return Err(invalid(
                at,
                "operation is not a contextual physical boundary",
            ));
        }
    }
    pse_quantity::AdmittedOutputBoundary::declared_contract(at.as_id(), source.clone(), destination)
        .map_err(|error| invalid(at, error.to_string()))
}

impl ReferenceTranslation {
    fn admitted(&self, registry: &pse_quantity::QuantityRegistry, at: DeclarationId) -> Result<()> {
        let first = self
            .anchors
            .first()
            .ok_or_else(|| invalid(at, "scientific anchor context is empty"))?;
        if self.anchors.iter().any(|anchor| {
            anchor.temperature != first.temperature || anchor.pressure != first.pressure
        }) {
            return Err(invalid(
                at,
                "scientific anchors must use one common physical state",
            ));
        }
        pse_quantity::ReferenceTranslation::admit_context(
            registry,
            self.source,
            self.target,
            first.temperature,
            first.pressure,
            &self
                .anchors
                .iter()
                .map(|anchor| anchor.member)
                .collect::<Vec<_>>(),
            &self.provenance,
        )
        .map_err(|error| invalid(at, error.to_string()))
    }
    fn frame(&self, h: &mut pse_ids::FramedHasher) {
        h.id(&self.source.as_id())
            .id(&self.target.as_id())
            .id(&self.source_anchor.as_id())
            .id(&self.target_anchor.as_id())
            .id(&self.component_kind.as_id());
        h.str(&pse_authoring::dsl::render_expr(&self.temperature))
            .str(&pse_authoring::dsl::render_expr(&self.pressure));
        h.u64(self.provenance.len() as u64);
        for id in &self.provenance {
            h.id(id);
        }
        h.u64(self.anchors.len() as u64);
        for anchor in &self.anchors {
            h.id(&anchor.member);
            for value in [anchor.temperature, anchor.pressure] {
                h.id(&value.quantity().as_id()).u64(value.bits());
            }
            for value in &anchor.values {
                h.str(&pse_authoring::dsl::render_expr(value));
            }
        }
    }
}

/// Check contextual intrinsic signatures before actual instance owners are selected.
pub(crate) fn call_type(
    name: &str,
    args: &[pse_authoring::dsl::Expr],
    env: &BTreeMap<String, crate::Type>,
    package: &crate::CheckedPackage,
    context: &crate::TypeContext<'_>,
    at: DeclarationId,
) -> Result<Option<crate::Type>> {
    use crate::Type;
    use pse_authoring::dsl;
    if !matches!(name, "transfer" | "reorient" | "reflect") {
        return Ok(None);
    }
    let (value, orientation) = match (name, args) {
        ("transfer", [value, _, orientation]) | ("reflect", [_, value, orientation]) => {
            (value, orientation)
        }
        ("reorient", [value, orientation]) => (value, orientation),
        _ => return Err(invalid(at, "contextual operation arity")),
    };
    let direction = match dsl::render_expr(orientation).as_str() {
        "Into" => TransferDirection::Into,
        "OutOf" => TransferDirection::OutOf,
        _ => return Err(invalid(at, "explicit Into or OutOf direction required")),
    };
    let actual = crate::expression::infer(value, env, package, context, at, None)?;
    let quantity = actual
        .quantity_scheme()
        .ok_or_else(|| invalid(at, "contextual intrinsic requires a physical payload"))?
        .clone();
    let boundary = if name == "transfer" {
        if actual.physical_refinement().is_some() {
            return Err(invalid(
                at,
                "an existing transfer requires reorient or reflect",
            ));
        }
        let contract = quantity
            .resolve_contract_with_evidence(
                context.quantities,
                &BTreeMap::new(),
                context.preconditions,
            )
            .map_err(|error| invalid(at, error.to_string()))?;
        let q = context
            .quantities
            .quantity_type(
                contract
                    .require_named()
                    .map_err(|error| invalid(at, error.to_string()))?,
            )
            .map_err(|error| invalid(at, error.to_string()))?;
        let kind = context
            .quantities
            .kind(q.key.kind)
            .map_err(|error| invalid(at, error.to_string()))?;
        if q.key.reference_state.is_some()
            || !kind.extensive
            || kind.addition_kind != pse_quantity::QuantityAdditionKind::Additive
        {
            return Err(invalid(
                at,
                "a transfer requires a datum-free extensive additive payload",
            ));
        }
        let Type::Boundary(id) =
            crate::expression::infer(&args[1], env, package, context, at, None)?
        else {
            return Err(invalid(at, "transfer target must be a declared boundary"));
        };
        BoundaryRef::Declared(id)
    } else {
        let Some(PhysicalRefinement::Transfer { boundary, .. }) = actual.physical_refinement()
        else {
            return Err(invalid(at, "reorientation/reflection requires a transfer"));
        };
        if name == "reorient" {
            boundary.clone()
        } else {
            let id = crate::expression::indexed_declaration_reference(
                &args[0], env, package, context, at,
            )?;
            let exchange = package.declarations[&id]
                .value
                .exchange
                .as_ref()
                .ok_or_else(|| invalid(at, "reflection names an exchange"))?;
            let mut endpoint_env =
                crate::expression::declaration_environment(package, context, id)?;
            for index in &exchange.indices {
                let domain = dsl::parse_expr(&index.domain)
                    .map_err(|error| invalid(at, error.to_string()))?;
                let (Type::Set(element) | Type::Continuous(_, element)) =
                    crate::expression::infer(&domain, &endpoint_env, package, context, id, None)?
                else {
                    return Err(invalid(at, "exchange index domain must be a set"));
                };
                endpoint_env.insert(index.name.clone(), *element);
            }
            let endpoint = |text: &str| -> Result<DeclarationId> {
                let expr = dsl::parse_expr(text).map_err(|error| invalid(at, error.to_string()))?;
                match crate::expression::infer(&expr, &endpoint_env, package, context, id, None)? {
                    Type::Boundary(id) => Ok(id),
                    _ => Err(invalid(at, "exchange endpoint requires a boundary")),
                }
            };
            let (first, second) = (endpoint(&exchange.from)?, endpoint(&exchange.to)?);
            let selected = if boundary.declaration() == first {
                second
            } else if boundary.declaration() == second {
                first
            } else {
                return Err(invalid(at, "transfer is not an exchange endpoint"));
            };
            BoundaryRef::Declared(selected)
        }
    };
    Ok(Some(Type::RefinedQuantity {
        quantity,
        refinement: PhysicalRefinement::Transfer {
            boundary,
            direction,
        },
    }))
}
impl crate::PhysicalOperation {
    /// Frame the scientific operation and its selected context, independently of rendering.
    pub fn frame(&self, h: &mut pse_ids::FramedHasher) {
        use crate::PhysicalOperation as O;
        match self {
            O::Coordinate { map, slot } => {
                h.str("coordinate").id(&map.as_id()).id(&slot.as_id());
            }
            O::ReducedLaw {
                map,
                reconstruction,
            } => {
                h.str("reduced-law")
                    .id(&map.as_id())
                    .id(&reconstruction.as_id());
            }
            O::Reconstruction {
                map,
                reconstruction,
                reference,
            } => {
                h.str("reconstruction")
                    .id(&map.as_id())
                    .id(&reconstruction.as_id())
                    .id(&reference.as_id());
            }
            O::Response { witness, potential } => {
                h.str("response").str(witness).bool(potential.is_some());
                if let Some(potential) = potential {
                    h.id(&potential.as_id());
                }
            }
            O::ReferenceTranslation(value) => {
                h.str("reference-translation");
                value.frame(h);
            }
            O::ReferenceAnchorMean {
                translation,
                source,
                coefficient,
            } => {
                h.str("reference-anchor-mean")
                    .bool(*source)
                    .id(&coefficient.as_id());
                translation.frame(h);
            }
            O::Transfer {
                source,
                result,
                factor,
                exchange,
            } => {
                h.str("transfer").bool(source.is_some());
                if let Some(source) = source {
                    source.frame(h);
                }
                result.frame(h);
                h.part(&factor.to_le_bytes()).bool(exchange.is_some());
                if let Some(exchange) = exchange {
                    h.id(&exchange.as_id());
                }
            }
            O::TransferMagnitude { source } => {
                h.str("transfer-contribution");
                source.frame(h);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_ids::SemanticId;
    fn id(n: u8) -> DeclarationId {
        SemanticId::from_bytes([n; 16]).into()
    }
    fn boundary(instance: u8, coordinate: i64) -> BoundaryRef {
        BoundaryRef::Bound {
            instance: id(instance).as_id().into(),
            declaration: id(9),
            coordinates: vec![Value::Integer(coordinate)],
        }
    }
    fn model(text: &str, root: &str) -> Result<crate::SpecializedModel> {
        let (registry, _) = crate::kernel_types::physical();
        let checks = pse_quantity::PhysicalPreconditions::new(
            pse_quantity::generated::standard_preconditions(),
        )
        .unwrap();
        let context = crate::TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: &registry,
            preconditions: &checks,
            scope: &crate::PhysicalScope::default(),
        };
        let package = crate::check(&crate::kernel_types::try_source(text)?, &context)?;
        crate::specialize(
            &package,
            package.names[root],
            InstanceId::from_bytes([7; 16]),
            &crate::Bindings::default(),
            crate::Limits::default(),
        )
    }
    #[test]
    fn directed_transfers_bind_actual_instances_and_consume_one_orientation() {
        let text = r#"package p {
          def Cell { boundary surface; var heat:Transfer<EnergyTransferRate,surface,Into>;
            accumulate energy:Power boundary surface accounting tolerance 0.001{W};
            contribute energy role directed=heat;
          }
          def Root { child hot:Cell=Cell(); child cold:Cell=Cell();
            exchange pair between hot.surface and cold.surface;
            eq reflection:hot.heat==reflect(pair,cold.heat,Into);
          }
        }"#;
        let prepared = model(text, "p.Root").unwrap();
        assert_eq!(prepared.exchanges.len(), 1);
        let closures = prepared.closures.values().collect::<Vec<_>>();
        assert_eq!(closures.len(), 2);
        assert_ne!(closures[0].boundary, closures[1].boundary);
        assert!(
            closures
                .iter()
                .all(|closure| closure.terms.len() == 1 && closure.terms[0].sign() == 1.0)
        );
        assert!(prepared.functions.values().any(|function| matches!(
            function.physical_operation,
            Some(crate::PhysicalOperation::Transfer {
                factor: -1,
                exchange: Some(_),
                ..
            })
        )));
        assert!(
            model(
                &text.replace("role directed=heat", "role negative=heat"),
                "p.Root"
            )
            .is_err()
        );
        assert!(
            model(
                &text.replace("role directed=heat", "role directed=reorient(heat,OutOf)"),
                "p.Root"
            )
            .is_err()
        );
        assert!(
            model(
                &text.replace("reflect(pair,cold.heat,Into)", "cold.heat"),
                "p.Root"
            )
            .is_err()
        );
    }
    #[test]
    fn conservation_zero_retains_material_energy_type_with_directed_transfer() {
        let text = r#"package p { def Root {
          boundary wall;
          var material:Power;
          var heat:EnergyTransferRate;
          accumulate energy:Power boundary wall conservation tolerance 0.001{W};
          contribute energy role inflow=material;
          contribute energy role directed=transfer(heat,wall,Into);
        }}"#;
        let prepared = model(text, "p.Root").unwrap();
        let closure = prepared.closures.values().next().unwrap();
        assert_eq!(closure.terms.len(), 2);
        assert!(
            prepared
                .equations
                .iter()
                .any(|row| { row.id == pse_ids::named_id(closure.id, "conservation") })
        );
        assert!(
            model(
                &text.replace("transfer(heat,wall,Into)", "transfer(heat,wall,OutOf)"),
                "p.Root"
            )
            .is_err()
        );
        // The generated equation context does not grant a type to an authored free zero.
        let untyped = text.replace(
            "var material:Power;",
            "var material:Power; eq free_zero:0{W}==transfer(heat,wall,Into);",
        );
        assert!(model(&untyped, "p.Root").is_err());
    }
    #[test]
    fn inherited_transfer_boundary_resolves_from_effective_interface_members() {
        let text = r#"package p {
          interface Base { boundary wall; }
          interface Other { boundary unrelated; }
          interface Derived extends Base { var heat:Transfer<EnergyTransferRate,wall,Into>; }
          def Root:Derived { let q:EnergyTransferRate=25{W}; eq directed:heat==transfer(q,wall,Into); }
        }"#;
        let prepared = model(text, "p.Root").unwrap();
        assert!(prepared.functions.values().any(|function| matches!(
            function.physical_operation,
            Some(crate::PhysicalOperation::Transfer {
                result: PhysicalRefinement::Transfer {
                    boundary: BoundaryRef::Bound { .. },
                    ..
                },
                ..
            })
        )));
        assert!(
            model(
                &text.replace(
                    "EnergyTransferRate,wall,Into",
                    "EnergyTransferRate,unrelated,Into"
                ),
                "p.Root"
            )
            .is_err()
        );
    }
    #[test]
    fn indexed_boundaries_and_exchanges_check_and_bind_actual_coordinates() {
        let text = r#"package p { def Root {
          entity kind position {} entity position a {} entity position b {}
          entity kind other {} entity other wrong {}
          set positions:Set<position>={a,b};
          boundary wall[i in positions]; boundary opposite[i in positions];
          exchange pair[i in positions] between wall[i] and opposite[i];
          let q:EnergyTransferRate=25{W};
          var heat[i in positions]:Transfer<EnergyTransferRate,wall,Into>;
          var cold_heat[i in positions]:Transfer<EnergyTransferRate,opposite,Into>;
          eq directed[j in positions]:heat[j]==transfer(q,wall[j],Into);
          eq reflected[j in positions]:cold_heat[j]==reflect(pair[j],transfer(q,wall[j],Into),Into);
        }}"#;
        let prepared = model(text, "p.Root").unwrap();
        assert_eq!(prepared.exchanges.len(), 2);
        let mut coordinates = BTreeSet::new();
        for exchange in prepared.exchanges.values() {
            let (
                BoundaryRef::Bound {
                    coordinates: first, ..
                },
                BoundaryRef::Bound {
                    coordinates: second,
                    ..
                },
            ) = (&exchange.first, &exchange.second)
            else {
                panic!("indexed exchange endpoints must be bound");
            };
            assert_eq!(first.len(), 1);
            assert_eq!(first, second);
            coordinates.insert(first.clone());
            assert!(prepared.functions.values().any(|function| matches!(
                &function.physical_operation,
                Some(crate::PhysicalOperation::Transfer { result: PhysicalRefinement::Transfer { boundary, .. }, factor: -1, exchange: Some(_), .. }) if boundary == &exchange.second
            )));
        }
        assert_eq!(coordinates.len(), 2);
        for invalid in [
            text.replace("wall[j],Into", "wall,Into"),
            text.replace("wall[j],Into", "wall[j,j],Into"),
            text.replace("wall[j],Into", "wall[wrong],Into"),
            text.replace("reflect(pair[j]", "reflect(pair"),
            text.replace("reflect(pair[j]", "reflect(pair[j,j]"),
            text.replace("reflect(pair[j]", "reflect(pair[wrong]"),
            text.replace("reflect(pair[j]", "reflect(pair[a]"),
        ] {
            assert!(model(&invalid, "p.Root").is_err(), "{invalid}");
        }
    }
    #[test]
    fn child_inherited_exchanges_use_the_declaration_namespace() {
        let text = r#"package p {
          entity kind position {} entity position a {} entity position b {}
          set members:Set<position>={a,b};
          interface Base {
            set positions:Set<position>={a,b};
            boundary wall[i in positions]; boundary opposite[i in positions];
            exchange pair[i in positions] between wall[i] and opposite[i];
          }
          def Cell:Base {}
          def Root {
            entity kind other {} entity other wrong {}
            set positions:Set<other>={wrong}; boundary wall[i in positions];
            child cell:Cell=Cell(); child other_cell:Cell=Cell(); var q:EnergyTransferRate;
            eq reflected[j in members]:transfer(q,cell.opposite[j],Into)==reflect(cell.pair[j],transfer(q,cell.wall[j],Into),Into);
          }
        }"#;
        let prepared = model(text, "p.Root").unwrap();
        assert_eq!(prepared.exchanges.len(), 4);
        let reflected = prepared
            .functions
            .values()
            .filter_map(|function| match &function.physical_operation {
                Some(crate::PhysicalOperation::Transfer {
                    result: PhysicalRefinement::Transfer { boundary, .. },
                    factor: -1,
                    exchange: Some(_),
                    ..
                }) => Some(boundary),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(reflected.len(), 2);
        for boundary in reflected {
            let BoundaryRef::Bound {
                instance,
                coordinates,
                ..
            } = boundary
            else {
                panic!("child reflection target must be bound");
            };
            assert_ne!(*instance, InstanceId::from_bytes([7; 16]));
            assert_eq!(coordinates.len(), 1);
            assert!(
                prepared
                    .exchanges
                    .values()
                    .any(|exchange| boundary == &exchange.second)
            );
        }
        for invalid in [
            text.replace("reflect(cell.pair[j]", "reflect(cell.pair"),
            text.replace("reflect(cell.pair[j]", "reflect(cell.pair[wrong]"),
            text.replace("reflect(cell.pair[j]", "reflect(cell.pair[a]"),
            text.replace("transfer(q,cell.wall[j]", "transfer(q,wall[wrong]"),
            text.replace("transfer(q,cell.wall[j]", "transfer(q,other_cell.wall[j]"),
        ] {
            assert!(model(&invalid, "p.Root").is_err(), "{invalid}");
        }
    }
    #[test]
    fn all_transfer_conventions_reflect_and_reorient_once() {
        let (hot, cold) = (boundary(1, 0), boundary(2, 0));
        let pair = PairedExchange::admit(id(10), hot.clone(), cold.clone()).unwrap();
        for source_direction in [TransferDirection::Into, TransferDirection::OutOf] {
            for target_direction in [TransferDirection::Into, TransferDirection::OutOf] {
                let source = PhysicalRefinement::Transfer {
                    boundary: hot.clone(),
                    direction: source_direction,
                };
                let (same, same_sign) = source.reorient(target_direction, id(11)).unwrap();
                let (opposite, opposite_sign) =
                    pair.reflect(&source, &cold, target_direction).unwrap();
                for magnitude in [-25.0, 0.0, 25.0] {
                    assert_eq!(
                        f64::from(source_direction.coefficient()) * magnitude,
                        f64::from(target_direction.coefficient() * same_sign) * magnitude
                    );
                    assert_eq!(
                        f64::from(source_direction.coefficient()) * magnitude
                            + f64::from(target_direction.coefficient() * opposite_sign) * magnitude,
                        0.0
                    );
                }
                assert_eq!(
                    same,
                    PhysicalRefinement::Transfer {
                        boundary: hot.clone(),
                        direction: target_direction
                    }
                );
                assert_eq!(
                    opposite,
                    PhysicalRefinement::Transfer {
                        boundary: cold.clone(),
                        direction: target_direction
                    }
                );
            }
        }
    }
    #[test]
    fn transfers_refuse_wrong_owner_coordinate_and_duplicate_sign() {
        let owner = boundary(1, 0);
        let into = PhysicalRefinement::Transfer {
            boundary: owner.clone(),
            direction: TransferDirection::Into,
        };
        into.contribution(&owner, false, id(11)).unwrap();
        assert!(into.contribution(&owner, true, id(11)).is_err());
        assert!(into.contribution(&boundary(2, 0), false, id(11)).is_err());
        assert!(into.contribution(&boundary(1, 1), false, id(11)).is_err());
        let out = PhysicalRefinement::Transfer {
            boundary: owner.clone(),
            direction: TransferDirection::OutOf,
        };
        assert!(out.contribution(&owner, false, id(11)).is_err());
        assert!(PairedExchange::admit(id(10), owner.clone(), owner.clone()).is_err());
        assert!(PairedExchange::admit(id(10), owner, BoundaryRef::Declared(id(9))).is_err());
    }
}
