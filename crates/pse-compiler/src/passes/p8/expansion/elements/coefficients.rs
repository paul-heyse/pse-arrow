// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit coefficient products joined to actual admitted composition through DataFusion.
mod plan;
mod rows;
use super::{
    BTreeMap, BTreeSet, BoundIndexRef, CompilerError, Context, DomainKind, ExprGraph, IndexSet,
    NodeId, Opcode, Output, Payload, QuantityTypeId, Reservation, SemanticId, Term, compiled,
    invalid,
};
use crate::passes::native_rows::{AlgorithmInputs, workspace};
use pse_quantity::OperationId;

pub(super) struct Group {
    pub root: NodeId,
    pub quantity: QuantityTypeId,
    pub indices: IndexSet,
    pub operation: OperationId,
}
struct Declaration {
    group: SemanticId,
    product: SemanticId,
    owner: SemanticId,
    element: BoundIndexRef,
    species: BoundIndexRef,
    quantity: QuantityTypeId,
    scalar: QuantityTypeId,
    unit: pse_quantity::UnitId,
    operation: OperationId,
    mass: bool,
    header: compiled::element_projection_groups::Row,
}
pub(super) async fn build(
    context: &Context<'_, '_>,
    law: &compiled::law_applications::Row,
    term: &Term<'_>,
    axes: [BoundIndexRef; 2],
    graph: &mut ExprGraph,
    output: &mut Output<'_>,
    work: &mut dyn Reservation,
) -> Result<Group, CompilerError> {
    work.try_grow(8192)
        .map_err(pse_catalog::CatalogError::from)?;
    let [element, species] = axes;
    let declaration = declaration(context, law, term, element, species, output)?;
    let mut builder =
        compiled::element_projection_groups::Builder::with_registry(context.ctx.registry, 1)?;
    builder.push(declaration.header.clone())?;
    let header = builder
        .finish()?
        .retained(context.ctx.reserver, context.ctx.cancel)?;
    let session = workspace(
        &context.inputs.session,
        &BTreeMap::from([(compiled::element_projection_groups::RELATION_KEY, header)]),
        context.ctx.cancel,
    )?;
    let mut arguments = AlgorithmInputs::new(context.ctx.reserver, "P8:element-coefficients");
    let complete = arguments
        .execute(
            plan::members(&session, context.ctx.registry)?,
            &session,
            context.ctx.cancel,
        )
        .await?;
    let actual = rows::read(&complete, &context.inputs.sources, context.ctx.registry)?;
    rows::emit(context, &declaration, &actual, graph, output, work)?;
    let indices = IndexSet::try_from_iter([element, species])
        .map_err(|_| invalid("element coefficient axes conflict"))?;
    let root = graph.insert(
        Opcode::Gather,
        Payload::Gather {
            group: declaration.group,
            coordinate_map: vec![(element.bound_index, 0), (species.bound_index, 1)],
        },
        &[],
        Some(declaration.owner),
    )?;
    Ok(Group {
        root,
        quantity: declaration.quantity,
        indices,
        operation: declaration.operation,
    })
}
fn declaration(
    context: &Context<'_, '_>,
    law: &compiled::law_applications::Row,
    term: &Term<'_>,
    element: BoundIndexRef,
    species: BoundIndexRef,
    output: &mut Output<'_>,
) -> Result<Declaration, CompilerError> {
    let source = QuantityTypeId::from_id(term.contribution.quantity_type_id);
    let actual = context.physical.quantity_type(source)?;
    let basis = actual
        .key
        .basis
        .ok_or_else(|| invalid("element contribution has no supported explicit basis"))?
        .as_id();
    let template = law.law_template_id;
    let contracts = &context.inputs.element_projection_contracts;
    let mut matches = contracts
        .iter()
        .filter(|row| row.law_template_id == template && row.source_basis_id == basis);
    let contract = matches
        .next()
        .ok_or_else(|| invalid("no declared element projection for actual source basis"))?;
    if matches.next().is_some() {
        return Err(invalid("element source basis projection is ambiguous"));
    }
    output.use_row(contract);
    let expected = context
        .physical
        .quantity_type(QuantityTypeId::from_id(contract.source_quantity_type_id))?;
    let mut source_key = actual.key.clone();
    source_key.shape.clear();
    let mut expected_key = expected.key.clone();
    expected_key.shape.clear();
    if source_key != expected_key {
        return Err(invalid(
            "element contribution differs from its complete declared source quantity",
        ));
    }
    let mass = match contract.formula.as_str() {
        "molar_count" => false,
        "mass_count_over_mw" => true,
        _ => return Err(invalid("unknown element coefficient formula")),
    };
    let basis_kind = context
        .physical
        .basis(pse_quantity::BasisId::from_id(basis))?
        .kind
        .as_str();
    if basis_kind != if mass { "mass" } else { "molar" } {
        return Err(invalid(
            "element coefficient formula differs from actual basis declaration",
        ));
    }
    let quantity = QuantityTypeId::from_id(contract.coefficient_quantity_type_id);
    let coefficient = context.physical.quantity_type(quantity)?;
    let unit = pse_quantity::UnitId::from_id(contract.coefficient_unit_id);
    if coefficient.key.shape != [DomainKind::Element, DomainKind::Species]
        || coefficient.canonical_unit != unit
    {
        return Err(invalid(
            "element coefficient must declare exact ordered Element/Species shape and canonical unit",
        ));
    }
    let mut scalar_key = coefficient.key.clone();
    scalar_key.shape.clear();
    let scalar = context.physical.resolve_key(&scalar_key)?;
    let owner = law.owner_instance_id;
    let application = law.application_id;
    let contribution = term.contribution.contribution_id;
    let group = pse_ids::named_id(
        application,
        &format!("pse:element-coefficient-group:v1:{}", contribution.to_hex()),
    );
    let domain_ids = [element.domain.as_id(), species.domain.as_id()];
    let product = pse_templates::identity::domain_product_id(&domain_ids);
    let header = compiled::element_projection_groups::Row {
        group_id: group,
        product_id: product,
        application_id: application,
        contribution_id: contribution,
        owner_instance_id: owner,
        law_template_id: template,
        source_basis_id: basis,
        element_domain_id: domain_ids[0],
        species_domain_id: domain_ids[1],
        domain_ids: domain_ids.to_vec(),
        quantity_type_id: quantity.as_id(),
        derivation_id: group,
    };
    Ok(Declaration {
        group,
        product,
        owner,
        element,
        species,
        quantity,
        scalar,
        unit,
        operation: OperationId::from_id(contract.multiplication_operation_id),
        mass,
        header,
    })
}
