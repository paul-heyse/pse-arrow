// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual local package source inspection without a numerical evaluation or native solve.
use super::*;
use crate::workflow::{modeling::ModelingAnalysis, tests as fixture};
use pse_compiler::workspace::ModelingCaseBindings;
use std::{collections::BTreeSet, time::Duration};

async fn prepared_cells(
    branching: bool,
) -> (Runtime, ModelingPackage, ModelingDiagnosticPreparation) {
    let runtime = fixture::runtime();
    let services = runtime.execution_services();
    let source = if branching {
        "package p { def Cell {param g:Scalar=1; var x:Scalar; let a:Scalar=(if g>0 then log(x) else log(-x)); let b:Scalar=a*a+x; annotation valid x(-1,1); annotation valid b(-2,2); eq e:b==0;} def Root {child left:Cell=Cell(); child right:Cell=Cell();} }"
    } else {
        "package p { def Cell {var x:Scalar; let a:Scalar=x*x; let b:Scalar=a*a+x; eq e:b==0;} def Root {child left:Cell=Cell(); child right:Cell=Cell();} }"
    };
    let declarations = pse_authoring::language::parse(
        source,
        SemanticId::from_bytes([117; 16]),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap();
    let root = declarations
        .iter()
        .find(|row| row.name == "Root")
        .unwrap()
        .declaration_id;
    let package = runtime
        .modeling_package(declarations, fixture::physical())
        .await
        .unwrap();
    let analysis = ModelingAnalysis {
        root,
        instance: pse_modeling::specialize::root_instance(root),
        bindings: Bindings::default(),
        limits: Limits::default(),
        case: ModelingCaseBindings {
            values: BTreeMap::from([("left.x".into(), 0.), ("right.x".into(), 0.)]),
            ..Default::default()
        },
        order: DerivativeOrder::First,
        compiler: fixture::compiler_profile(),
        solver: fixture::profile(),
        numerical: Default::default(),
    };
    let prepared = package
        .prepare_diagnostics(
            services.preparation(),
            &analysis,
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    (runtime, package, prepared)
}

#[tokio::test]
async fn kernel_source_map_preserves_shared_occurrences_original_sources_and_static_alternatives_without_evaluation()
 {
    let (runtime, _package, prepared) = prepared_cells(true).await;
    // Both log branches are invalid at the supplied point. Static inspection has
    // no reason to evaluate either branch, choose a current arm, or qualify a point.
    let model = prepared.model.model.compiled();
    let x = model
        .model
        .symbols
        .values()
        .filter(|symbol| symbol.lineage.path.ends_with(".x"))
        .map(|symbol| symbol.id)
        .collect::<Vec<_>>();
    assert_eq!(x.len(), 2);
    assert!(x.iter().all(|id| prepared.model.values.scalars[id] == 0.));
    let before = prepared.model.values.scalars.clone();
    let (result, counts) = crate::math::counted(runtime.modeling_diagnostic_source_map(
        &prepared,
        2 << 20,
        Duration::from_secs(10),
        &crate::CancelSource::new(),
    ))
    .await;
    let owner = result.unwrap();
    let document = owner.document();
    assert!(matches!(
        document.scope,
        SourceMapScope::StaticEligibleAlternatives
    ));
    assert_eq!(prepared.model.values.scalars, before);
    assert_eq!(
        counts,
        crate::math::PreparationCounts::default(),
        "inspection neither prepares a new case nor rebinds the invalid point"
    );
    assert_eq!(
        document.source_identity,
        prepared.model.case.compiled().plan.structure().key()
    );
    assert_eq!(document.numerical_identity, prepared.numerics.key);
    assert!(
        document
            .fields
            .iter()
            .all(|field| field.document == Some(SemanticId::from_bytes([117; 16])))
    );
    let equations = model
        .model
        .equations
        .iter()
        .map(|row| row.id)
        .collect::<BTreeSet<_>>();
    let occurrences = document
        .instances
        .iter()
        .filter(|instance| equations.contains(&instance.instance))
        .collect::<Vec<_>>();
    assert_eq!(occurrences.len(), 2);
    assert_eq!(
        occurrences[0].body, occurrences[1].body,
        "shared arithmetic retains separate original occurrence bindings"
    );
    assert_ne!(occurrences[0].instance, occurrences[1].instance);
    // Only explicit validity-check targets own checked-member tokens. All
    // original locals also remain in the separate instantiated member map;
    // ordinary consumed inputs are joined through slots, not invented checks.
    let checked = |instance: &SourceInstanceDocument| {
        instance
            .checked_members
            .iter()
            .map(|(_, original)| *original)
            .collect::<BTreeSet<_>>()
    };
    let first = checked(occurrences[0]);
    let second = checked(occurrences[1]);
    assert!(
        first.len() >= 2 && second.len() >= 2,
        "explicitly checked computed locals and consumed inputs all retain original attribution"
    );
    assert!(
        first.is_disjoint(&second),
        "a shared source token never substitutes a sibling occurrence's member"
    );
    for instance in occurrences {
        let equation = document
            .members
            .iter()
            .find(|member| member.member == instance.instance)
            .unwrap();
        let locals = document
            .members
            .iter()
            .filter(|member| {
                member.instance == equation.instance
                    && (member.path.ends_with(".a") || member.path.ends_with(".b"))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            locals.len(),
            2,
            "all original computed locals survive independently of check-token eligibility"
        );
        assert!(
            instance.checked_members.iter().any(|(_, actual)| locals
                .iter()
                .any(|local| local.member == *actual && local.path.ends_with(".b"))),
            "the explicitly checked computed local joins its original instance"
        );
        assert!(
            instance.slots.iter().all(|slot| {
                document.members.iter().any(|member| {
                    member.member == slot.member && member.instance == equation.instance
                })
            }),
            "every consumed input retains the original sibling instance"
        );
        assert!(!instance.demands.is_empty());
        assert!(
            instance
                .contributions
                .iter()
                .any(|contribution| matches!(contribution.target,
            SourceContributionTarget::Row { row } if row == instance.instance))
        );
        for demand in &instance.demands {
            let template = document
                .templates
                .iter()
                .find(|template| template.template == demand.template)
                .unwrap();
            assert_eq!(template.body, instance.body);
            assert_eq!(demand.outputs, template.outputs);
            assert_eq!(demand.coordinates, template.coordinates);
            let regions = template
                .arithmetic
                .iter()
                .map(|origin| origin.region)
                .collect::<BTreeSet<_>>();
            assert!(
                regions.len() >= 2,
                "both static branch alternatives survive in this selected source view"
            );
            assert!(
                template
                    .arithmetic
                    .iter()
                    .all(|origin| origin.start_output < origin.end_output)
            );
            assert!(
                template
                    .arithmetic
                    .iter()
                    .all(|origin| origin.source.is_some() == origin.source_unavailable.is_none())
            );
        }
    }
    assert_eq!(document.rows.len(), 2);
    assert_eq!(document.columns.len(), 2);
    assert!(
        document.fixed_inputs.len() >= 2,
        "fixed predicate parameters remain consumed inputs"
    );
    assert!(
        document
            .native_pullback_unavailable
            .contains("no transformed native attempt")
    );
    assert!(
        !serde_json::to_string(&owner)
            .unwrap()
            .contains("executed_branch"),
        "static eligibility never claims an executed arm"
    );
}

#[tokio::test]
async fn kernel_source_map_explicit_second_product_labels_value_check_origins_without_extra_actions_or_evaluation()
 {
    let (runtime, _package, mut prepared) = prepared_cells(false).await;
    prepared.model.case = runtime
        .shared
        .math()
        .prepare_order(
            prepared.model.case.clone(),
            DerivativeOrder::Second,
            pse_diagnostics::Cancellation::default(),
        )
        .await
        .unwrap();
    let owner = runtime
        .modeling_diagnostic_source_map(
            &prepared,
            2 << 20,
            Duration::from_secs(10),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let weighted = owner
        .document()
        .templates
        .iter()
        .filter(|template| template.product == ProductKind::WeightedHessian)
        .collect::<Vec<_>>();
    assert!(
        !weighted.is_empty(),
        "the control explicitly prepared the Second weighted-matrix product"
    );
    for template in weighted {
        assert!(matches!(
            template.arithmetic_mapping,
            ArithmeticMappingScope::OriginalValueChecksForWeightedProduct
        ));
        assert!(!template.arithmetic.is_empty());
        assert!(
            template
                .arithmetic
                .iter()
                .all(|origin| origin.check_order == DerivativeOrder::Value),
            "retained original Value checks are never labelled derivative-kernel instructions"
        );
    }
    assert!(
        owner.document().templates.iter().all(|template| !matches!(
            template.product,
            ProductKind::JacobianAction | ProductKind::WeightedHessianAction
        )),
        "inspection does not silently prepare unrequested action products"
    );
    // A control-bearing product is deliberately not flattened. Its source ranges
    // belong to the selected compiled product, rather than a fake Value-only view.
    let (runtime, _package, mut branch) = prepared_cells(true).await;
    branch.model.case = runtime
        .shared
        .math()
        .prepare_order(
            branch.model.case.clone(),
            DerivativeOrder::Second,
            pse_diagnostics::Cancellation::default(),
        )
        .await
        .unwrap();
    let branch = runtime
        .modeling_diagnostic_source_map(
            &branch,
            2 << 20,
            Duration::from_secs(10),
            &crate::CancelSource::new(),
        )
        .await
        .unwrap();
    let selected = branch
        .document()
        .templates
        .iter()
        .filter(|template| template.product == ProductKind::WeightedHessian)
        .collect::<Vec<_>>();
    assert!(!selected.is_empty());
    assert!(selected.iter().all(|template| matches!(
        template.arithmetic_mapping,
        ArithmeticMappingScope::SelectedCompiledProduct
    )));
    assert!(selected.iter().all(|template| {
        template
            .arithmetic
            .iter()
            .any(|origin| origin.check_order == DerivativeOrder::Second)
    }));
}

#[tokio::test]
async fn kernel_source_map_finite_byte_clock_and_cancel_refusals_leave_prepared_source_and_admission_intact()
 {
    let (runtime, _package, prepared) = prepared_cells(true).await;
    let cancel = crate::CancelSource::new();
    // Warm only the explicitly requested immutable assembly before comparing
    // refusal cleanup; the source owner and its cache retain their normal leases.
    let initial = runtime
        .modeling_diagnostic_source_map(&prepared, 2 << 20, Duration::from_secs(10), &cancel)
        .await
        .unwrap();
    let original = serde_json::to_vec(&initial).unwrap();
    drop(initial);
    let service = runtime.shared.math();
    let before = service.test_admission_snapshot();
    assert!(
        runtime
            .modeling_diagnostic_source_map(&prepared, 1, Duration::from_secs(10), &cancel)
            .await
            .is_err()
    );
    assert!(
        runtime
            .modeling_diagnostic_source_map(&prepared, 2 << 20, Duration::ZERO, &cancel)
            .await
            .is_err()
    );
    let expired = crate::CancelSource::new().with_deadline(Some(std::time::Instant::now()));
    assert!(
        runtime
            .modeling_diagnostic_source_map(&prepared, 2 << 20, Duration::from_secs(10), &expired)
            .await
            .is_err(),
        "mapping cannot renew the caller's preparation clock"
    );
    assert!(
        !expired.token().is_cancelled(),
        "deadline remains independent from cancellation"
    );
    let stopped = crate::CancelSource::new();
    stopped.cancel();
    assert!(
        runtime
            .modeling_diagnostic_source_map(&prepared, 2 << 20, Duration::from_secs(10), &stopped)
            .await
            .is_err()
    );
    assert_eq!(
        service.test_admission_snapshot(),
        before,
        "refused source inspection releases its own finite permission and no unrelated owner"
    );
    let recovered = runtime
        .modeling_diagnostic_source_map(&prepared, 2 << 20, Duration::from_secs(10), &cancel)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_vec(&recovered).unwrap(),
        original,
        "failed inspection does not mutate retained source maps or numerical inputs"
    );
}

fn retained_map_with_unused_fixed_formal() -> ModelingSourceMapDocument {
    let row = SemanticId::from_bytes([121; 16]);
    let variable = SemanticId::from_bytes([122; 16]);
    let fixed = SemanticId::from_bytes([123; 16]);
    let body = ContentHash::from_bytes([124; 32]);
    ModelingSourceMapDocument {
        version: Default::default(),
        source_identity: body,
        numerical_identity: body,
        scope: SourceMapScope::StaticEligibleAlternatives,
        rows: vec![row],
        columns: vec![variable],
        fixed_inputs: vec![],
        row_nominals: vec![(row, 1.)],
        variable_nominals: vec![(variable, 1.)],
        objective_signs: vec![],
        fields: vec![],
        members: vec![SourceMemberDocument {
            member: fixed,
            declaration: fixed,
            instance: row,
            path: "cell.unused_fixed".into(),
        }],
        templates: vec![SourceTemplateDocument {
            template: 0,
            body,
            order: DerivativeOrder::First,
            product: ProductKind::SelectedPartials,
            outputs: vec![0],
            coordinates: vec![0],
            arithmetic_mapping: ArithmeticMappingScope::SelectedCompiledProduct,
            arithmetic: vec![],
        }],
        instances: vec![SourceInstanceDocument {
            ordinal: 0,
            instance: row,
            body,
            checked_members: vec![],
            slots: vec![
                SourceSlotDocument {
                    formal: 0,
                    member: variable,
                    scale: 1.,
                    offset: 0.,
                    fixed: false,
                },
                SourceSlotDocument {
                    formal: 1,
                    member: fixed,
                    scale: 2.,
                    offset: 3.,
                    fixed: true,
                },
            ],
            contributions: vec![SourceContributionDocument {
                output: 0,
                target: SourceContributionTarget::Row { row },
                scale: 1.,
            }],
            demands: vec![SourceDemandDocument {
                demand: EvaluationDemand::Constraints,
                template: 0,
                outputs: vec![0],
                coordinates: vec![0],
            }],
        }],
        native_pullback_unavailable: "no transformed native pullback".into(),
    }
}

#[test]
fn retained_source_map_allows_unused_original_fixed_formals_outside_consumed_frame() {
    // Producer-shaped complete original gather: only formal zero is demanded by
    // the selected compiled template, so formal one has no compact fixed read.
    let mut map = retained_map_with_unused_fixed_formal();
    let unused = &map.instances[0].slots[1];
    assert!(!map.fixed_inputs.contains(&unused.member));
    assert!(
        !map.instances[0].demands[0]
            .coordinates
            .contains(&unused.formal)
    );
    map.validate().unwrap();
    let restored: ModelingSourceMapDocument =
        serde_json::from_slice(&serde_json::to_vec(&map).unwrap()).unwrap();
    restored.validate().unwrap();
    assert_eq!(restored.instances[0].slots[1].offset, 3.);
    map.instances[0].slots[1].fixed = false;
    assert!(
        map.validate().is_err(),
        "absent free columns cannot be declared free slots"
    );
    map.instances[0].slots[1].fixed = true;
    map.instances[0].slots[0].fixed = true;
    assert!(
        map.validate().is_err(),
        "actual free columns cannot be declared fixed slots"
    );
}

#[test]
fn retained_source_map_rejects_contradictory_member_lineage_in_either_order() {
    let map = retained_map_with_unused_fixed_formal();
    let original = map.members[0].clone();
    let mut repeated = map.clone();
    repeated.members.push(original.clone());
    repeated.validate().unwrap();
    for field in 0..3 {
        let mut conflicting = original.clone();
        match field {
            0 => conflicting.declaration = SemanticId::from_bytes([125; 16]),
            1 => conflicting.instance = SemanticId::from_bytes([126; 16]),
            _ => conflicting.path = "other.unused_fixed".into(),
        }
        for members in [
            vec![original.clone(), conflicting.clone()],
            vec![conflicting, original.clone()],
        ] {
            let mut contradictory = map.clone();
            contradictory.members = members;
            assert!(
                contradictory.validate().is_err(),
                "first-match order cannot select member meaning"
            );
        }
    }
}
