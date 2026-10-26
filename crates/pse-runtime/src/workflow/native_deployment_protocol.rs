// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Product deployment documents and pre-effect materialization.

use super::{ManagedPrimaryReceiver, NativeAllocation, NativeExecutionAllocation};
use crate::workflow::{WorkflowError, contract};
use pse_model::document::Version;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Production profile names are local deployment vocabulary, not scientific semantics.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum ProductionProfileName {
    /// Ordinary storage service and independently capped managed worker.
    Service,
    /// Ordinary bounded functional work.
    Functional,
    /// Explicit widened functional work.
    Wide,
    /// Dedicated timing role.
    Timing,
    /// Full finite scientific execution.
    Exclusive,
    /// Sixteen-lane scientific reference execution.
    Reference,
    /// Separate durable-test observer role.
    ExclusiveObserver,
}

/// One production declaration; host admission remains a separate owner.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProductionProfile {
    /// Selected production role.
    pub name: ProductionProfileName,
    /// Complete service and execution envelope.
    pub total_memory_bytes: usize,
    /// Separately charged storage process cap.
    pub server_memory_bytes: usize,
    /// Exact managed primary process cap.
    pub native_worker_memory_bytes: usize,
    /// Shared scientific runtime policy within that primary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<NativeExecutionAllocation>,
}

/// The separately typed production section of agent-capacity.toml.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProductionProfiles {
    /// Declaration interpretation.
    pub version: Version<1>,
    /// Default service role; explicit operator choices retain their own scope.
    pub default_profile: ProductionProfileName,
    /// Separately typed default capability request allowances.
    pub capabilities: CapabilityDefaults,
    /// Distinct production roles, preserving their different physical scopes.
    pub profiles: Vec<ProductionProfile>,
}

/// Host capability request defaults, distinct from managed receiver pools.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CapabilityDefaults {
    /// Interactive accounted pool default.
    pub memory_limit_bytes: std::num::NonZeroUsize,
    /// Interactive scientific CPU permit default.
    pub threads: std::num::NonZeroUsize,
    /// Explicit bounded spill allowance.
    pub max_spill_bytes: std::num::NonZeroUsize,
}

impl ProductionProfiles {
    /// Materialize a selected declared role through the one allocation validator.
    /// # Errors
    /// Duplicate/missing roles or invalid relational resource policy.
    pub fn select(&self, name: ProductionProfileName) -> Result<NativeAllocation, WorkflowError> {
        let mut names = std::collections::BTreeSet::new();
        for profile in &self.profiles {
            if !names.insert(profile.name) {
                return Err(contract("duplicate production execution profile"));
            }
        }
        let profile = self
            .profiles
            .iter()
            .find(|profile| profile.name == name)
            .ok_or_else(|| contract("selected production execution profile is absent"))?;
        NativeAllocation::materialize(
            profile.total_memory_bytes,
            profile.server_memory_bytes,
            1,
            profile.native_worker_memory_bytes,
            profile.execution,
        )
    }
    /// Read only the production-owned declaration from the existing host policy file.
    /// # Errors
    /// Unreadable, malformed, unsupported or missing production declarations.
    pub fn read(path: &Path) -> Result<Self, WorkflowError> {
        #[derive(Deserialize)]
        struct Declaration {
            production: ProductionProfiles,
        }
        use std::io::Read;
        let mut bytes = String::new();
        let file =
            std::fs::File::open(path).map_err(|error| WorkflowError::Input(error.to_string()))?;
        if !file
            .metadata()
            .map_err(|error| WorkflowError::Input(error.to_string()))?
            .is_file()
        {
            return Err(contract("production declaration must be a regular file"));
        }
        file.take(1024 * 1024 + 1)
            .read_to_string(&mut bytes)
            .map_err(|error| WorkflowError::Input(error.to_string()))?;
        if bytes.len() > 1024 * 1024 {
            return Err(contract(
                "production declaration exceeds its bounded envelope",
            ));
        }
        let declaration: Declaration =
            toml::from_str(&bytes).map_err(|error| WorkflowError::Input(error.to_string()))?;
        Ok(declaration.production)
    }
}

