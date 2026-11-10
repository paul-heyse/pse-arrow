// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Ordered pure-property tables over the existing selected observation artifact owner.
use super::*;
use pse_compiler::workspace::Profile;
use pse_math::binding::CaseValues;
use std::collections::BTreeSet;
impl ModelingPackage {
    /// Evaluate an ordered independent population of the same selected authored
    /// properties. Source admission, local physical transport and applicability stay
    /// attached to each original point; provider-effect populations refuse this route.
    pub async fn observe_batch(
        &self,
        services: &crate::workflow::PreparationServices,
        model: ModelingPreparation,
        rows: BTreeSet<SemanticId>,
        points: Vec<(SemanticId, CaseValues)>,
        profile: Profile,
        cancel: &crate::CancelSource,
    ) -> Result<Arc<pse_columnar::Leased<pse_math::assembly::CaseBatchOutcome>>, WorkflowError>
    {
        self.check_service_bindings(services)?;
        let driver = services.math().preparation_driver(cancel)?;
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                if let Some(original) = &model.original
                    && let Some(program) = self
                        .numerical_observation_program(
                            services,
                            None,
                            original.root,
                            original.instance,
                            original.bindings.clone(),
                            original.limits,
                            rows.iter().copied().collect(),
                            Vec::new(),
                            pse_kernels::DerivativeOrder::Value,
                            profile,
                            &driver,
                        )
                        .await?
                {
                    return services
                        .math()
                        .evaluate_numerical_observation_batch(program, points, &driver)
                        .await
                        .map_err(Into::into);
                }
                let program = self
                    .observation_program(services, &model, &rows, profile, &driver)
                    .await?;
                services
                    .math()
                    .evaluate_observation_batch(program, points, &driver)
                    .await
                    .map_err(Into::into)
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_modeling::{Bindings, Limits};
    #[cfg(feature = "solver-ipopt")]
    #[tokio::test]
    async fn ordered_authored_caloric_table_uses_shared_population_and_original_temperature_domain()
    {
        let runtime = crate::workflow::tests::runtime();
        let text = format!(
            r#"{}
package caloric_population {{ use correlations @"1.0.0";
def Root {{var T:Temperature;var reference:Temperature;var h:DeltaH;var s:DeltaS;
eq heat:correlations.dippr100_dh_value(reference,T,75.3{{J/(mol*K)}},0{{J/(mol*K^2)}},0{{J/(mol*K^3)}},0{{J/(mol*K^4)}},0{{J/(mol*K^5)}})==h;
eq entropy:correlations.dippr100_ds_value(reference,T,75.3{{J/(mol*K)}},0{{J/(mol*K^2)}},0{{J/(mol*K^3)}},0{{J/(mol*K^4)}},0{{J/(mol*K^5)}})==s;
}} }}"#,
            crate::reference_source_fixture::reference_family_sources("bounded-reference")
        );
        let declarations = crate::reference_source_fixture::rows(&text);
        let root = declarations
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, crate::workflow::tests::physical())
            .await
            .unwrap();
        let driver = crate::CancelSource::new();
        let services = runtime.preparation_services();
        let model = package
            .prepare(
                &services,
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings {
                    demand: vec!["T".into(), "reference".into(), "h".into(), "s".into()],
                    ..Default::default()
                },
                Limits::default(),
                &driver,
            )
            .await
            .unwrap();
        let symbol = |name: &str| {
            model
                .compiled()
                .model
                .symbols
                .values()
                .find(|symbol| symbol.lineage.path == format!("Root.{name}"))
                .unwrap()
                .id
        };
        let t = symbol("T");
        let reference = symbol("reference");
        let h = symbol("h");
        let s = symbol("s");
        let rows = model
            .compiled()
            .model
            .equations
            .iter()
            .map(|row| row.id)
            .collect::<BTreeSet<_>>();
        let heat = model
            .compiled()
            .model
            .equations
            .iter()
            .find(|row| row.lineage.path == "Root.heat")
            .unwrap()
            .id;
        let entropy = model
            .compiled()
            .model
            .equations
            .iter()
            .find(|row| row.lineage.path == "Root.entropy")
            .unwrap()
            .id;
        let heat = rows.iter().position(|id| *id == heat).unwrap();
        let entropy = rows.iter().position(|id| *id == entropy).unwrap();
        let temperatures = [
            300_f64.next_up(),
            320.,
            250.,
            0.,
            300.,
            350.,
            280.,
            300_f64.next_down(),
        ];
        let points = || {
            temperatures
                .into_iter()
                .enumerate()
                .map(|(i, value)| {
                    (
                        SemanticId::from_bytes([(i % 3 + 1) as u8; 16]),
                        CaseValues {
                            scalars: BTreeMap::from([
                                (t, value),
                                (reference, 300.),
                                (h, 0.),
                                (s, 0.),
                            ]),
                        },
                    )
                })
                .collect()
        };
        #[cfg(feature = "compiled-evaluator")]
        let evaluators = [
            pse_math::library::Evaluator::Interpreted,
            pse_math::library::Evaluator::CompiledX4 {
                module_bytes: 16 << 20,
            },
        ];
        #[cfg(not(feature = "compiled-evaluator"))]
        let evaluators = [pse_math::library::Evaluator::Interpreted];
        for evaluator in evaluators {
            let mut profile = crate::workflow::tests::compiler_profile();
            profile.optimization.evaluator = evaluator;
            for repeat in 0..2 {
                let before = services.math().preparation_effects();
                let table = package
                    .observe_batch(
                        &services,
                        model.clone(),
                        rows.clone(),
                        points(),
                        profile,
                        &driver,
                    )
                    .await
                    .unwrap();
                assert_eq!(table.points().len(), temperatures.len());
                for (i, temperature) in temperatures.into_iter().enumerate() {
                    assert_eq!(
                        table.points()[i].id,
                        SemanticId::from_bytes([(i % 3 + 1) as u8; 16])
                    );
                    if temperature == 0. {
                        assert!(table.row(i).is_none());
                        assert!(matches!(
                            table.points()[i].outcome,
                            pse_math::batch::PointOutcome::Failed(
                                pse_math::MathError::Instance { .. }
                            )
                        ));
                    } else {
                        let values = table.row(i).unwrap();
                        let dh = 75.3 * (temperature - 300.);
                        let ds = 75.3 * ((temperature - 300.) / 300.).ln_1p();
                        assert!(
                            (values[heat] - dh).abs() <= dh.abs() * 4e-14 + 1e-29,
                            "heat at {temperature}: {} vs {dh}",
                            values[heat]
                        );
                        assert!(
                            (values[entropy] - ds).abs() <= ds.abs() * 4e-14 + 1e-29,
                            "entropy at {temperature}: {} vs {ds}",
                            values[entropy]
                        );
                    }
                }
                if repeat != 0 {
                    assert_eq!(
                        services.math().preparation_effects().completions,
                        before.completions,
                        "compatible table reuses mathematical preparation"
                    );
                }
            }
        }
    }
    #[tokio::test]
    async fn ordered_property_batch_admits_multiinstance_population_on_original_runtime_job() {
        let runtime = crate::workflow::tests::runtime();
        let declarations=pse_authoring::language::parse("package p {def Cell {var x:Scalar;eq response:log1p(x)==0;}def Root {child a:Cell=Cell();child b:Cell=Cell();}}",SemanticId::from_bytes([118;16]),pse_authoring::language::IdentityPolicy::Named,Default::default()).unwrap();
        let root = declarations
            .iter()
            .find(|d| d.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, crate::workflow::tests::physical())
            .await
            .unwrap();
        let driver = crate::CancelSource::new();
        let services = runtime.preparation_services();
        let model = package
            .prepare(
                &services,
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                &driver,
            )
            .await
            .unwrap();
        let x = model
            .compiled()
            .model
            .symbols
            .values()
            .filter(|s| s.lineage.path.ends_with(".x"))
            .map(|s| s.id)
            .collect::<Vec<_>>();
        assert_eq!(x.len(), 2);
        let rows = model
            .compiled()
            .model
            .equations
            .iter()
            .map(|e| e.id)
            .collect::<BTreeSet<_>>();
        assert_eq!(rows.len(), 2);
        let points = (0..8)
            .map(|index| {
                (
                    SemanticId::from_bytes([index; 16]),
                    CaseValues {
                        scalars: BTreeMap::from([(x[0], 1e-30), (x[1], 1e-30)]),
                    },
                )
            })
            .collect();
        let result = package
            .observe_batch(
                &services,
                model,
                rows,
                points,
                crate::workflow::tests::compiler_profile(),
                &driver,
            )
            .await
            .unwrap();
        assert_eq!(result.points().len(), 8);
        for index in 0..8 {
            assert_eq!(result.row(index), Some([1e-30, 1e-30].as_slice()));
        }
    }
    #[tokio::test]
    async fn ordered_property_batch_reuses_selected_artifact_and_retains_original_failures() {
        let runtime = crate::workflow::tests::runtime();
        let text = "package p {def Root {var x:Scalar;eq response:log1p(x)==0;}}";
        let declarations = pse_authoring::language::parse(
            text,
            SemanticId::from_bytes([119; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|d| d.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, crate::workflow::tests::physical())
            .await
            .unwrap();
        let driver = crate::CancelSource::new();
        let services = runtime.preparation_services();
        let model = package
            .prepare(
                &services,
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                Limits::default(),
                &driver,
            )
            .await
            .unwrap();
        let x = model
            .compiled()
            .model
            .symbols
            .values()
            .find(|s| s.lineage.path == "Root.x")
            .unwrap()
            .id;
        let row = model
            .compiled()
            .model
            .equations
            .iter()
            .find(|e| e.lineage.path == "Root.response")
            .unwrap()
            .id;
        let ids = [
            SemanticId::from_bytes([1; 16]),
            SemanticId::from_bytes([2; 16]),
            SemanticId::from_bytes([1; 16]),
        ];
        let points = ids
            .into_iter()
            .zip([1e-30, -1., -1e-30])
            .map(|(id, value)| {
                (
                    id,
                    CaseValues {
                        scalars: BTreeMap::from([(x, value)]),
                    },
                )
            })
            .collect();
        let result = package
            .observe_batch(
                &services,
                model.clone(),
                BTreeSet::from([row]),
                points,
                crate::workflow::tests::compiler_profile(),
                &driver,
            )
            .await
            .unwrap();
        assert_eq!(
            result.points().iter().map(|p| p.id).collect::<Vec<_>>(),
            ids
        );
        assert_eq!(result.row(0).unwrap()[0].to_bits(), 1e-30_f64.to_bits());
        assert_eq!(result.row(2).unwrap()[0].to_bits(), (-1e-30_f64).to_bits());
        assert!(result.row(1).is_none());
        assert!(
            matches!(&result.points()[1].outcome,pse_math::batch::PointOutcome::Failed(pse_math::MathError::Instance {cause,..}) if matches!(cause.as_ref(),pse_math::MathError::Domain{..}))
        );
        let empty = package
            .observe_batch(
                &services,
                model,
                BTreeSet::from([row]),
                vec![],
                crate::workflow::tests::compiler_profile(),
                &driver,
            )
            .await
            .unwrap();
        assert!(empty.points().is_empty());
        assert!(!empty.canceled());
    }
}
