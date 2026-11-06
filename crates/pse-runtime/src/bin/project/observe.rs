// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One process-root stop source through input, interpretation, output and owned finish.
use pse_host::observer_io::{Reader, Writer};
use pse_runtime::{CancelSource, tooling::reader};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub(super) fn run() -> Result<i32> {
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    executor.block_on(async {
        let stop = CancelSource::new();
        let signal = super::signals::ProcessInterrupt::new(stop.clone())?;
        let result = async {
            let input = Reader::new(std::io::stdin())?;
            let output = Writer::new(std::io::stdout())?;
            transfer(input, output, stop.token()).await
        }
        .await;
        // Stop is a request. The encoder/collector/connection and I/O owners have
        // returned or joined before the root signal listener itself is joined.
        signal.finish().await;
        result
    })
}

async fn transfer(
    input: Reader,
    output: Writer,
    cancel: pse_columnar::CancellationToken,
) -> Result<i32> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 16 << 10];
    while bytes.len() <= reader::REQUEST_BYTES {
        let remaining = (reader::REQUEST_BYTES + 1 - bytes.len()).min(chunk.len());
        let count = cancel
            .until_cancelled(input.read(&mut chunk[..remaining]))
            .await??;
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..count]);
        // Ready local inputs must not starve the process-root signal task.
        tokio::task::yield_now().await;
    }
    cancel.checkpoint()?;
    let worker_stop = cancel.clone();
    // Existing interpretation owns synchronous joins and its local executor.
    // Never abandon this blocking task on cancellation: its token stops query/
    // capture work, then the original task returns after its actual cleanup.
    let encoded =
        tokio::task::spawn_blocking(move || reader::encode(&bytes, &worker_stop)).await??;
    deliver(encoded, &output, &cancel).await
}

