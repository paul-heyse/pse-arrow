// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physical trajectories and exact execution provenance use generated owned relations.
use super::{RunReport, RunRequest, RunResult, WorkflowError, contract, relation};
use pse_ids::SemanticId;
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::runtime::computation_runs,
};
use std::collections::BTreeMap;
impl RunResult {
    pub(super) fn encode_simulation(&self) -> Result<BTreeMap<SemanticId,FieldCheckedBatch>,WorkflowError> {
        let RunRequest::Simulation(p)=&self.request else{return Err(contract("simulation request mismatch"));};
        let mut batches=match &self.report {Ok(RunReport::Simulation(r))=>r.tables()?,_=>BTreeMap::new()};
        let registry=&self.runtime.registry;
        let mut header=computation_runs::Builder::with_registry(registry,1).map_err(relation)?;
        header.push(self.completion().map_err(|e|contract(e.to_string()))?.computation.clone().ok_or_else(||contract("simulation completion absent"))?).map_err(relation)?;
        batches.insert(computation_runs::RELATION_ID,header.finish().map_err(relation)?);
        batches.extend(p.source.source_tables()?);
        self.retain_sources(&mut batches)?;
        Ok(batches)
    }
    pub(super) fn retain_sources(
        &self,
        batches: &mut BTreeMap<SemanticId, FieldCheckedBatch>,
    ) -> Result<(), WorkflowError> {
        self.numerical_tables(batches)?;
        use pse_relations::generated::runtime::run_lineage;
        let completion = self.completion().map_err(|e| contract(e.to_string()))?;
        let mut lineage =
            run_lineage::Builder::with_registry(&self.runtime.registry, completion.lineage.len())
                .map_err(relation)?;
        for row in &completion.lineage {
            lineage.push(row.clone()).map_err(relation)?;
        }
        batches.insert(
            run_lineage::RELATION_ID,
            lineage.finish().map_err(relation)?,
        );
        let cancel = pse_columnar::CancellationToken::new();
        for batch in batches.values_mut() {
            *batch = batch
                .retained(&self.runtime.shared.pool(), &cancel)
                .map_err(relation)?;
        }
        Ok(())
    }
}
