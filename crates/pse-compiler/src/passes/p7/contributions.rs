// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{Realizer, invalid};
use crate::CompilerError;
use crate::passes::physical_subject::{member, phase};
use pse_quantity::{DomainKind, QuantityTypeId};
use pse_relations::generated::{authored, compiled, enums::ExpressionRootRole, inferred};

impl Realizer<'_> {
    #[expect(
        clippy::too_many_lines,
        reason = "contributions keeps the native relation inputs and dependency ordered assembly visible in one place"
    )]
    pub(super) fn contributions(
        &mut self,
        instance: &inferred::instances::Row,
    ) -> Result<(), CompilerError> {
        let declarations = self
            .inventory
            .contributions
            .iter()
            .filter(|row| row.template_id == instance.template_id)
            .cloned()
            .collect::<Vec<_>>();
        for declaration in declarations {
            if !self.guard_enabled(instance.instance_id, declaration.guard_id)? {
                continue;
            }
            let contracts = self
                .inventory
                .contribution_contracts
                .iter()
                .filter(|row| row.contribution_decl_id == declaration.contribution_decl_id)
                .cloned()
                .collect::<Vec<_>>();
            let [contract] = contracts.as_slice() else {
                return Err(invalid(
                    "contribution requires one explicit physical/index contract",
                ));
            };
            let domains = self.domains(instance.instance_id, &contract.indexed_by)?;
            let product = self.product(&domains)?;
            let quantity = self
                .physical
                .quantity_type(QuantityTypeId::from_id(contract.quantity_type_id))?;
            let shape = domains
                .iter()
                .map(|domain| {
                    self.domain_facts
                        .get(domain)
                        .map(|facts| facts.kind)
                        .ok_or_else(|| invalid("contribution actual domain absent"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            if quantity.key.shape != shape {
                return Err(invalid(
                    "contribution physical claim differs from its actual domain shape",
                ));
            }
            let basis = quantity.key.basis.map(pse_quantity::BasisId::as_id);
            validate_subject_axes(
                contract.subject.kind.as_str(),
                member!(&contract.subject, axis, position),
                phase!(&contract.subject, axis, position),
                &shape,
            )?;
            let scopes = self
                .inventory
                .scopes
                .iter()
                .filter(|row| {
                    row.template_id == instance.template_id && row.name == declaration.scope
                })
                .collect::<Vec<_>>();
            let [scope] = scopes.as_slice() else {
                return Err(invalid(
                    "contribution scope key is absent or ambiguous under actual template",
                ));
            };
            let bindings = self
                .inventory
                .scope_bindings
                .iter()
                .filter(|row| {
                    row.scope_decl_id == scope.scope_id
                        && row.owner_instance_id == instance.instance_id
                })
                .collect::<Vec<_>>();
            let [scope] = bindings.as_slice() else {
                return Err(invalid(
                    "contribution scope lacks exact actual instance binding",
                ));
            };
            let scope_id = scope.scope_id;
            let source = self.source(
                authored::template_contributions::RELATION_ID,
                "contribution_decl_id",
                declaration.contribution_decl_id,
                "expression",
            )?;
            let env = self.prepare(instance, &source)?;
            let root = self.instantiate_roots(&source, &[source.root_id], &env)?[0];
            let transfer = self.transfer(
                instance,
                &declaration,
                contract,
                &source,
                &env,
                root,
                &domains,
            )?;
            let id = pse_ids::named_id(
                instance.instance_id,
                &format!(
                    "pse:contribution:v1:{}",
                    declaration.contribution_decl_id.to_hex()
                ),
            );
            self.append(
                "compiled.contributions",
                compiled::contributions::Row {
                    contribution_id: id,
                    contribution_decl_id: declaration.contribution_decl_id,
                    owner_instance_id: instance.instance_id,
                    scope_id,
                    product_id: product,
                    source_id: source.source_id,
                    expression_root: root.0,
                    law_family: declaration.law_family,
                    quantity_type_id: contract.quantity_type_id,
                    basis_id: basis,
                    orientation: declaration.orientation,
                    transfer_connection_id: transfer,
                    subject: super::super::native_rows::transfer_value(
                        &contract.subject,
                        self.registry,
                        "compiled.contributions",
                        "subject",
                    )?,
                    derivation_id: Self::derivation(
                        instance.instance_id,
                        source.source_id,
                        "contribution",
                    ),
                },
            )?;
            self.expression_root(id, ExpressionRootRole::Contribution, root, &source, &env)?;
        }
        Ok(())
    }
}
fn validate_subject_axes(
    kind: &str,
    subject_axis: Option<i64>,
    phase_axis: Option<i64>,
    shape: &[DomainKind],
) -> Result<(), CompilerError> {
    if let Some(axis) = subject_axis {
        let expected = match kind {
            "species" | "phase_species" => DomainKind::Species,
            "element" => DomainKind::Element,
            _ => return Err(invalid("subject kind has no indexed subject contract")),
        };
        if usize::try_from(axis).ok().and_then(|axis| shape.get(axis)) != Some(&expected) {
            return Err(invalid(
                "subject axis does not have its actual required domain kind",
            ));
        }
    }
    if phase_axis.is_some_and(|axis| {
        usize::try_from(axis).ok().and_then(|axis| shape.get(axis)) != Some(&DomainKind::Phase)
    }) {
        return Err(invalid("phase axis is not an actual phase domain"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonmaterial_subjects_allow_independent_phase_coordinates() {
        for kind in ["total", "energy", "momentum"] {
            validate_subject_axes(kind, None, None, &[]).unwrap();
            validate_subject_axes(
                kind,
                None,
                Some(1),
                &[DomainKind::PortSet, DomainKind::Phase],
            )
            .unwrap();
            assert!(validate_subject_axes(kind, None, Some(0), &[DomainKind::Species]).is_err());
            assert!(validate_subject_axes(kind, Some(0), None, &[DomainKind::Phase]).is_err());
        }
    }
}
