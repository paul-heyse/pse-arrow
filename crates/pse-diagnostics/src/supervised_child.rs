// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Synchronous ownership of a freshly spawned process group, never setsid containment.
use crate::{Cancellation, process_facts::{ProcessFacts, ProcessIdentity, ProcessRead, ProcessReader}};
use std::{io::{self,Read}, os::{fd::{AsFd,OwnedFd},unix::process::CommandExt}, process::{Child,Command,ExitStatus,Stdio}, sync::Arc, time::{Duration,Instant}};

/// Unknown drain never silently releases compiler ownership.
#[derive(Clone, Copy, Debug)]
pub enum DrainFailurePolicy {
    /// Retain the caller stack, capacity, directory and readers indefinitely on unknown drain.
    Park,
    /// Refuse after this finite drain allowance and retry once on owner drop.
    Refuse(Duration),
}
/// A bounded prefix reader; the caller prepays its stack and buffer before spawn.
#[derive(Clone, Copy, Debug)]
pub struct StderrCapture {
    /// Retained prefix; every other byte is drained and discarded.
    pub bytes: usize,
    /// Explicit prepaid reader stack.
    pub stack_bytes: usize,
}
/// Supervision refusal, distinct from the scientific child's exit status.
#[derive(Debug, thiserror::Error)]
pub enum SupervisionError {
    /// Caller requested stop; ownership still requires drain.
    #[error("supervised process cancellation requested")]
    Cancelled,
    /// Original absolute deadline expired; cancellation is unchanged.
    #[error("supervised process deadline expired")]
    Deadline,
    /// Kernel or local procfs access failed.
    #[error("supervised process I/O: {0}")]
    Io(#[from] io::Error),
    /// Missing, changed or foreign identity prevents permission to act.
    #[error("supervised process identity unavailable: {0}")]
    Identity(String),
    /// No complete group-drain proof within the diagnostic allowance.
    #[error("owned process group did not drain; leader remains unreaped")]
    Drain,
    /// The owned bounded output reader failed.
    #[error("supervised stderr reader: {0}")]
    Reader(String),
}

crate::impl_diagnostic! {
    SupervisionError,
    code(this) { Some(match this { Self::Cancelled=>crate::DiagnosticCode::RuntimeCancelled, Self::Deadline=>crate::DiagnosticCode::RuntimeResourceLimit, _=>crate::DiagnosticCode::RuntimeInfrastructure }) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}

/// A pidfd-verified leader remains unreaped until every observed owned member exits.
/// Group membership is not proof about descendants which escape with setsid.
#[derive(Debug)]
pub struct SupervisedChild {
    child: Child,
    process_fd: Option<OwnedFd>,
    identity: Option<ProcessIdentity>,
    finished: Option<ExitStatus>,
    reader: Option<std::thread::JoinHandle<io::Result<Vec<u8>>>>,
    stderr: Option<Vec<u8>>,
    policy: DrainFailurePolicy,
    facts: ProcessReader,
}
impl SupervisedChild {
    /// Spawn a new group. Optional stderr capture is Park-owned and already prepaid.
    pub fn spawn(command:&mut Command, policy:DrainFailurePolicy, stderr:Option<StderrCapture>) -> Result<Self,SupervisionError> {
        if let DrainFailurePolicy::Refuse(allowance)=policy {
            if allowance.is_zero() || Instant::now().checked_add(allowance).is_none() {
                return Err(SupervisionError::Drain);
            }
        }
        if let Some(capture)=stderr {
            if capture.bytes==0 || capture.stack_bytes==0 || !matches!(policy,DrainFailurePolicy::Park) {
                return Err(SupervisionError::Reader("bounded stderr requires prepaid nonzero Park ownership".into()));
            }
            command.stderr(Stdio::piped());
        }
        let facts=ProcessReader::default();
        let observer_namespace=present(facts.observer_namespace())?;
        let current=present(facts.process(std::process::id()))?;
        if current.identity.pid_namespace!=observer_namespace {
            return Err(SupervisionError::Identity("observer namespace changed before spawn".into()));
        }
        present(facts.spawning_namespace(&observer_namespace))?;
        command.process_group(0);
        let child=command.spawn()?;
        let mut owner=Self {child,process_fd:None,identity:None,finished:None,reader:None,stderr:None,policy,facts};
        // Child ownership reserves this freshly spawned leader even if identity
        // acquisition fails. Drop uses the same explicit drain failure policy.
        owner.admit(&current.identity)?;
        if let Some(capture)=stderr {
            let mut pipe=owner.child.stderr.take().ok_or_else(|| SupervisionError::Reader("stderr pipe absent".into()))?;
            owner.reader=Some(std::thread::Builder::new().name("pse-child-stderr".into()).stack_size(capture.stack_bytes).spawn(move || {
                let mut prefix=Vec::with_capacity(capture.bytes);
                let mut chunk=[0_u8;1024];
                loop {
                    let size=match pipe.read(&mut chunk) { Err(error) if error.kind()==io::ErrorKind::Interrupted=>continue, result=>result? };
                    if size==0 { return Ok(prefix); }
                    let keep=size.min(capture.bytes-prefix.len());
                    prefix.extend_from_slice(&chunk[..keep]);
                }
            })?);
        }
        Ok(owner)
    }
    fn admit(&mut self, inherited:&ProcessIdentity)->Result<(),SupervisionError> {
        let before=present(self.facts.stat(self.pid()))?;
        let fd=rustix::process::pidfd_open(pid(self.pid())?,rustix::process::PidfdFlags::empty()).map_err(io::Error::from)?;
        self.process_fd=Some(fd);
        let fd=self.process_fd.as_ref().ok_or_else(||SupervisionError::Identity("leader pidfd absent".into()))?;
        // pidfd_open(2): exclusive unreaped Child custody also covers a child
        // which exited before open. P_PIDFD rejects ECHILD rather than assuming
        // a numeric PID still denotes our child. WNOWAIT retains the PGID anchor.
        rustix::process::waitid(rustix::process::WaitId::PidFd(fd.as_fd()),rustix::process::WaitIdOptions::EXITED|rustix::process::WaitIdOptions::NOWAIT|rustix::process::WaitIdOptions::NOHANG).map_err(io::Error::from)?;
        let after=present(self.facts.stat(self.pid()))?;
        if before.start_ticks!=after.start_ticks || before.process_group!=after.process_group || after.process_group!=pid(self.pid())?.as_raw_pid() {
            return Err(SupervisionError::Identity("spawned leader start ticks or process group changed across pidfd open".into()));
        }
        let identity=ProcessIdentity {boot_id:inherited.boot_id.clone(),pid_namespace:inherited.pid_namespace.clone(),pid:self.pid(),start_ticks:after.start_ticks};
        // Do not require live procfs namespace metadata once this descriptor
        // establishes whole-thread-group exit. Inherited facts were checked
        // for the actual spawning thread before fork, and a
        // process's active PID namespace cannot change (pid_namespaces(7)).
        // Readiness proves this process exited, never that its group drained.
        if !exited(fd)? {
            let live=(|| {
                let facts=present(self.facts.process(self.pid()))?;
                verify(&facts,&facts,&identity)?;
                if facts.stat.start_ticks!=after.start_ticks || facts.stat.process_group!=after.process_group {
                    return Err(SupervisionError::Identity("spawned leader changed after pidfd open".into()));
                }
                Ok(())
            })();
            if let Err(error)=live {if !exited(fd)? {return Err(error);}}
        }
        self.identity=Some(identity);
        Ok(())
    }
    /// Actual freshly spawned PID.
    pub fn pid(&self)->u32 {self.child.id()}
    /// Post-pidfd-open verified identity; present for every successful spawn.
    pub fn identity(&self)->Option<&ProcessIdentity> {self.identity.as_ref()}
    /// Retained completion only after whole-group drain and leader reap.
    pub fn retained_status(&self)->Option<ExitStatus> {self.finished}
    /// Transfer the launch exec gate to its caller.
    pub fn take_stdin(&mut self)->Option<std::process::ChildStdin> {self.child.stdin.take()}
    /// Transfer stdout ownership to the caller's existing reader.
    pub fn take_stdout(&mut self)->Option<std::process::ChildStdout> {self.child.stdout.take()}
    /// Transfer stderr when the optional shared reader was not selected.
    pub fn take_stderr(&mut self)->Option<std::process::ChildStderr> {self.child.stderr.take()}
    /// Take the bounded prefix after drain joined its reader.
    pub fn take_stderr_prefix(&mut self)->Option<Vec<u8>> {self.stderr.take()}
    /// Wait on the held pidfd and cancellation eventfd using the caller's original clock.
    /// Readiness never reaps or establishes group drain.
    pub fn wait_ready(&self, control:Option<&Cancellation>, deadline:Option<Instant>)->Result<(),SupervisionError> {
        if self.finished.is_some() {return Ok(());}
        let fd=self.process_fd.as_ref().ok_or_else(|| SupervisionError::Identity("leader pidfd absent".into()))?;
        wait_exit(fd,control,deadline)?;
        rustix::process::waitid(rustix::process::WaitId::Pid(pid(self.pid())?),rustix::process::WaitIdOptions::EXITED|rustix::process::WaitIdOptions::NOWAIT).map_err(io::Error::from)?;
        check(control,deadline)
    }
    /// Observe completion without reaping the PGID reservation early.
    pub fn try_wait(&mut self)->Result<Option<ExitStatus>,SupervisionError> {
        if let Some(status)=self.finished {return Ok(Some(status));}
        let result=rustix::process::waitid(rustix::process::WaitId::Pid(pid(self.pid())?),rustix::process::WaitIdOptions::EXITED|rustix::process::WaitIdOptions::NOWAIT|rustix::process::WaitIdOptions::NOHANG).map_err(io::Error::from)?;
        if result.is_none() {return Ok(None);}
        self.drain().map(Some)
    }
    /// Wait for the leader, then independently drain every owned member.
    pub fn wait(&mut self)->Result<ExitStatus,SupervisionError> {
        if let Some(status)=self.finished {return Ok(status);}
        self.wait_ready(None,None)?;
        self.drain()
    }
    /// Kill the owned group and await complete group drain before reaping its leader.
    pub fn cancel(&mut self)->Result<ExitStatus,SupervisionError> {self.drain()}
    /// Post-open verified live member descriptors, a snapshot rather than a drain proof.
    pub fn members(&self)->Result<Vec<OwnedFd>,SupervisionError> {self.members_until(None)}
    fn members_until(&self,deadline:Option<Instant>)->Result<Vec<OwnedFd>,SupervisionError> {
        if self.finished.is_some() {return Err(SupervisionError::Identity("leader already reaped".into()));}
        rustix::process::waitid(rustix::process::WaitId::Pid(pid(self.pid())?),rustix::process::WaitIdOptions::EXITED|rustix::process::WaitIdOptions::NOWAIT|rustix::process::WaitIdOptions::NOHANG).map_err(io::Error::from)?;
        let expected=self.identity.as_ref().ok_or_else(|| SupervisionError::Identity("leader identity absent".into()))?;
        let mut members=Vec::new();
        for entry in std::fs::read_dir(self.facts.root())? {
            if deadline.is_some_and(|deadline|Instant::now()>=deadline) {return Err(SupervisionError::Drain);}
            let entry=entry?;
            let Some(member)=entry.file_name().to_str().and_then(|name|name.parse::<u32>().ok()) else {continue};
            if member==self.pid() {continue;}
            let stat=match self.facts.stat(member) {
                ProcessRead::Present(stat)=>stat,
                ProcessRead::Absent=>continue,
                other=>return Err(observation_error(other)),
            };
            if stat.process_group!=pid(self.pid())?.as_raw_pid() {continue;}
            let fd=match rustix::process::pidfd_open(pid(member)?,rustix::process::PidfdFlags::empty()) {
                Ok(fd)=>fd,Err(rustix::io::Errno::SRCH)=>continue,Err(error)=>return Err(io::Error::from(error).into()),
            };
            // Do not require live procfs namespace metadata after this exact
            // descriptor establishes whole-thread-group exit. Only that proof
            // permits skipping it, without ever signalling it.
            if exited(&fd)? {continue;}
            let after=match self.facts.process(member) {
                ProcessRead::Present(facts)=>facts,
                other=> {if exited(&fd)? {continue;} return Err(observation_error(other));}
            };
            if let Err(error)=verify(&after,&after,expected) {if exited(&fd)? {continue;} return Err(error);}
            if after.stat.start_ticks!=stat.start_ticks || after.stat.process_group!=stat.process_group {
                if exited(&fd)? {continue;}
                return Err(SupervisionError::Identity("member start ticks or process group changed across descriptor open".into()));
            }
            // A zombie TGID can still have running threads. Only pidfd readiness
            // establishes whole-thread-group exit, never the stat state alone.
            if !exited(&fd)? {members.push(fd);}
        }
        Ok(members)
    }
    /// Drain under the selected failure policy. Cancellation/deadline do not release issued work.
    pub fn drain(&mut self)->Result<ExitStatus,SupervisionError> {
        if let Some(status)=self.finished {return Ok(status);}
        let deadline=match self.policy {DrainFailurePolicy::Park=>None,DrainFailurePolicy::Refuse(allowance)=>Some(Instant::now().checked_add(allowance).ok_or(SupervisionError::Drain)?)};
        match self.drain_inner(deadline) {
            Ok(status)=>Ok(status),
            Err(error) if matches!(self.policy,DrainFailurePolicy::Park) && self.finished.is_none()=>park_unknown(error),
            Err(error)=>Err(error),
        }
    }
    fn drain_inner(&mut self,deadline:Option<Instant>)->Result<ExitStatus,SupervisionError> {
        loop {
            if deadline.is_some_and(|deadline|Instant::now()>=deadline) {return Err(SupervisionError::Drain);}
            // ECHILD does not reserve a numeric PGID. Refuse before any group
            // signal if external reaping violated exclusive leader custody.
            rustix::process::waitid(rustix::process::WaitId::Pid(pid(self.pid())?),rustix::process::WaitIdOptions::EXITED|rustix::process::WaitIdOptions::NOWAIT|rustix::process::WaitIdOptions::NOHANG).map_err(io::Error::from)?;
            if let Some(fd)=&self.process_fd {
                match rustix::process::pidfd_send_signal(fd,rustix::process::Signal::KILL) {
                    Ok(())|Err(rustix::io::Errno::SRCH)=>{},Err(error)=>return Err(io::Error::from(error).into()),
                }
            }
            match rustix::process::kill_process_group(pid(self.pid())?,rustix::process::Signal::KILL) {
                Ok(())|Err(rustix::io::Errno::SRCH)=>{},Err(error)=>return Err(io::Error::from(error).into()),
            }
            let members=self.members_until(deadline)?;
            if members.is_empty() {break;}
            for member in &members {
                match rustix::process::pidfd_send_signal(member,rustix::process::Signal::KILL) {
                    Ok(())|Err(rustix::io::Errno::SRCH)=>{},Err(error)=>return Err(io::Error::from(error).into()),
                }
            }
            for member in &members {wait_exit(member,None,deadline).map_err(|error|if matches!(error,SupervisionError::Deadline){SupervisionError::Drain}else{error})?;}
            // A member may have forked before SIGKILL. Reinventory after exact exits.
        }
        // Even an empty member list says nothing about the still-live leader.
        if let Some(fd)=&self.process_fd {wait_exit(fd,None,deadline)?;}
        let status=self.child.wait()?;
        self.finished=Some(status);
        if let Some(reader)=self.reader.take() {
            self.stderr=Some(reader.join().map_err(|_|SupervisionError::Reader("reader panicked".into()))??);
        }
        Ok(status)
    }
}
impl Drop for SupervisedChild {
    fn drop(&mut self) {if self.finished.is_none() {let _=self.drain();}}
}
fn pid(pid:u32)->Result<rustix::process::Pid,SupervisionError> {
    i32::try_from(pid).ok().and_then(rustix::process::Pid::from_raw).ok_or_else(||SupervisionError::Identity("PID extent".into()))
}
fn present<T>(read:ProcessRead<T>)->Result<T,SupervisionError> {
    match read {ProcessRead::Present(facts)=>Ok(facts),other=>Err(observation_error(other))}
}
fn observation_error<T>(read:ProcessRead<T>)->SupervisionError {
    SupervisionError::Identity(match read {ProcessRead::Absent=>"process absent".into(),ProcessRead::Unavailable(reason)=>reason,ProcessRead::Changed=>"process changed".into(),ProcessRead::Present(_)=>"unexpected process observation".into()})
}
fn verify(before:&ProcessFacts,after:&ProcessFacts,owner:&ProcessIdentity)->Result<(),SupervisionError> {
    if before.identity!=after.identity || after.identity.boot_id!=owner.boot_id || after.identity.pid_namespace!=owner.pid_namespace {
        return Err(SupervisionError::Identity("boot, namespace or start ticks changed across pidfd open".into()));
    }
    Ok(())
}
fn check(control:Option<&Cancellation>,deadline:Option<Instant>)->Result<(),SupervisionError> {
    if control.is_some_and(Cancellation::is_cancelled) {Err(SupervisionError::Cancelled)}
    else if deadline.is_some_and(|deadline|Instant::now()>=deadline) {Err(SupervisionError::Deadline)} else {Ok(())}
}
fn exited(fd:&OwnedFd)->Result<bool,SupervisionError> {
    use rustix::event::{PollFd,PollFlags,Timespec};
    let mut descriptors=[PollFd::new(fd,PollFlags::IN)];
    loop {
        match rustix::event::poll(&mut descriptors,Some(&Timespec{tv_sec:0,tv_nsec:0})) {
            Err(rustix::io::Errno::INTR)=>continue,Err(error)=>return Err(io::Error::from(error).into()),Ok(_)=>break,
        }
    }
    let ready=descriptors[0].revents();
    if ready.intersects(PollFlags::NVAL|PollFlags::ERR) {return Err(SupervisionError::Identity("pidfd exit readiness unavailable".into()));}
    Ok(ready.intersects(PollFlags::IN|PollFlags::HUP))
}
fn wait_exit(fd:&OwnedFd,control:Option<&Cancellation>,deadline:Option<Instant>)->Result<(),SupervisionError> {
    use rustix::{event::{EventfdFlags,PollFd,PollFlags,Timespec},io::Errno};
    #[derive(Debug)]
    struct Signal(OwnedFd);
    impl std::task::Wake for Signal {
        fn wake(self:Arc<Self>) {self.wake_by_ref();}
        fn wake_by_ref(self:&Arc<Self>) {while let Err(Errno::INTR)=rustix::io::write(&self.0,&1_u64.to_ne_bytes()) {}}
    }
    let signal=control.map(|_|rustix::event::eventfd(0,EventfdFlags::CLOEXEC|EventfdFlags::NONBLOCK).map(|fd|Arc::new(Signal(fd))).map_err(io::Error::from)).transpose()?;
    let _watch=control.zip(signal.as_ref()).map(|(control,signal)|control.watch(&std::task::Waker::from(signal.clone())));
    loop {
        check(control,deadline)?;
        let timeout=deadline.map(|deadline|Timespec::try_from(deadline.saturating_duration_since(Instant::now()))).transpose().map_err(io::Error::other)?;
        let mut descriptors=vec![PollFd::new(fd,PollFlags::IN)];
        if let Some(signal)=&signal {descriptors.push(PollFd::new(&signal.0,PollFlags::IN));}
        match rustix::event::poll(&mut descriptors,timeout.as_ref()) {
            Err(Errno::INTR)=>continue,Err(error)=>return Err(io::Error::from(error).into()),Ok(_)=>{},
        }
        check(control,deadline)?;
        let ready=descriptors[0].revents();
        if ready.intersects(PollFlags::NVAL|PollFlags::ERR) {return Err(SupervisionError::Identity("pidfd exit notification unavailable".into()));}
        if ready.intersects(PollFlags::IN|PollFlags::HUP) {return Ok(());}
    }
}
fn park_unknown(error:SupervisionError)->! {
    if let Some(context)=crate::observation::current() {
        context.text("process.supervision_drain_fault",&error.to_string());
        context.text("process.supervision_state","blocked-unproved-drain");
    }
    loop {std::thread::park();}
}

#[cfg(test)]
mod tests;
