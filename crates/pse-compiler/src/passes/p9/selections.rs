// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete P6 winners bind actual state contexts and declared output correspondences.
mod bindings;
mod native;

use super::invalid;
use crate::{
    CompilerError,
    passes::{native_rows::AlgorithmInputs, p3::SelectedRoot},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, MemoryReserver, SemanticId};
use pse_relations::generated::{authored, enums::MethodFamily, inferred, reference};
use pse_schema::Registry;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(super) struct Method {
    pub instance: SemanticId,
    pub scope: SemanticId,
    pub state: SemanticId,
    pub package: SemanticId,
    pub specification: reference::method_specs::Row,
    pub existing_state: bool,
}
#[derive(Clone, Debug)]
pub(super) struct Request {
    pub requirement: inferred::property_requirements::Row,
    pub instance: SemanticId,
    pub provision: reference::method_provisions::Row,
}
#[derive(Debug)]
pub(super) struct Selection {
    pub methods: BTreeMap<SemanticId, Method>,
    pub roots: Vec<SelectedRoot>,
    pub requests: Vec<Request>,
}

#[expect(
    clippy::too_many_lines,
    reason = "select keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
pub(super) async fn select(
    session: &SnapshotSession,
    registry: &Registry,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
) -> Result<(Selection, AlgorithmInputs), CompilerError> {
    let mut arguments = AlgorithmInputs::new(reserver, "P9:selected-method-arguments");
    let candidates = native::candidates(&mut arguments, session, registry, cancel).await?;
    let mappings = bindings::state_parameters(&mut arguments, session, registry, cancel).await?;
    let mut result = Selection {
        methods: BTreeMap::new(),
        roots: Vec::new(),
        requests: Vec::new(),
    };
    for native::Candidate {
        requirement,
        resolution,
        resolution_key,
        scope,
        state,
        method,
        provision,
    } in candidates
    {
        cancel.checkpoint()?;
        let inferred::method_resolutions::InferredMethodResolutionsFieldOutcomeSelected::Resolved(
            selected,
        ) = resolution.outcome.selected()?
        else {
            return Err(invalid(
                "selected property requirement is unresolved or ambiguous",
            ));
        };
        let template = match method.realization.selected()? {
            reference::method_specs::ReferenceMethodSpecsFieldRealizationSelected::EquationTemplate(value) => Some(value.template_id),
            reference::method_specs::ReferenceMethodSpecsFieldRealizationSelected::Kernel(_) => None,
        };
        if selected.method_id != method.method_id {
            return Err(invalid(
                "method resolution differs from actual selected specification",
            ));
        }
        let equation = template.is_some();
        let template_output = matches!(provision.output.selected()?, reference::method_provisions::ReferenceMethodProvisionsFieldOutputSelected::TemplateSymbol(_));
        if equation != template_output {
            return Err(invalid(
                "selected method and provision use different producer routes",
            ));
        }
        let existing_state = equation && method.family == MethodFamily::StateDefinition;
        let instance = if existing_state {
            if Some(state.template_id) != template {
                return Err(invalid(
                    "state definition method differs from actual selected state template",
                ));
            }
            state.instance_id
        } else {
            pse_ids::named_id(
                scope.state_scope_id,
                &format!("pse:method-instance:v1:{}", method.method_id.to_hex()),
            )
        };
        if let Some(previous) = result.methods.get(&instance) {
            if previous.scope != scope.state_scope_id
                || previous.specification != method
                || previous.state != state.instance_id
                || previous.package != scope.property_package_id
            {
                return Err(invalid(
                    "selected method instance has conflicting actual meanings",
                ));
            }
        } else {
            if equation && !existing_state {
                let template =
                    template.ok_or_else(|| invalid("equation method template absent"))?;
                let assignments = mappings
                    .get(&method.method_id)
                    .map(
                        |name| authored::instances::AuthoredInstancesFieldParamValuesItem {
                            name: name.clone(),
                            value: state.instance_id.to_string(),
                        },
                    )
                    .into_iter()
                    .collect();
                result.roots.push(SelectedRoot {
                    instance: authored::instances::Row {
                        instance_id: instance,
                        parent_instance_id: Some(state.instance_id),
                        template_id: template,
                        name: format!("method_{}", method.method_id.to_hex()),
                        param_values: assignments,
                        feature_values: Vec::new(),
                        property_package_id: Some(scope.property_package_id),
                        reaction_package_id: None,
                        doc: "Selected method instance from actual P6 resolution".to_owned(),
                    },
                    source_relation: inferred::method_resolutions::RELATION_ID,
                    source_key: resolution_key,
                });
            }
            result.methods.insert(
                instance,
                Method {
                    instance,
                    scope: scope.state_scope_id,
                    state: state.instance_id,
                    package: scope.property_package_id,
                    specification: method.clone(),
                    existing_state,
                },
            );
        }
        result.requests.push(Request {
            requirement,
            instance,
            provision: provision.clone(),
        });
    }
    Ok((result, arguments))
}