/// Operations accepted by the standalone product deployment validator.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub enum DeploymentRequest {
    /// Materialize operator-selected values before any configuration publication.
    Materialize {
        /// Complete service and execution envelope.
        total_memory_bytes: usize,
        /// Separately placed storage process.
        server_memory_bytes: usize,
        /// Independently capped managed worker slots.
        native_workers: usize,
        /// Hard process cap for each managed worker.
        native_worker_memory_bytes: usize,
        /// Optional shared scientific execution policy.
        execution: Option<NativeExecutionAllocation>,
    },
    /// Validate an existing materialized allocation without reinterpreting defaults.
    Validate {
        /// Original declared allocation.
        allocation: NativeAllocation,
    },
    /// Select an upstream production role from its explicit declaration file.
    Profile {
        /// Original production declaration path.
        declaration: PathBuf,
        /// Production role selected by this consumer.
        profile: ProductionProfileName,
    },
}

/// Actual product materialization, independent of environment/systemd realization.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeploymentResponse {
    /// Validated complete allocation.
    pub allocation: NativeAllocation,
}
impl DeploymentRequest {
    /// Apply one bounded pre-effect request through the canonical validator.
    /// # Errors
    /// Invalid allocation, declaration or selected production role.
    pub fn materialize(self) -> Result<DeploymentResponse, WorkflowError> {
        let allocation = match self {
            Self::Materialize {
                total_memory_bytes,
                server_memory_bytes,
                native_workers,
                native_worker_memory_bytes,
                execution,
            } => NativeAllocation::materialize(
                total_memory_bytes,
                server_memory_bytes,
                native_workers,
                native_worker_memory_bytes,
                execution,
            )?,
            Self::Validate { allocation } => {
                allocation.validate()?;
                allocation
            }
            Self::Profile {
                declaration,
                profile,
            } => ProductionProfiles::read(&declaration)?.select(profile)?,
        };
        Ok(DeploymentResponse { allocation })
    }
}

/// Shared execution-owned fields projected from service and receiver configurations.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ManagedExecutionPolicy {
    /// One already materialized resource allocation.
    pub resources: NativeAllocation,
    /// Exact managed receiver association, when selected.
    #[serde(default)]
    pub primary_receiver: Option<ManagedPrimaryReceiver>,
}

/// Registered context document shares the allocation and receiver owners.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RegisteredNativeContext {
    /// Registered-context interpretation.
    pub version: Version<1>,
    /// Exactly associated database.
    pub database: String,
    /// Same actual execution policy consumed at the base configuration boundary.
    #[serde(flatten)]
    pub policy: ManagedExecutionPolicy,
    /// Immutable receiver state root.
    pub receiver_state: PathBuf,
    /// Never-reinterpreted receiver generation.
    pub receiver_generation: String,
}

/// Supervisor readiness states are observations, never stop or release authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum DeploymentReadiness {
    /// Listener and protocol are ready.
    Ready,
    /// The admitted service is starting.
    Starting,
    /// The service is stopping.
    Stopping,
    /// The owning maintenance lifetime has admission closed.
    Maintenance,
    /// Exact absence is established and may be acted on.
    ActionableAbsent,
    /// Required authority cannot be observed.
    Unavailable,
    /// Observed authority differs from its selected binding.
    Mismatch,
}

