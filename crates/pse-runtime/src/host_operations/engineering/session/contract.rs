// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable prepared method metadata and original allocations through response encoding.
use super::*;

pub(super) struct PreparedContract {
    document: EngineeringContract,
    extent: usize,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl std::fmt::Debug for PreparedContract {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedEngineeringContract")
            .field("extent", &self.extent)
            .finish_non_exhaustive()
    }
}
pub(in crate::host_operations::engineering) struct ContractOutput {
    pub(in crate::host_operations::engineering) document: EngineeringContract,
    pub(in crate::host_operations::engineering) owner: Arc<dyn pse_columnar::PayloadOwner>,
}

impl EngineeringService {
    pub(in crate::host_operations::engineering) fn describe_contract(
        &self,
    ) -> Result<ContractOutput, EngineeringServiceError> {
        self.describe_contract_world(
            &self.contract,
            super::super::generated::DESCRIPTORS,
            crate::host_operations::engineering_contract::methods,
        )
    }

    pub(in crate::host_operations::engineering) fn describe_study_contract(
        &self,
    ) -> Result<ContractOutput, EngineeringServiceError> {
        self.describe_contract_world(
            &self.explicit_study_contract,
            pse_host::engineering_client::DESCRIPTORS,
            crate::host_operations::engineering_contract::explicit_study_methods,
        )
    }

    fn describe_contract_world(
        &self,
        cache: &Mutex<Option<Arc<PreparedContract>>>,
        descriptor: &[u8],
        methods: impl FnOnce() -> Result<Vec<EngineeringMethodContract>, pse_contract::boundary_projection::ProjectionError>,
    ) -> Result<ContractOutput, EngineeringServiceError> {
        use sha2::Digest as _;
        let mut prepared = cache
            .lock()
            .map_err(|_| infrastructure("prepared contract lock poisoned"))?;
        let math = self.runtime.preparation_services();
        if prepared.is_none() {
            // Graph construction is source-static, but still belongs to this
            // service's actual allocation before any graph/response copy.
            let scratch_extent = descriptor
                .len()
                .checked_mul(16)
                .and_then(|bytes| bytes.checked_add(1 << 20))
                .ok_or_else(|| resource("contract preparation extent overflow"))?;
            let _scratch = math
                .math()
                .reserve("engineering:contract-source-graph", scratch_extent)
                .map_err(owner)?;
            let methods = methods().map_err(owner)?;
            let method_extent = methods.iter().try_fold(
                methods
                    .capacity()
                    .checked_mul(size_of::<EngineeringMethodContract>())
                    .ok_or_else(|| resource("method metadata extent overflow"))?,
                |bytes, method| {
                    [
                        method.name.capacity(),
                        method.input.capacity(),
                        method.output.capacity(),
                        method.input_direction.capacity(),
                        method.output_direction.capacity(),
                    ]
                    .into_iter()
                    .try_fold(bytes, |total, bytes| {
                        total
                            .checked_add(bytes)
                            .ok_or_else(|| resource("method metadata extent overflow"))
                    })
                },
            )?;
            let semantics = "blueprint §19; §20; §21.1; §21.5; §21.7";
            let extent = descriptor
                .len()
                .checked_add(method_extent)
                .and_then(|bytes| {
                    bytes.checked_add(semantics.len() + size_of::<EngineeringContract>() + 4096)
                })
                .ok_or_else(|| resource("prepared contract extent overflow"))?;
            let owner = math
                .math()
                .reserve("engineering:prepared-contract", extent)
                .map_err(owner)?;
            let document = EngineeringContract {
                descriptor: descriptor.to_vec(),
                descriptor_sha256: sha2::Sha256::digest(descriptor).to_vec(),
                semantics: semantics.into(),
                methods,
            };
            *prepared = Some(Arc::new(PreparedContract {
                document,
                extent,
                _owner: owner,
            }));
        }
        let original = prepared
            .as_ref()
            .ok_or_else(|| infrastructure("prepared contract was not retained"))?
            .clone();
        drop(prepared);
        let extent = original
            .extent
            .checked_mul(8)
            .and_then(|bytes| bytes.checked_add(65536))
            .ok_or_else(|| resource("contract encoding extent overflow"))?;
        let encoding = math
            .math()
            .reserve("engineering:contract-response", extent)
            .map_err(owner)?;
        let document = original.document.clone();
        Ok(ContractOutput {
            document,
            owner: Arc::new((original, encoding)),
        })
    }
}
