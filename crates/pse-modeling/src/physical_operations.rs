// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declaration-owned authorizations for physical reconstruction and contextual changes.
use pse_model::generated::identities::DeclarationId;
use crate::{CheckedPackage, Function, Result, Type, TypeContext, invalid};
use pse_authoring::{dsl, language::TypeNode};
use pse_quantity::scheme::Scheme;
use std::collections::{BTreeMap, BTreeSet};

/// A checked scientific operation retained by specialization and mathematical lowering.
/// Its result role comes from the admitted declaration, never dimension-only inference.
#[derive(Clone, Debug, PartialEq)]
pub enum PhysicalOperation {
    /// A dimensionless slot of one declared physical coordinate map.
    Coordinate {
        /// Owning map.
        map: DeclarationId,
        /// Owning slot.
        slot: DeclarationId,
    },
    /// A reduced law may consume only slots of its declared map inside its scalar body.
    ReducedLaw {
        /// Owning coordinate map.
        map: DeclarationId,
        /// Admitted reconstruction family.
        reconstruction: DeclarationId,
    },
    /// Reconstruction of a reduced law through its declared map and normalization.
    Reconstruction {
        /// Physical coordinate map.
        map: DeclarationId,
        /// Reconstruction family.
        reconstruction: DeclarationId,
        /// Selected reference convention.
        reference: DeclarationId,
    },
    /// An authored response formula whose selected potential supplies its derivatives.
    Response {
        /// Formal potential function; specialization retains its actual selected identity.
        witness: String,
        /// Actual reconstructed potential selected by specialization.
        potential: Option<DeclarationId>,
    },
    /// A datum translation between independently anchored physical reference states.
    ReferenceTranslation(crate::contextual::ReferenceTranslation),
    /// A finite weighted reference anchor assembled from the exact selected members.
    ReferenceAnchorMean {
        /// Admitted paired scientific anchors at common physical conditions.
        translation: crate::contextual::ReferenceTranslation,
        /// Whether this mean belongs to the source datum (otherwise the target).
        source: bool,
        /// Neutral numerical coefficient used only after finite member admission.
        coefficient: pse_quantity::QuantityTypeId,
    },
    /// Admission, reorientation or paired reflection of a directed physical transfer.
    Transfer {
        /// Required input context, absent only for initial binding of a physical rate.
        source: Option<crate::PhysicalRefinement>,
        /// Established output context.
        result: crate::PhysicalRefinement,
        /// The single orientation factor applied to the numerical rate.
        factor: i8,
        /// Selected paired-exchange declaration for reflection.
        exchange: Option<DeclarationId>,
    },
    /// Consumption of an admitted transfer at its actual receiving ledger boundary.
    TransferMagnitude {
        /// Context consumed exactly once by the checked contribution.
        source: crate::PhysicalRefinement,
    },
}

fn arguments<'a>(
    values: impl Iterator<Item = (&'a str, &'a [TypeNode], Option<&'a str>)>,
    context: &TypeContext<'_>, names: &BTreeMap<String, Type>, at: DeclarationId,
) -> Result<Vec<(String, Type)>> {
    let mut seen = BTreeSet::new();
    values.map(|(name, nodes, default)| {
        if default.is_some() || !seen.insert(name) {
            return Err(invalid(at, "a physical operation has distinct explicit arguments without defaults"));
        }
        Ok((name.to_owned(), context.resolve(nodes, &BTreeSet::new(), names, at)?))
    }).collect()
}

fn function(id: DeclarationId, arguments: Vec<(String, Type)>, result: Type, body: &str, operation: PhysicalOperation) -> Result<Function> {
    Ok(Function {
        physical_operation: Some(operation), reduction: None, validity: None,
        envelopes: Vec::new(), validity_reads: crate::envelope::Reads::default(),
        external: None, continuity: None, id, variables: BTreeSet::new(), arguments,
        result, body: Some(dsl::parse_expr(body).map_err(|e| invalid(id, e.to_string()))?),
    })
}

fn insert(p: &mut CheckedPackage, function: Function) {
    p.types.insert(function.id, Type::Function { arguments: function.arguments.clone(), result: Box::new(function.result.clone()) });
    p.functions.insert(function.id, function);
}

