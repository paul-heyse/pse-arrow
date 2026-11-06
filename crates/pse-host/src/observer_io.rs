// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Observer-owned descriptors; cancellation leaves no blocking pipe worker behind.
#![allow(
    unsafe_code,
    reason = "Tokio1.53.3 registration requires an explicit stable owned-descriptor invariant"
)]
use std::{
    fs::OpenOptions,
    io,
    os::{
        fd::{AsFd, AsRawFd, OwnedFd},
        unix::fs::OpenOptionsExt as _,
    },
};
use tokio::{io::Interest, io::unix::AsyncFd};

/// A single output descriptor whose pending write can be dropped without a worker.
#[derive(Debug)]
pub struct Writer(Destination);
/// A single input descriptor whose pending read can be dropped without a worker.
#[derive(Debug)]
pub struct Reader(Destination);
#[derive(Debug)]
enum Destination {
    Pollable {
        descriptor: AsyncFd<OwnedFd>,
        socket: bool,
    },
    Direct(OwnedFd),
}
impl Writer {
    /// Duplicate regular files to preserve their actual shared offset and append mode.
    /// Reopen pipes/terminals independently, so O_NONBLOCK never changes an ancestor's fd.
    /// Socket writes use a per-call nonblocking flag on the duplicated original socket.
    /// # Errors
    /// Descriptor duplication, independent reopen, or native readiness registration failure.
    pub fn new(source: impl AsFd) -> io::Result<Self> {
        Ok(Self(destination(source, true)?))
    }
    /// Write all supplied bytes. A dropped future may have written a prefix; it never
    /// promises an atomic frame, durable file, publication, or rollback.
    /// # Errors
    /// The original native write/readiness error, including zero progress.
    pub async fn write(&self, mut bytes: &[u8]) -> io::Result<()> {
        while !bytes.is_empty() {
            let chunk = &bytes[..bytes.len().min(64 << 10)];
            let written = match &self.0 {
                Destination::Direct(descriptor) => {
                    rustix::io::write(descriptor, chunk).map_err(io::Error::from)
                }
                Destination::Pollable { descriptor, socket } => {
                    let mut ready = descriptor.writable().await?;
                    match ready.try_io(|registered| {
                        if *socket {
                            rustix::net::send(
                                registered.get_ref(),
                                chunk,
                                rustix::net::SendFlags::DONTWAIT | rustix::net::SendFlags::NOSIGNAL,
                            )
                            .map_err(io::Error::from)
                        } else {
                            rustix::io::write(registered.get_ref(), chunk).map_err(io::Error::from)
                        }
                    }) {
                        Ok(result) => result,
                        Err(_) => continue,
                    }
                }
            };
            match written {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "observer output accepted no bytes",
                    ));
                }
                Ok(count) => bytes = &bytes[count..],
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
            tokio::task::yield_now().await;
        }
        Ok(())
    }
}
impl Reader {
    /// Duplicate regular files preserving their shared offset, independently reopen pipes/
    /// terminals, and use per-call nonblocking socket reads. Original flags stay unchanged.
    /// # Errors
    /// Descriptor duplication, independent reopen, or native readiness registration failure.
    pub fn new(source: impl AsFd) -> io::Result<Self> {
        Ok(Self(destination(source, false)?))
    }
    /// Read the next prefix, or zero at EOF. Dropping a pending read releases only this
    /// wait; no detached read task can later consume the caller's bytes.
    /// # Errors
    /// The original native read/readiness error.
    pub async fn read(&self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        loop {
            let result = match &self.0 {
                Destination::Direct(descriptor) => {
                    rustix::io::read(descriptor, &mut *bytes).map_err(io::Error::from)
                }
                Destination::Pollable { descriptor, socket } => {
                    let mut ready = descriptor.readable().await?;
                    match ready.try_io(|registered| {
                        if *socket {
                            rustix::net::recv(
                                registered.get_ref(),
                                &mut *bytes,
                                rustix::net::RecvFlags::DONTWAIT,
                            )
                            .map(|(count, _)| count)
                            .map_err(io::Error::from)
                        } else {
                            rustix::io::read(registered.get_ref(), &mut *bytes)
                                .map_err(io::Error::from)
                        }
                    }) {
                        Ok(result) => result,
                        Err(_) => continue,
                    }
                }
            };
            match result {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                other => return other,
            }
        }
    }
}
fn destination(source: impl AsFd, writable: bool) -> io::Result<Destination> {
    use rustix::fs::{FileType, OFlags};
    let original = rustix::io::dup(source)?;
    let kind = FileType::from_raw_mode(rustix::fs::fstat(&original)?.st_mode);
    if kind == FileType::RegularFile {
        // Regular files have no epoll readiness. Writes are local kernel operations,
        // chunked below; there is no detached blocking task to abandon on cancellation.
        return Ok(Destination::Direct(original));
    }
    let socket = kind == FileType::Socket;
    let descriptor = if socket {
        original
    } else {
        let reopened = OpenOptions::new()
            .read(!writable)
            .write(writable)
            .custom_flags((OFlags::NONBLOCK | OFlags::CLOEXEC | OFlags::NOCTTY).bits() as i32)
            .open(format!("/proc/self/fd/{}", original.as_raw_fd()))?;
        reopened.into()
    };
    // SAFETY: OwnedFd keeps this one native descriptor open and unchanged for the
    // whole registration lifetime. No mutable reference or raw-fd replacement escapes.
    match unsafe {
        AsyncFd::register_with_interest(
            descriptor,
            if writable {
                Interest::WRITABLE
            } else {
                Interest::READABLE
            },
        )
    } {
        Ok(descriptor) => Ok(Destination::Pollable { descriptor, socket }),
        Err(error) => {
            let (descriptor, cause) = error.into_parts();
            if kind == FileType::CharacterDevice
                && cause.raw_os_error() == Some(rustix::io::Errno::PERM.raw_os_error())
            {
                // /dev/null and /dev/full reject epoll. Their independently reopened
                // descriptor is already nonblocking; unsupported WouldBlock stays an error.
                Ok(Destination::Direct(descriptor))
            } else {
                Err(cause)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read as _, Seek as _, Write as _},
        time::Duration,
    };
    #[tokio::test]
    async fn regular_input_preserves_original_shared_offset_and_eof() {
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(b"prefix-selected").unwrap();
        file.seek(io::SeekFrom::Start(7)).unwrap();
        let input = Reader::new(&file).unwrap();
        let mut bytes = [0; 8];
        assert_eq!(input.read(&mut bytes).await.unwrap(), 8);
        assert_eq!(&bytes, b"selected");
        assert_eq!(file.stream_position().unwrap(), 15);
        assert_eq!(input.read(&mut bytes).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn cancelled_socket_input_does_not_consume_later_bytes_or_change_flags() {
        let (source, mut sender) = std::os::unix::net::UnixStream::pair().unwrap();
        let flags = rustix::fs::fcntl_getfl(&source).unwrap();
        let input = Reader::new(&source).unwrap();
        let mut bytes = [0; 8];
        assert!(
            tokio::time::timeout(Duration::from_millis(25), input.read(&mut bytes))
                .await
                .is_err()
        );
        assert_eq!(rustix::fs::fcntl_getfl(&source).unwrap(), flags);
        drop(input);
        sender.write_all(b"original").unwrap();
        let mut source = source;
        source.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"original");
    }
    #[tokio::test]
    async fn regular_file_keeps_original_offset_and_append_mode() {
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(b"prefix-tail").unwrap();
        file.seek(io::SeekFrom::Start(7)).unwrap();
        Writer::new(&file).unwrap().write(b"view").await.unwrap();
        assert_eq!(file.stream_position().unwrap(), 11);
        file.seek(io::SeekFrom::Start(0)).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"prefix-view");
        rustix::fs::fcntl_setfl(&file, rustix::fs::OFlags::APPEND).unwrap();
        file.seek(io::SeekFrom::Start(0)).unwrap();
        Writer::new(&file).unwrap().write(b"-append").await.unwrap();
        assert_eq!(file.stream_position().unwrap(), 18);
    }
    #[tokio::test]
    async fn stalled_socket_can_be_cancelled_without_changing_original_flags_or_leaving_a_writer() {
        let (source, mut consumer) = std::os::unix::net::UnixStream::pair().unwrap();
        let flags = rustix::fs::fcntl_getfl(&source).unwrap();
        let writer = Writer::new(&source).unwrap();
        let bytes = vec![b'x'; 4 << 20];
        assert!(
            tokio::time::timeout(Duration::from_millis(25), writer.write(&bytes))
                .await
                .is_err()
        );
        assert_eq!(rustix::fs::fcntl_getfl(&source).unwrap(), flags);
        drop(writer);
        drop(source);
        let mut observed = Vec::new();
        consumer.read_to_end(&mut observed).unwrap();
        assert!(!observed.is_empty() && observed.len() < bytes.len());
        assert!(observed.iter().all(|byte| *byte == b'x'));
    }
}
