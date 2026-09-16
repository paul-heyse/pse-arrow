// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Manifest-selected immutable bindings. Encodings select bytes, never semantic truth.

use std::collections::BTreeMap;

use pse_ids::{CancellationToken, EncodingChecksum, SemanticId, SnapshotKind};
use serde::{Deserialize, Serialize};

use super::super::control::{self, OwnedControl};
use super::super::{invocation, membership::AdmissionContext, verify::admission};
use crate::{Catalog, CatalogError, Manifest, session::SessionSemantics, snapshot::ManifestRef};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PolicyWire {
    pub(super) policy_id: SemanticId,
    pub(super) snapshot: ManifestRef,
    pub(super) port: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct InvocationWire {
    #[serde(deserialize_with = "required_document_source")]
    pub(super) document_source: Option<ManifestRef>,
    #[serde(deserialize_with = "crate::store::verify::unique_map")]
    pub(super) policies: BTreeMap<String, PolicyWire>,
    pub(super) engine: Option<SessionSemantics>,
}

fn required_document_source<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ManifestRef>, D::Error> {
    Option::deserialize(deserializer)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::store) struct ContextWire {
    version: u32,
    pub(super) producer: Option<SemanticId>,
    #[serde(deserialize_with = "crate::store::verify::unique_map")]
    pub(super) parents: BTreeMap<String, ManifestRef>,
    pub(super) invocation: Option<InvocationWire>,
}

impl ContextWire {
    pub(in crate::store) fn matches(&self, context: &AdmissionContext) -> bool {
        self.producer == context.stage_pass
            && self.parents.len() == context.parents.len()
            && self.parents.iter().all(|(role, reference)| {
                context
                    .parents
                    .get(role)
                    .is_some_and(|parent| parent.manifest_ref() == *reference)
            })
            && match (&self.invocation, &context.invocation) {
                (None, None) => true,
                (Some(wire), Some(actual)) => {
                    wire.document_source
                        == actual
                            .document_source
                            .as_ref()
                            .map(|source| source.manifest_ref())
                        && wire.engine == actual.engine
                        && wire.policies.len() == actual.policies.len()
                        && wire.policies.iter().all(|(role, policy)| {
                            actual.policies.get(role).is_some_and(|actual| {
                                actual.policy_id == policy.policy_id
                                    && actual.port == policy.port
                                    && actual.snapshot.manifest_ref() == policy.snapshot
                            })
                        })
                }
                _ => false,
            }
    }
    fn extent(&self) -> Result<usize, CatalogError> {
        let slots = super::super::stage_owned::map_slots(&self.parents)?;
        let bytes = self
            .parents
            .keys()
            .try_fold(control::add(512, slots)?, |bytes, role| {
                control::add(bytes, role.capacity())
            })?;
        self.invocation.as_ref().map_or(Ok(bytes), |invocation| {
            let bytes = control::add(
                bytes,
                super::super::stage_owned::map_slots(&invocation.policies)?,
            )?;
            let bytes = invocation
                .policies
                .iter()
                .try_fold(bytes, |bytes, (role, policy)| {
                    control::add(
                        bytes,
                        control::add(role.capacity(), policy.port.capacity())?,
                    )
                })?;
            control::add(
                bytes,
                invocation::engine_extent(invocation.engine.as_ref())?,
            )
        })
    }

    fn validate(&self, manifest: &Manifest) -> Result<(), CatalogError> {
        if self.version != 3 {
            return Err(CatalogError::UnknownVersion {
                field: "admission binding".to_owned(),
                value: self.version.to_string(),
            });
        }
        if self.parents.len() != manifest.semantic_parents.len()
            || manifest.semantic_parents.iter().any(|parent| {
                self.parents
                    .get(&parent.role)
                    .is_none_or(|reference| reference.snapshot_id != parent.snapshot_id)
            })
        {
            return Err(admission(
                "admission binding",
                "wrong exact parent role/identity set",
            ));
        }
        let producer_matches = match (manifest.snapshot_kind, self.producer) {
            (SnapshotKind::Stage, Some(producer)) => manifest
                .compiler
                .passes
                .iter()
                .any(|pass| pass.pass_id == producer),
            (SnapshotKind::Stage, None) | (_, Some(_)) => false,
            (_, None) => self.invocation.is_none(),
        };
        if !producer_matches {
            return Err(admission(
                "admission binding",
                "missing or inconsistent producing pass",
            ));
        }
        Ok(())
    }

    fn forecast(context: &AdmissionContext) -> Result<usize, CatalogError> {
        let slots =
            super::super::stage_owned::map_extent::<String, ManifestRef>(context.parents.len())?;
        let bytes = context
            .parents
            .keys()
            .try_fold(control::add(512, slots)?, |bytes, role| {
                control::add(bytes, role.capacity())
            })?;
        context.invocation.as_ref().map_or(Ok(bytes), |invocation| {
            let slots = super::super::stage_owned::map_extent::<String, PolicyWire>(
                invocation.policies.len(),
            )?;
            let bytes = invocation.policies.iter().try_fold(
                control::add(bytes, slots)?,
                |bytes, (role, policy)| {
                    control::add(
                        bytes,
                        control::add(role.capacity(), policy.port.capacity())?,
                    )
                },
            )?;
            control::add(
                bytes,
                invocation::engine_extent(invocation.engine.as_ref())?,
            )
        })
    }

    fn from_context(context: &AdmissionContext) -> Self {
        Self {
            version: 3,
            producer: context.stage_pass,
            parents: context
                .parents
                .iter()
                .map(|(role, parent)| (role.clone(), parent.manifest_ref()))
                .collect(),
            invocation: context
                .invocation
                .as_ref()
                .map(|invocation| InvocationWire {
                    document_source: invocation
                        .document_source
                        .as_ref()
                        .map(|source| source.manifest_ref()),
                    policies: invocation
                        .policies
                        .iter()
                        .map(|(role, policy)| {
                            (
                                role.clone(),
                                PolicyWire {
                                    policy_id: policy.policy_id,
                                    snapshot: policy.snapshot.manifest_ref(),
                                    port: policy.port.clone(),
                                },
                            )
                        })
                        .collect(),
                    engine: invocation.engine.clone(),
                }),
        }
    }
}

impl Catalog {
    pub(in crate::store) async fn publish_admission_binding(
        &self,
        context: &AdmissionContext,
        cancel: &CancellationToken,
    ) -> Result<Option<EncodingChecksum>, CatalogError> {
        if context.parents.is_empty()
            && context.stage_pass.is_none()
            && context.invocation.is_none()
        {
            return Ok(None);
        }
        let mut reservation = self.reserver.open("store:admission-binding-write");
        reservation.try_grow(ContextWire::forecast(context)?)?;
        let binding = ContextWire::from_context(context);
        let bytes = crate::store::encode::control(
            &binding,
            self.reserver.as_ref(),
            "store:admission-binding-encode",
            self.limits.max_control_bytes,
        )?;
        let checksum = pse_ids::encoding_checksum(&bytes);
        self.ensure_create(
            &crate::store::layout::context_path(&checksum),
            bytes,
            cancel,
        )
        .await?;
        Ok(Some(checksum))
    }

    pub(in crate::store) async fn read_admission_binding(
        &self,
        manifest: &Manifest,
        cancel: &CancellationToken,
    ) -> Result<OwnedControl<ContextWire>, CatalogError> {
        let Some(binding) = &manifest.admission_binding else {
            if manifest.snapshot_kind != SnapshotKind::Model
                || !manifest.semantic_parents.is_empty()
            {
                return Err(admission(
                    "admission binding",
                    "unsupported context-dependent manifest without current admission binding",
                ));
            }
            let mut reservation = self.reserver.open("store:admission-binding-read");
            reservation.try_grow(512)?;
            return Ok(OwnedControl::new(
                ContextWire::from_context(&AdmissionContext::default()),
                pse_ids::ReservationLease::new(reservation),
            ));
        };
        let checksum = binding.encoding_checksum;
        let path = crate::store::layout::context_path(&checksum);
        let bytes = self
            .read_bytes(&path, self.limits.max_control_bytes, cancel)
            .await?;
        if pse_ids::encoding_checksum(&bytes) != checksum {
            return Err(admission(
                "admission binding",
                "selected encoding checksum differs",
            ));
        }
        let reservation = control::decode_reservation(
            &bytes,
            self.reserver.as_ref(),
            "store:admission-binding-read",
        )?;
        let binding: ContextWire = serde_json::from_slice(&bytes)
            .map_err(|error| admission("admission binding", &error.to_string()))?;
        binding.validate(manifest)?;
        let lease = control::retain(reservation, binding.extent()?)?;
        Ok(OwnedControl::new(binding, lease))
    }
}

#[cfg(test)]
mod tests {
    use super::InvocationWire;

    #[test]
    fn document_source_selection_must_be_present_even_when_explicitly_empty() {
        let incomplete = serde_json::json!({"policies": {}, "engine": null});
        assert!(serde_json::from_value::<InvocationWire>(incomplete).is_err());
        let complete = serde_json::json!({"document_source": null, "policies": {}, "engine": null});
        assert!(serde_json::from_value::<InvocationWire>(complete).is_ok());
    }
}
