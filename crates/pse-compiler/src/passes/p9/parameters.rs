// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact selected coefficients are values only through their declared source coordinate map.
use super::{invalid, selections::Selection};
use crate::{
    CompilerError, PassContext,
    passes::{
        native_outputs::{SourceKey, Sources},
        native_rows::{AlgorithmInputs, Located, located_input},
        parameter_indices::ParameterIndexProjector,
    },
};
use pse_catalog::session::SnapshotSession;
use pse_ids::SemanticId;
use pse_quantity::{DomainKind, QuantityRegistry, QuantityTypeId, UnitId};
use pse_relations::generated::{compiled, enums::SymbolRole, inferred, normalized, reference};
use std::collections::BTreeSet;
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct Binding {
    pub row: compiled::method_parameter_bindings::Row,
    pub support: BTreeSet<SourceKey>,
}
pub(super) struct Bindings {
    pub rows: Vec<Binding>,
    _lease: Arc<pse_ids::ReservationLease>,
    _arguments: AlgorithmInputs,
}
pub(super) async fn bind(
    selection: &Selection,
    ctx: &PassContext<'_>,
    session: &SnapshotSession,
    sources: &Sources,
) -> Result<Bindings, CompilerError> {
    let registry = ctx.registry;
    let reserver = ctx.reserver;
    let cancel = ctx.cancel;
    let physical = ctx.physical()?.quantities();
    let mut lease = reserver.open("P9:parameter-binding-rows");
    let mut arguments = AlgorithmInputs::new(reserver, "P9:parameter-arguments");
    let inventory = Inventory::new(&mut arguments, session, sources, ctx).await?;
    let projector = ParameterIndexProjector::new(registry, session, sources, reserver);
    let mut output = Vec::new();
    for method in selection
        .methods
        .values()
        .filter(|method| !method.existing_state && method.specification.template_id.is_some())
    {
        for parameter in inventory
            .parameters
            .iter()
            .filter(|row| row.method_id == method.specification.method_id)
        {
            cancel.checkpoint()?;
            let template = method
                .specification
                .template_id
                .ok_or_else(|| invalid("selected coefficient template absent"))?;
            let symbol = unique(
                inventory
                    .symbols
                    .iter()
                    .filter(|row| row.template_id == template && row.name == parameter.name),
                "method parameter symbol",
            )?;
            if symbol.role != SymbolRole::Parameter {
                return Err(invalid(
                    "method parameter declaration is not a parameter-role symbol",
                ));
            }
            let (domains, mut support) = inventory.domains(method.instance, &symbol.indexed_by)?;
            let (scalar, canonical) =
                check_type(physical, parameter, symbol, &domains, &inventory.domains)?;
            support.insert(parameter.source.clone());
            support.insert(symbol.source.clone());
            support.insert(
                unique(
                    inventory
                        .instances
                        .iter()
                        .filter(|row| row.instance_id == method.instance),
                    "selected method instance",
                )?
                .source
                .clone(),
            );
            for domain in &domains {
                support.insert(
                    unique(
                        inventory
                            .domains
                            .iter()
                            .filter(|row| row.domain_id == *domain),
                        "parameter domain",
                    )?
                    .source
                    .clone(),
                );
            }
            let product = pse_templates::identity::domain_product_id(&domains);
            let actual_product = unique(
                inventory
                    .products
                    .iter()
                    .filter(|row| row.product_id == product || row.domain_ids == domains),
                "method parameter product",
            )?;
            if actual_product.product_id != product || actual_product.domain_ids != domains {
                return Err(invalid(
                    "method parameter product differs from its ordered factor vector",
                ));
            }
            support.insert(actual_product.source.clone());
            let tuples = inventory
                .tuples
                .iter()
                .filter(|row| row.product_id == product)
                .collect::<Vec<_>>();
            if domains.is_empty() && (tuples.len() != 1 || !tuples[0].tuple.is_empty()) {
                return Err(invalid(
                    "method scalar product is not exactly the empty tuple",
                ));
            }
            let mut seen = BTreeSet::new();
            for tuple in tuples {
                if !seen.insert(tuple.tuple.clone()) {
                    return Err(invalid("method parameter tuple repeated"));
                }
                let projected = projector
                    .project(
                        method.specification.method_id,
                        &parameter.name,
                        &domains,
                        &tuple.tuple,
                        cancel,
                    )
                    .await?;
                lease
                    .try_grow(
                        projected
                            .values
                            .len()
                            .checked_mul(128)
                            .and_then(|n| n.checked_add(parameter.name.len().saturating_mul(8)))
                            .and_then(|n| {
                                n.checked_add(projected.sources.len().saturating_mul(256))
                            })
                            .and_then(|n| n.checked_add(4096))
                            .ok_or_else(|| invalid("parameter output extent overflow"))?,
                    )
                    .map_err(pse_ids::CanonError::from)?;
                let candidates = inventory
                    .values
                    .iter()
                    .filter(|row| {
                        row.owner_entity_id == method.package
                            && row.parameter_kind == parameter.name
                            && row.index == projected.values
                    })
                    .collect::<Vec<_>>();
                if candidates.is_empty() && !parameter.required {
                    continue;
                }
                let [value] = candidates.as_slice() else {
                    return Err(invalid(
                        "required method coefficient key is absent or repeated",
                    ));
                };
                if !value.value.is_finite() {
                    return Err(invalid("method coefficient is nonfinite"));
                }
                pse_quantity::convert_spec_for_type(
                    physical.unit(UnitId::from_id(value.unit_id))?,
                    physical.unit(canonical)?,
                    &physical.quantity_type(scalar)?.key,
                )?;
                let mut actual_support = support.clone();
                actual_support.extend(projected.sources.iter().cloned());
                actual_support.insert(tuple.source.clone());
                actual_support.insert(value.source.clone());
                output.push(Binding {
                    row: compiled::method_parameter_bindings::Row {
                        method_instance_id: method.instance,
                        symbol_decl_id: symbol.symbol_decl_id,
                        index: tuple.tuple.clone(),
                        source_owner: method.package,
                        parameter_kind: parameter.name.clone(),
                        source_index: projected.values.clone(),
                        value: value.value,
                        unit_id: value.unit_id,
                        quantity_type_id: scalar.as_id(),
                        derivation_id: SemanticId::NIL,
                    },
                    support: actual_support,
                });
            }
        }
    }
    Ok(Bindings {
        rows: output,
        _lease: pse_ids::ReservationLease::new(lease),
        _arguments: arguments,
    })
}
fn check_type(
    physical: &QuantityRegistry,
    parameter: &reference::method_parameters::Row,
    symbol: &normalized::template_symbols::Row,
    domains: &[SemanticId],
    facts: &[Located<normalized::domains::Row>],
) -> Result<(QuantityTypeId, UnitId), CompilerError> {
    let declared = physical.quantity_type(QuantityTypeId::from_id(parameter.quantity_type_id))?;
    let symbol_type = physical.quantity_type(QuantityTypeId::from_id(symbol.quantity_type_id))?;
    let shape = domains
        .iter()
        .map(|domain| {
            let facts = unique(
                facts.iter().filter(|row| row.domain_id == *domain),
                "method coefficient domain",
            )?;
            DomainKind::parse(facts.kind.as_str())
                .ok_or_else(|| invalid("coefficient domain kind absent"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if declared.key != symbol_type.key
        || declared.key.shape != shape
        || parameter
            .indexed_by
            .iter()
            .map(|kind| kind.as_str())
            .ne(shape.iter().map(|kind| kind.as_str()))
    {
        return Err(invalid(
            "method coefficient full quantity or ordered domain contract differs",
        ));
    }
    pse_quantity::convert_spec_for_type(
        physical.unit(UnitId::from_id(parameter.natural_unit_id))?,
        physical.unit(declared.canonical_unit)?,
        &declared.key,
    )?;
    let mut key = declared.key.clone();
    key.shape.clear();
    let scalar = physical.resolve_key(&key)?;
    Ok((scalar, physical.quantity_type(scalar)?.canonical_unit))
}
struct Inventory {
    parameters: Vec<Located<reference::method_parameters::Row>>,
    symbols: Vec<Located<normalized::template_symbols::Row>>,
    bindings: Vec<Located<normalized::instance_domain_bindings::Row>>,
    domains: Vec<Located<normalized::domains::Row>>,
    products: Vec<Located<normalized::domain_products::Row>>,
    tuples: Vec<Located<inferred::valid_index_tuples::Row>>,
    values: Vec<Located<normalized::parameter_values::Row>>,
    instances: Vec<Located<inferred::instances::Row>>,
}
impl Inventory {
    async fn new(
        arguments: &mut AlgorithmInputs,
        session: &SnapshotSession,
        sources: &Sources,
        ctx: &PassContext<'_>,
    ) -> Result<Self, CompilerError> {
        Ok(Self {
            parameters: located_input(arguments, session, sources, ctx.registry, ctx.cancel)
                .await?,
            symbols: located_input(arguments, session, sources, ctx.registry, ctx.cancel).await?,
            bindings: located_input(arguments, session, sources, ctx.registry, ctx.cancel).await?,
            domains: located_input(arguments, session, sources, ctx.registry, ctx.cancel).await?,
            products: located_input(arguments, session, sources, ctx.registry, ctx.cancel).await?,
            tuples: located_input(arguments, session, sources, ctx.registry, ctx.cancel).await?,
            values: located_input(arguments, session, sources, ctx.registry, ctx.cancel).await?,
            instances: located_input(arguments, session, sources, ctx.registry, ctx.cancel).await?,
        })
    }
    fn domains(
        &self,
        instance: SemanticId,
        names: &[String],
    ) -> Result<(Vec<SemanticId>, BTreeSet<SourceKey>), CompilerError> {
        let mut domains = Vec::new();
        let mut support = BTreeSet::new();
        for name in names {
            let binding = unique(
                self.bindings
                    .iter()
                    .filter(|row| row.instance_id == instance && row.domain_name == *name),
                "method coefficient domain binding",
            )?;
            domains.push(binding.domain_id);
            support.insert(binding.source.clone());
        }
        Ok((domains, support))
    }
}
fn unique<T>(mut values: impl Iterator<Item = T>, label: &str) -> Result<T, CompilerError> {
    let result = values
        .next()
        .ok_or_else(|| invalid(format!("{label} absent")))?;
    if values.next().is_some() {
        return Err(invalid(format!("{label} ambiguous")));
    }
    Ok(result)
}
