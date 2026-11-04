// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive bounded inspection of the actual retained executable registrations.

use super::{ModelingPackage, WorkflowError, contract};
use pse_ids::ContentHash;
use pse_contract::boundary_projection::{Direction, LocalRule, ValidateProjection as _};
use std::sync::Arc;

const MAXIMUM_PROVIDER_INSPECTION_BYTES: usize = 64 << 10;

pse_contract::projection_struct! {
/// Select actual retained provider registrations without constructing a native worker.
#[derive(Clone, Debug)]
pub struct ProviderInspectionRequest {
    projection: { name: "ProviderInspectionRequest", rust_path: crate::workflow::ProviderInspectionRequest,
        version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
    /// Exact package-local registration name; absence requests the finite ordered page.
    pub name: Option<String> => { tag: 1, direction: Direction::Input, rules: [LocalRule::Utf8Bytes { min: 0, max: 1024 }] };
    /// Original name-ordered inventory offset, ignored for exact-name selection.
    pub offset: usize => { tag: 2, direction: Direction::Input, rules: [] };
    /// One through sixteen registrations; the complete encoded response must fit64KiB.
    pub limit: usize => { tag: 3, direction: Direction::Input, rules: [LocalRule::UnsignedRange { min: 1, max: 16 }] };
}
}

pse_contract::projection_struct! {
/// Original retained factory meaning. A descriptor does not prove a new native worker can start.
#[derive(Clone, Debug)]
pub struct ProviderInspectionEntry {
    projection: { name: "ProviderInspectionEntry", rust_path: crate::workflow::ProviderInspectionEntry,
        version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Actual package-local retained registration name or the explicitly requested absent name.
    pub name: String => { tag: 1, direction: Direction::Output, rules: [] };
    /// Actual physically admitted factory descriptor; absent means no retained registration.
    pub registration: Option<pse_kernels::ProviderSpec> => { tag: 2, direction: Direction::Output, rules: [] };
    /// Actual immutable factory/envelope configuration identity, absent with the registration.
    pub configuration: Option<ContentHash> => { tag: 3, direction: Direction::Output, rules: [] };
}
}

pse_contract::projection_struct! {
/// Exact native source and finite retained-factory observations, never provider preparation.
#[derive(Clone, Debug)]
pub struct ProviderInspectionPage {
    projection: { name: "ProviderInspectionPage", rust_path: crate::workflow::ProviderInspectionPage,
        version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Exact original model and physical attribution.
    pub source: crate::workflow::OperationSource => { tag: 1, direction: Direction::Output, rules: [] };
    /// Complete original retained registration count, independently from selected rows.
    pub total: usize => { tag: 2, direction: Direction::Output, rules: [] };
    /// Actual original inventory offset.
    pub offset: usize => { tag: 3, direction: Direction::Output, rules: [] };
    /// Next original inventory offset, absent only when the selection is exhausted.
    pub next_offset: Option<usize> => { tag: 4, direction: Direction::Output, rules: [] };
    /// Actual retained registrations or one explicit absent-name observation.
    pub entries: Vec<ProviderInspectionEntry> => { tag: 5, direction: Direction::Output, rules: [LocalRule::Length { min: 0, max: 16 }] };
}
}

/// Native selected metadata with its original allocation custody.
#[derive(Debug)]
pub struct ProviderInspectionOutput {
    /// Original bounded passive page.
    pub page: ProviderInspectionPage,
    /// Original allocation retained through encoded response consumption.
    pub owner: Arc<dyn pse_columnar::PayloadOwner>,
}

impl ModelingPackage {
    /// Inspect actual retained factories without `worker`, `create_scoped` or evaluation.
    /// # Errors
    /// Invalid finite selection, source mismatch, cancellation and original pool refusals propagate.
    pub fn inspect_providers(
        &self,
        services: &crate::workflow::PreparationServices,
        request: &ProviderInspectionRequest,
        cancel: &crate::CancelSource,
    ) -> Result<ProviderInspectionOutput, WorkflowError> {
        let _activity = services.admit_activity()?;
        self.check_service_bindings(services)?;
        request.validate_projection().map_err(|error| {
            WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
        })?;
        super::canonical::checkpoint(cancel)?;
        let too_large = || {
            WorkflowError::Resource(
                "native selected provider metadata exceeds 64KiB; narrow the page".into(),
            )
        };
        // Attribute the selected response to the actual source without cloning
        // its optional research/context owners before their storage is bounded.
        let mut source_extent = 4096usize;
        if let Some(source) = &self.research_source {
            for value in [
                &source.issuing_store,
                &source.selection,
                &source.dataset,
                &source.revision,
                &source.physical_context,
            ]
            .into_iter()
            .chain(&source.publication)
            {
                source_extent = source_extent.saturating_add(value.capacity());
            }
        }
        if let Some(selection) = &self.project_context {
            source_extent = source_extent
                .saturating_add(selection.cut.issuing_store.capacity())
                .saturating_add(selection.cut.project.as_str().len())
                .saturating_add(pse_model::HeapUsage::heap_bytes(&selection.context));
        }
        if source_extent > MAXIMUM_PROVIDER_INSPECTION_BYTES {
            return Err(too_large());
        }
        let selected = |name: &str| {
            request
                .name
                .as_deref()
                .is_none_or(|selected| selected == name)
        };
        let range = self
            .providers
            .iter()
            .filter(|(name, _)| selected(name))
            .skip(if request.name.is_some() {
                0
            } else {
                request.offset
            })
            .take(request.limit);
        let extent = range
            .clone()
            .try_fold(source_extent, |size, (name, registration)| {
                let spec = registration.spec();
                let extent = size
                    .checked_add(name.capacity())
                    .and_then(|value| value.checked_add(size_of::<ProviderInspectionEntry>()))
                    .and_then(|value| {
                        value.checked_add(
                            spec.inputs
                                .capacity()
                                .checked_mul(size_of::<pse_kernels::Port>())?,
                        )
                    })
                    .and_then(|value| {
                        value.checked_add(
                            spec.outputs
                                .capacity()
                                .checked_mul(size_of::<pse_kernels::Port>())?,
                        )
                    })
                    .and_then(|value| value.checked_add(spec.shapes.retained_bytes()))
                    .ok_or_else(|| contract("selected provider metadata extent overflow"))?;
                if extent > MAXIMUM_PROVIDER_INSPECTION_BYTES {
                    return Err(too_large());
                }
                Ok(extent)
            })?;
        let owner = services
            .math()
            .reserve("modeling:provider-inspection", extent)?;
        let mut entries = range
            .map(|(name, registration)| ProviderInspectionEntry {
                name: name.clone(),
                registration: Some(registration.spec().clone()),
                configuration: Some(registration.configuration_key()),
            })
            .collect::<Vec<_>>();
        if entries.is_empty()
            && let Some(name) = &request.name
        {
            entries.push(ProviderInspectionEntry {
                name: name.clone(),
                registration: None,
                configuration: None,
            });
        }
        super::canonical::checkpoint(cancel)?;
        let next = request.offset.saturating_add(request.limit);
        Ok(ProviderInspectionOutput {
            page: ProviderInspectionPage {
                source: crate::workflow::OperationSource::of(self),
                total: self.providers.len(),
                offset: if request.name.is_some() {
                    0
                } else {
                    request.offset
                },
                next_offset: (request.name.is_none() && next < self.providers.len())
                    .then_some(next),
                entries,
            },
            owner,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "actual retained provider inspection controls"
    )]
    use super::*;
    use pse_ids::SemanticId;
    use pse_kernels::{
        AdmittedProvider, DerivativeOrder, DerivativeSource, Port, ProviderFactory, ProviderShapes,
        ProviderSpec, Registration,
    };
    use std::{
        collections::BTreeMap,
        sync::atomic::{AtomicUsize, Ordering},
    };

    #[derive(Debug)]
    struct ObservedFactory {
        spec: ProviderSpec,
        configuration: ContentHash,
        constructions: Arc<AtomicUsize>,
    }
    impl ProviderFactory for ObservedFactory {
        fn spec(&self) -> &ProviderSpec {
            &self.spec
        }
        fn configuration_key(&self) -> ContentHash {
            self.configuration
        }
        fn create(&self) -> Result<pse_kernels::ProviderBinding, pse_kernels::ProviderError> {
            self.constructions.fetch_add(1, Ordering::SeqCst);
            Err(pse_kernels::ProviderError::Contract(
                "inspection must not construct this worker".into(),
            ))
        }
    }

    async fn shaped_registrations(
        runtime: &crate::workflow::Runtime,
        selections: &[(&str, usize)],
    ) -> (
        ModelingPackage,
        Arc<AtomicUsize>,
        BTreeMap<String, ProviderSpec>,
    ) {
        let physical = crate::workflow::tests::physical();
        let quantity = physical.quantities.neutral_dimensionless().unwrap();
        let unit = physical
            .quantities
            .quantity_type(quantity)
            .unwrap()
            .canonical_unit;
        let constructions = Arc::new(AtomicUsize::new(0));
        let mut registrations = BTreeMap::new();
        let mut originals = BTreeMap::new();
        for &(name, cells) in selections {
            let id = pse_ids::named_id(SemanticId::from_bytes([71; 16]), name);
            let inputs = (0..cells)
                .map(|index| Port {
                    id: pse_ids::named_id(id, &format!("input/{index}")),
                    quantity,
                    unit,
                })
                .collect::<Vec<_>>();
            let shape = pse_kernels::ProviderShape {
                id: pse_ids::named_id(id, "logical-input"),
                axes: vec![pse_ids::named_id(id, "axis")],
                coordinates: inputs.iter().map(|port| vec![port.id]).collect(),
                cells: inputs.iter().map(|port| port.id).collect(),
            };
            let spec = ProviderSpec {
                id,
                revision: ContentHash::from_bytes([72; 32]),
                data: ContentHash::from_bytes([73; 32]),
                shapes: ProviderShapes {
                    inputs: vec![shape],
                    outputs: Vec::new(),
                },
                derivative_source: DerivativeSource::Analytic,
                inputs,
                outputs: vec![Port {
                    id: pse_ids::named_id(id, "output"),
                    quantity,
                    unit,
                }],
                derivatives: DerivativeOrder::First,
                smoothness: DerivativeOrder::First,
            };
            let registration = Registration::bind(
                AdmittedProvider::new(spec.clone(), &physical.quantities).unwrap(),
                Arc::new(ObservedFactory {
                    spec: spec.clone(),
                    configuration: ContentHash::from_bytes([74; 32]),
                    constructions: constructions.clone(),
                }),
            )
            .unwrap();
            originals.insert(name.to_owned(), spec);
            registrations.insert(name.to_owned(), registration);
        }
        let declarations = pse_authoring::language::parse(
            "package selected { param x:Scalar=1; }",
            SemanticId::from_bytes([75; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let package = runtime
            .modeling_package_registered(declarations, physical, registrations)
            .await
            .unwrap();
        (package, constructions, originals)
    }

    #[tokio::test]
    async fn provider_inspection_refuses_direct_reads_after_original_caller_closure() {
        let runtime = crate::workflow::tests::runtime();
        let (package, constructions, _) = shaped_registrations(&runtime, &[("law", 1)]).await;
        let services = runtime.preparation_services();
        let pool = services.pool();
        let before = pool.reserved();
        let effects = runtime.native().test_admission_snapshot();
        runtime.close();
        assert!(matches!(
            package.inspect_providers(
                &services,
                &ProviderInspectionRequest {
                    name: Some("law".into()),
                    offset: 0,
                    limit: 1
                },
                &crate::CancelSource::new()
            ),
            Err(WorkflowError::CallerClosed)
        ));
        assert_eq!(pool.reserved(), before);
        assert_eq!(runtime.native().test_admission_snapshot(), effects);
        assert_eq!(constructions.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn provider_inspection_bounds_original_nested_storage_before_cloning_and_allows_narrower_cuts()
     {
        let runtime = crate::workflow::tests::runtime();
        let (package, constructions, originals) =
            shaped_registrations(&runtime, &[("a", 512), ("b", 512), ("c-large", 1024)]).await;
        let services = runtime.preparation_services();
        let pool = services.pool();
        let before = pool.reserved();
        let effects = runtime.native().test_admission_snapshot();
        let cancel = crate::CancelSource::new();
        let ports_only = ["a", "b"]
            .into_iter()
            .map(|name| {
                let spec = &originals[name];
                (spec.inputs.capacity() + spec.outputs.capacity()) * size_of::<Port>()
                    + size_of::<ProviderInspectionEntry>()
                    + name.len()
            })
            .sum::<usize>()
            + 4096;
        assert!(
            ports_only < MAXIMUM_PROVIDER_INSPECTION_BYTES,
            "the combined refusal must include original nested shape storage"
        );
        assert!(matches!(
            package.inspect_providers(
                &services,
                &ProviderInspectionRequest {
                    name: None,
                    offset: 0,
                    limit: 2
                },
                &cancel
            ),
            Err(WorkflowError::Resource(_))
        ));
        assert_eq!(
            pool.reserved(),
            before,
            "oversized cuts refuse before selected allocation"
        );
        assert!(matches!(
            package.inspect_providers(
                &services,
                &ProviderInspectionRequest {
                    name: Some("c-large".into()),
                    offset: 0,
                    limit: 1
                },
                &cancel
            ),
            Err(WorkflowError::Resource(_))
        ));
        assert_eq!(
            pool.reserved(),
            before,
            "a single oversized descriptor also refuses before cloning"
        );
        for (offset, name) in [(0, "a"), (1, "b")] {
            let selected = package
                .inspect_providers(
                    &services,
                    &ProviderInspectionRequest {
                        name: None,
                        offset,
                        limit: 1,
                    },
                    &cancel,
                )
                .unwrap();
            assert_eq!(
                selected.page.source,
                crate::workflow::OperationSource::of(&package)
            );
            assert_eq!(selected.page.total, 3);
            assert_eq!(selected.page.offset, offset);
            assert_eq!(selected.page.next_offset, Some(offset + 1));
            assert_eq!(selected.page.entries.len(), 1);
            assert_eq!(selected.page.entries[0].name, name);
            assert_eq!(
                selected.page.entries[0].registration.as_ref(),
                Some(&originals[name])
            );
            assert!(pool.reserved() > before);
            assert!(pool.reserved() - before <= MAXIMUM_PROVIDER_INSPECTION_BYTES);
            drop(selected);
            assert_eq!(pool.reserved(), before);
        }
        let exact = package
            .inspect_providers(
                &services,
                &ProviderInspectionRequest {
                    name: Some("b".into()),
                    offset: usize::MAX,
                    limit: 16,
                },
                &cancel,
            )
            .unwrap();
        assert_eq!(
            exact.page.entries[0].registration.as_ref(),
            Some(&originals["b"])
        );
        assert_eq!(exact.page.offset, 0);
        assert_eq!(exact.page.next_offset, None);
        drop(exact);
        assert_eq!(pool.reserved(), before);
        assert_eq!(runtime.native().test_admission_snapshot(), effects);
        assert_eq!(
            constructions.load(Ordering::SeqCst),
            0,
            "bounded inspection never constructs a worker for any retained registration"
        );
    }

    #[tokio::test]
    async fn provider_inspection_retains_actual_factory_facts_without_native_construction() {
        let runtime = crate::workflow::tests::runtime();
        let physical = crate::workflow::tests::physical();
        let quantity = physical.quantities.neutral_dimensionless().unwrap();
        let unit = physical
            .quantities
            .quantity_type(quantity)
            .unwrap()
            .canonical_unit;
        let spec = ProviderSpec {
            shapes: ProviderShapes::default(),
            derivative_source: DerivativeSource::Analytic,
            id: SemanticId::from_bytes([41; 16]),
            revision: ContentHash::from_bytes([42; 32]),
            data: ContentHash::from_bytes([43; 32]),
            inputs: vec![Port {
                id: SemanticId::from_bytes([44; 16]),
                quantity,
                unit,
            }],
            outputs: vec![Port {
                id: SemanticId::from_bytes([45; 16]),
                quantity,
                unit,
            }],
            derivatives: DerivativeOrder::First,
            smoothness: DerivativeOrder::First,
        };
        let constructions = Arc::new(AtomicUsize::new(0));
        let registration = Registration::bind(
            AdmittedProvider::new(spec.clone(), &physical.quantities).unwrap(),
            Arc::new(ObservedFactory {
                spec: spec.clone(),
                configuration: ContentHash::from_bytes([46; 32]),
                constructions: constructions.clone(),
            }),
        )
        .unwrap();
        let configuration = registration.configuration_key();
        let declarations = pse_authoring::language::parse(
            "package selected { param x:Scalar=1; }",
            SemanticId::from_bytes([47; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let package = runtime
            .modeling_package_registered(
                declarations,
                physical,
                BTreeMap::from([("law".into(), registration)]),
            )
            .await
            .unwrap();
        let services = runtime.preparation_services();
        let cancel = crate::CancelSource::new();
        let pool = services.pool();
        let before = pool.reserved();
        let selected = package
            .inspect_providers(
                &services,
                &ProviderInspectionRequest {
                    name: Some("law".into()),
                    offset: 0,
                    limit: 1,
                },
                &cancel,
            )
            .unwrap();
        assert_eq!(
            selected.page.source,
            crate::workflow::OperationSource::of(&package)
        );
        assert_eq!(selected.page.total, 1);
        assert_eq!(selected.page.entries[0].registration.as_ref(), Some(&spec));
        assert_eq!(selected.page.entries[0].configuration, Some(configuration));
        assert!(
            pool.reserved() > before,
            "original selected metadata remains charged while retained"
        );
        drop(selected);
        assert_eq!(pool.reserved(), before);
        let absent = package
            .inspect_providers(
                &services,
                &ProviderInspectionRequest {
                    name: Some("absent".into()),
                    offset: 0,
                    limit: 1,
                },
                &cancel,
            )
            .unwrap();
        assert_eq!(absent.page.entries.len(), 1);
        assert!(absent.page.entries[0].registration.is_none());
        assert!(absent.page.entries[0].configuration.is_none());
        assert_eq!(constructions.load(Ordering::SeqCst), 0);
        cancel.cancel();
        assert!(
            package
                .inspect_providers(
                    &services,
                    &ProviderInspectionRequest {
                        name: None,
                        offset: 0,
                        limit: 1
                    },
                    &cancel
                )
                .is_err()
        );
        assert_eq!(constructions.load(Ordering::SeqCst), 0);
    }
}
