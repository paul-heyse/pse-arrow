// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared sparse fit assembly, numerical policy and ownership after source interpretation.
use super::*;

pub(super) struct PreparedExperiments {
    pub execution_identity: ContentHash,
    pub declaration: FitDeclaration,
    pub lineage: pse_model::lineage::Fitted,
    pub profile: FitProfile,
    pub order: DerivativeOrder,
    pub variables: Vec<Variable>,
    pub rows: Vec<SemanticId>,
    pub bounds: Vec<(f64, f64)>,
    pub initial: Vec<f64>,
    pub parameter_ports: Vec<Port>,
    pub parameter_columns: Vec<Option<OriginalCol>>,
    pub experiments: Vec<Experiment>,
    pub measurements: Vec<Measurement>,
    pub targets: Vec<TargetSpec>,
    pub declarations: Vec<SourcedRequirement>,
    pub bytes: usize,
    pub physical_cells: usize,
}
impl PreparedExperiments {
    pub(super) fn finish(
        self,
        runtime: super::super::Runtime,
        quantities: Arc<pse_quantity::QuantityRegistry>,
        source_identity: ContentHash,
        reservation: datafusion::execution::memory_pool::MemoryReservation,
    ) -> Result<FitProblem, WorkflowError> {
        let Self {
            execution_identity,
            declaration: d,
            lineage,
            profile,
            order,
            variables: vars,
            rows,
            bounds,
            initial,
            parameter_ports,
            parameter_columns,
            experiments,
            measurements,
            mut targets,
            declarations,
            mut bytes,
            physical_cells,
        } = self;
        let policy = &runtime.shared.budget().math;
        let q = &quantities;
        let experiment_metadata = experiments.iter().try_fold(0usize, |total, experiment| {
            let bytes = match experiment {
                Experiment::Steady(s) => s
                    .providers
                    .len()
                    .checked_mul(256)
                    .and_then(|n| {
                        n.checked_add(
                            s.values.scalars.len() * 128
                                + s.coordinates.capacity() * size_of::<(SemanticId, OriginalCol)>()
                                + s.variables.capacity() * size_of::<pse_math::binding::Variable>(),
                        )
                    })
                    .ok_or_else(|| contract("fit experiment metadata"))?,
                Experiment::Transient(s) => s
                    .program
                    .metadata_bytes()
                    .checked_add(s.bindings.capacity() * size_of::<Binding>())
                    .and_then(|n| n.checked_add(s.parameters.capacity() * size_of::<f64>()))
                    .and_then(|n| n.checked_add(s.output_ports.capacity() * size_of::<Port>()))
                    .ok_or_else(|| contract("fit transient metadata"))?,
            };
            total
                .checked_add(bytes)
                .ok_or_else(|| contract("fit experiment metadata"))
        })?;
        // Finite metadata allowance includes coordinate/policy projections and
        // opaque library metadata. It is accounting, not a private layout or RSS claim.
        let metadata_bytes = [
            d.parameters.len(),
            d.observations.len(),
            vars.len(),
            rows.len(),
            experiments.len(),
        ]
        .into_iter()
        .try_fold(0usize, |n, v| n.checked_add(v))
        .and_then(|n| n.checked_mul(1024))
        .and_then(|n| n.checked_add(policy.foreign_bytes))
        .and_then(|n| n.checked_add(experiment_metadata))
        .ok_or_else(|| contract("fit preparation metadata allowance"))?;
        // Each contribution may coexist with pair/refill maps, canonical CSC,
        // transpose and symbolic-product scratch. Bound construction before faer.
        let layout_limit = policy
            .workspace_bytes
            .checked_sub(metadata_bytes)
            .map(|n| n / 256)
            .filter(|n| *n > 0)
            .ok_or_else(|| contract("fit preparation memory allowance"))?
            .min(profile.max_cells);
        let layout = Arc::new(
            sparse::Layout::new(
                &experiments,
                &measurements,
                &parameter_columns,
                rows.len(),
                vars.len(),
                order,
                layout_limit,
            )
            .map_err(crate::math::MathRuntimeError::from)?,
        );
        let cells = layout
            .cells
            .checked_add(physical_cells)
            .ok_or_else(|| contract("fit sparse derivative/report extent"))?;
        bytes = bytes
            .checked_add(
                physical_cells
                    .checked_mul(512)
                    .ok_or_else(|| contract("fit physical report storage"))?,
            )
            .ok_or_else(|| contract("fit physical report storage"))?
            .checked_add(
                cells
                    .checked_mul(64)
                    .ok_or_else(|| contract("fit derivative storage"))?,
            )
            .ok_or_else(|| contract("fit storage"))?;
        let neutral = q
            .neutral_dimensionless()
            .ok_or_else(|| contract("fitting requires the bound neutral quantity"))?;
        targets.push(TargetSpec {
            id: SemanticId::NIL,
            kind: NumericalTarget::Objective,
            quantity: neutral,
            unit: q
                .quantity_type(neutral)
                .map_err(|e| contract(e.to_string()))?
                .canonical_unit,
            integer: false,
            declared_tolerance: None,
        });
        let numerics = Arc::new(
            pse_math::numerics::resolve(q, &targets, &declarations, &profile.solver.numerics)
                .map_err(math)?,
        );
        let columns: Vec<_> = vars.iter().map(|v| v.id).collect();
        let normalization = Normalization::from_policy(&numerics, &columns, &rows).map_err(math)?;
        let tolerances = native::quality::Tolerances::from_policy(&numerics, &columns, &rows)
            .map_err(crate::math::MathRuntimeError::from)?;
        let accuracy =
            native::solve::ResolvedAccuracy::resolve(&numerics.policy, &tolerances, &normalization)
                .map_err(crate::math::MathRuntimeError::from)?;
        if vars.is_empty()
            && matches!(&profile.solver.presolve, native::presolve::Policy::Explicit { required, .. } if !required.is_empty())
        {
            return Err(contract(
                "all-fixed fitting evaluates directly and cannot apply required native presolve passes",
            ));
        }
        let mut h = FramedHasher::new(pse_ids::Frame::FitSourceV1);
        h.hash(&source_identity);
        d.frame(&mut h);
        let source = h.finish_hash();
        let mut h = FramedHasher::new(pse_ids::Frame::FitProfileV3);
        h.hash(
            &crate::math::solves::profile_key(&profile.solver)
                .map_err(crate::math::MathRuntimeError::from)?,
        );
        h.u64(profile.rank_tolerance.to_bits())
            .u64(profile.max_cells as u64)
            .str(profile.derivatives.as_str());
        for (id, p) in &profile.simulations {
            h.id(&id.as_id())
                .hash(&crate::workflow::dynamics::profile_identity(p));
        }
        // Mode identity is the complete serde encoding, never a hand-written field list (F09).
        for (id, modes) in &profile.modes {
            let encoded = serde_json::to_string(modes)
                .map_err(|e| contract(format!("fit mode encoding: {e}")))?;
            h.id(&id.as_id()).str(&encoded);
        }
        let profile_key = h.finish_hash();
        let mut h = FramedHasher::new(pse_ids::Frame::FitPreparedV1);
        h.hash(&source)
            .hash(&profile_key)
            .hash(&numerics.key)
            .hash(&execution_identity);
        let contract = OracleContract {
            identity: source,
            variables: vars,
            rows,
            derivatives: order,
            smoothness: order,
        };
        let retained = metadata_bytes
            .checked_add(layout.retained_bytes())
            .filter(|n| *n <= reservation.size())
            .ok_or_else(|| super::contract("fit preparation retained allowance"))?;
        reservation.shrink(reservation.size() - retained);
        Ok(FitProblem {
            runtime,
            quantities,
            source_identity: source,
            declaration: d,
            lineage,
            profile,
            key: h.finish_hash(),
            profile_key,
            numerics,
            normalization,
            tolerances,
            accuracy,
            bytes,
            contract,
            layout,
            bounds,
            initial,
            parameter_ports,
            parameter_columns,
            experiments,
            measurements,
            _owner: pse_columnar::AllocationLease::new(reservation),
        })
    }
}