async fn deliver(
    encoded: reader::EncodedResponse,
    output: &Writer,
    cancel: &pse_columnar::CancellationToken,
) -> Result<i32> {
    if let Err(error) = cancel.checkpoint() {
        return Err(encoded.delivery_failed(error.into()).into());
    }
    match cancel.until_cancelled(output.write(encoded.bytes())).await {
        Err(error) => Err(encoded.delivery_failed(error.into()).into()),
        Ok(Err(error)) => Err(encoded.delivery_failed(error.into()).into()),
        // The full frame was accepted by the output; a later stop cannot revoke that fact.
        Ok(Ok(())) => Ok(encoded.exit_code()),
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "owned descriptor and process controls assert exact fixture setup and original outcomes"
    )]
    use super::*;
    use std::{
        io::{Read as _, Write as _},
        os::unix::net::UnixStream,
        time::Duration,
    };

    #[tokio::test]
    async fn cancelled_input_has_no_detached_reader_or_changed_original_flags() {
        let (input, mut sender) = UnixStream::pair().unwrap();
        let (output, _receiver) = UnixStream::pair().unwrap();
        let flags = rustix::fs::fcntl_getfl(&input).unwrap();
        let token = pse_columnar::CancellationToken::new();
        let work = transfer(
            Reader::new(&input).unwrap(),
            Writer::new(output).unwrap(),
            token.clone(),
        );
        tokio::pin!(work);
        assert!(futures_util::poll!(work.as_mut()).is_pending());
        token.cancel();
        let error = tokio::time::timeout(Duration::from_secs(5), work)
            .await
            .unwrap()
            .unwrap_err();
        assert!(matches!(
            error.downcast_ref::<pse_columnar::CanonError>(),
            Some(pse_columnar::CanonError::Cancelled)
        ));
        assert_eq!(rustix::fs::fcntl_getfl(&input).unwrap(), flags);
        sender.write_all(b"still owned by original").unwrap();
        sender.shutdown(std::net::Shutdown::Write).unwrap();
        let mut original = input;
        let mut remaining = String::new();
        original.read_to_string(&mut remaining).unwrap();
        assert_eq!(remaining, "still owned by original");
    }

    #[tokio::test]
    async fn ordinary_request_uses_the_same_owned_response_encoder() {
        let (input, mut sender) = UnixStream::pair().unwrap();
        let (output, mut receiver) = UnixStream::pair().unwrap();
        let request = serde_json::to_vec(&reader::ReaderRequest {
            protocol: reader::Protocol::ProductReader,
            version: Default::default(),
            correlation: "original-observer".into(),
            access: reader::Access::Passive,
            request: reader::Request::Discover,
        })
        .unwrap();
        sender.write_all(&request).unwrap();
        sender.shutdown(std::net::Shutdown::Write).unwrap();
        let expected = reader::encode(&request, &pse_columnar::CancellationToken::new()).unwrap();
        assert_eq!(
            transfer(
                Reader::new(input).unwrap(),
                Writer::new(output).unwrap(),
                pse_columnar::CancellationToken::new()
            )
            .await
            .unwrap(),
            expected.exit_code()
        );
        let mut bytes = Vec::new();
        receiver.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, expected.bytes());
    }

    #[tokio::test]
    async fn cancelled_stalled_response_keeps_only_a_provisional_prefix() {
        use std::io::{Seek as _, SeekFrom};
        let mut input = tempfile::tempfile().unwrap();
        let request = serde_json::to_vec(&reader::ReaderRequest {
            protocol: reader::Protocol::ProductReader,
            version: Default::default(),
            correlation: "stalled-schema-response".into(),
            access: reader::Access::Passive,
            request: reader::Request::Schema {
                contract: reader::Contract::Response,
            },
        })
        .unwrap();
        input.write_all(&request).unwrap();
        input.seek(SeekFrom::Start(0)).unwrap();
        let (output, mut receiver) = UnixStream::pair().unwrap();
        rustix::net::sockopt::set_socket_send_buffer_size(&output, 4096).unwrap();
        let expected = reader::encode(&request, &pse_columnar::CancellationToken::new()).unwrap();
        assert!(
            expected.bytes().len()
                > rustix::net::sockopt::socket_send_buffer_size(&output).unwrap()
        );
        let token = pse_columnar::CancellationToken::new();
        let mut worker = tokio::spawn(transfer(
            Reader::new(input).unwrap(),
            Writer::new(output).unwrap(),
            token.clone(),
        ));
        let input = Reader::new(&receiver).unwrap();
        let mut first = [0; 1];
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(10), input.read(&mut first))
                .await
                .unwrap()
                .unwrap(),
            1
        );
        token.cancel();
        let error = tokio::time::timeout(Duration::from_secs(5), &mut worker)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        assert!(matches!(
            error.downcast_ref::<reader::ReaderError>(),
            Some(reader::ReaderError::Columnar(
                pse_columnar::CanonError::Cancelled
            ))
        ));
        drop(input);
        let mut partial = first.to_vec();
        receiver.read_to_end(&mut partial).unwrap();
        assert!(partial.len() < expected.bytes().len());
        assert!(expected.bytes().starts_with(&partial));
        assert!(serde_json::from_slice::<serde_json::Value>(&partial).is_err());
    }

    #[tokio::test]
    async fn original_refusal_survives_late_stop_and_broken_output() {
        for stopped in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let request = reader::ReaderRequest {
                protocol: reader::Protocol::ProductReader,
                version: Default::default(),
                correlation: "original-missing-member".into(),
                access: reader::Access::Passive,
                request: reader::Request::Investigate {
                    request: reader::InvestigationRequest::FrozenInspect {
                        evidence: directory.path().join("missing-original.json"),
                        request: pse_runtime::tooling::frozen_inspection::FrozenInspectionRequest {
                            version: Default::default(),
                            inspection: pse_runtime::tooling::inspection::InspectionRequest::new(
                                pse_diagnostics::observation::OperationId {
                                    process_instance: [19, 3],
                                    sequence: 42,
                                },
                            ),
                            window:
                                pse_runtime::tooling::frozen_inspection::RecordedWindow::Recorded,
                        },
                    },
                },
            };
            let token = pse_columnar::CancellationToken::new();
            let encoded = reader::encode(&serde_json::to_vec(&request).unwrap(), &token).unwrap();
            assert_eq!(encoded.exit_code(), 2);
            let (output, receiver) = UnixStream::pair().unwrap();
            let mut receiver = Some(receiver);
            if stopped {
                token.cancel();
            } else {
                drop(receiver.take());
            }
            let error = deliver(encoded, &Writer::new(output).unwrap(), &token)
                .await
                .unwrap_err();
            let reader::ReaderError::Delivery {
                response,
                transport,
            } = *error.downcast::<reader::ReaderError>().unwrap()
            else {
                panic!("a completed original refusal must survive failed delivery");
            };
            assert!(matches!(*response, reader::ReaderError::Identity(_)));
            if stopped {
                assert!(matches!(
                    *transport,
                    reader::ReaderError::Columnar(pse_columnar::CanonError::Cancelled)
                ));
            } else {
                assert!(
                    matches!(*transport, reader::ReaderError::Io(ref cause) if cause.kind() == std::io::ErrorKind::BrokenPipe)
                );
            }
        }
    }

    #[test]
    fn process_root_signals_cancel_a_stalled_input_without_waiting_for_eof() -> Result<()> {
        const CHILD: &str = "PSE_OBSERVE_INPUT_SIGNAL_CONTROL";
        if std::env::var_os(CHILD).is_some() {
            let error = run().unwrap_err();
            assert!(matches!(
                error.downcast_ref::<pse_columnar::CanonError>(),
                Some(pse_columnar::CanonError::Cancelled)
            ));
            return Ok(());
        }
        for signal in ["-INT", "-TERM"] {
            let mut child = std::process::Command::new(std::env::current_exe()?)
                .args(["--exact", "observe::tests::process_root_signals_cancel_a_stalled_input_without_waiting_for_eof", "--nocapture"])
                .env(CHILD, "1")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .spawn()?;
            let outcome = (|| -> Result<()> {
                let deadline = std::time::Instant::now() + Duration::from_secs(10);
                loop {
                    if let Some(status) = child.try_wait()? {
                        return Err(format!(
                            "observer exited before signal registration: {status}"
                        )
                        .into());
                    }
                    let status = std::fs::read_to_string(format!("/proc/{}/status", child.id()))?;
                    let caught = status
                        .lines()
                        .find_map(|line| line.strip_prefix("SigCgt:\t"))
                        .map(|mask| u64::from_str_radix(mask.trim(), 16))
                        .transpose()?
                        .unwrap_or(0);
                    if caught & ((1 << 1) | (1 << 14)) == ((1 << 1) | (1 << 14)) {
                        break;
                    }
                    if std::time::Instant::now() >= deadline {
                        return Err("observer did not register its signal owners".into());
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                if !std::process::Command::new("kill")
                    .args([signal, &child.id().to_string()])
                    .status()?
                    .success()
                {
                    return Err("observer signal delivery failed".into());
                }
                // Keep the parent's stdin writer open: EOF cannot be the reason it exits.
                while child.try_wait()?.is_none() {
                    if std::time::Instant::now() >= deadline {
                        return Err("observer did not settle after its stop request".into());
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                if !child.wait()?.success() {
                    return Err("observer child did not retain typed cancellation".into());
                }
                Ok(())
            })();
            let _ = child.kill();
            let _ = child.wait();
            outcome?;
        }
        Ok(())
    }
}
