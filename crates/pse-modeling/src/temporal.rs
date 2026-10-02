// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Analysis-owned temporal child composition; no unit laws are interpreted here.
use crate::{CheckedPackage, Result, Type, TypeContext, invalid};
use pse_authoring::language::AuthoredModelingDeclarationsFieldValueBindingIndicesItem as Index;
use pse_model::generated::enums::ModelingDeclarationKind as Kind;

pub(crate) const COORDINATE: &str = "__temporal_coordinate";

/// The domain authority of an authored or analysis-owned member coordinate.
pub(crate) enum IndexDomain {
    Authored {
        position: usize,
    },
    Temporal {
        policy: crate::DeclarationId,
        axis: crate::DeclarationId,
    },
}

pub(crate) fn index_domain(
    package: &CheckedPackage,
    declaration: crate::DeclarationId,
    position: usize,
) -> Result<IndexDomain> {
    if let Some((policy, axis, _)) = package.temporal.get(&declaration) {
        let owned = package.declarations[&declaration]
            .value
            .binding
            .as_ref()
            .and_then(|binding| binding.indices.first())
            .is_some_and(|index| index.name == COORDINATE);
        if !owned {
            return Err(invalid(
                *policy,
                "analysis-owned temporal coordinate absent",
            ));
        }
        if position == 0 {
            return Ok(IndexDomain::Temporal {
                policy: *policy,
                axis: *axis,
            });
        }
        return Ok(IndexDomain::Authored {
            position: position - 1,
        });
    }
    Ok(IndexDomain::Authored { position })
}

pub(crate) fn time_quantity(
    ty: &Type,
    context: &TypeContext<'_>,
    id: crate::DeclarationId,
) -> Result<pse_quantity::QuantityTypeId> {
    let Type::Quantity(scheme) = ty else {
        return Err(invalid(id, "temporal axis requires physical time"));
    };
    let quantity = scheme
        .resolve_with_evidence(
            context.quantities,
            &Default::default(),
            context.preconditions,
        )
        .map_err(|error| invalid(id, error.to_string()))?;
    let definition = context
        .quantities
        .quantity_type(quantity)
        .map_err(|error| invalid(id, error.to_string()))?;
    let unit = context
        .quantities
        .unit(definition.canonical_unit)
        .map_err(|error| invalid(id, error.to_string()))?;
    if unit.is_affine
        || unit.dimension != pse_quantity::DimensionVector::base(pse_quantity::BaseDimension::Time)
    {
        return Err(invalid(
            id,
            "temporal axis requires a non-affine time quantity",
        ));
    }
    Ok(quantity)
}

pub(crate) fn admit(package: &mut CheckedPackage, context: &TypeContext<'_>) -> Result<()> {
    let policies = package
        .declarations
        .values()
        .filter_map(|row| {
            row.value
                .temporal
                .as_ref()
                .map(|policy| (row.declaration_id, policy.clone()))
        })
        .collect::<Vec<_>>();
    for (id, policy) in policies {
        let target = package
            .resolve(id, &policy.target)
            .ok_or_else(|| invalid(id, "temporal child is not visible"))?;
        if package.declarations[&target].value.kind != Kind::Child {
            return Err(invalid(
                id,
                "temporal composition requires a child hierarchy",
            ));
        }
        let axis = package
            .resolve(id, &policy.axis)
            .ok_or_else(|| invalid(id, "temporal axis is not visible"))?;
        let Some(Type::Continuous(_, element)) = package.types.get(&axis) else {
            return Err(invalid(
                id,
                "temporal composition requires a continuous axis",
            ));
        };
        time_quantity(element, context, id)?;
        if policy.argument.contains('.') || policy.argument.is_empty() {
            return Err(invalid(
                id,
                "temporal binding names one constructor argument",
            ));
        }
        if package
            .temporal
            .insert(target, (id, axis, policy.argument))
            .is_some()
        {
            return Err(invalid(id, "competing temporal policies for one child"));
        }
        let binding = package
            .declarations
            .get_mut(&target)
            .and_then(|row| row.value.binding.as_mut())
            .ok_or_else(|| invalid(id, "temporal child binding absent"))?;
        if binding.indices.iter().any(|index| index.name == COORDINATE) {
            return Err(invalid(id, "temporal coordinate is owned by the analysis"));
        }
        binding.indices.insert(
            0,
            Index {
                name: COORDINATE.into(),
                domain: policy.axis,
            },
        );
    }
    Ok(())
}
