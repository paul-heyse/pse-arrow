// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Private coupling of source construction, exact P2 inputs and its completed obligations.
use pse_catalog::{Catalog, CatalogError, source_production::SourceProducer};
use pse_relations::columnar::FieldCheckedBatch;
use pse_schema::model::RelationKey;
use std::{any::Any, collections::BTreeMap, sync::Arc};

pub(crate) struct SourceArguments {
    pub(crate) candidate: pse_authoring::change_set::OwnedCandidateSnapshot,
    pub(crate) rows: BTreeMap<RelationKey, FieldCheckedBatch>,
    pub(crate) validation: Arc<pse_rules::invariants::InvariantReport>,
    pub(crate) registry: Arc<pse_schema::Registry>,
}
#[derive(Debug)]
pub(super) struct RegisteredSources;
impl SourceProducer for RegisteredSources {
    fn complete(
        &self,
        catalog: &Catalog,
        arguments: &(dyn Any + Send + Sync),
    ) -> Result<BTreeMap<RelationKey, FieldCheckedBatch>, CatalogError> {
        let args = arguments
            .downcast_ref::<SourceArguments>()
            .ok_or_else(|| refused("different source completion type"))?;
        if !Arc::ptr_eq(&args.registry, catalog.registry())
            || args.validation.error_count() != 0
            || args.candidate.changes.documents().is_none()
        {
            return Err(refused(
                "source construction or actual P2 completion is absent",
            ));
        }
        // The private caller retained the exact rows used by P2 and the candidate
        // constructor. No marker, digest or reconstructed validator grants this handoff.
        Ok(args.rows.clone())
    }
}
fn refused(reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: "source completion".to_owned(),
        reason: reason.to_owned(),
    }
}
