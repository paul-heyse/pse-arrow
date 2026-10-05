// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original-space authored solves use the same retained result and publication boundary.
use super::{RunRequest, RunResult, WorkflowError, contract, relation};
use crate::math::solves::Outcome;
use pse_ids::SemanticId;
use pse_relations::{
    columnar::{FieldCheckedBatch, RelationRow},
    generated::{
        enums::DualQualification,
        runtime::{
            incumbents, infeasibility_certificates as certificates, modeling_checks,
            modeling_findings, modeling_reports, solution_pool as pool,
            solve_constraints as constraints, solve_metrics as metrics, solve_runs as runs,
            solve_strategy_events, solve_variables as variables,
        },
    },
};
use std::collections::BTreeMap;
impl RunResult {
    pub(super) fn encode_modeling(
        &self,
    ) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, WorkflowError> {
        let RunRequest::Modeling(requests) = &self.request else {
            return Err(contract("authored solve request mismatch"));
        };
        let registry = &self.runtime.registry;
        let bytes = requests
            .iter()
            .try_fold(0usize, |total, request| {
                let declaration = request.model.case.compiled().plan.structure();
                let cells = declaration
                    .variables()
                    .len()
                    .checked_add(declaration.parameters().len())?
                    .checked_add(declaration.rows().len())?;
                total.checked_add(cells.checked_mul(2048)?)?.checked_add(
                    request
                        .profile
                        .controls
                        .report_allowance()
                        .ok()?
                        .checked_mul(8)?,
                )
            })
            .ok_or_else(|| contract("authored sequence export extent"))?;
        let _scratch = self
            .runtime
            .shared
            .math()
            .reserve("modeling:solve-export", bytes)?;
        let validation = self.runtime.validation_context()?;
        let mut run_rows = runs::Builder::with_registry(registry, requests.len(), &validation)
            .map_err(relation)?;
        for row in &self
            .completion()
            .map_err(|e| contract(e.to_string()))?
            .solves
        {
            run_rows.push(row.clone()).map_err(relation)?;
        }
        let pool = self.runtime.shared.pool();
        let cancel = pse_columnar::CancellationToken::new();
        let validation = self.runtime.validation_context()?;
        let mut collection =
            pse_relations::columnar::Collection::new(registry, &pool, &cancel, &validation);
        collection
            .ensure::<modeling_checks::Row>()
            .map_err(relation)?;
        collection
            .ensure::<modeling_reports::Row>()
            .map_err(relation)?;
        collection
            .ensure::<modeling_findings::Row>()
            .map_err(relation)?;
        collection
            .ensure::<solve_strategy_events::Row>()
            .map_err(relation)?;
        collection
            .ensure::<pse_model::generated::runtime::solve_strategy_products::Row>()
            .map_err(relation)?;
        let mut variable_rows =
            variables::Builder::with_registry(registry, 0, &validation).map_err(relation)?;
        let mut constraint_rows =
            constraints::Builder::with_registry(registry, 0, &validation).map_err(relation)?;
        let mut metric_rows =
            metrics::Builder::with_registry(registry, 0, &validation).map_err(relation)?;
        let mut pool_rows =
            pool::Builder::with_registry(registry, 0, &validation).map_err(relation)?;
        let mut certificate_rows =
            certificates::Builder::with_registry(registry, 0, &validation).map_err(relation)?;
        let mut local = super::local_analysis::Rows::new(registry, &validation)?;
        // A durable run publishes its incumbent stream as the store held it when the
        // attempt ended (Plan 22 I13); an ephemeral run keeps its retained incumbents in
        // runtime.solve_metrics.
        let mut incumbent_rows =
            incumbents::Builder::with_registry(registry, 0, &validation).map_err(relation)?;
        if let super::RunDurability::Durable(record) = &self.durability {
            let stored = record
                .incumbents
                .as_ref()
                .map_err(|error| WorkflowError::Shared(error.clone()))?;
            for incumbent in stored {
                incumbent_rows
                    .push(incumbents::Row {
                        run_id: self.run_id,
                        seq: incumbent.seq,
                        step: i64::from(incumbent.step),
                        elapsed_seconds: incumbent.elapsed_seconds,
                        phase: incumbent.phase.clone(),
                        objective: incumbent.objective,
                        dual_bound: incumbent.dual_bound,
                        gap: incumbent.gap,
                        nodes: incumbent.nodes,
                        seconds: incumbent.seconds,
                        solution_id: incumbent.solution_id.map(|s| s.as_id()),
                    })
                    .map_err(relation)?;
            }
        }
        for (ordinal, request) in requests.iter().enumerate() {
            let step = ordinal as i64;
            let result = self.modeling_result(ordinal);
            let trace = result
                .and_then(|result| result.strategy.as_ref())
                .or_else(|| {
                    self.modeling_error(ordinal)
                        .and_then(WorkflowError::strategy_trace)
                });
            if let Some(trace) = trace {
                for row in trace
                    .product_rows(self.run_id, ordinal)
                    .map_err(crate::math::MathRuntimeError::from)?
                {
                    collection.push(row).map_err(relation)?;
                }
                for row in trace
                    .rows(self.run_id, ordinal)
                    .map_err(crate::math::MathRuntimeError::from)?
                {
                    collection.push(row).map_err(relation)?;
                }
            }
            // A staged result owns the actual rebound case admitted for this attempt.
            // Unattempted requests still publish the admission completed at submission.
            let prepared = result.map_or(request, |result| &result.prepared);
            let declaration = prepared.model.case.compiled().plan.structure();
            let admission = prepared
                .solve
                .route_decision()
                .ok_or_else(|| contract("retained solve admission absent"))?;
            let identity = prepared.admission_identity()?;
            collection
                .push(admission.row(identity, step))
                .map_err(relation)?;
            if let Some(assessment) = &admission.structure {
                collection
                    .push(assessment.row(identity, step))
                    .map_err(relation)?;
            }
            let outcome = result.map(|r| &r.outcome);
            let native = match outcome {
                Some(Outcome::Native(r)) => Some(r.as_ref()),
                _ => None,
            };
            let constant = match outcome {
                Some(Outcome::Constant(r)) => Some(r.as_ref()),
                _ => None,
            };
            let observation = native
                .and_then(|r| r.observation.as_ref())
                .or_else(|| constant.map(|r| &r.observation));
            let candidate = native.and_then(|r| r.candidate.as_ref());
            // Multipliers read from a KKT point that certified the requested sensitivities
            // are sensitivity certified: independent active gradients, strict
            // complementarity and second-order sufficiency hold (ADR-0118 item 10).
            let certified = native
                .and_then(|r| r.evidence.sensitivity.as_ref())
                .is_some_and(|p| p.sensitivities.is_ok());
            let qualified = |o: &pse_backend_native::quality::Observation| {
                if o.dual_error.is_some() {
                    DualQualification::UnavailableOrInvalid
                } else if certified {
                    DualQualification::SensitivityCertified
                } else {
                    DualQualification::EvaluatedKktNotSensitivityCertified
                }
            };

            let coordinates: BTreeMap<_, _> = native
                .map(|r| {
                    r.variables
                        .iter()
                        .enumerate()
                        .map(|(i, id)| (*id, i))
                        .collect()
                })
                .unwrap_or_default();
            let original_coordinates: BTreeMap<_, _> = constant
                .map(|r| r.coordinates.iter().copied().collect())
                .unwrap_or_default();
            let dual_status = native
                .and_then(|r| r.observation.as_ref())
                .map_or(DualQualification::Unavailable, qualified);
            for v in declaration.variables() {
                let p = &v.port;
                let ix = coordinates.get(&p.id).copied();
                let value = if v.fixed {
                    prepared.model.values.scalars.get(&p.id).copied()
                } else if constant.is_some() {
                    original_coordinates.get(&p.id).copied()
                } else {
                    ix.and_then(|i| candidate.and_then(|c| c.primal.get(i).copied()))
                };
                let tolerance = if v.fixed {
                    None
                } else {
                    let i = prepared
                        .model
                        .case
                        .compiled()
                        .plan
                        .columns()
                        .iter()
                        .position(|id| *id == p.id);
                    i.and_then(|i| prepared.solve.tolerances().variables.get(i).copied())
                };
                variable_rows
                    .push(variables::Row {
                        run_id: self.run_id,
                        step,
                        symbol_id: p.id,
                        quantity_id: Some(p.quantity.as_id()),
                        unit_id: Some(p.unit.as_id()),
                        fixed: v.fixed,
                        parameter: false,
                        domain: Some(v.domain),
                        value,
                        lower: v.lower,
                        upper: v.upper,
                        lower_violation: value.map(|x| v.lower.map_or(0.0, |l| (l - x).max(0.0))),
                        upper_violation: value.map(|x| v.upper.map_or(0.0, |u| (x - u).max(0.0))),
                        tolerance,
                        lower_dual: ix.and_then(|i| {
                            candidate.and_then(|c| {
                                c.bound_dual.as_ref().and_then(|(l, _)| l.get(i).copied())
                            })
                        }),
                        upper_dual: ix.and_then(|i| {
                            candidate.and_then(|c| {
                                c.bound_dual.as_ref().and_then(|(_, u)| u.get(i).copied())
                            })
                        }),
                        reduced_cost: ix.and_then(|i| {
                            candidate.and_then(|c| {
                                c.reduced_costs.as_ref().and_then(|v| v.get(i).copied())
                            })
                        }),
                        stationarity: ix.and_then(|i| {
                            observation.and_then(|o| {
                                o.stationarity.as_ref().and_then(|v| v.get(i).copied())
                            })
                        }),
                        dual_qualification: dual_status,
                    })
                    .map_err(relation)?;
            }
            for p in declaration.parameters() {
                variable_rows
                    .push(variables::Row {
                        run_id: self.run_id,
                        step,
                        symbol_id: p.id,
                        quantity_id: Some(p.quantity.as_id()),
                        unit_id: Some(p.unit.as_id()),
                        fixed: true,
                        parameter: true,
                        domain: None,
                        value: prepared.model.values.scalars.get(&p.id).copied(),
                        lower: None,
                        upper: None,
                        lower_violation: None,
                        upper_violation: None,
                        tolerance: None,
                        lower_dual: None,
                        upper_dual: None,
                        reduced_cost: None,
                        stationarity: None,
                        dual_qualification: DualQualification::NotApplicableParameter,
                    })
                    .map_err(relation)?;
            }
            let rows = declaration.rows();
            for (i, r) in rows.iter().enumerate() {
                let unit = prepared
                    .source
                    .physical
                    .quantities
                    .quantity_type(r.quantity)
                    .map_err(super::math)?
                    .canonical_unit
                    .as_id();
                constraint_rows
                    .push(constraints::Row {
                        run_id: self.run_id,
                        step,
                        row_id: r.id,
                        quantity_id: Some(r.quantity.as_id()),
                        unit_id: Some(unit),
                        value: observation.and_then(|o| o.values.get(i).copied()),
                        lower: r.lower.is_finite().then_some(r.lower),
                        upper: r.upper.is_finite().then_some(r.upper),
                        equality_residual: observation
                            .and_then(|o| o.equality_residuals.get(i).copied().flatten()),
                        lower_violation: observation
                            .and_then(|o| o.lower_violations.get(i).copied()),
                        upper_violation: observation
                            .and_then(|o| o.upper_violations.get(i).copied()),
                        tolerance: prepared.solve.tolerances().rows.get(i).copied(),
                        dual: candidate
                            .and_then(|c| c.row_dual.as_ref().and_then(|v| v.get(i).copied())),
                        dual_qualification: dual_status,
                    })
                    .map_err(relation)?;
            }

            if let Some(native) = native {
                let stored = self.stored_events(step)?;
                let events = stored.as_deref().map_or(
                    super::results::StepEvents::Retained,
                    super::results::StepEvents::Stored,
                );
                super::results::push_native_metrics(
                    &mut metric_rows,
                    self.run_id,
                    step,
                    native,
                    events,
                )?;
                if let Some(row) = super::results::certificate_row(self.run_id, step, native) {
                    certificate_rows.push(row).map_err(relation)?;
                }
            }
            // Ranked pooled solutions over the report's free variables (ADR-0105 §8).
            for solution in native
                .and_then(|r| r.global.as_ref())
                .map_or(&[][..], |g| g.pool.as_slice())
            {
                let rank =
                    i64::try_from(solution.rank).map_err(|_| contract("solution pool rank"))?;
                for (symbol_id, value) in native
                    .map_or(&[][..], |r| r.variables.as_slice())
                    .iter()
                    .zip(&solution.primal)
                {
                    pool_rows
                        .push(pool::Row {
                            run_id: self.run_id,
                            step,
                            rank,
                            symbol_id: *symbol_id,
                            value: *value,
                            objective: solution.objective,
                            feasible: solution.feasible,
                        })
                        .map_err(relation)?;
                }
            }
            // Every quantity a sensitivity request asked for, certified or withheld.
            if let Some(sensitivity) = &request.profile.sensitivity {
                local.push(&super::local_analysis::Step {
                    run_id: self.run_id,
                    step,
                    request: sensitivity,
                    report: native,
                    candidate: candidate.is_some(),
                    structure: declaration,
                    quantities: &request.source.physical.quantities,
                })?;
            }
            if let Some(result) = result {
                for row in &result.checks {
                    collection.push(row.clone()).map_err(relation)?;
                }
                for row in &result.reports {
                    collection.push(row.clone()).map_err(relation)?;
                }
            }
        }
        // Failures first, then the informational bound tightenings admission recorded for
        // each step's case (ADR-0103 item 4).
        let tightenings = requests.iter().flat_map(|request| {
            request
                .model
                .tightenings
                .iter()
                .map(pse_modeling::DomainTightening::boundary_diagnostic)
        });
        for (ordinal, finding) in self
            .capture_diagnostics()
            .into_iter()
            .chain(tightenings)
            .enumerate()
        {
            collection
                .push(super::modeling::analysis_tables::finding_row(
                    self.run_id,
                    ordinal as i64,
                    &finding,
                ))
                .map_err(relation)?;
        }
        let sources = requests
            .iter()
            .map(|p| p.source.source_tables())
            .collect::<Result<Vec<_>, _>>()?;
        let mut batches = sequence_sources(sources, &self.runtime, &validation)?;
        batches.extend(
            collection
                .finish()
                .map_err(relation)?
                .into_values()
                .map(|b| (b.relation_id(), b)),
        );
        batches.extend([
            (runs::RELATION_ID, run_rows.finish().map_err(relation)?),
            (
                variables::RELATION_ID,
                variable_rows.finish().map_err(relation)?,
            ),
            (
                constraints::RELATION_ID,
                constraint_rows.finish().map_err(relation)?,
            ),
            (
                metrics::RELATION_ID,
                metric_rows.finish().map_err(relation)?,
            ),
            (pool::RELATION_ID, pool_rows.finish().map_err(relation)?),
            (
                certificates::RELATION_ID,
                certificate_rows.finish().map_err(relation)?,
            ),
            (
                incumbents::RELATION_ID,
                incumbent_rows.finish().map_err(relation)?,
            ),
        ]);
        batches.extend(local.finish()?);
        self.retain_sources(&mut batches)?;
        Ok(batches)
    }
}

