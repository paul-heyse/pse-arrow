// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read-only cgroup aggregate properties, fenced to the original manager incarnation.
use super::*;

/// Native unit aggregates; these cannot be attributed exclusively to a scientific operation.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
pub struct NativeUnitAccounting {
    /// Unit and descendant cgroup CPU time since activation, in native nanoseconds.
    pub cpu_usage_nanoseconds: Option<u64>,
    /// Current whole-unit memory, in native bytes; absent after unavailable accounting.
    pub memory_current_bytes: Option<u64>,
    /// Current whole-unit task count; absent when the manager reports unavailable accounting.
    pub tasks_current: Option<u64>,
    /// Original service Result, absent for scopes and slices.
    pub service_result: Option<String>,
    /// Independent property reads and kernel membership checks are not an atomic snapshot.
    pub atomic_snapshot: bool,
}
fn available(value: u64) -> Option<u64> {
    (value != u64::MAX).then_some(value)
}
impl HostManager {
    /// Observe a recorded unit root without constructing a launch intent or mutation handle.
    pub(crate) async fn observe_custody_binding(
        &self,
        original_manager: &str,
        name: &str,
        binding: &UnitBinding,
    ) -> Result<UnitObservation, HostError> {
        self.validate_view()?;
        if original_manager != self.identity_key()
            || binding.invocation.len() != 16
            || binding.group.is_empty()
            || binding.device == 0
            || binding.inode == 0
        {
            return Err(HostError::Ownership(
                "recorded native manager/root identity is unavailable or changed".into(),
            ));
        }
        let observed = self.observe(name).await?;
        let group = Group::open(&binding.group, false)?;
        if observed.invocation != binding.invocation
            || observed.control_group != binding.group
            || group.device != binding.device
            || group.inode != binding.inode
        {
            return Err(HostError::Ownership(
                "recorded original unit incarnation or kernel group changed".into(),
            ));
        }
        self.validate_view()?;
        Ok(observed)
    }
    /// Authenticate a full publishing process identity inside a recorded foreground root.
    /// This borrows opened membership facts only, never scope or subgroup control.
    pub(crate) async fn authenticate_foreground_member(
        &self,
        original_manager: &str,
        name: &str,
        binding: &UnitBinding,
        process: &ProcessIdentity,
    ) -> Result<UnitObservation, HostError> {
        let observed = self
            .observe_custody_binding(original_manager, name, binding)
            .await?;
        if !self.native_workload(&observed).await? {
            return Err(HostError::Ownership(
                "recorded foreground unit is not the original native workload".into(),
            ));
        }
        let membership =
            CurrentMembership::authenticate_process(self, observed, process.clone()).await?;
        if !membership.matches_root(binding) {
            return Err(HostError::Ownership(
                "foreground publisher original native root changed".into(),
            ));
        }
        membership.validate_process(self).await?;
        Ok(membership.root().clone())
    }
    pub(crate) async fn native_accounting(
        &self,
        original_manager: &str,
        name: &str,
        binding: &UnitBinding,
    ) -> Result<NativeUnitAccounting, HostError> {
        tokio::time::timeout(Duration::from_secs(5), async {
            self.observe_custody_binding(original_manager, name, binding)
                .await?;
            let unit = self.unit(name).await?;
            let path = unit.inner().path().to_owned();
            let (cpu, memory, tasks, result) = if name.ends_with(".service") {
                let service = ServiceProxy::builder(&self.connection)
                    .destination(self.owner.as_str())
                    .path(path)
                    .cache_properties(CacheProperties::No)
                    .build()
                    .await?;
                let (cpu, memory, tasks, result) = tokio::try_join!(
                    service.cpu_usage_n_sec(),
                    service.memory_current(),
                    service.tasks_current(),
                    service.result()
                )?;
                (cpu, memory, tasks, Some(result))
            } else if name.ends_with(".scope") {
                let scope = ScopeProxy::builder(&self.connection)
                    .destination(self.owner.as_str())
                    .path(path)
                    .cache_properties(CacheProperties::No)
                    .build()
                    .await?;
                let (cpu, memory, tasks) = tokio::try_join!(
                    scope.cpu_usage_n_sec(),
                    scope.memory_current(),
                    scope.tasks_current()
                )?;
                (cpu, memory, tasks, None)
            } else if name.ends_with(".slice") {
                let slice = SliceProxy::builder(&self.connection)
                    .destination(self.owner.as_str())
                    .path(path)
                    .cache_properties(CacheProperties::No)
                    .build()
                    .await?;
                let (cpu, memory, tasks) = tokio::try_join!(
                    slice.cpu_usage_n_sec(),
                    slice.memory_current(),
                    slice.tasks_current()
                )?;
                (cpu, memory, tasks, None)
            } else {
                return Err(HostError::Ownership(
                    "native accounting unit type is unsupported".into(),
                ));
            };
            self.observe_custody_binding(original_manager, name, binding)
                .await?;
            Ok(NativeUnitAccounting {
                cpu_usage_nanoseconds: available(cpu),
                memory_current_bytes: available(memory),
                tasks_current: available(tasks),
                service_result: result,
                atomic_snapshot: false,
            })
        })
        .await
        .map_err(|_| HostError::Deadline("original native unit accounting".into()))?
    }
}
