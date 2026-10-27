// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! I/O-owned process observations around procfs-core's exact source parsers.
use procfs_core::{
    FromRead,
    process::{Stat, Status},
};
use std::{
    io,
    path::{Path, PathBuf},
};

/// A read outcome; missing facts never establish that a different namespace's owner died.
#[derive(Debug)]
pub enum ProcessRead<T> {
    /// Complete facts at this read boundary.
    Present(T),
    /// The selected process path disappeared.
    Absent,
    /// Access, parsing or required identity facts are unavailable.
    Unavailable(String),
    /// Reads disagree about the process identity.
    Changed,
}
/// Raw kernel identity, with no conversion of process ticks to wall time.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProcessIdentity {
    /// Kernel boot identifier.
    pub boot_id: String,
    /// Kernel PID namespace link, e.g. `pid:[4026531836]`.
    pub pid_namespace: String,
    /// PID as seen by this observer.
    pub pid: u32,
    /// Raw stat field22, without clock arithmetic.
    pub start_ticks: u64,
}
/// Stat facts parsed by the exact-release library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessStat {
    /// Kernel PID.
    pub pid: u32,
    /// Raw scheduler state; Z is not proof that every thread exited.
    pub state: char,
    /// Parent PID in this observer's namespace.
    pub parent_pid: u32,
    /// Kernel process group.
    pub process_group: i32,
    /// Raw process start ticks.
    pub start_ticks: u64,
}
/// A coherent stat/boot/namespace observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessFacts {
    /// Stable ownership identity.
    pub identity: ProcessIdentity,
    /// Current stat values; mutable values are not identity.
    pub stat: ProcessStat,
}
/// Selected status facts with byte units resolved once at the reader boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessStatus {
    /// PID from status.
    pub pid: u32,
    /// Parent PID, not a memory quantity.
    pub parent_pid: u32,
    /// Peak resident set in bytes; absent remains unknown.
    pub peak_resident_bytes: Option<u64>,
}
/// Local procfs opening and consistency owner. Supervision uses the default kernel root.
#[derive(Clone, Debug)]
pub struct ProcessReader {
    root: PathBuf,
}
impl Default for ProcessReader {
    fn default() -> Self {
        Self {
            root: PathBuf::from("/proc"),
        }
    }
}
impl ProcessReader {
    /// Select an explicit procfs image for observation or deterministic fixtures.
    /// An arbitrary image supplies no permission to signal kernel processes.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
    /// Root for a complete numeric process inventory.
    pub fn root(&self) -> &Path {
        &self.root
    }
    /// Verify that numeric procfs paths use this caller's PID view, then read
    /// the actual observer namespace through self rather than a numeric guess.
    pub fn observer_namespace(&self) -> ProcessRead<String> {
        let read = || -> io::Result<String> {
            let observer = std::fs::read_link(self.root.join("self"))?;
            let observer = observer
                .to_str()
                .and_then(|pid| pid.parse::<u32>().ok())
                .ok_or_else(|| io::Error::other("procfs observer PID unavailable"))?;
            if observer != std::process::id() {
                return Err(io::Error::other(
                    "procfs PID view differs from observer namespace",
                ));
            }
            // Status NSpid is ordered from the procfs mount's PID namespace
            // through nested inner namespaces (proc_pid_status(5), NStgid/NSpid;
            // Linux fs/proc/base.c proc_single_show passes the mount PID namespace
            // to array.c task_state).
            // A multi-level list is unsupported even if outer/inner numbers
            // coincide. Read self/status, not a guessed numeric process path.
            let bytes = std::fs::read(self.root.join("self/status"))?;
            let status = Status::from_read(bytes.as_slice()).map_err(io::Error::other)?;
            if u32::try_from(status.pid).ok() != Some(observer)
                || u32::try_from(status.tgid).ok() != Some(observer)
            {
                return Err(io::Error::other("procfs observer status identity changed"));
            }
            observer_pid_view(status.nspid.as_deref(), observer)?;
            std::fs::read_link(self.root.join("self/ns/pid"))?
                .into_os_string()
                .into_string()
                .map_err(|_| io::Error::other("observer PID namespace unavailable"))
        };
        match read() {
            Ok(namespace) => ProcessRead::Present(namespace),
            Err(error) => ProcessRead::Unavailable(error.to_string()),
        }
    }
    /// Verify the actual spawning thread inherits this PID namespace into its
    /// children. A process leader's namespace-for-children need not be this thread's.
    pub(crate) fn spawning_namespace(&self, expected: &str) -> ProcessRead<()> {
        let read = || -> io::Result<()> {
            // pid_namespaces(7): setns/unshare changes only future children's
            // PID namespace; the caller's own membership cannot change.
            for _ in 0..2 {
                for namespace in ["pid", "pid_for_children"] {
                    let actual =
                        std::fs::read_link(self.root.join("thread-self/ns").join(namespace))?;
                    if actual.to_str() != Some(expected) {
                        return Err(io::Error::other(
                            "spawning thread PID namespace or namespace-for-children differs from observer",
                        ));
                    }
                }
            }
            Ok(())
        };
        match read() {
            Ok(()) => ProcessRead::Present(()),
            Err(error) => ProcessRead::Unavailable(error.to_string()),
        }
    }
    /// Read stat alone, allowing inventory to filter foreign groups before namespace access.
    pub fn stat(&self, pid: u32) -> ProcessRead<ProcessStat> {
        match std::fs::read(self.root.join(pid.to_string()).join("stat")) {
            Ok(bytes) => match parse_stat(&bytes) {
                Ok(stat) if stat.pid == pid => ProcessRead::Present(stat),
                Ok(_) => ProcessRead::Changed,
                Err(error) => ProcessRead::Unavailable(error.to_string()),
            },
            Err(error) => read_error(error),
        }
    }
    /// Read required identity around stat a second time. Namespace/boot failures remain unknown.
    pub fn process(&self, pid: u32) -> ProcessRead<ProcessFacts> {
        let before = match self.stat(pid) {
            ProcessRead::Present(stat) => stat,
            ProcessRead::Absent => return ProcessRead::Absent,
            ProcessRead::Unavailable(error) => return ProcessRead::Unavailable(error),
            ProcessRead::Changed => return ProcessRead::Changed,
        };
        let identity = || -> io::Result<(String, String)> {
            let boot = std::fs::read_to_string(self.root.join("sys/kernel/random/boot_id"))?;
            let namespace = std::fs::read_link(self.root.join(pid.to_string()).join("ns/pid"))?;
            let namespace = namespace
                .into_os_string()
                .into_string()
                .map_err(|_| io::Error::other("non-UTF8 PID namespace"))?;
            if boot.trim().is_empty() || namespace.is_empty() {
                return Err(io::Error::other("empty process identity"));
            }
            Ok((boot.trim().to_owned(), namespace))
        };
        let first = match identity() {
            Ok(value) => value,
            Err(error) => return ProcessRead::Unavailable(error.to_string()),
        };
        let after = match self.stat(pid) {
            ProcessRead::Present(stat) => stat,
            ProcessRead::Absent => return ProcessRead::Changed,
            ProcessRead::Unavailable(error) => return ProcessRead::Unavailable(error),
            ProcessRead::Changed => return ProcessRead::Changed,
        };
        let second = match identity() {
            Ok(value) => value,
            Err(error) => return ProcessRead::Unavailable(error.to_string()),
        };
        if before.start_ticks != after.start_ticks || first != second {
            return ProcessRead::Changed;
        }
        ProcessRead::Present(ProcessFacts {
            identity: ProcessIdentity {
                boot_id: first.0,
                pid_namespace: first.1,
                pid,
                start_ticks: after.start_ticks,
            },
            stat: after,
        })
    }
    /// Read status with checked KiB-to-byte conversion; no inferred zero peak.
    pub fn status(&self, pid: u32) -> ProcessRead<ProcessStatus> {
        match std::fs::read(self.root.join(pid.to_string()).join("status")) {
            Ok(bytes) => match parse_status(&bytes) {
                Ok(status) if status.pid == pid => ProcessRead::Present(status),
                Ok(_) => ProcessRead::Changed,
                Err(error) => ProcessRead::Unavailable(error.to_string()),
            },
            Err(error) => read_error(error),
        }
    }
}
fn observer_pid_view(hierarchy: Option<&[i32]>, observer: u32) -> io::Result<()> {
    let observer = i32::try_from(observer).map_err(io::Error::other)?;
    if hierarchy != Some(std::slice::from_ref(&observer)) {
        return Err(io::Error::other(
            "procfs PID namespace hierarchy is missing, changed or unsupported",
        ));
    }
    Ok(())
}
fn read_error<T>(error: io::Error) -> ProcessRead<T> {
    if error.kind() == io::ErrorKind::NotFound {
        ProcessRead::Absent
    } else {
        ProcessRead::Unavailable(error.to_string())
    }
}
/// Parse stat using procfs-core, guarding its unchecked delimiter slice boundary.
pub fn parse_stat(bytes: &[u8]) -> io::Result<ProcessStat> {
    // Match the parser's lossy UTF8/trim normalization before guarding slices.
    let text = String::from_utf8_lossy(bytes);
    let bytes = text.trim().as_bytes();
    let start = bytes
        .iter()
        .position(|&b| b == b'(')
        .ok_or_else(|| io::Error::other("stat has no comm"))?;
    let end = bytes
        .iter()
        .rposition(|&b| b == b')')
        .ok_or_else(|| io::Error::other("stat has no comm end"))?;
    if start < 2
        || end <= start
        || bytes.get(start - 1) != Some(&b' ')
        || bytes.get(end + 1) != Some(&b' ')
        || bytes.get(end + 2).is_none()
    {
        return Err(io::Error::other("malformed stat delimiters"));
    }
    let stat = Stat::from_read(bytes).map_err(io::Error::other)?;
    Ok(ProcessStat {
        pid: u32::try_from(stat.pid).map_err(io::Error::other)?,
        state: stat.state,
        parent_pid: u32::try_from(stat.ppid).map_err(io::Error::other)?,
        process_group: stat.pgrp,
        start_ticks: stat.starttime,
    })
}
/// procfs-core0.18.0 Status::parse_with_kb strips kB and retains KiB numbers.
pub fn parse_status(bytes: &[u8]) -> io::Result<ProcessStatus> {
    let status = Status::from_read(bytes).map_err(io::Error::other)?;
    Ok(ProcessStatus {
        pid: u32::try_from(status.pid).map_err(io::Error::other)?,
        parent_pid: u32::try_from(status.ppid).map_err(io::Error::other)?,
        peak_resident_bytes: status
            .vmhwm
            .map(|kib| {
                kib.checked_mul(1024)
                    .ok_or_else(|| io::Error::other("VmHWM byte extent"))
            })
            .transpose()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn stat(ticks: u64) -> String {
        // Fields3..37. Field22 is independently placed at index19 after state.
        let mut fields = vec!["0".to_owned(); 35];
        fields[0] = "S".into();
        fields[1] = "7".into();
        fields[2] = "456".into();
        fields[19] = ticks.to_string();
        format!("123 (worker ) ( name) {}", fields.join(" "))
    }
    #[test]
    fn process_facts_stat_comm_max_ticks_and_malformed_boundaries() {
        let facts = parse_stat(stat(u64::MAX).as_bytes()).unwrap();
        assert_eq!(
            facts,
            ProcessStat {
                pid: 123,
                state: 'S',
                parent_pid: 7,
                process_group: 456,
                start_ticks: u64::MAX
            }
        );
        for bad in [
            "",
            "(",
            "()",
            "123 (x)",
            "123 (x) S 1 2",
            "123 x) S 1 2",
            "123 (x)   ",
            "123 (x)\u{2003}",
        ] {
            assert!(parse_stat(bad.as_bytes()).is_err(), "{bad}");
        }
    }
    #[test]
    fn process_facts_status_kibibytes_are_converted_once_and_overflow_refuses() {
        let status = "Name: fixture\nState: S\nTgid: 123\nPid: 123\nPPid: 7\nTracerPid: 0\nUid: 1 1 1 1\nGid: 1 1 1 1\nFDSize: 64\nGroups: 1\nThreads: 1\nSigQ: 0/1\nSigPnd: 0\nShdPnd: 0\nSigBlk: 0\nSigIgn: 0\nSigCgt: 0\nCapInh: 0\nCapPrm: 0\nCapEff: 0\nVmHWM: 123 kB\n";
        let facts = parse_status(status.as_bytes()).unwrap();
        assert_eq!(facts.parent_pid, 7);
        assert_eq!(facts.peak_resident_bytes, Some(125952));
        assert!(
            parse_status(
                status
                    .replace("123 kB", &format!("{} kB", u64::MAX))
                    .as_bytes()
            )
            .is_err()
        );
    }
    #[test]
    fn process_facts_live_identity_and_missing_are_explicit() {
        let reader = ProcessReader::default();
        assert!(matches!(
            reader.process(std::process::id()),
            ProcessRead::Present(_)
        ));
        let namespace = match reader.observer_namespace() {
            ProcessRead::Present(namespace) => namespace,
            other => panic!("{other:?}"),
        };
        assert!(matches!(
            reader.spawning_namespace(&namespace),
            ProcessRead::Present(())
        ));
        assert!(matches!(reader.process(u32::MAX), ProcessRead::Absent));
        let reader = ProcessReader::new(PathBuf::from("/dev/null"));
        assert!(matches!(reader.process(1), ProcessRead::Unavailable(_)));
    }
    #[test]
    fn process_facts_missing_namespace_denied_and_changed_are_not_absence() {
        let directory = std::env::temp_dir().join(format!(
            "pse-process-facts-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        struct Fixture(PathBuf);
        impl Drop for Fixture {
            fn drop(&mut self) {
                std::fs::remove_dir_all(&self.0).unwrap();
            }
        }
        let _fixture = Fixture(directory.clone());
        std::fs::create_dir(directory.join("123")).unwrap();
        std::fs::write(
            directory.join("123/stat"),
            stat(42).replacen("123 (", "456 (", 1),
        )
        .unwrap();
        let reader = ProcessReader::new(directory.clone());
        assert!(matches!(reader.stat(123), ProcessRead::Changed));
        std::fs::write(directory.join("123/stat"), stat(42)).unwrap();
        std::fs::create_dir_all(directory.join("sys/kernel/random")).unwrap();
        std::fs::write(directory.join("sys/kernel/random/boot_id"), b"fixture-boot").unwrap();
        assert!(matches!(reader.process(123), ProcessRead::Unavailable(_)));
        std::os::unix::fs::symlink("123", directory.join("self")).unwrap();
        assert!(matches!(
            reader.observer_namespace(),
            ProcessRead::Unavailable(_)
        ));
        assert!(matches!(
            read_error::<ProcessStat>(io::Error::from(io::ErrorKind::PermissionDenied)),
            ProcessRead::Unavailable(_)
        ));
    }

    #[test]
    fn process_facts_observer_view_rejects_coincident_nested_ids_and_missing_hierarchy() {
        assert!(observer_pid_view(Some(&[123]), 123).is_ok());
        for hierarchy in [
            None,
            Some(&[][..]),
            Some(&[122][..]),
            Some(&[123, 123][..]),
            Some(&[42, 123][..]),
        ] {
            assert!(observer_pid_view(hierarchy, 123).is_err());
        }
    }

    #[test]
    fn process_facts_spawning_thread_namespace_for_children_is_independent_of_leader() {
        use std::os::unix::fs::symlink;
        let directory = std::env::temp_dir().join(format!(
            "pse-spawning-namespace-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(directory.join("thread-self/ns")).unwrap();
        std::fs::create_dir_all(directory.join("self/ns")).unwrap();
        struct Fixture(PathBuf);
        impl Drop for Fixture {
            fn drop(&mut self) {
                std::fs::remove_dir_all(&self.0).unwrap();
            }
        }
        let _fixture = Fixture(directory.clone());
        for path in [
            "thread-self/ns/pid",
            "self/ns/pid",
            "self/ns/pid_for_children",
        ] {
            symlink("pid:[expected]", directory.join(path)).unwrap();
        }
        let reader = ProcessReader::new(directory.clone());
        assert!(matches!(
            reader.spawning_namespace("pid:[expected]"),
            ProcessRead::Unavailable(_)
        ));
        let children = directory.join("thread-self/ns/pid_for_children");
        symlink("pid:[different]", &children).unwrap();
        assert!(matches!(
            reader.spawning_namespace("pid:[expected]"),
            ProcessRead::Unavailable(_)
        ));
        std::fs::remove_file(&children).unwrap();
        symlink("pid:[expected]", &children).unwrap();
        assert!(matches!(
            reader.spawning_namespace("pid:[expected]"),
            ProcessRead::Present(())
        ));
        assert!(matches!(
            reader.spawning_namespace("pid:[different]"),
            ProcessRead::Unavailable(_)
        ));
    }
}
