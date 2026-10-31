// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded search frames through the existing cancellable native writer.
use std::io::{self, Write};
use tokio::sync::mpsc;

/// Closing the receiver releases producer backpressure before actual CPU settlement.
pub(super) struct Frames(pub mpsc::Sender<Vec<u8>>);
impl Write for Frames {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0
            .blocking_send(bytes.to_vec())
            .map_err(|_| io::Error::from(io::ErrorKind::BrokenPipe))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(super) async fn stdout(mut frames: mpsc::Receiver<Vec<u8>>) -> io::Result<()> {
    let output = crate::output_writer::Writer::new(io::stdout())?;
    while let Some(frame) = frames.recv().await {
        output.write(&frame).await?;
    }
    Ok(())
}
