// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Hydration uses exact protected fit products, never a caller's run-id assertion.
use super::{CovarianceOrigin, FitCovarianceRecord, ParameterCovariance};
use crate::workflow::{Runtime, WorkflowError, contract, relation};
use pse_columnar::{AllocationLease, CancellationToken};
use pse_model::scalars::FiniteBound;
use pse_relations::{columnar::RelationRow, generated::{enums::DerivedQuantity, runtime::{
    parameter_covariances, local_validity, scientific_contexts,
}}};
use std::sync::Arc;

impl Runtime {
    async fn covariance_rows<T: RelationRow + Send + Sync>(
        &self, record: &FitCovarianceRecord, name: &str, maximum: usize,
        cancel: &CancellationToken,
    ) -> Result<(Vec<T>, Arc<Vec<Arc<AllocationLease>>>), WorkflowError> {
        let mut reader = self.results(&record.run, &record.attempt, name, 0,
            u64::try_from(maximum).map_err(|_| contract("covariance row extent"))?
                .checked_add(1).ok_or_else(|| contract("covariance row extent"))?,
            cancel.clone()).await?;
        let mut rows = Vec::new();
        let mut owners = Vec::new();
        while let Some(batch) = reader.next_relation_batch().await? {
            cancel.checkpoint().map_err(pse_relations::RelationError::from).map_err(relation)?;
            let bytes = batch.batch().get_array_memory_size();
            let reservation = super::super::result_export::working(&self.shared.pool(),
                "uncertainty:fit-hydration-copy", &[(bytes, 2), (batch.batch().num_rows(), 1024)])?;
            owners.push(AllocationLease::new(reservation));
            rows.extend(T::rows(&batch).map_err(relation)?);
            if rows.len() > maximum { return Err(contract("fit covariance source exceeds its declared row inventory")); }
        }
        Ok((rows, Arc::new(owners)))
    }

    /// Hydrate an exact qualified canonical fit covariance, including its original
    /// parameter order, physical coordinates, scales, interpretation and local validity.
    /// Missing or withheld inputs refuse; no caller-provided matrix is certified here.
    ///
    /// # Errors
    /// Missing, partial, incompatible or unqualified fit products, resource refusal,
    /// cancellation, or a protected canonical read failure.
    pub async fn fit_covariance(
        &self, record: FitCovarianceRecord, cancel: &CancellationToken,
    ) -> Result<ParameterCovariance, WorkflowError> {
        let (mut covariance, covariance_owner) = self.covariance_rows::<parameter_covariances::Row>(
            &record, "runtime.parameter_covariances", 1, cancel).await?;
        let (mut contexts, context_owner) = self.covariance_rows::<scientific_contexts::Row>(
            &record, "runtime.scientific_contexts", 1, cancel).await?;
        let (validities, validity_owner) = self.covariance_rows::<local_validity::Row>(
            &record, "runtime.local_validity", 64, cancel).await?;
        if covariance.len() != 1 || contexts.len() != 1 {
            return Err(contract("the exact fit covariance or its scientific context is unavailable"));
        }
        let covariance = covariance.remove(0);
        let context = contexts.remove(0);
        if context.run_id != covariance.run_id || context.step != 0 || context.fit_id.is_none() {
            return Err(contract("covariance source is not the selected fit's scientific context"));
        }
        let mut matches = validities.into_iter().filter(|validity| validity.run_id == covariance.run_id
            && validity.step == 0 && validity.quantity == DerivedQuantity::ParameterCovariance);
        let validity = matches.next().ok_or_else(|| contract("fit covariance local validity is unavailable"))?;
        if matches.next().is_some() || !validity.validity.certified {
            return Err(contract("fit covariance local validity is not uniquely qualified"));
        }
        let finite = |values: Vec<f64>| values.into_iter().map(FiniteBound::try_new)
            .collect::<Result<Vec<_>, _>>().map_err(|error| contract(error.to_string()));
        let mut input = ParameterCovariance {
            version: pse_model::document::Version,
            origin: CovarianceOrigin::QualifiedFit { run_id: covariance.run_id, record: Some(record) },
            parameters: covariance.parameters, parameter_quantities: covariance.parameter_quantities,
            parameter_units: covariance.parameter_units, values: finite(covariance.values)?,
            scales: finite(covariance.parameter_scales)?, verified: None,
        };
        input.admit().map_err(|error| contract(error.to_string()))?;
        let content = input.content().map_err(|error| contract(error.to_string()))?;
        input.verified = Some(Arc::new(crate::math::settings::VerifiedFitCovariance {
            content, source: context.source_identity, physical: context.physical_identity,
            validity: validity.validity,
            _owner: Arc::new((covariance_owner, context_owner, validity_owner)),
        }));
        Ok(input)
    }

    pub(in crate::workflow) async fn reconcile_covariance(
        &self, input: &mut ParameterCovariance, cancel: &CancellationToken,
    ) -> Result<(), WorkflowError> {
        input.admit().map_err(|error| contract(error.to_string()))?;
        if matches!(input.origin, CovarianceOrigin::External { .. }) { return Ok(()); }
        if input.verified_fit().is_ok() { return Ok(()); }
        let CovarianceOrigin::QualifiedFit { record: Some(record), .. } = &input.origin else {
            return Err(contract("a decoded in-process covariance needs an available exact canonical fit reference"));
        };
        let actual = self.fit_covariance(record.clone(), cancel).await?;
        if input.content().map_err(|error| contract(error.to_string()))?
            != actual.content().map_err(|error| contract(error.to_string()))?
        {
            return Err(contract("submitted fit covariance differs from the actual qualified fit content, order, units or scales"));
        }
        input.verified = actual.verified;
        Ok(())
    }
}
