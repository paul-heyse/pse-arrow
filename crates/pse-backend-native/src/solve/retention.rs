// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Completed envelope capacity; opaque analysis and native state remain separately bounded.
use super::*;
use std::mem::{align_of, size_of};

struct Extent(usize);
impl Extent {
    fn add(&mut self, bytes: usize) -> Option<()> {
        self.0 = self.0.checked_add(bytes)?;
        Some(())
    }
    fn vector<T>(&mut self, values: &Vec<T>) -> Option<()> {
        self.add(values.capacity().checked_mul(size_of::<T>())?)
    }
    fn optional_vector<T>(&mut self, values: &Option<Vec<T>>) -> Option<()> {
        if let Some(values) = values {
            self.vector(values)?;
        }
        Some(())
    }
    fn text(&mut self, value: &Option<String>) -> Option<()> {
        if let Some(value) = value {
            self.add(value.capacity())?;
        }
        Some(())
    }
    fn tree<K, V>(&mut self, length: usize) -> Option<()> {
        // The pinned Rust alloc BTree node has 11 key/value slots, a parent pointer,
        // two u16 fields and (for an internal node) 12 child pointers. Charge one
        // complete internal node per entry, plus a possibly allocated empty root.
        // Seven maximum-alignment paddings bound field/final layout padding without
        // depending on private field offsets. Every occupied node owns an entry.
        let alignment = align_of::<K>()
            .max(align_of::<V>())
            .max(align_of::<usize>());
        let node = size_of::<K>()
            .checked_add(size_of::<V>())?
            .checked_mul(11)?
            .checked_add(size_of::<usize>().checked_mul(13)?)?
            .checked_add(size_of::<u16>().checked_mul(2)?)?
            .checked_add(alignment.checked_mul(7)?)?;
        self.add(length.checked_add(1)?.checked_mul(node)?)
    }
    fn metric(&mut self, value: &Metric) -> Option<()> {
        match value {
            Metric::Text(value) => self.add(value.capacity())?,
            Metric::Integer(_) | Metric::Real(_) | Metric::Bool(_) | Metric::Unavailable(_) => {}
        }
        Some(())
    }
    fn metrics(&mut self, values: &BTreeMap<String, Metric>) -> Option<()> {
        self.tree::<String, Metric>(values.len())?;
        for (key, value) in values {
            self.add(key.capacity())?;
            self.metric(value)?;
        }
        Some(())
    }
    fn options(&mut self, values: &Options) -> Option<()> {
        self.tree::<String, OptionValue>(values.len())?;
        for (key, value) in values {
            self.add(key.capacity())?;
            match value {
                OptionValue::Text(value) => self.add(value.capacity())?,
                OptionValue::Bool(_) | OptionValue::Integer(_) | OptionValue::Real(_) => {}
            }
        }
        Some(())
    }
    fn strings(&mut self, values: &BTreeMap<String, String>) -> Option<()> {
        self.tree::<String, String>(values.len())?;
        for (key, value) in values {
            self.add(key.capacity())?;
            self.add(value.capacity())?;
        }
        Some(())
    }
    fn quality(&mut self, value: &crate::quality::Quality) -> Option<()> {
        let crate::quality::Quality {
            rows,
            bounds,
            integrality,
            normalized_max: _,
        } = value;
        self.vector(rows)?;
        self.vector(bounds)?;
        self.vector(integrality)
    }
    fn observation(&mut self, value: &crate::quality::Observation) -> Option<()> {
        let crate::quality::Observation {
            sources,
            objective: _,
            values,
            bounds,
            equality_residuals,
            lower_violations,
            upper_violations,
            stationarity,
            complementarity,
            dual_error,
        } = value;
        for source in sources {
            let pse_math::assembly::OutputValue {
                instance: _,
                output: _,
                value: _,
            } = source;
        }
        self.vector(sources)?;
        self.vector(values)?;
        self.vector(bounds)?;
        self.vector(equality_residuals)?;
        self.vector(lower_violations)?;
        self.vector(upper_violations)?;
        self.optional_vector(stationarity)?;
        self.optional_vector(complementarity)?;
        self.text(dual_error)
    }
    fn warm(&mut self, value: &WarmStart) -> Option<()> {
        let WarmStart {
            origin,
            compatibility,
            payload,
        } = value;
        if let Some(SeedOrigin { run: _, attempt: _ }) = origin {}
        let Compatibility {
            layout: _,
            profile: _,
            data: _,
            backend: _,
        } = compatibility;
        match payload {
            WarmPayload::Root(value) => self.vector(value)?,
            WarmPayload::Nlp {
                primal,
                bounds,
                rows,
                barrier: _,
                working,
            } => {
                self.vector(primal)?;
                self.optional_vector(rows)?;
                if let Some((lower, upper)) = bounds {
                    self.vector(lower)?;
                    self.vector(upper)?;
                }
                if let Some(working) = working {
                    let WorkingSet {
                        transformation: _,
                        #[cfg(feature = "pounce")]
                            active: _,
                    } = working;
                    #[cfg(feature = "pounce")]
                    {
                        let pounce_rs::sqp::WorkingSet {
                            bounds,
                            constraints,
                        } = &working.active;
                        self.vector(bounds)?;
                        self.vector(constraints)?;
                    }
                }
            }
            WarmPayload::Highs {
                primal,
                dual,
                basis,
            } => {
                self.optional_vector(primal)?;
                if let Some((columns, rows)) = dual {
                    self.vector(columns)?;
                    self.vector(rows)?;
                }
                if let Some(Basis { columns, rows }) = basis {
                    self.vector(columns)?;
                    self.vector(rows)?;
                }
            }
        }
        Some(())
    }
    fn presolve_options(&self, value: &pounce_presolve::PresolveOptions) {
        let pounce_presolve::PresolveOptions {
            enabled: _,
            certify_tol: _,
            bound_tightening: _,
            redundant_constraint_removal: _,
            linear_eq_reduction: _,
            licq_check: _,
            print_level: _,
            max_passes: _,
            licq_action: _,
            warm_z_bounds: _,
            bound_mult_init_val: _,
            auxiliary: _,
            auxiliary_tol: _,
            auxiliary_max_block_dim: _,
            auxiliary_wall_time_fraction: _,
            auxiliary_coupling: _,
            auxiliary_diagnostics: _,
            fbbt: _,
            fbbt_tol: _,
            fbbt_max_iter: _,
            fbbt_max_constraints: _,
        } = value;
    }
}