/// Narrow service observation transport; no full runtime configuration replica.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeploymentObservation {
    /// Selected observed readiness state.
    pub phase: DeploymentReadiness,
    /// Exact service state directory.
    pub state: PathBuf,
    /// Selected service generation, when known.
    pub instance_id: Option<String>,
    /// Selected managed unit.
    pub unit: String,
    /// Owner-authored explanation of the observation.
    pub cause: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn production_profiles_preserve_exact_reference_and_distinct_fixture_roles() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let profiles = ProductionProfiles::read(&root.join(".config/agent-capacity.toml")).unwrap();
        let reference = profiles.select(ProductionProfileName::Reference).unwrap();
        assert_eq!(reference.total_memory_bytes, 160 << 30);
        assert_eq!(reference.native_worker_memory_bytes, 140 << 30);
        assert_eq!(reference.execution.unwrap().case_lanes, 16);
        assert_eq!(reference.execution.unwrap().pool_memory_bytes, 128 << 30);
        let service = profiles.select(profiles.default_profile).unwrap();
        assert_eq!(service.total_memory_bytes, 48 << 30);
        assert_eq!(service.native_worker_memory_bytes, 32 << 30);
        assert!(service.execution.is_none());
        assert_eq!(profiles.capabilities.memory_limit_bytes.get(), 16 << 30);
        assert_eq!(profiles.capabilities.threads.get(), 16);
        assert_eq!(profiles.capabilities.max_spill_bytes.get(), 1 << 30);
        let functional = profiles.select(ProductionProfileName::Functional).unwrap();
        let wide = profiles.select(ProductionProfileName::Wide).unwrap();
        assert_eq!(functional.execution.unwrap().case_lanes, 2);
        assert_eq!(wide.execution.unwrap().case_lanes, 4);
        assert_ne!(
            functional.native_worker_memory_bytes,
            wide.native_worker_memory_bytes
        );
        for name in [
            ProductionProfileName::Service,
            ProductionProfileName::Functional,
            ProductionProfileName::Wide,
            ProductionProfileName::Timing,
            ProductionProfileName::Exclusive,
            ProductionProfileName::Reference,
            ProductionProfileName::ExclusiveObserver,
        ] {
            profiles.select(name).unwrap().validate().unwrap();
        }
    }
    #[test]
    fn one_validator_rejects_overflow_and_relationally_impossible_allocations() {
        let valid = NativeAllocation::materialize(4 << 30, 1 << 30, 2, 1 << 30, None).unwrap();
        assert!(
            DeploymentRequest::Validate { allocation: valid }
                .materialize()
                .is_ok()
        );
        assert!(NativeAllocation::materialize(2 << 30, 1 << 30, 2, 1 << 30, None).is_err());
        assert!(NativeAllocation::materialize(usize::MAX, 1 << 30, 32, usize::MAX, None).is_err());
        let mut invalid = valid;
        invalid.surrealmx_memory_limit_bytes = invalid.server_memory_bytes + 1;
        assert!(invalid.validate().is_err());
    }
    #[test]
    fn standalone_protocol_refuses_missing_version_unknown_fields_and_invalid_execution() {
        type Request = crate::boundary_documents::Versioned<DeploymentRequest, 1>;
        assert!(serde_json::from_str::<Request>(r#"{"operation":"materialize","total_memory_bytes":4294967296,"server_memory_bytes":1073741824,"native_workers":1,"native_worker_memory_bytes":1073741824,"execution":null}"#).is_err());
        assert!(serde_json::from_str::<Request>(r#"{"document_version":1,"operation":"profile","declaration":"/config","profile":"reference","unknown":true}"#).is_err());
        assert!(serde_json::from_str::<Request>(r#"{"document_version":2,"operation":"profile","declaration":"/config","profile":"reference"}"#).is_err());
    }
}

/// Immutable runtime inputs selected by source-side preparation, never a setup recipe.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreparedReceiverEnvironment {
    /// Standalone runtime-input interpretation.
    pub version: Version<1>,
    /// Original selected capabilities; thread/resource budgets remain separately owned.
    pub capabilities: Vec<String>,
    /// Selected nonsecret path, loader and compiler inputs only.
    pub values: std::collections::BTreeMap<String, String>,
    /// Exact managed native generation receipts; bytes are still checked by their owner.
    pub provider_receipts: std::collections::BTreeMap<PathBuf, String>,
}