fn sequence_sources(
    mut sources: Vec<BTreeMap<SemanticId, FieldCheckedBatch>>,
    runtime: &super::Runtime,
    validation: &pse_relations::validate::ValidationContext,
) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, WorkflowError> {
    let mut batches = if sources.is_empty() {
        BTreeMap::new()
    } else {
        sources.remove(0)
    };
    // One package already exported and validated its complete source context.
    // Rebuilding it here would repeat the same validation for every study point.
    if sources.is_empty() {
        return Ok(batches);
    }
    let registry = &runtime.registry;
    let maximum_tables = batches
        .len()
        .checked_add(3)
        .ok_or_else(|| contract("sequence source extent"))?;
    let owner = runtime.shared.math().reserve(
        "modeling:sequence-source-map",
        super::modeling::source_map_extent(maximum_tables)?,
    )?;
    macro_rules! merge_source {
        ($module:ident,$key:expr) => {{
            use pse_relations::generated::authored::$module as wire;
            let mut rows = BTreeMap::new();
            for batch in batches
                .get(&wire::RELATION_ID)
                .into_iter()
                .chain(sources.iter().filter_map(|s| s.get(&wire::RELATION_ID)))
            {
                for row in wire::Row::rows(batch).map_err(relation)? {
                    if let Some(old) = rows.insert(($key)(&row), row.clone())
                        && old != row
                    {
                        return Err(contract("conflicting source context in authored sequence"));
                    }
                }
            }
            let mut builder =
                wire::Builder::with_registry(registry, rows.len(), validation).map_err(relation)?;
            for row in rows.into_values() {
                builder.push(row).map_err(relation)?;
            }
            batches.insert(wire::RELATION_ID, builder.finish().map_err(relation)?);
        }};
    }
    merge_source!(
        modeling_declarations,
        |r: &pse_relations::generated::authored::modeling_declarations::Row| r.declaration_id
    );
    merge_source!(
        documents,
        |r: &pse_relations::generated::authored::documents::Row| r.document_id
    );
    merge_source!(
        packages,
        |r: &pse_relations::generated::authored::packages::Row| r.package_id
    );
    for table in batches.values_mut() {
        *table = table.clone().with_export_owner(owner.clone());
    }
    Ok(batches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_relations::generated::authored::modeling_declarations as wire;
    use std::sync::Arc;

    #[tokio::test]
    async fn all_fixed_publication_keeps_constant_values_without_kkt_evidence() {
        use super::super::tests as fixture;
        use pse_relations::generated::enums::NativeRunState;
        let runtime = fixture::runtime();
        let declarations = pse_authoring::language::parse(
            "package p { def Root { param p:Scalar=4; var x:Scalar; annotation start x(7); eq balance:x+p==11; annotation check x(x>0); } }",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime
            .modeling_package(declarations, fixture::physical())
            .unwrap();
        let mut case = pse_compiler::workspace::ModelingCaseBindings {
            variables: BTreeMap::from([(
                "x".into(),
                pse_compiler::workspace::ModelingVariableState {
                    fixed: Some(true),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let requested = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                case.clone(),
                pse_kernels::DerivativeOrder::First,
                fixture::compiler_profile(),
                fixture::profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let symbol = |path: &str| {
            let suffix = format!(".{path}");
            requested
                .model
                .model
                .compiled()
                .model
                .symbols
                .values()
                .find(|s| s.lineage.path == path || s.lineage.path.ends_with(&suffix))
                .unwrap()
                .id
        };
        let (x, p) = (symbol("x"), symbol("p"));
        case.values.extend([("x".into(), 5.), ("p".into(), 6.)]);
        let rebound = package
            .prepare_solve(
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                case,
                pse_kernels::DerivativeOrder::First,
                fixture::compiler_profile(),
                fixture::profile(),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let completed = package
            .solve_case(
                rebound,
                fixture::compiler_profile(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let result = RunResult::joined(
            completed.run_id,
            runtime.clone(),
            RunRequest::Modeling(vec![requested]),
            None,
            Ok(super::super::RunReport::Modeling(vec![completed])),
        )
        .finished(None, false)
        .await;
        assert!(result.usable());
        let Outcome::Constant(report) = &result.modeling_result(0).unwrap().outcome else {
            panic!("all-fixed original evaluation must have no native attempt");
        };
        assert!(report.coordinates.is_empty());
        let rows = variables::Row::rows(&result.table("runtime.solve_variables").unwrap()).unwrap();
        assert_eq!(rows.len(), 2);
        let fixed = rows.iter().find(|r| r.symbol_id == x).unwrap();
        assert_eq!(fixed.value, Some(5.));
        assert!(fixed.fixed && !fixed.parameter);
        assert_eq!(fixed.dual_qualification, DualQualification::Unavailable);
        assert!(
            fixed.lower_dual.is_none()
                && fixed.upper_dual.is_none()
                && fixed.stationarity.is_none()
                && fixed.reduced_cost.is_none()
        );
        let parameter = rows.iter().find(|r| r.symbol_id == p).unwrap();
        assert_eq!(parameter.value, Some(6.));
        assert_eq!(
            parameter.dual_qualification,
            DualQualification::NotApplicableParameter
        );
        let rows =
            constraints::Row::rows(&result.table("runtime.solve_constraints").unwrap()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].value, report.observation.values.first().copied());
        assert_eq!(rows[0].equality_residual, Some(0.));
        assert!(rows[0].dual.is_none());
        assert_eq!(rows[0].dual_qualification, DualQualification::Unavailable);
        let rows = runs::Row::rows(&result.table("runtime.solve_runs").unwrap()).unwrap();
        assert_eq!(rows[0].state, NativeRunState::ConstantEvaluation);
        assert!(
            rows[0].backend.is_none()
                && rows[0].termination.is_none()
                && rows[0].native_code.is_none()
                && rows[0].native_status.is_none()
        );
        assert!(
            metrics::Row::rows(&result.table("runtime.solve_metrics").unwrap())
                .unwrap()
                .is_empty()
        );
    }

    #[cfg(feature = "solver-kinsol")]
    #[tokio::test]
    async fn complete_original_publication_uses_reconstructed_and_rebound_values() {
        use super::super::{ModelingAnalysis, RunReport, tests as fixture};
        use pse_relations::generated::enums::NativeRunState;
        for permitted in [true, false] {
            let runtime = fixture::runtime_on(
                512 << 20,
                crate::math::MathPolicy {
                    worker_bytes: 128 << 20,
                    workspace_bytes: 128 << 20,
                    foreign_bytes: 32 << 20,
                    ..Default::default()
                },
            );
            let pool = runtime.shared.pool();
            let threshold = if permitted { 3 } else { 5 };
            let declarations = pse_authoring::language::parse(
                &format!("package p {{ def Root {{ param p:Scalar=2; var x:Scalar; var y:Scalar; var z:Scalar; annotation start x(0); annotation start y(0); annotation start z(0); eq first:x==1; eq second:y==x+1; eq third:z==2*y; annotation check p(p>{threshold}); annotation report p(\"bound parameter\"); }} }}"),
                SemanticId::NIL,
                pse_authoring::language::IdentityPolicy::Named,
                Default::default(),
            ).unwrap();
            let root = declarations
                .iter()
                .find(|r| r.name == "Root")
                .unwrap()
                .declaration_id;
            let package = runtime
                .modeling_package(declarations, fixture::physical())
                .unwrap();
            let mut analysis = ModelingAnalysis {
                root,
                instance: pse_modeling::specialize::root_instance(root),
                bindings: Default::default(),
                limits: Default::default(),
                case: Default::default(),
                order: pse_kernels::DerivativeOrder::First,
                compiler: fixture::compiler_profile(),
                solver: crate::math::solves::SolverProfile {
                    intent: pse_backend_native::solve::SolveIntent::Root,
                    selection: pse_backend_native::solve::SolverSelection::Explicit(
                        pse_backend_native::solve::Backend::Kinsol,
                    ),
                    ..Default::default()
                },
                numerical: Default::default(),
            };
            let cancel = crate::CancelSource::new();
            let requested = package.prepare_analysis(&analysis, &cancel).await.unwrap();
            let symbol = |path: &str| {
                let suffix = format!(".{path}");
                requested
                    .model
                    .model
                    .compiled()
                    .model
                    .symbols
                    .values()
                    .find(|s| s.lineage.path == path || s.lineage.path.ends_with(&suffix))
                    .unwrap()
                    .id
            };
            let (p, x, y, z) = (symbol("p"), symbol("x"), symbol("y"), symbol("z"));
            // The executed step retains its rebound preparation; the submitted request
            // remains immutable, as it does for staged workflows.
            analysis.case.values.insert("p".into(), 4.);
            let rebound = package.prepare_analysis(&analysis, &cancel).await.unwrap();
            assert_eq!(
                rebound
                    .solve
                    .automatic_block_count(&Arc::default())
                    .unwrap(),
                3
            );
            let completed = package
                .solve_case(rebound, analysis.compiler, &cancel)
                .await
                .unwrap_or_else(|error| panic!("{}", error));
            assert_eq!(completed.accepted, permitted);
            let expected = BTreeMap::from([(p, 4.), (x, 1.), (y, 2.), (z, 4.)]);
            if permitted {
                let Outcome::Constant(report) = &completed.outcome else {
                    panic!("complete original block schedule must have no outer native attempt");
                };
                assert_eq!(report.coordinates.len(), 3);
                assert_eq!(report.component_reports().len(), 3);
                for (id, value) in &report.coordinates {
                    assert!((value - expected[id]).abs() < 1e-7);
                }
            }
            let result = RunResult::joined(
                completed.run_id,
                runtime.clone(),
                RunRequest::Modeling(vec![requested]),
                None,
                Ok(RunReport::Modeling(vec![completed])),
            )
            .finished(None, false)
            .await;
            assert_eq!(result.usable(), permitted);
            let completion_before = serde_json::to_value(result.completion().unwrap()).unwrap();
            let escaped = result.table("runtime.solve_variables").unwrap();
            let rows = variables::Row::rows(&escaped).unwrap();
            assert_eq!(rows.len(), expected.len());
            for row in &rows {
                assert!(
                    (row.value.unwrap() - expected[&row.symbol_id]).abs() < 1e-7,
                    "{row:?}"
                );
                assert_eq!(row.parameter, row.symbol_id == p);
                assert_eq!(row.fixed, row.symbol_id == p);
                if permitted {
                    assert!(
                        row.lower_dual.is_none()
                            && row.upper_dual.is_none()
                            && row.reduced_cost.is_none()
                            && row.stationarity.is_none()
                    );
                    assert_eq!(
                        row.dual_qualification,
                        if row.parameter {
                            DualQualification::NotApplicableParameter
                        } else {
                            DualQualification::Unavailable
                        }
                    );
                }
            }
            let constraints =
                constraints::Row::rows(&result.table("runtime.solve_constraints").unwrap())
                    .unwrap();
            assert_eq!(constraints.len(), 3);
            for row in constraints {
                assert!(row.equality_residual.unwrap().abs() < 1e-7);
                assert!(row.value.is_some());
                if permitted {
                    assert!(row.dual.is_none());
                    assert_eq!(row.dual_qualification, DualQualification::Unavailable);
                }
            }
            let checks =
                modeling_checks::Row::rows(&result.table("runtime.modeling_checks").unwrap())
                    .unwrap();
            assert_eq!(checks.len(), 1);
            assert_eq!(checks[0].satisfied, permitted);
            let reports =
                modeling_reports::Row::rows(&result.table("runtime.modeling_reports").unwrap())
                    .unwrap();
            assert_eq!(reports.len(), 1);
            assert_eq!(reports[0].value, 4.);
            let runs = runs::Row::rows(&result.table("runtime.solve_runs").unwrap()).unwrap();
            if permitted {
                assert_eq!(runs[0].state, NativeRunState::ConstantEvaluation);
                assert!(
                    runs[0].backend.is_none()
                        && runs[0].termination.is_none()
                        && runs[0].native_code.is_none()
                        && runs[0].native_status.is_none()
                );
                assert!(
                    metrics::Row::rows(&result.table("runtime.solve_metrics").unwrap())
                        .unwrap()
                        .is_empty()
                );
            }
            assert_eq!(
                serde_json::to_value(result.completion().unwrap()).unwrap(),
                completion_before,
                "publication cannot requalify the point"
            );
            drop((result, package, runtime));
            assert!(
                pool.reserved() > 0,
                "escaped checked rows retain their owners"
            );
            assert_eq!(variables::Row::rows(&escaped).unwrap(), rows);
            drop(escaped);
            assert_eq!(
                pool.reserved(),
                0,
                "last table releases the retained source, result and buffer charges"
            );
        }
    }

    #[test]
    fn single_source_keeps_retained_buffers_and_sequence_conflicts_are_refused() {
        let runtime = super::super::tests::runtime();
        let rows = pse_authoring::language::parse(
            "package application {}",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let package = runtime
            .modeling_package(rows.clone(), super::super::tests::physical())
            .unwrap();
        let source = package.source_tables().unwrap();
        let validation = runtime.validation_context().unwrap();
        let expected_keys = source.keys().copied().collect::<Vec<_>>();
        let expected = source[&wire::RELATION_ID].clone();
        let single = sequence_sources(vec![source], &runtime, &validation).unwrap();
        assert_eq!(single.keys().copied().collect::<Vec<_>>(), expected_keys);
        assert!(Arc::ptr_eq(
            &single[&wire::RELATION_ID].batch().columns()[0],
            &expected.batch().columns()[0],
        ));
        let matching = sequence_sources(
            vec![
                package.source_tables().unwrap(),
                package.source_tables().unwrap(),
            ],
            &runtime,
            &validation,
        )
        .unwrap();
        assert_eq!(
            wire::Row::rows(&matching[&wire::RELATION_ID]).unwrap(),
            rows
        );

        let mut conflict = rows;
        conflict[0].name = "conflicting_name".into();
        let conflicting = package
            .with_declarations(conflict)
            .unwrap()
            .source_tables()
            .unwrap();
        assert!(sequence_sources(vec![single, conflicting], &runtime, &validation).is_err());
    }
}
