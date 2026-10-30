// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Retire only this authenticated finalizer's existing context claim after its own duties.
//! Original supervisor, launcher, entry and payload I/O obligations remain protected.
use super::*;
use crate::{
    generation::{BoundFinalizerUse, FinalizerClaimLocator, FinalizerUseScope},
    invocation::InvocationStore,
    managed::recovery::RecoveredResources,
};

struct PublicationBinding {
    directory: PathBuf,
    invocation: CliInvocationId,
}
impl PublicationBinding {
    fn new(directory: &InvocationDirectory) -> Self {
        Self {
            directory: directory.path().to_owned(),
            invocation: directory.id().clone(),
        }
    }
    fn matches(&self, directory: &InvocationDirectory) -> bool {
        self.directory == directory.path() && &self.invocation == directory.id()
    }
}
pub(super) struct ClosedLaunches(PublicationBinding);
pub(super) struct RequestCleaned(PublicationBinding);
pub(super) struct FinalizationPublished(PublicationBinding);
pub(super) struct TerminalPublished(PublicationBinding);
pub(super) struct LocalClaim {
    bound: BoundFinalizerUse,
}
pub(super) struct LocalDuties {
    closed: ClosedLaunches,
    request: RequestCleaned,
    finalization: FinalizationPublished,
    terminal: TerminalPublished,
}
#[derive(Default)]
pub(super) struct LocalFinalization {
    pub(super) claim: Option<LocalClaim>,
    pub(super) closed: Option<ClosedLaunches>,
    pub(super) request: Option<RequestCleaned>,
    pub(super) recovered: Option<super::super::recovery::RecoveryCustody>,
}
impl LocalFinalization {
    pub(super) fn locator(&self) -> Option<&FinalizerClaimLocator> {
        self.claim.as_ref().map(LocalClaim::locator)
    }
    pub(super) fn finish(
        self,
        finalization: FinalizationPublished,
        terminal: TerminalPublished,
        context: &supervisor::Context,
        original: Option<&supervisor::ControllerRecord>,
        root: &std::path::Path,
    ) -> Result<(), HostError> {
        match (
            self.claim,
            self.closed,
            self.request,
            self.recovered,
            original,
        ) {
            (Some(claim), Some(closed), Some(request), Some(recovered), Some(original)) => {
                match recovered.into_settled() {
                    Ok(resources) => claim.complete(
                        resources,
                        LocalDuties {
                            closed,
                            request,
                            finalization,
                            terminal,
                        },
                        context,
                        original,
                        root,
                    ),
                    Err(_retained) => Ok(()),
                }
            }
            _ => Ok(()),
        }
    }
}

impl LocalClaim {
    pub(super) async fn bind(
        context: &mut supervisor::Context,
        root: &std::path::Path,
        original: &supervisor::ControllerRecord,
    ) -> Result<Self, HostError> {
        let scope = FinalizerUseScope::new(
            std::fs::canonicalize(root)?,
            context.directory.path().to_owned(),
            context.directory.id().clone(),
            context.manager.identity_key(),
            context.record.controller.clone(),
            original.binding.clone(),
            original.process.clone(),
        )?;
        let usage = context.take_finalizer_use().await?;
        let bound = usage.bind_finalizer(scope)?;
        Ok(Self { bound })
    }
    pub(super) fn locator(&self) -> &FinalizerClaimLocator {
        self.bound.locator()
    }
    fn complete(
        self,
        resources: RecoveredResources,
        duties: LocalDuties,
        context: &supervisor::Context,
        original: &supervisor::ControllerRecord,
        root: &std::path::Path,
    ) -> Result<(), HostError> {
        let directory = &context.directory;
        if !resources.matches(
            directory,
            &context.manager,
            &context.record.controller,
            original,
        ) || self.bound.scope() != &resources.finalizer_scope(root)?
            || !duties.closed.0.matches(directory)
            || !duties.request.0.matches(directory)
            || !duties.finalization.0.matches(directory)
            || !duties.terminal.0.matches(directory)
        {
            return Err(HostError::Ownership(
                "local finalizer claim completion differs from original native/publication scope"
                    .into(),
            ));
        }
        // This non-deserializable completion owns the actual native resource report and
        // successful local I/O receipts. It grants no release over another process's claims.
        LocalCompletion {
            claim: self.bound,
            _resources: resources,
        }
        .retire()
    }
}
struct LocalCompletion {
    claim: BoundFinalizerUse,
    _resources: RecoveredResources,
}
impl LocalCompletion {
    fn retire(self) -> Result<(), HostError> {
        self.claim.settle_current()
    }
}
pub(super) async fn close(
    directory: &InvocationDirectory,
    deadline: Instant,
) -> Result<ClosedLaunches, HostError> {
    loop {
        match directory.try_launch_lock() {
            Ok(guard) => {
                guard.close()?;
                return Ok(ClosedLaunches(PublicationBinding::new(directory)));
            }
            Err(HostError::Io(error)) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err(HostError::Deadline(
                        "original finalizer launch exclusion".into(),
                    ));
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(error) => return Err(error),
        }
    }
}
pub(super) fn discard_request(
    directory: &InvocationDirectory,
) -> Result<(bool, RequestCleaned), HostError> {
    let removed = directory.discard_request()?;
    Ok((removed, RequestCleaned(PublicationBinding::new(directory))))
}
pub(super) fn publish_finalization<T: Serialize>(
    directory: &InvocationDirectory,
    value: &T,
) -> Result<FinalizationPublished, HostError> {
    directory.publish_evidence(InvocationEvidence::Finalization, value)?;
    Ok(FinalizationPublished(PublicationBinding::new(directory)))
}
pub(super) fn observe_terminal(
    store: &InvocationStore,
    id: &CliInvocationId,
) -> Result<Option<TerminalPublished>, HostError> {
    let found = store.result::<Box<serde_json::value::RawValue>, ManagedSummary>(id)?;
    found
        .map(|_| {
            store
                .directory(id)
                .map(|directory| TerminalPublished(PublicationBinding::new(&directory)))
        })
        .transpose()
}
pub(super) fn publish_terminal(
    directory: &InvocationDirectory,
    terminal: &TerminalResult<ManagedSummary>,
) -> Result<TerminalPublished, HostError> {
    directory.publish_result(terminal)?;
    Ok(TerminalPublished(PublicationBinding::new(directory)))
}
#[cfg(test)]
mod tests;
