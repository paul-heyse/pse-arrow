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
    pub(crate) async fn native_accounting(
        &self,
        intent: &LaunchIntent,
        binding: &UnitBinding,
    ) -> Result<NativeUnitAccounting, HostError> {
        tokio::time::timeout(Duration::from_secs(5), async {
            self.validate_binding(intent, binding).await?;
            let unit = self.unit(intent.unit()).await?;
            let path = unit.inner().path().to_owned();
            let (cpu, memory, tasks, result) = if intent.unit().ends_with(".service") {
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
            } else if intent.unit().ends_with(".scope") {
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
            } else if intent.unit().ends_with(".slice") {
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
            self.validate_binding(intent, binding).await?;
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
