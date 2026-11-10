// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One current protected source selection for mathematical acquisition/publication.
use super::*;
use crate::math::portable::ValidatedReplayNamespace;
use pse_operations::canonical_selection::SelectedRead;
use std::{
    future::Future,
    time::{Duration, Instant},
};

pub(super) struct ProtectedPreparation<'a> {
    deployment: &'a crate::workflow::CanonicalDeployment,
    read: SelectedRead,
    namespace: Option<ValidatedReplayNamespace>,
}
impl<'a> ProtectedPreparation<'a> {
    pub(super) async fn open(
        package: &ModelingPackage,
        source: Option<&'a crate::workflow::SourceServices>,
        math: &Arc<crate::math::MathService>,
        versions: &BTreeMap<String, String>,
        driver: &crate::CancelSource,
        deadline: Instant,
    ) -> Result<Option<Self>, WorkflowError> {
        let Some((deployment, origin)) = source
            .and_then(|source| source.canonical().ok())
            .zip(package.canonical_revision())
        else {
            return Ok(None);
        };
        canonical::checkpoint(driver)?;
        let protection = deployment
            .store()
            .protect(origin.clone(), Duration::from_secs(600))
            .await?;
        let mut current = Self {
            deployment,
            read: SelectedRead::new(protection),
            namespace: None,
        };
        let control = pse_diagnostics::Cancellation::default();
        let result: Result<(), WorkflowError> = run(driver, &control, async {
            canonical::checkpoint(driver)?;
            let logicals = versions.keys().cloned().collect::<Vec<_>>();
            let resolved = deployment
                .store()
                .resolve_logicals(&mut current.read, &logicals)
                .await?;
            canonical::checkpoint(driver)?;
            if resolved
                .iter()
                .map(|member| (member.logical.clone(), member.version.clone()))
                .collect::<BTreeMap<_, _>>()
                != *versions
            {
                return Err(contract(
                    "mathematical source origin differs from admitted selection",
                ));
            }
            current.namespace = crate::math::portable::qualify_replay(
                math,
                deployment.producer().cloned(),
                deadline,
                driver,
            )
            .await?;
            canonical::checkpoint(driver)?;
            Ok(())
        })
        .await;
        if let Err(error) = result {
            return settle(Err(error), current.close().await);
        }
        Ok(Some(current))
    }
    pub(super) fn receiving(
        &mut self,
    ) -> Option<(
        &pse_operations::canonical::CanonicalStore,
        &mut SelectedRead,
        &ValidatedReplayNamespace,
    )> {
        Some((
            self.deployment.store(),
            &mut self.read,
            self.namespace.as_ref()?,
        ))
    }
    pub(super) fn publication(
        &self,
    ) -> Option<(
        &pse_operations::canonical::CanonicalStore,
        &SelectedRead,
        &ValidatedReplayNamespace,
    )> {
        Some((
            self.deployment.store(),
            &self.read,
            self.namespace.as_ref()?,
        ))
    }
    async fn close(self) -> Result<(), WorkflowError> {
        Ok(self.read.finish().await?)
    }
}

/// Stop the native leaf on the original cancellation/deadline, then drain work
/// before releasing its selected read or retained reconstruction/publication owners.
pub(super) async fn run<T>(
    driver: &crate::CancelSource,
    control: &pse_diagnostics::Cancellation,
    operation: impl Future<Output = Result<T, WorkflowError>>,
) -> Result<T, WorkflowError> {
    canonical::checkpoint(driver)?;
    let (result, _) = pse_diagnostics::stop::drain_on_stop(
        pse_diagnostics::stop::StopSignal::stopped(driver),
        operation,
        |_| control.cancel(),
    )
    .await;
    match result {
        Ok(value) => {
            canonical::checkpoint(driver)?;
            Ok(value)
        }
        Err(error) => Err(error),
    }
}
pub(super) async fn finish<T>(
    current: Option<ProtectedPreparation<'_>>,
    operation: Result<T, WorkflowError>,
) -> Result<T, WorkflowError> {
    let cleanup = if let Some(current) = current {
        current.close().await
    } else {
        Ok(())
    };
    settle(operation, cleanup)
}
fn settle<T>(
    operation: Result<T, WorkflowError>,
    cleanup: Result<(), WorkflowError>,
) -> Result<T, WorkflowError> {
    match (operation, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(operation), Err(cleanup)) => Err(WorkflowError::Settlement {
            operation: Box::new(operation),
            cleanup: Box::new(cleanup),
        }),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
    }
}
