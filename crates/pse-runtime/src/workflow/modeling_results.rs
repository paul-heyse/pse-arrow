// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Original-space authored solves use the same retained result and publication boundary.
use super::{RunRequest,RunReport,RunResult,WorkflowError,contract,relation};
use crate::math::solves::Outcome;
use pse_ids::SemanticId;
use pse_relations::{columnar::{FieldCheckedBatch,RelationRow},generated::{enums::DualQualification,runtime::{solve_runs as runs,solve_variables as variables,solve_constraints as constraints,solve_metrics as metrics,solution_pool as pool,modeling_checks,modeling_reports,modeling_findings}}};
use std::collections::BTreeMap;
impl RunResult {
    pub(super) fn encode_modeling(&self)->Result<BTreeMap<SemanticId,FieldCheckedBatch>,WorkflowError> {
        let RunRequest::Modeling(requests)=&self.request else {return Err(contract("authored solve request mismatch"));};
        let registry=&self.runtime.registry;
        let bytes=requests.iter().try_fold(0usize,|total,request|{
            let declaration=request.model.case.compiled().plan.structure();
            let cells=declaration.variables().len().checked_add(declaration.parameters().len())?.checked_add(declaration.rows().len())?;
            total.checked_add(cells.checked_mul(2048)?)?.checked_add(request.profile.controls.report_allowance().ok()?.checked_mul(8)?)
        }).ok_or_else(||contract("authored sequence export extent"))?;
        let _scratch=self.runtime.shared.math().reserve("modeling:solve-export",bytes)?;
        let mut run_rows=runs::Builder::with_registry(registry,requests.len()).map_err(relation)?;
        for row in &self.completion().map_err(|e|contract(e.to_string()))?.solves {run_rows.push(row.clone()).map_err(relation)?;}
        let pool=self.runtime.shared.pool();
        let cancel=pse_columnar::CancellationToken::new();
        let mut collection=pse_relations::columnar::Collection::new(registry,&pool,&cancel);
        collection.ensure::<modeling_checks::Row>().map_err(relation)?;
        collection.ensure::<modeling_reports::Row>().map_err(relation)?;
        collection.ensure::<modeling_findings::Row>().map_err(relation)?;
        let mut variable_rows=variables::Builder::with_registry(registry,0).map_err(relation)?;
        let mut constraint_rows=constraints::Builder::with_registry(registry,0).map_err(relation)?;
        let mut metric_rows=metrics::Builder::with_registry(registry,0).map_err(relation)?;
        let mut pool_rows=pool::Builder::with_registry(registry,0).map_err(relation)?;
        for (ordinal,request) in requests.iter().enumerate() {
        let declaration=request.model.case.compiled().plan.structure();
        let step=ordinal as i64;
        let result=match &self.report {Ok(RunReport::Modeling(r))=>r.get(ordinal),_=>None};
        let outcome=result.map(|r|&r.outcome);
        let native=match outcome {Some(Outcome::Native(r))=>Some(r.as_ref()),_=>None};
        let constant=match outcome {Some(Outcome::Constant(r))=>Some(r.as_ref()),_=>None};
        let observation=native.and_then(|r|r.observation.as_ref()).or_else(||constant.map(|r|&r.observation));
        let candidate=native.and_then(|r|r.candidate.as_ref());

            let coordinates: BTreeMap<_, _> = native
                .map(|r| {
                    r.variables
                        .iter()
                        .enumerate()
                        .map(|(i, id)| (*id, i))
                        .collect()
                })
                .unwrap_or_default();
            for v in declaration.variables() {
                let p = &v.port;
                let ix = coordinates.get(&p.id).copied();
                let value = if v.fixed {
                    request.model.values.scalars.get(&p.id).copied()
                } else {
                    ix.and_then(|i| candidate.and_then(|c| c.primal.get(i).copied()))
                };
                let tolerance = if v.fixed {
                    None
                } else {
                    let i = request.model.case
                        .compiled()
                        .plan
                        .columns()
                        .iter()
                        .position(|id| *id == p.id);
                    i.and_then(|i| request.solve.tolerances().variables.get(i).copied())
                };
                let dual_status = match observation {
                    Some(o) if o.dual_error.is_none() => {
                        DualQualification::EvaluatedKktNotSensitivityCertified
                    }
                    Some(o) if o.dual_error.is_some() => DualQualification::UnavailableOrInvalid,
                    _ => DualQualification::Unavailable,
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
                        value: request.model.values.scalars.get(&p.id).copied(),
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
            let rows = request.model.case.compiled().plan.structure().rows();
            for (i, r) in rows.iter().enumerate() {
                let unit = request.source.physical
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
                        tolerance: request.solve.tolerances().rows.get(i).copied(),
                        dual: candidate
                            .and_then(|c| c.row_dual.as_ref().and_then(|v| v.get(i).copied())),
                        dual_qualification: observation.map_or(
                            DualQualification::Unavailable,
                            |o| {
                                if o.dual_error.is_none() {
                                    DualQualification::EvaluatedKktNotSensitivityCertified
                                } else {
                                    DualQualification::UnavailableOrInvalid
                                }
                            },
                        ),
                    })
                    .map_err(relation)?;
            }

        if let Some(native)=native {
            let stored=self.stored_events(step)?;
            let events=stored.as_deref().map_or(super::results::StepEvents::Retained,super::results::StepEvents::Stored);
            super::results::push_native_metrics(&mut metric_rows,self.run_id,step,native,events)?;
        }
        // Ranked pooled solutions over the report's free variables (ADR-0105 §8).
        for solution in native.and_then(|r|r.global.as_ref()).map_or(&[][..],|g|g.pool.as_slice()) {
            let rank=i64::try_from(solution.rank).map_err(|_|contract("solution pool rank"))?;
            for (symbol_id,value) in native.map_or(&[][..],|r|r.variables.as_slice()).iter().zip(&solution.primal) {
                pool_rows.push(pool::Row{run_id:self.run_id,step,rank,symbol_id:*symbol_id,value:*value,objective:solution.objective,feasible:solution.feasible}).map_err(relation)?;
            }
        }
        if let Some(result)=result {
            for row in &result.checks {collection.push(row.clone()).map_err(relation)?;}
            for row in &result.reports {collection.push(row.clone()).map_err(relation)?;}
        }
        }
        for (ordinal,error) in self.capture_diagnostics().iter().enumerate() {
            collection.push(super::modeling::analysis_tables::finding_row(self.run_id,ordinal as i64,error)).map_err(relation)?;
        }
        let sources=requests.iter().map(|p|p.source.source_tables()).collect::<Result<Vec<_>,_>>()?;
        let mut batches=sources.first().cloned().unwrap_or_default();
        macro_rules! merge_source {
            ($module:ident,$key:expr) => {{
                use pse_relations::generated::authored::$module as wire;
                let mut rows=BTreeMap::new();
                for batch in sources.iter().filter_map(|s|s.get(&wire::RELATION_ID)) {
                    for row in wire::Row::rows(batch).map_err(relation)? {
                        if let Some(old)=rows.insert(($key)(&row),row.clone()) && old!=row {return Err(contract("conflicting source context in authored sequence"));}
                    }
                }
                let mut builder=wire::Builder::with_registry(registry,rows.len()).map_err(relation)?;
                for row in rows.into_values() {builder.push(row).map_err(relation)?;}
                batches.insert(wire::RELATION_ID,builder.finish().map_err(relation)?);
            }};
        }
        merge_source!(modeling_declarations,|r:&pse_relations::generated::authored::modeling_declarations::Row|r.declaration_id);
        merge_source!(documents,|r:&pse_relations::generated::authored::documents::Row|r.document_id);
        merge_source!(packages,|r:&pse_relations::generated::authored::packages::Row|r.package_id);
        merge_source!(package_quantity_aliases,|r:&pse_relations::generated::authored::package_quantity_aliases::Row|(r.package_id,r.name.clone()));
        batches.extend(collection.finish().map_err(relation)?.into_values().map(|b|(b.relation_id(),b)));
        batches.extend([
            (runs::RELATION_ID,run_rows.finish().map_err(relation)?),
            (variables::RELATION_ID,variable_rows.finish().map_err(relation)?),
            (constraints::RELATION_ID,constraint_rows.finish().map_err(relation)?),
            (metrics::RELATION_ID,metric_rows.finish().map_err(relation)?),
            (pool::RELATION_ID,pool_rows.finish().map_err(relation)?),
        ]);
        self.retain_sources(&mut batches)?;
        Ok(batches)
    }
}
