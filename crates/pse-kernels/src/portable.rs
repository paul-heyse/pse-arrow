// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Opaque immutable provider descriptions. Native factories stay attempt-owned.
use super::*;
use serde::{Deserialize, Serialize};
const VERSION: u32 = 2;
pse_contract::projection_struct! {
/// Untrusted decoded provider data; it contains no callback, factory or native handle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderRecord {
    projection: { name: "KernelProviderRecord", rust_path: crate::portable::ProviderRecord,
        version: VERSION, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Representation interpretation, independent of native provider semantic identity.
    version: u32 => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: VERSION as u64, max: VERSION as u64 }] };
    /// Original native physical/implementation key, checked again during contextual replay.
    key: ContentHash => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// The actual native descriptor; serialized data alone never supplies executable admission.
    spec: ProviderSpec => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Required] };
}
}
#[cfg(test)]
mod tests {
    use super::*;
    use pse_contract::boundary_projection::{
        ConstructionRole, Direction, ProjectionGraph, ProjectionRecord,
    };

    #[test]
    fn portable_record_projects_untrusted_data_with_one_owned_interpretation() {
        let graph = ProjectionGraph::of::<ProviderRecord>().unwrap();
        let definition = &graph.definitions()["KernelProviderRecord"];
        assert_eq!(definition.version, VERSION);
        assert_eq!(<ProviderRecord as ProjectionRecord>::VERSION, VERSION);
        assert_eq!(definition.direction, Direction::Both);
        assert_eq!(definition.construction, ConstructionRole::Data);
        for name in [
            "KernelProviderPort",
            "KernelProviderShape",
            "KernelProviderShapes",
            "KernelProviderSpec",
        ] {
            assert_eq!(graph.definitions()[name].direction, Direction::Both);
        }
    }
}
impl ProviderRecord {
    /// Complete retained provider descriptor fields, excluding attempt-local factories.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(self.spec.heap_bytes())
    }
    /// Capture already admitted exact provider meaning, without worker construction.
    pub fn capture(value: &AdmittedProvider) -> Self {
        Self {
            version: VERSION,
            key: value.spec().key().0,
            spec: value.spec().clone(),
        }
    }
    /// Restore inside the caller-qualified physical receipt session. Port admission is
    /// retained exactly; execution still needs an independently matching factory.
    pub fn restore(&self) -> Result<AdmittedProvider, ProviderError> {
        pse_quantity::resolved::receipts::require_record(self).map_err(|error| match error {
            pse_quantity::QuantityError::RecordMembership(source) => {
                ProviderError::RecordMembership(source)
            }
            error => ProviderError::Contract(error.to_string()),
        })?;
        if self.version != VERSION {
            return Err(ProviderError::Contract(
                "unsupported provider record".into(),
            ));
        }
        if self.spec.key().0 != self.key {
            return Err(ProviderError::Contract(
                "provider record identity differs".into(),
            ));
        }
        Ok(AdmittedProvider(self.spec.clone()))
    }
}
