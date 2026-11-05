// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source graph selection and domain policy composition, separate from the shared wire adapter.
use super::ProjectionCodegenError;
use buffa_descriptor::DynamicMessage;
use pse_contract::boundary_projection::Direction;
use pse_contract_proto::{DescriptorPolicy, FieldContext, MetadataError, NativeSelection};

pub(super) type DomainCheck =
    fn(&DynamicMessage, FieldContext<'_>) -> Result<Option<Vec<u8>>, ProjectionCodegenError>;
#[derive(Debug)]
pub(super) struct SelectionNames {
    pub(super) interpretations: Vec<(String, u32)>,
    pub(super) owners: Vec<String>,
    pub(super) fields: Vec<String>,
    pub(super) methods: Vec<String>,
    pub(super) facets: Vec<(String, String)>,
    pub(super) facet_fields: Vec<(String, String, String)>,
}
pub(super) fn native_selection(
    graph: &pse_contract::boundary_projection::ProjectionGraph,
    package: &str,
    direction: Direction,
) -> SelectionNames {
    use pse_contract::boundary_projection::Shape;
    let mut selection = SelectionNames {
        interpretations: vec![],
        owners: vec![],
        fields: vec![],
        methods: vec![],
        facets: vec![],
        facet_fields: vec![],
    };
    for owner in graph.definitions().values() {
        let name = format!("{package}.{}", owner.name);
        selection.owners.push(name.clone());
        selection
            .interpretations
            .push((name.clone(), owner.version));
        match &owner.shape {
            Shape::Record(fields) => {
                for field in fields
                    .iter()
                    .filter(|field| field.direction.includes(direction))
                {
                    let path = format!("{name}.{}", field.name);
                    if super::registry_facets::has_options(field) {
                        selection
                            .facets
                            .push((path.clone(), super::registry_facets::EXTENSION.into()));
                        for required in super::registry_facets::required_fields(field) {
                            selection.facet_fields.push((
                                path.clone(),
                                super::registry_facets::EXTENSION.into(),
                                required.into(),
                            ));
                        }
                    }
                    selection.fields.push(path);
                }
            }
            Shape::Alternatives(alternatives) => {
                for alternative in alternatives {
                    selection
                        .fields
                        .push(format!("{name}.{}", super::snake_case(&alternative.name)));
                    for field in alternative
                        .fields
                        .iter()
                        .filter(|field| field.direction.includes(direction))
                    {
                        let path = format!("{name}.Alternative{}.{}", alternative.tag, field.name);
                        if super::registry_facets::has_options(field) {
                            selection
                                .facets
                                .push((path.clone(), super::registry_facets::EXTENSION.into()));
                            for required in super::registry_facets::required_fields(field) {
                                selection.facet_fields.push((
                                    path.clone(),
                                    super::registry_facets::EXTENSION.into(),
                                    required.into(),
                                ));
                            }
                        }
                        selection.fields.push(path);
                    }
                }
            }
            Shape::StringEnum(_) => selection.fields.push(format!("{name}.value")),
        }
    }
    for service in graph.services().values() {
        for method in &service.methods {
            selection
                .methods
                .push(format!("{package}.{}.{}", service.name, method.name));
        }
    }
    selection
}

/// Explicit selected domain policies; unknown ownership never falls back to wire
/// compatibility. Owner callbacks validate typed payloads under the acquired pool.
pub(super) struct TypedPolicy<'a> {
    validators: &'a [(&'a str, DomainCheck)],
    interpretations: &'a [(&'a str, u32)],
    selection: NativeSelection<'a>,
}
impl<'a> TypedPolicy<'a> {
    pub(super) fn new(
        validators: &'a [(&'a str, DomainCheck)],
        interpretations: &'a [(&'a str, u32)],
        selection: NativeSelection<'a>,
    ) -> Self {
        Self {
            validators,
            interpretations,
            selection,
        }
    }
    fn validate(
        &self,
        facet: &pse_contract::metadata::Facet<'_>,
    ) -> pse_contract::metadata::Disposition {
        use pse_contract::metadata::Disposition;
        if facet.interpretation_available
            && self
                .validators
                .iter()
                .any(|(name, _)| *name == facet.policy)
        {
            Disposition::Compatible
        } else {
            Disposition::Unknown
        }
    }
}
impl DescriptorPolicy for TypedPolicy<'_> {
    fn native_selection(&self) -> Option<NativeSelection<'_>> {
        Some(self.selection)
    }
    fn normalize_decoded_facet(
        &self,
        extension: &str,
        value: &DynamicMessage,
        field: FieldContext<'_>,
    ) -> Option<Result<Option<Vec<u8>>, MetadataError>> {
        self.validators
            .iter()
            .find(|(name, _)| *name == extension)
            .map(|(_, validate)| {
                validate(value, field).map_err(|source| match source {
                    ProjectionCodegenError::Metadata(source) if source.is_unavailable() => source,
                    source => MetadataError::domain(extension, source),
                })
            })
    }
}
impl pse_contract::metadata::InterpretationPolicy for TypedPolicy<'_> {
    fn name(&self) -> &str {
        "pse.contract.v2/native-crossing+registry-execution+canonical-publication"
    }
    fn supports_interpretation(&self, subject: &str, version: u32) -> bool {
        self.interpretations.contains(&(subject, version))
    }
    fn check_facet(
        &self,
        facet: &pse_contract::metadata::Facet<'_>,
    ) -> pse_contract::metadata::Disposition {
        self.validate(facet)
    }
    fn compare_facet(
        &self,
        old: &pse_contract::metadata::Facet<'_>,
        new: &pse_contract::metadata::Facet<'_>,
    ) -> pse_contract::metadata::Disposition {
        use pse_contract::metadata::Disposition;
        match (self.validate(old), self.validate(new)) {
            (Disposition::Compatible, Disposition::Compatible) => {
                if old.value == new.value {
                    Disposition::Compatible
                } else {
                    Disposition::Incompatible
                }
            }
            (Disposition::Incompatible, _) | (_, Disposition::Incompatible) => {
                Disposition::Incompatible
            }
            _ => Disposition::Unknown,
        }
    }
    fn introduced_facet(
        &self,
        new: &pse_contract::metadata::Facet<'_>,
    ) -> pse_contract::metadata::Disposition {
        use pse_contract::metadata::Disposition;
        if self.validate(new) == Disposition::Compatible {
            Disposition::Incompatible
        } else {
            Disposition::Unknown
        }
    }
}

#[cfg(test)]
mod tests;
