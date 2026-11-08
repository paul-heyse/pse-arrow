// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::{Cancellation, Child, Instant, OwnedFd, SupervisionError, check, io, wait_poll};
use rustix::{
    event::{PollFd, PollFlags},
    fs::{OFlags, fcntl_getfl, fcntl_setfl},
    io::Errno,
};

/// Independent observations for a finite child's stdout and stderr.
#[derive(Debug)]
pub struct CapturedOutput {
    /// Standard output.
    pub stdout: CapturedStream,
    /// Standard error.
    pub stderr: CapturedStream,
}

/// A prepaid prefix and the completion facts for one original pipe.
#[derive(Debug)]
pub struct CapturedStream {
    /// Retained bytes, limited to the requested extent.
    pub prefix: Vec<u8>,
    /// Bytes were observed beyond the prefix.
    pub overflow: bool,
    /// The pipe reached EOF.
    pub complete: bool,
    limit: usize,
}

impl CapturedStream {
    fn reserve(limit: usize) -> Result<Self, SupervisionError> {
        let mut prefix = Vec::new();
        prefix.try_reserve_exact(limit).map_err(io::Error::other)?;
        Ok(Self {
            prefix,
            overflow: false,
            complete: false,
            limit,
        })
    }

    fn read(&mut self, pipe: &mut Option<OwnedFd>) -> Result<(), SupervisionError> {
        let Some(fd) = pipe else {
            return Ok(());
        };
        let mut chunk = [0_u8; 8192];
        match rustix::io::read(fd, &mut chunk) {
            Ok(0) => {
                self.complete = true;
                *pipe = None;
            }
            Ok(size) => {
                let retained = size.min(self.limit - self.prefix.len());
                self.prefix.extend_from_slice(&chunk[..retained]);
                self.overflow |= retained < size;
            }
            Err(Errno::AGAIN | Errno::INTR) => {}
            Err(error) => return Err(io::Error::from(error).into()),
        }
        Ok(())
    }
}

impl CapturedOutput {
    pub(super) fn reserve(stdout: usize, stderr: usize) -> Result<Self, SupervisionError> {
        Ok(Self {
            stdout: CapturedStream::reserve(stdout)?,
            stderr: CapturedStream::reserve(stderr)?,
        })
    }
}

#[derive(Debug)]
pub(super) struct Capture {
    stdout: Option<OwnedFd>,
    stderr: Option<OwnedFd>,
    pub(super) output: CapturedOutput,
}

impl Capture {
    #[cfg(test)]
    pub(super) fn replace_stdout_for_test(&mut self, pipe: OwnedFd) {
        self.stdout = Some(pipe);
        self.output.stdout.complete = false;
    }

    pub(super) fn new(child: &mut Child, output: CapturedOutput) -> Result<Self, SupervisionError> {
        fn nonblocking(pipe: Option<impl Into<OwnedFd>>) -> Result<OwnedFd, SupervisionError> {
            let fd = pipe
                .ok_or_else(|| SupervisionError::Reader("bounded pipe absent".into()))?
                .into();
            let flags = fcntl_getfl(&fd).map_err(io::Error::from)?;
            fcntl_setfl(&fd, flags | OFlags::NONBLOCK).map_err(io::Error::from)?;
            Ok(fd)
        }
        Ok(Self {
            stdout: Some(nonblocking(child.stdout.take())?),
            stderr: Some(nonblocking(child.stderr.take())?),
            output,
        })
    }

    pub(super) fn wait(
        &mut self,
        leader: Option<&OwnedFd>,
        control: Option<&Cancellation>,
        deadline: Instant,
    ) -> Result<(), SupervisionError> {
        if leader.is_none() && self.stdout.is_none() && self.stderr.is_none() {
            return Ok(());
        }
        wait_poll(control, Some(deadline), |signal, timeout| {
            let fds = [leader, self.stdout.as_ref(), self.stderr.as_ref(), signal];
            let mut descriptors: Vec<_> = fds
                .iter()
                .flatten()
                .map(|fd| PollFd::new(*fd, PollFlags::IN))
                .collect();
            rustix::event::poll(&mut descriptors, timeout).map_err(io::Error::from)?;
            check(control, Some(deadline))?;
            let mut events = descriptors.iter().map(PollFd::revents);
            let [leader_ready, stdout_ready, stderr_ready, _] = fds.map(|fd| {
                fd.and_then(|_| events.next())
                    .unwrap_or_else(PollFlags::empty)
            });
            if (leader_ready | stdout_ready | stderr_ready)
                .intersects(PollFlags::ERR | PollFlags::NVAL)
            {
                return Err(SupervisionError::Reader(
                    "capture descriptor unavailable".into(),
                ));
            }
            let ready = PollFlags::IN | PollFlags::HUP;
            // Bound each drain step so noisy streams cannot starve one another or the deadline.
            if stdout_ready.intersects(ready) {
                self.output.stdout.read(&mut self.stdout)?;
            }
            if stderr_ready.intersects(ready) {
                self.output.stderr.read(&mut self.stderr)?;
            }
            Ok(leader_ready.intersects(ready)
                || leader.is_none() && self.stdout.is_none() && self.stderr.is_none())
        })
    }
}