/// Establish map slots and reconstruction families before ordinary function signatures.
/// # Errors
/// A declaration lacks its map, has malformed explicit arguments, or names an invalid witness.
pub(crate) fn admit_signatures(p: &mut CheckedPackage, context: &TypeContext<'_>) -> Result<()> {
    let rows = p.declarations.values().cloned().collect::<Vec<_>>();
    for row in &rows {
        if let Some(value) = &row.value.reconstruction {
            let map = p.resolve(row.declaration_id, &value.map).filter(|map| matches!(p.types.get(map), Some(Type::CoordinateMap(_))))
                .ok_or_else(|| invalid(row.declaration_id, "reconstruction names a declared coordinate map"))?;
            p.types.insert(row.declaration_id, Type::Reconstruction { declaration: row.declaration_id, map });
        }
    }
    let scalar = context.quantities.neutral_dimensionless().ok_or_else(|| invalid(pse_ids::SemanticId::NIL, "neutral dimensionless type absent"))?;
    for row in &rows {
        let id = row.declaration_id;
        if let Some(slot) = &row.value.coordinate_slot {
            let map = row.parent_id.ok_or_else(|| invalid(id, "a coordinate slot belongs to a map"))?;
            let declaration = p.declarations.get(&map).and_then(|r| r.value.coordinate_map.as_ref()).ok_or_else(|| invalid(id, "a coordinate slot belongs to a map"))?;
            let names = p.named_types(map);
            let mut args = arguments(declaration.arguments.iter().map(|a| (a.name.as_str(), a.r#type.as_slice(), a.default_value.as_deref())), context, &names, map)?;
            for index in &slot.indices {
                let Some(Type::Set(element)) = args.iter().find(|(name, _)| name == &index.domain).map(|(_, ty)| ty) else {
                    return Err(invalid(id, "a coordinate slot index ranges over an explicit finite-set argument"));
                };
                let element = (**element).clone();
                if args.iter().any(|(name, _)| name == &index.name) {
                    return Err(invalid(id, "a coordinate slot index has a distinct formal name"));
                }
                args.push((index.name.clone(), element));
            }
            let result = Type::RefinedQuantity { quantity: Scheme::Concrete(scalar), refinement: crate::PhysicalRefinement::Coordinate {map, slot:id} };
            let mut checked = function(id, args, result, &slot.expression, PhysicalOperation::Coordinate {map, slot:id})?;
            checked.validity = declaration.validity.as_ref().map(|text| dsl::parse_predicate(text).map_err(|e| invalid(map, e.to_string()))).transpose()?;
            insert(p, checked);
        }
    }
    for row in &rows {
        let id = row.declaration_id;
        if let Some(value) = &row.value.reconstruction {
            let Type::Reconstruction {map, ..} = p.types[&id] else { return Err(invalid(id, "missing reconstruction map")); };
            let names = p.named_types(id);
            let args = arguments(value.arguments.iter().map(|a| (a.name.as_str(), a.r#type.as_slice(), a.default_value.as_deref())), context, &names, id)?;
            let map_declaration = p.declarations[&map].value.coordinate_map.as_ref().ok_or_else(|| invalid(id, "missing coordinate map"))?;
            let map_args = arguments(map_declaration.arguments.iter().map(|a| (a.name.as_str(), a.r#type.as_slice(), a.default_value.as_deref())), context, &p.named_types(map), map)?;
            if args != map_args { return Err(invalid(id, "reconstruction arguments exactly match its coordinate map")); }
            let result = context.resolve(&value.return_type, &BTreeSet::new(), &names, id)?;
            if !matches!(result, Type::Quantity(_)) { return Err(invalid(id, "reconstruction declares a physical potential result")); }
            let reference = p.resolve(id, &value.reference).ok_or_else(|| invalid(id, "reconstruction reference convention is not declared"))?;
            let mut law_args = Vec::new();
            let mut call_args = Vec::new();
            for slot in p.children.get(&map).into_iter().flatten() {
                let declaration = &p.declarations[slot];
                let Some(indices) = declaration.value.coordinate_slot.as_ref().map(|slot| &slot.indices) else { return Err(invalid(map, "a coordinate map owns only coordinate slots")); };
                let checked = p.functions.get(slot).ok_or_else(|| invalid(id, "coordinate slot signature absent"))?;
                let element = checked.result.clone();
                let slot_type = if indices.is_empty() { element } else {
                    let axes = checked.arguments[args.len()..].iter().map(|(_, ty)| match ty {
                        Type::Entity(id) | Type::Enum(id) => Ok(*id),
                        _ => Err(invalid(*slot, "indexed coordinate slots require finite declared entity kinds")),
                    }).collect::<Result<_>>()?;
                    Type::Indexed {element: Box::new(element), axes}
                };
                law_args.push((declaration.name.clone(), slot_type));
                let path = p.names.iter().find_map(|(path, selected)| (*selected == *slot).then_some(path)).ok_or_else(|| invalid(*slot, "coordinate slot has no qualified identity"))?;
                call_args.push(format!("{path}({})", args.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>().join(",")));
            }
            if law_args.is_empty() { return Err(invalid(map, "a coordinate map has at least one declared slot")); }
            for (name, ty) in &args {
                if ty.quantity_scheme().is_none() && !matches!(ty, Type::Indexed {..}) {
                    law_args.push((name.clone(), ty.clone()));
                    call_args.push(name.clone());
                }
            }
            let reduced = Type::RefinedQuantity {quantity: Scheme::Concrete(scalar), refinement: crate::PhysicalRefinement::ReducedLaw {map, reconstruction:id}};
            let mut formals = vec![("reduced_law".into(), Type::Function {arguments:law_args, result:Box::new(reduced)})];
            formals.extend(args);
            let body = format!("({})*reduced_law({})", value.normalization, call_args.join(","));
            let checked = function(id, formals, result, &body, PhysicalOperation::Reconstruction {map, reconstruction:id, reference})?;
            p.functions.insert(id, checked);
        }
        if let Some(value) = &row.value.response {
            let names = p.named_types(id);
            let args = arguments(value.arguments.iter().map(|a| (a.name.as_str(), a.r#type.as_slice(), a.default_value.as_deref())), context, &names, id)?;
            let Some(Type::Function {result: potential, ..}) = args.iter().find(|(name, _)| name == &value.witness).map(|(_, ty)| ty) else {
                return Err(invalid(id, "a response witness is an explicit physical-potential function argument"));
            };
            if !matches!(potential.as_ref(), Type::Quantity(_)) {
                return Err(invalid(id, "a response witness returns a physical potential"));
            }
            let result = context.resolve(&value.return_type, &BTreeSet::new(), &names, id)?;
            if !matches!(result, Type::Quantity(_)) {
                return Err(invalid(id, "a response declares a physical result"));
            }
            let checked = function(id, args, result, &value.body, PhysicalOperation::Response {witness: value.witness.clone(), potential:None})?;
            let mut used = false;
            if let Some(body) = &checked.body {
                body.walk(|expr| match &expr.kind {
                    dsl::ExprKind::NamedCall {name, ..} if name == &value.witness => used = true,
                    dsl::ExprKind::Partial {function, ..} if function == &value.witness => used = true,
                    _ => {},
                });
            }
            if !used { return Err(invalid(id, "a response formula must consume its selected potential or its partial derivatives")); }
            insert(p, checked);
        }
    }
    Ok(())
}

/// Admit the numerical formal interface of a declared reduced law.
/// # Errors
/// A numeric argument is unrefined, belongs to another map or names a non-slot declaration.
pub(crate) fn admit_reduced_law(p: &CheckedPackage, function: &mut Function) -> Result<()> {
    let Some(crate::PhysicalRefinement::ReducedLaw {map, reconstruction}) = function.result.physical_refinement() else { return Ok(()); };
    let (map, reconstruction) = (*map, *reconstruction);
    fn argument(p: &CheckedPackage, ty: &Type, map: DeclarationId, at: DeclarationId, found: &mut bool) -> Result<()> {
        match ty {
            Type::RefinedQuantity {refinement: crate::PhysicalRefinement::Coordinate {map: owner, slot}, ..}
                if *owner == map && p.declarations.get(slot).is_some_and(|row| row.parent_id == Some(map) && row.value.coordinate_slot.is_some()) => { *found = true; Ok(()) },
            Type::Indexed {element, ..} => argument(p, element, map, at, found),
            Type::Quantity(_) | Type::RefinedQuantity {..} | Type::Function {..} => Err(invalid(at, "every numerical reduced-law argument is a declared slot of its own coordinate map")),
            _ => Ok(()),
        }
    }
    let mut found = false;
    for (_, ty) in &function.arguments { argument(p, ty, map, function.id, &mut found)?; }
    if !found { return Err(invalid(function.id, "a reduced law consumes its declared map coordinates")); }
    function.physical_operation = Some(PhysicalOperation::ReducedLaw {map, reconstruction});
    Ok(())
}

impl PhysicalOperation {
    /// Bind the actual response witness only when its reachable definition reconstructs
    /// a declared physical potential. Equal signatures alone confer no scientific role.
    /// # Errors
    /// The selected function has no reconstruction authority in its composed closure.
    pub(crate) fn bind_response_witness(&mut self, name: &str, selected: DeclarationId, p: &CheckedPackage, at: DeclarationId) -> Result<()> {
        let Self::Response {witness, potential} = self else { return Ok(()); };
        if witness != name { return Ok(()); }
        fn reconstructed(id: DeclarationId, p: &CheckedPackage, visited: &mut BTreeSet<DeclarationId>) -> bool {
            if !visited.insert(id) { return false; }
            let Some(function) = p.functions.get(&id) else { return false; };
            if matches!(function.physical_operation, Some(PhysicalOperation::Reconstruction {..})) { return true; }
            let mut calls = Vec::new();
            if let Some(body) = &function.body {
                body.walk(|expr| if let dsl::ExprKind::NamedCall {name, args} = &expr.kind {
                    let name = if name == "reconstruct" { args.first().map(dsl::render_expr) } else { Some(name.clone()) };
                    if let Some(called) = name.and_then(|name| p.resolve(id, &name)) { calls.push(called); }
                });
            }
            calls.into_iter().any(|called| reconstructed(called, p, visited))
        }
        if !reconstructed(selected, p, &mut BTreeSet::new()) {
            return Err(invalid(at, "a response witness must retain a declared physical reconstruction in its actual selected closure"));
        }
        *potential = Some(selected);
        Ok(())
    }

    /// Admit the checked body's physical result at this declaration-owned output boundary.
    /// # Errors
    /// Nominal context, physical dimensions or existing contextual obligations disagree.
    pub fn admit_result(&self, actual: &Type, target: &Type, context: &TypeContext<'_>, at: DeclarationId) -> Result<pse_quantity::AdmittedOutputBoundary> {
        if matches!(self, Self::ReferenceTranslation(_) | Self::ReferenceAnchorMean {..} | Self::Transfer {..} | Self::TransferMagnitude {..}) {
            return crate::contextual::admit_operation_result(self, actual, target, context, at);
        }
        if actual.physical_refinement().is_some() {
            return Err(invalid(at, "a physical operation must explicitly consume its input refinement"));
        }
        let source = actual.quantity_scheme().ok_or_else(|| invalid(at, "a physical operation requires a numerical body"))?
            .resolve_contract_with_evidence(context.quantities, &BTreeMap::new(), context.preconditions).map_err(|e| invalid(at, e.to_string()))?;
        self.admit_numeric_result(&source, target, context.quantities, context.preconditions, at)
    }

    /// Reuse retained declaration authority when lowering the checked physical body.
    /// # Errors
    /// The numerical contract differs from the operation's admitted physical output.
    pub fn admit_numeric_result(&self, source: &pse_quantity::ResolvedPhysicalContract, target: &Type, registry: &pse_quantity::QuantityRegistry, checker: &dyn pse_quantity::infer::InvariantChecker, at: DeclarationId) -> Result<pse_quantity::AdmittedOutputBoundary> {
        if matches!(self, Self::ReferenceTranslation(_) | Self::ReferenceAnchorMean {..} | Self::Transfer {..} | Self::TransferMagnitude {..}) {
            return crate::contextual::admit_operation_numeric_result(self, source, target, registry, checker, at);
        }
        let destination = target.quantity_scheme().ok_or_else(|| invalid(at, "a physical operation requires a numerical result"))?
            .resolve_contract_with_evidence(registry, &BTreeMap::new(), checker).map_err(|e| invalid(at, e.to_string()))?;
        let target_id = destination.require_named().map_err(|e| invalid(at, e.to_string()))?;
        match self {
            Self::Coordinate {map, slot} if target.physical_refinement() == Some(&crate::PhysicalRefinement::Coordinate {map:*map, slot:*slot}) => {
                // The declared map owns consumption of a dimensionless physical ratio.
                // Its complete source remains in the authorization; the boundary below
                // checks dimensions without mistaking TemperatureRatio for plain Scalar.
            }
            Self::ReducedLaw {map, reconstruction} if target.physical_refinement() == Some(&crate::PhysicalRefinement::ReducedLaw {map:*map, reconstruction:*reconstruction}) => {
                if source.named_id() != registry.neutral_dimensionless() {
                    return Err(invalid(at, "a reduced-law body produces a checked neutral scalar"));
                }
            }
            Self::Response {..} | Self::Reconstruction {..} if target.physical_refinement().is_none() => {
                let key = &registry.quantity_type(target_id).map_err(|e| invalid(at, e.to_string()))?.key;
                let bases = source.qualified_factors().iter().filter_map(|factor| factor.key.basis).collect::<BTreeSet<_>>();
                let consumes_basis = key.basis.is_none() && bases.len() <= 1
                    && source.representation().dimension().exponent(pse_quantity::BaseDimension::Amount).is_zero();
                if source.axes() != destination.axes() || source.indices() != destination.indices() || source.qualified_factors().iter().any(|factor| {
                    factor.key.basis.is_some_and(|basis| Some(basis) != key.basis && !consumes_basis)
                        || factor.key.reference_state.is_some_and(|reference| Some(reference) != key.reference_state)
                        || factor.key.subject_kind.is_some_and(|subject| Some(subject) != key.subject_kind)
                }) {
                    return Err(invalid(at, "a physical response cannot erase or replace existing basis, datum, subject or free binders"));
                }
            }
            _ => return Err(invalid(at, "physical operation result has the wrong nominal contract")),
        }
        pse_quantity::AdmittedOutputBoundary::declared(at.as_id(), source.clone(), target_id, registry).map_err(|e| invalid(at, e.to_string()))
    }

    /// Explicitly consume admitted numerical roles only inside this operation's body.
    /// The original signature remains nominal at every call boundary.
    pub fn body_arguments(&self, arguments: &[(String, Type)]) -> Vec<(String, Type)> {
        fn consume(ty: &Type, map: DeclarationId) -> Type {
            match ty {
                Type::RefinedQuantity {quantity, refinement: crate::PhysicalRefinement::Coordinate {map: owner, ..}} if *owner == map => Type::Quantity(quantity.clone()),
                Type::Indexed {element, axes} => Type::Indexed {element: Box::new(consume(element, map)), axes: axes.clone()},
                _ => ty.clone(),
            }
        }
        arguments.iter().map(|(name, ty)| (name.clone(), match self {
            Self::ReferenceAnchorMean {coefficient, ..} => Type::Quantity(Scheme::Concrete(*coefficient)),
            Self::ReducedLaw {map, ..} => consume(ty, *map),
            Self::Reconstruction {map, reconstruction, ..} => match ty {
                Type::Function {arguments, result} => match result.as_ref() {
                    Type::RefinedQuantity {quantity, refinement: crate::PhysicalRefinement::ReducedLaw {map: owner, reconstruction: family}} if owner == map && family == reconstruction => Type::Function {arguments:arguments.clone(), result:Box::new(Type::Quantity(quantity.clone()))},
                    _ => ty.clone(),
                },
                _ => ty.clone(),
            },
            _ => ty.clone(),
        })).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel_types::{physical, source};

    fn admitted(text: &str) -> Result<CheckedPackage> {
        let (registry, _) = physical();
        crate::check(&source(text), &TypeContext {formula_authority: None,quantities: &registry, preconditions: &pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions()).unwrap(), scope: &crate::PhysicalScope::default()})
    }

    #[test]
    fn coordinate_slots_are_nominal_and_reduced_bodies_consume_only_their_map() {
        let declarations = r#"package p {
          entity kind convention {}
          entity convention ideal {}
          coordinate map first(x: Length) { slot x = x/1{m}; }
          coordinate map second(x: Length) { slot x = x/1{m}; }
          reconstruction family for first(x: Length) -> Length reference ideal = 1{m};
          fn law(x: Coordinate<first.x>) -> Reduced<family> = x*x;
        }"#;
        let checked = admitted(declarations).unwrap();
        let law = checked.names["p.law"];
        assert!(matches!(checked.functions[&law].physical_operation, Some(PhysicalOperation::ReducedLaw {..})));
        let wrong_map = declarations.replace("fn law(x: Coordinate<first.x>)", "fn law(x: Coordinate<second.x>)");
        assert!(admitted(&wrong_map).is_err());
        let untyped = declarations.replace("fn law(x: Coordinate<first.x>)", "fn law(x: Scalar)");
        assert!(admitted(&untyped).is_err());
        let raw = declarations.replace("fn law(x: Coordinate<first.x>) -> Reduced<family>", "fn law(x: Coordinate<first.x>) -> Scalar");
        assert!(admitted(&raw).is_err());
    }

    #[test]
    fn response_cannot_claim_an_unused_potential_witness() {
        assert!(admitted(r#"package p {
            response counterfeit from potential(x: Length,potential: Fn(x: Length)->Length) -> Length = x;
        }"#).is_err());
    }

    #[test]
    fn coordinate_slot_cannot_relabel_a_physical_magnitude() {
        assert!(admitted("package p { coordinate map bad(x: Length) { slot x = x; } }").is_err());
    }

    #[test]
    fn physical_reconstruction_specializes_exact_maps_and_selected_response_witness() {
        let text = r#"package p {
          entity kind convention {} entity convention ideal {}
          entity kind item {} entity item a {} entity item b {}
          set members: Set<item> = {a,b};
          coordinate map coords(x: Length, n: Length[item], members: Set<item>) {
            slot x = x/1{m}; slot n[j in members] = n[j]/1{m};
          }
          reconstruction family for coords(x: Length,n: Length[item],members:Set<item>) -> Length reference ideal = 1{m};
          fn law(x:Coordinate<coords.x>,n:Coordinate<coords.n>[item],members:Set<item>)->Reduced<family> = x*sum(j in members | n[j]);
          fn potential(x:Length,n:Length[item],members:Set<item>)->Length = reconstruct(family,law,x,n,members);
          response result from potential(x:Length,n:Length[item],members:Set<item>,potential:Fn(x:Length,n:Length[item],members:Set<item>)->Length)->Length = potential(x,n,members);
          def Root { var x:Length; var n[j in members]:Length; eq equation:result(x,n,members,potential)==4{m}; }
        }"#;
        let package = admitted(text).unwrap();
        let root = package.names["p.Root"];
        let model = crate::specialize(&package,root,crate::InstanceId::from_id(pse_ids::SemanticId::NIL),&crate::Bindings::default(),crate::Limits::default()).unwrap();
        assert!(model.functions.values().any(|f| matches!(f.physical_operation, Some(PhysicalOperation::Reconstruction {..}))));
        assert!(model.functions.values().any(|f| matches!(f.physical_operation, Some(PhysicalOperation::Response {potential:Some(_), ..}))));
        let false_witness = text.replace("reconstruct(family,law,x,n,members)", "x");
        let package = admitted(&false_witness).unwrap();
        assert!(crate::specialize(&package,package.names["p.Root"],crate::InstanceId::from_id(pse_ids::SemanticId::NIL),&crate::Bindings::default(),crate::Limits::default()).is_err());
    }

    #[test]
    fn reconstruction_consumes_common_basis_only_after_amount_cancellation() {
        let (registry, names) = physical();
        let prerequisites = pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions()).unwrap();
        let contract = |name: &str| pse_quantity::ResolvedPhysicalContract::named(names[name],pse_quantity::IndexSet::new(),&registry).unwrap();
        let multiply = |a, b| pse_quantity::resolved::infer_operation(&pse_quantity::infer::OpRequest::Mul,&[a,b],None,&registry,&prerequisites).unwrap().result;
        let molar = multiply(contract("GasConstant"),contract("DeltaTemperature"));
        let extensive = multiply(molar.clone(),contract("TotalAmount"));
        let id = DeclarationId::from_id(pse_ids::SemanticId::from_bytes([91;16]));
        let operation = PhysicalOperation::Reconstruction {map:id,reconstruction:id,reference:id};
        let target = Type::Quantity(Scheme::Concrete(names["ResidualHelmholtzEnergy"]));
        assert!(extensive.at_boundary(names["ResidualHelmholtzEnergy"],&registry).is_err());
        operation.admit_numeric_result(&extensive,&target,&registry,&prerequisites,id).unwrap();
        assert!(operation.admit_numeric_result(&molar,&target,&registry,&prerequisites,id).is_err());
        let component_amount = multiply(molar.clone(),contract("Amount"));
        assert!(operation.admit_numeric_result(&component_amount,&target,&registry,&prerequisites,id).is_err());
        let referenced = multiply(contract("DeltaH"),contract("TotalAmount"));
        assert!(operation.admit_numeric_result(&referenced,&target,&registry,&prerequisites,id).is_err());
    }
}
