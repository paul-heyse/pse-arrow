// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable fitting declarations belong to the authored modeling source.
use super::{ModelingPackage, WorkflowError, contract};
use pse_ids::SemanticId;
use pse_relations::{
    columnar::{FieldCheckedBatch, RelationRow},
    generated::authored::fit_cases,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// Exact fitting declaration from the schema registry.
pub type FitDeclaration = fit_cases::Row;

/// Registry-owned experiment and parameter selection intent; measurements belong to typed records.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FitDeclarations {
    /// Declared fits with their parameters, experiments and observation selections.
    pub fits: Vec<fit_cases::Row>,
    #[serde(skip)]
    _owners: Vec<Arc<pse_columnar::AllocationLease>>,
}
impl FitDeclarations {
    pub(crate) fn from_owned_rows(
        fits: Vec<fit_cases::Row>,
        owners: Vec<Arc<pse_columnar::AllocationLease>>,
    ) -> Self {
        Self {
            fits,
            _owners: owners,
        }
    }
    pub(crate) fn from_batches(
        batches: &BTreeMap<SemanticId, FieldCheckedBatch>,
    ) -> Result<Self, WorkflowError> {
        Ok(Self {
            fits: batches
                .get(&fit_cases::RELATION_ID)
                .map(fit_cases::Row::rows)
                .transpose()
                .map_err(super::super::relation)?
                .unwrap_or_default(),
            _owners: Vec::new(),
        })
    }
}
impl ModelingPackage {
    /// Attach generated fitting declarations to this immutable source revision.
    pub async fn with_fit_declarations(
        mut self,
        services: &crate::workflow::SourceServices,
        data: FitDeclarations,
    ) -> Result<Self, WorkflowError> {
        self.check_service_bindings(services.preparation())?;
        let unique = |ids: Vec<SemanticId>| ids.iter().collect::<BTreeSet<_>>().len() == ids.len();
        if !unique(data.fits.iter().map(|r| r.fit_id.as_id()).collect()) {
            return Err(contract("duplicate fit declaration identity"));
        }
        self.revision = self.replace_fits(services, &data.fits).await?;
        Ok(self)
    }
}
