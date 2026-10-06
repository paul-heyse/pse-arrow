// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Reusable actual-reference C4 request fixture; coupled execution belongs to Plan 25k.
use super::*;
use crate::workflow::tests::{compiler_profile, physical, runtime_on};
use pse_structural::flowsheet::{Decision, Policy};

#[path = "../../../../../../tests/fixtures/reference-source-fixture.rs"]
mod source_fixture;

pub(in crate::workflow) struct ReferenceRecycleFixture {
    pub package: ModelingPackage,
    pub analysis: ModelingAnalysis,
    pub selection: ModelingFlowSelection,
    pub request: RecycleRequest,
}

/// Original source documents and canonical compiler inventories supply the request.
/// The explicitly selected unit procedures never turn a capability refusal into an
/// implicit change of strategy. The authored staged simultaneous fixtures remain alternatives.
pub(in crate::workflow) async fn actual_reference_recycle_fixture(
    cancel: &crate::CancelSource,
) -> Result<ReferenceRecycleFixture, WorkflowError> {
    let source = format!(
        "{}\n{}\n{}\n{}\n{}",
        source_fixture::reference_sources(),
        include_str!("../../../../../../packages/reference/process/models/mixer.pse"),
        include_str!("../../../../../../packages/reference/process/models/flash.pse"),
        include_str!("../../../../../../packages/reference/campaign/models/references.pse"),
        include_str!("../../../../../../packages/reference/campaign/models/recycle-flash.pse")
    );
    let rows = source_fixture::rows(&source);
    let namespace = rows
        .iter()
        .find(|r| r.parent_id.is_none() && r.name == "recycle_flash")
        .ok_or_else(|| contract("reference recycle package absent"))?
        .declaration_id;
    let root = rows
        .iter()
        .find(|r| r.parent_id == Some(namespace) && r.name == "recycle_flash_conditional_handoff")
        .ok_or_else(|| contract("reference recycle root absent"))?
        .declaration_id;
    // Full original source admission, including the imported reference closure,
    // needs larger declared workspace and evaluator storage than the tiny scalar controls.
    let runtime = runtime_on(
        4 << 30,
        crate::math::MathPolicy {
            workspace_bytes: 1 << 30,
            worker_bytes: 256 << 20,
            foreign_bytes: 8 << 20,
            ..Default::default()
        },
    );
    let mut physical = physical();
    physical.preconditions = Arc::new(
        pse_quantity::PhysicalPreconditions::new(pse_quantity::generated::standard_preconditions())
            .map_err(crate::workflow::math)?,
    );
    physical.key =
        pse_compiler::workspace::physical_identity(&physical.quantities, &physical.preconditions);
    let package = runtime.modeling_package(rows, physical).await?;
    let analysis = package
        .declared_execution(
            root,
            compiler_profile(),
            super::super::tests::profile(SolveIntent::Root),
            Default::default(),
            Default::default(),
            cancel,
        )
        .await?
        .analysis;
    // declared_execution admits the authored fixture's original fresh-feed fixes.
    let resolved = package
        .resolve_case(
            analysis.root,
            analysis.instance,
            analysis.bindings.clone(),
            analysis.limits,
            analysis.case.clone(),
            DerivativeOrder::First,
            analysis.compiler,
            analysis.solver.clone(),
            analysis.numerical.clone(),
            Default::default(),
            false,
            cancel,
        )
        .await?;
    let model = &resolved.model.model.compiled().model;
    let prefix = format!("{}.root", model.instances[&analysis.instance].path);
    let mut units = Vec::new();
    let mut nodes = BTreeSet::new();
    for (name, input_names) in [
        ("feed", &[][..]),
        ("mixer", &["inlet_ports"][..]),
        ("heater", &["inlet_port"][..]),
        ("flash", &["inlet_port"][..]),
        ("splitter", &["inlet_port"][..]),
    ] {
        let path = format!("{prefix}.{name}");
        let node = model
            .instances
            .values()
            .find(|i| i.path == path)
            .ok_or_else(|| contract(format!("reference unit absent: {name}")))?;
        nodes.insert(node.id);
        let owned = model
            .material_ports
            .values()
            .filter(|p| p.lineage.instance == node.id);
        let mut inputs = BTreeSet::new();
        let mut outputs = BTreeSet::new();
        for port in owned {
            let member = port
                .lineage
                .path
                .strip_prefix(&format!("{path}."))
                .ok_or_else(|| contract("reference port owner path"))?;
            if input_names
                .iter()
                .any(|name| member == *name || member.starts_with(&format!("{name}[")))
            {
                inputs.insert(port.id);
            } else {
                outputs.insert(port.id);
            }
        }
        if outputs.is_empty() || name != "feed" && inputs.is_empty() {
            return Err(contract("reference causal directions have no boundary"));
        }
        let realization = if name == "feed" {
            CausalUnitRealization::ExplicitMap
        } else {
            let coordinates = expand_unit_ports(model, &inputs)?;
            let symbols = coordinates
                .iter()
                .map(|id| model.ports[id].symbol)
                .collect();
            let inventory =
                pse_compiler::workspace::CompilerWorkspace::modeling_conditional_unit_inventory(
                    resolved.model.model.compiled(),
                    resolved.model.case.compiled().plan.structure(),
                    node.id.as_id(),
                    &symbols,
                )
                .map_err(MathRuntimeError::from)?;
            CausalUnitRealization::Conditional {
                residuals: inventory.residuals,
                unknowns: inventory.unknowns,
                solver: Box::new(crate::math::settings::SolveSettings {
                    intent: SolveIntent::Root,
                    backend: Some(Backend::Kinsol),
                    ..Default::default()
                }),
            }
        };
        units.push(CausalUnitRequest {
            node: node.id.as_id(),
            inputs,
            outputs,
            realization,
        });
    }
    let tear = model
        .connections
        .values()
        .find(|c| c.lineage.path == format!("{prefix}.recycle"))
        .ok_or_else(|| contract("reference recycle occurrence absent"))?
        .id;
    let selection = ModelingFlowSelection {
        nodes,
        connections: model
            .connections
            .keys()
            .map(|id| {
                (
                    *id,
                    Decision {
                        id: *id,
                        cost: 1.0,
                        policy: if *id == tear {
                            Policy::Mandatory
                        } else {
                            Policy::Forbidden
                        },
                    },
                )
            })
            .collect(),
    };
    Ok(ReferenceRecycleFixture {
        package,
        analysis,
        selection,
        request: RecycleRequest {
            tears: BTreeSet::from([tear]),
            units,
            anderson: 0,
            damping: pse_model::scalars::Fraction::try_new(1.0)
                .map_err(|e| contract(e.to_string()))?,
        },
    })
}