impl SolveReport {
    /// Conservative completed reporting capacity, with actual vector/string capacities.
    /// The caller keeps its source-dimension allowance in addition to this envelope bound.
    /// Unknown bulky extensions retain the original complete reporting allowance instead;
    /// this operation never sizes or discharges opaque native/session/analysis owners.
    pub fn completed_report_allowance(&self) -> Result<Option<usize>, ProblemError> {
        if self.global.is_some()
            || self.evidence.local.is_some()
            || self.evidence.sensitivity.is_some()
            || self.evidence.root_response.is_some()
            || self.evidence.root_predictor.is_some()
            || self.evidence.inverse_reduced_hessian.is_some()
            || self.evidence.contradiction.is_some()
            || self
                .preprocessing
                .as_ref()
                .is_some_and(|value| value.proof.is_some())
        {
            return Ok(None);
        }
        #[cfg(feature = "highs")]
        if self.highs_diagnostics.is_some() {
            return Ok(None);
        }
        self.visible_report_allowance()
            .map(Some)
            .ok_or_else(|| ProblemError::memory("completed reporting capacity overflow"))
    }
    fn visible_report_allowance(&self) -> Option<usize> {
        // Exhaustive inventories make a new envelope or evidence field require an
        // accounting decision rather than silently omitting an owned payload.
        let Self {
            callback_failure: _,
            validation_failure: _,
            evidence: _,
            failure_owner: _,
            observation: _,
            preprocessing: _,
            owner: _,
            #[cfg(feature = "highs")]
                highs_diagnostics: _,
            #[cfg(feature = "pounce")]
                pounce_statistics: _,
            certificate: _,
            global: _,
            backend: _,
            variables: _,
            rows: _,
            termination: _,
            candidate: _,
            quality: _,
            metrics: _,
            options: _,
            native_defaults: _,
            provenance: _,
            events: _,
            dropped_events: _,
            warm_start: _,
            start_receipt: _,
            qualification: _,
            least_infeasible: _,
        } = self;
        let Evidence {
            work: _,
            abandoned: _,
            callback: _,
            start_submitted: _,
            reused_native_state: _,
            restart: _,
            working_set_submitted: _,
            kkt: _,
            coefficient: _,
            original_bound: _,
            conic: _,
            local: _,
            output_accuracy: _,
            sensitivity: _,
            root_response: _,
            root_predictor: _,
            inverse_reduced_hessian: _,
            global: _,
            contradiction: _,
        } = &self.evidence;
        let mut extent = Extent(size_of::<Self>());
        extent.add(self.failure_bytes())?;
        extent.vector(&self.variables)?;
        extent.vector(&self.rows)?;
        if let Some(accuracy) = &self.evidence.output_accuracy {
            extent.vector(&accuracy.goals)?;
            for goal in &accuracy.goals {
                extent.add(goal.declaration.provenance.capacity())?;
            }
            extent.vector(&accuracy.outputs)?;
            for evidence in accuracy
                .outputs
                .iter()
                .filter_map(|output| output.as_ref().ok())
            {
                extent.add(evidence.limitation.capacity())?;
            }
            extent.optional_vector(&accuracy.correction)?;
        }
        let NativeTermination {
            code: _,
            name,
            message,
            category: _,
            assurance: _,
        } = &self.termination;
        extent.add(name.capacity())?;
        extent.text(message)?;
        if let Some(candidate) = &self.candidate {
            let Candidate {
                kind: _,
                primal: _,
                objective: _,
                row_dual: _,
                bound_dual: _,
                reduced_costs: _,
                slacks: _,
                commitment: _,
            } = candidate;
            extent.vector(&candidate.primal)?;
            extent.optional_vector(&candidate.row_dual)?;
            extent.optional_vector(&candidate.reduced_costs)?;
            extent.optional_vector(&candidate.slacks)?;
            if let Some((lower, upper)) = &candidate.bound_dual {
                extent.vector(lower)?;
                extent.vector(upper)?;
            }
            if let Some(crate::transform::Commitment { columns }) = &candidate.commitment {
                extent.vector(columns)?;
            }
        }
        if let Some(value) = &self.observation {
            extent.observation(value)?;
        }
        if let Some(value) = &self.quality {
            extent.quality(value)?;
        }
        extent.metrics(&self.metrics)?;
        extent.options(&self.options)?;
        extent.options(&self.native_defaults)?;
        extent.strings(&self.provenance)?;
        extent.vector(&self.events)?;
        for event in &self.events {
            let Event {
                phase: _,
                elapsed: _,
                values: _,
                incumbent: _,
            } = event;
            extent.add(event.phase.capacity())?;
            extent.metrics(&event.values)?;
            if let Some(IncumbentEvent {
                objective: _,
                dual_bound: _,
                gap: _,
                nodes: _,
                seconds: _,
                primal,
            }) = &event.incumbent
            {
                extent.optional_vector(primal)?;
            }
        }
        if let Some(value) = &self.warm_start {
            extent.warm(value)?;
        }
        if let Some(value) = &self.start_receipt {
            let StartReceipt {
                previous_attempt: _,
                seed: _,
                sparse_seed: _,
                transformations: _,
                submitted: _,
            } = value;
            if let Some(seed) = &value.seed {
                extent.warm(seed)?;
            }
            if let Some(seed) = &value.sparse_seed {
                extent.tree::<SemanticId, f64>(seed.len())?;
            }
            extent.vector(&value.transformations)?;
            for transformation in &value.transformations {
                match transformation {
                    SeedTransformation::Presolve {
                        transformation: _,
                        passes,
                    } => extent.vector(passes)?,
                    SeedTransformation::Normalization(_)
                    | SeedTransformation::WorkingSet(_)
                    | SeedTransformation::InteriorRestart(_) => {}
                }
            }
        }
        if let Some(value) = &self.preprocessing {
            let crate::presolve::Report {
                requested: _,
                effective: _,
                passes: _,
                facts: _,
                transformation: _,
                dimensions: _,
                diagnostics: _,
                columns: _,
                rows: _,
                proof: _,
                working_set: _,
                resolution: _,
            } = value;
            extent.presolve_options(&value.effective);
            match &value.requested {
                crate::presolve::Policy::Explicit { options, required } => {
                    extent.presolve_options(options);
                    extent.tree::<crate::presolve::Pass, ()>(required.len())?;
                }
                crate::presolve::Policy::Auto | crate::presolve::Policy::Off => {}
            }
            extent
                .tree::<crate::presolve::Pass, crate::presolve::PassReport>(value.passes.len())?;
            for crate::presolve::PassReport {
                requested: _,
                eligible: _,
                applied: _,
                reason,
            } in value.passes.values()
            {
                extent.text(reason)?;
            }
            extent.strings(&value.diagnostics)?;
            extent.add(
                value
                    .columns
                    .capacity()
                    .checked_mul(size_of::<pse_math::index::OriginalCol>())?,
            )?;
            extent.add(
                value
                    .rows
                    .capacity()
                    .checked_mul(size_of::<pse_math::index::OriginalRow>())?,
            )?;
        }
        #[cfg(feature = "pounce")]
        if let Some(value) = &self.pounce_statistics {
            let pounce_rs::SolveStatistics {
                iteration_count: _,
                total_cpu_time_secs: _,
                total_sys_time_secs: _,
                total_wallclock_time_secs: _,
                num_obj_evals: _,
                num_constr_evals: _,
                num_obj_grad_evals: _,
                num_constr_jac_evals: _,
                num_hess_evals: _,
                final_objective: _,
                final_scaled_objective: _,
                final_dual_inf: _,
                final_constr_viol: _,
                final_compl: _,
                final_kkt_error: _,
                final_declared_constr_viol: _,
                final_declared_box_viol: _,
                final_unscaled_dual_inf: _,
                final_unscaled_constr_viol: _,
                final_unscaled_compl: _,
                final_unscaled_kkt_error: _,
                final_kkt_error_above_noise: _,
                final_mu: _,
                partitioned_elements: _,
                partitioned_dense_elements: _,
                partitioned_diagonal_elements: _,
                partitioned_stored_reals: _,
                fd_hessian_pattern_used: _,
                fd_hessian_nnz: _,
                fd_hessian_n: _,
                fd_hessian_groups: _,
                fd_hessian_rho_max: _,
                fd_hessian_coloring_fell_back: _,
                fd_hessian_objective_clique_widened: _,
                restoration_calls: _,
                restoration_inner_iters: _,
                restoration_outer_iters: _,
                restoration_wall_secs: _,
                quality_escalations: _,
                dual_divergence_signature: _,
                dual_divergence_retry_promoted: _,
                sqp_qp_solves: _,
                sqp_qp_working_set_changes: _,
                iterations,
            } = value.as_ref();
            for pounce_rs::IterRecord {
                iter: _,
                objective: _,
                inf_pr: _,
                inf_du: _,
                mu: _,
                d_norm: _,
                regularization: _,
                alpha_dual: _,
                alpha_primal: _,
                alpha_primal_char: _,
                ls_trials: _,
            } in iterations
            {}
            extent.add(size_of::<pounce_rs::SolveStatistics>())?;
            extent.vector(iterations)?;
        }
        if let Some(InfeasibilityCertificate {
            kind: _,
            accuracy: _,
            ray,
            verification: _,
        }) = &self.certificate
        {
            extent.vector(ray)?;
        }
        if let Some(LeastInfeasible { violated }) = &self.least_infeasible {
            extent.vector(violated)?;
        }
        Some(extent.0)
    }
}

