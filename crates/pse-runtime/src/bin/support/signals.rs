// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Process-root interruption requests stop without taking ownership of native drain.

pub(super) struct ProcessInterrupt {
    listener: tokio::task::JoinHandle<()>,
    cancellation: pse_runtime::CancelSource,
}

impl ProcessInterrupt {
    /// Register both signals before admitting work, rather than on a spawned task's first poll.
    pub(super) fn new(cancellation: pse_runtime::CancelSource) -> std::io::Result<Self> {
        use tokio::signal::unix::{SignalKind, signal};
        let mut interrupt = signal(SignalKind::interrupt())?;
        let mut terminate = signal(SignalKind::terminate())?;
        let stopped = cancellation.clone();
        let listener = tokio::spawn(async move {
            tokio::select! {
                _ = interrupt.recv() => {},
                _ = terminate.recv() => {},
            }
            stopped.cancel();
        });
        Ok(Self { listener, cancellation })
    }

    /// Join the signal task after the original work has drained.
    pub(super) async fn finish(mut self) {
        self.listener.abort();
        let _ = (&mut self.listener).await;
    }
}

impl Drop for ProcessInterrupt {
    fn drop(&mut self) {
        // Dropping a root future requests stop; its native supervisors still own actual joins.
        self.cancellation.cancel();
        self.listener.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn signal_owner_drop_stops_only_its_original_scope() -> std::io::Result<()> {
        let original = pse_runtime::CancelSource::new();
        let sibling = pse_runtime::CancelSource::new();
        let owner = ProcessInterrupt::new(original.clone())?;
        let listener = owner.listener.abort_handle();
        drop(owner);
        assert!(original.token().is_cancelled());
        assert!(!sibling.token().is_cancelled());
        // Abort is requested synchronously; wait for actual listener-future destruction.
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while !listener.is_finished() { tokio::task::yield_now().await; }
        }).await?;
        assert!(listener.is_finished());
        Ok(())
    }

    #[tokio::test]
    async fn signal_owner_finish_joins_listener() -> std::io::Result<()> {
        let stop = pse_runtime::CancelSource::new();
        let owner = ProcessInterrupt::new(stop)?;
        let listener = owner.listener.abort_handle();
        owner.finish().await;
        assert!(listener.is_finished());
        Ok(())
    }

    /// Run in a child test process so process signals never reach a neighboring test.
    #[test]
    fn process_signals_request_stop_and_keep_drain_owned() -> Result<(), Box<dyn std::error::Error>> {
        const CHILD: &str = "PSE_SIGNAL_DRAIN_CONTROL";
        if let Some(directory) = std::env::var_os(CHILD) {
            let directory = std::path::PathBuf::from(directory);
            let executor = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
            executor.block_on(async {
                let stop = pse_runtime::CancelSource::new();
                let owner = ProcessInterrupt::new(stop.clone())?;
                std::fs::write(directory.join("ready"), [])?;
                stop.cancelled().await;
                std::fs::write(directory.join("stopping"), [])?;
                while !directory.join("release").exists() {
                    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                }
                owner.finish().await;
                Ok::<_, std::io::Error>(())
            })?;
            return Ok(());
        }
        for signal in ["-INT", "-TERM"] {
            let directory = tempfile::tempdir()?;
            let mut child = std::process::Command::new(std::env::current_exe()?)
                .args(["--exact", "signals::tests::process_signals_request_stop_and_keep_drain_owned", "--nocapture"])
                .env(CHILD, directory.path())
                .spawn()?;
            let outcome = (|| {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
                for stage in ["ready", "stopping"] {
                    while !directory.path().join(stage).exists() {
                        if let Some(status) = child.try_wait()? {
                            return Err(format!("signal control exited before {stage}: {status}").into());
                        }
                        if std::time::Instant::now() >= deadline {
                            return Err(format!("signal control did not reach {stage}").into());
                        }
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                    if stage == "ready" {
                        if !std::process::Command::new("kill").args([signal, &child.id().to_string()]).status()?.success() {
                            return Err("signal delivery failed".into());
                        }
                    }
                }
                if child.try_wait()?.is_some() {
                    return Err("stop request cannot stand for drained work".into());
                }
                std::fs::write(directory.path().join("release"), [])?;
                while child.try_wait()?.is_none() {
                    if std::time::Instant::now() >= deadline {
                        return Err("signal control did not finish its drain".into());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                if !child.wait()?.success() {
                    return Err("signal control failed after drain".into());
                }
                Ok::<_, Box<dyn std::error::Error>>(())
            })();
            // Always settle the owned fixture, including a failed assertion/control.
            let _ = child.kill();
            let _ = child.wait();
            outcome?;
        }
        Ok(())
    }
}