#[tokio::test]
async fn conditional_unit_reference_request_declares_original_inventories_and_root_refusal() {
    let cancel = crate::CancelSource::new();
    let fixture = actual_reference_recycle_fixture(&cancel).await.unwrap();
    assert_eq!(fixture.request.units.len(), 5);
    assert_eq!(fixture.request.tears.len(), 1);
    assert!(matches!(
        fixture.request.units[0].realization,
        CausalUnitRealization::ExplicitMap
    ));
    for unit in &fixture.request.units[1..] {
        let CausalUnitRealization::Conditional {
            residuals,
            unknowns,
            solver,
        } = &unit.realization
        else {
            panic!("reference equation unit became an explicit fabricated map")
        };
        assert!(!residuals.is_empty() && !unknowns.is_empty());
        assert_eq!(solver.backend, Some(Backend::Kinsol));
    }
    let encoded = serde_json::to_value(&fixture.request).unwrap();
    let request: RecycleRequest = serde_json::from_value(encoded).unwrap();
    let error = fixture
        .package
        .prepare_recycle(&fixture.analysis, fixture.selection, request, &cancel)
        .await
        .unwrap_err();
    let WorkflowError::ConditionalAdmission { diagnostic, .. } = error else {
        panic!("selected reference unit capability refusal lost its source envelope: {error}")
    };
    assert_eq!(
        diagnostic.class,
        pse_model::diagnostic::BoundaryClass::Unsupported
    );
    assert_eq!(
        diagnostic.rule,
        pse_diagnostics::DiagnosticRule::ModelingConditionalUnitAdmissionUnsupported
    );
    let mixer = &fixture.request.units[1];
    assert!(diagnostic.sources.contains(&mixer.node));
    assert!(
        mixer
            .inputs
            .iter()
            .all(|id| diagnostic.sources.contains(id))
    );
    let CausalUnitRealization::Conditional {
        residuals,
        unknowns,
        ..
    } = &mixer.realization
    else {
        panic!("reference mixer must retain its conditional realization")
    };
    assert!(
        residuals
            .iter()
            .chain(unknowns)
            .all(|id| diagnostic.sources.contains(id))
    );
}