impl crate::quality::Observation {
    /// Visible retained observation capacity; inline storage is counted by its envelope.
    pub fn completed_report_allowance(&self) -> Option<usize> {
        let mut extent = Extent(0);
        extent.observation(self)?;
        Some(extent.0)
    }
}
impl crate::quality::Quality {
    /// Visible retained qualification capacity; inline storage is counted by its envelope.
    pub fn completed_report_allowance(&self) -> Option<usize> {
        let mut extent = Extent(0);
        extent.quality(self)?;
        Some(extent.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn report() -> SolveReport {
        let contract = crate::OracleContract {
            identity: ContentHash::from_bytes([1; 32]),
            variables: Vec::new(),
            rows: Vec::new(),
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let execution = Execution::new(Arc::new(AtomicBool::new(false)), &Controls::default());
        SolveReport::new(
            Backend::Kinsol,
            &contract,
            NativeTermination {
                code: 0,
                name: "success".into(),
                message: None,
                category: Termination::Success,
                assurance: Assurance::None,
            },
            &execution,
        )
    }
    #[test]
    fn completed_reporting_counts_spare_capacities_and_portable_warm_seed() {
        let mut report = report();
        let before = report.completed_report_allowance().unwrap().unwrap();
        let text = String::with_capacity(32768);
        let mut primal = Vec::with_capacity(8192);
        primal.push(1.);
        let storage = text.capacity() + primal.capacity() * size_of::<f64>();
        report
            .metrics
            .insert("allocated-empty-text".into(), Metric::Text(text));
        report.warm_start = Some(WarmStart {
            origin: None,
            compatibility: Compatibility {
                layout: ContentHash::from_bytes([2; 32]),
                profile: ContentHash::from_bytes([3; 32]),
                data: ContentHash::from_bytes([4; 32]),
                backend: Backend::Kinsol,
            },
            payload: WarmPayload::Root(primal),
        });
        assert!(report.completed_report_allowance().unwrap().unwrap() >= before + storage);
    }
    #[test]
    fn completed_reporting_preserves_typed_failure_and_unknown_extension() {
        let mut report = report();
        let cause = Arc::new(ProblemError::Limit {
            kind: crate::LimitKind::Work,
            detail: "actual evaluation allowance".into(),
        });
        report.callback_failure = Some(cause.clone());
        assert!(report.completed_report_allowance().unwrap().unwrap() >= cause.retained_bytes());
        assert!(Arc::ptr_eq(
            &report.shared_callback_failure().unwrap(),
            &cause
        ));
        report.evidence.root_response = Some(Err(crate::square_response::Withheld::Memory));
        assert_eq!(report.completed_report_allowance().unwrap(), None);
        assert!(Arc::ptr_eq(
            &report.shared_callback_failure().unwrap(),
            &cause
        ));
        let mut overflow = Extent(usize::MAX);
        assert_eq!(overflow.tree::<String, Metric>(1), None);
        assert_eq!(overflow.0, usize::MAX);
    }
}
