// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::io::{BufRead,BufReader};
fn spawn(script:&str)->SupervisedChild {
    let mut command=Command::new("/bin/sh");
    command.args(["-c",script]).stdout(Stdio::piped());
    SupervisedChild::spawn(&mut command,DrainFailurePolicy::Refuse(Duration::from_secs(3)),None).unwrap()
}
struct ProcImage(std::path::PathBuf);
impl ProcImage {
    fn new(processes:&[(u32,bool)])->Self {
        use std::os::unix::fs::symlink;
        let directory=std::env::temp_dir().join(format!("pse-supervision-exited-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(directory.join("sys/kernel/random")).unwrap();
        symlink("/proc/sys/kernel/random/boot_id",directory.join("sys/kernel/random/boot_id")).unwrap();
        for &(process,namespace) in processes {
            std::fs::create_dir(directory.join(process.to_string())).unwrap();
            symlink(format!("/proc/{process}/stat"),directory.join(format!("{process}/stat"))).unwrap();
            if namespace {symlink(format!("/proc/{process}/ns"),directory.join(format!("{process}/ns"))).unwrap();}
        }
        Self(directory)
    }
    fn reader(&self)->ProcessReader {ProcessReader::new(self.0.clone())}
}
impl Drop for ProcImage {fn drop(&mut self){std::fs::remove_dir_all(&self.0).unwrap();}}
#[test]
fn supervised_child_post_open_identity_refuses_changed_and_foreign_namespace() {
    let facts=present(ProcessReader::default().process(std::process::id())).unwrap();
    for kind in 0..3 {
        let mut changed=facts.clone();
        match kind {0=>changed.identity.start_ticks+=1,1=>changed.identity.pid_namespace.push_str("foreign"),_=>changed.identity.boot_id.push_str("foreign")}
        assert!(matches!(verify(&facts,&changed,&facts.identity),Err(SupervisionError::Identity(_))));
    }
    assert!(matches!(present::<ProcessFacts>(ProcessRead::Changed),Err(SupervisionError::Identity(_))));
    let mut foreign=facts.clone();foreign.identity.pid_namespace.push_str("foreign");
    assert!(verify(&foreign,&foreign,&facts.identity).is_err());
}
#[test]
fn supervised_child_unreaped_leader_reserves_group_until_members_exit() {
    let mut child=spawn("sleep 60 & printf '%s\\n' \"$!\"; exit 7");
    let mut line=String::new();
    BufReader::new(child.take_stdout().unwrap()).read_line(&mut line).unwrap();
    child.wait_ready(None,Some(Instant::now()+Duration::from_secs(3))).unwrap();
    // WNOWAIT must leave the leader visible and waitable during group drain.
    assert!(matches!(ProcessReader::default().stat(child.pid()),ProcessRead::Present(_)));
    let members=child.members().unwrap();
    assert_eq!(members.len(),1);
    assert!(!exited(&members[0]).unwrap());
    let mut foreign=spawn("sleep 60");
    assert_eq!(child.wait().unwrap().code(),Some(7));
    assert!(matches!(child.members(),Err(SupervisionError::Identity(_))));
    assert!(exited(&members[0]).unwrap());
    assert!(foreign.try_wait().unwrap().is_none());
    foreign.cancel().unwrap();
}
#[test]
fn supervised_child_cancellation_and_original_deadline_wake_without_early_release() {
    for cancel in [false,true] {
        let mut child=spawn("sleep 60 & wait");
        let control=Cancellation::default();
        let worker=if cancel {let control=control.clone();Some(std::thread::spawn(move||control.cancel()))}else{None};
        let result=child.wait_ready(Some(&control),if cancel {None}else{Some(Instant::now()+Duration::from_millis(10))});
        if cancel {assert!(matches!(result,Err(SupervisionError::Cancelled)));}else{assert!(matches!(result,Err(SupervisionError::Deadline)));assert!(!control.is_cancelled());}
        assert!(child.retained_status().is_none());
        child.cancel().unwrap();
        if let Some(worker)=worker {worker.join().unwrap();}
    }
}
#[test]
fn supervised_child_bounded_stderr_drains_noise_and_joins_before_return() {
    let mut command=Command::new("/bin/sh");
    command.args(["-c","i=0; while [ \"$i\" -lt 1024 ]; do printf '0123456789abcdef' >&2; i=$((i+1)); done"]);
    let mut child=SupervisedChild::spawn(&mut command,DrainFailurePolicy::Park,Some(StderrCapture{bytes:17,stack_bytes:64<<10})).unwrap();
    child.wait_ready(None,Some(Instant::now()+Duration::from_secs(3))).unwrap();
    assert!(child.drain().unwrap().success());
    assert_eq!(child.take_stderr_prefix().unwrap(),b"0123456789abcdef0");
    assert!(child.reader.is_none());
}
#[test]
fn supervised_child_refuse_preserves_leader_and_drop_retries_drain() {
    let mut child=spawn("sleep 60 & wait");
    let leader=child.pid();
    child.facts=ProcessReader::new("/dev/null".into());
    assert!(matches!(child.cancel(),Err(SupervisionError::Io(_))));
    assert!(child.retained_status().is_none());
    // Repair only the injected observation fault. Drop must retry and reap.
    child.facts=ProcessReader::default();
    drop(child);
    assert_eq!(rustix::process::waitid(rustix::process::WaitId::Pid(pid(leader).unwrap()),rustix::process::WaitIdOptions::EXITED|rustix::process::WaitIdOptions::NOWAIT|rustix::process::WaitIdOptions::NOHANG).unwrap_err(),rustix::io::Errno::CHILD);
}
#[test]
fn supervised_child_forking_group_cancel_reinventories_without_touching_foreign_child() {
    let mut child=spawn("i=0; while [ \"$i\" -lt 32 ]; do sleep 60 & printf '%s\\n' \"$!\"; i=$((i+1)); done; wait");
    let mut output=BufReader::new(child.take_stdout().unwrap());
    let mut first=String::new();output.read_line(&mut first).unwrap();
    let member=rustix::process::pidfd_open(pid(first.trim().parse().unwrap()).unwrap(),rustix::process::PidfdFlags::empty()).unwrap();
    let mut foreign=spawn("sleep 60");
    child.cancel().unwrap();
    assert!(exited(&member).unwrap());
    let mut remaining=String::new();output.read_to_string(&mut remaining).unwrap();
    assert!(foreign.try_wait().unwrap().is_none());
    foreign.cancel().unwrap();
}

// This subprocess entry is inert in the normal unit process. The parent control
// below selects it explicitly with a private fixture directory and owns its exit.
#[test]
fn supervised_child_park_fixture() {
    let Some(directory)=std::env::var_os("PSE_SUPERVISION_PARK_FIXTURE") else {return};
    let directory=std::path::PathBuf::from(directory);
    struct Custody(std::path::PathBuf);
    impl Drop for Custody {fn drop(&mut self){std::fs::write(self.0.join("released"),b"released").unwrap();}}
    let _custody=Custody(directory.clone());
    let mut command=Command::new("/bin/sleep");command.arg("60");
    let mut child=SupervisedChild::spawn(&mut command,DrainFailurePolicy::Park,None).unwrap();
    std::fs::write(directory.join("ready"),child.pid().to_string()).unwrap();
    child.facts=ProcessReader::new("/dev/null".into());
    // The initial group kill happens, but inaccessible inventory cannot prove
    // drain. Park must preserve the caller's custody guard and unreaped child.
    let _=child.cancel();
    panic!("Park returned after unknown drain");
}
#[test]
fn supervised_child_park_retains_custody_after_unknown_drain() {
    let directory=std::env::temp_dir().join(format!("pse-supervision-park-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir(&directory).unwrap();
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {fn drop(&mut self){std::fs::remove_dir_all(&self.0).unwrap();}}
    let _fixture=Fixture(directory.clone());
    let mut command=Command::new(std::env::current_exe().unwrap());
    command.args(["--exact","supervised_child::tests::supervised_child_park_fixture","--nocapture"])
        .env("PSE_SUPERVISION_PARK_FIXTURE",&directory).stdout(Stdio::null()).stderr(Stdio::null());
    let mut helper=SupervisedChild::spawn(&mut command,DrainFailurePolicy::Refuse(Duration::from_secs(3)),None).unwrap();
    let deadline=Instant::now()+Duration::from_secs(3);
    let member=loop {
        if let Ok(value)=std::fs::read_to_string(directory.join("ready")) {break value.parse::<u32>().unwrap();}
        assert!(Instant::now()<deadline,"Park fixture did not establish readiness");
        std::thread::sleep(Duration::from_millis(1));
    };
    let fd=rustix::process::pidfd_open(pid(member).unwrap(),rustix::process::PidfdFlags::empty()).unwrap();
    wait_exit(&fd,None,Some(deadline)).unwrap();
    assert!(matches!(helper.wait_ready(None,Some(Instant::now()+Duration::from_millis(10))),Err(SupervisionError::Deadline)));
    assert!(!directory.join("released").exists(),"unknown drain cannot release caller custody");
    helper.cancel().unwrap();
}

#[test]
fn supervised_child_member_namespace_unknown_refuses_without_releasing_owner() {
    use std::os::unix::fs::symlink;
    let mut child=spawn("sleep 60 & printf '%s\n' \"$!\"; wait");
    let mut line=String::new();BufReader::new(child.take_stdout().unwrap()).read_line(&mut line).unwrap();
    let member=line.trim().parse::<u32>().unwrap();
    let directory=std::env::temp_dir().join(format!("pse-supervision-namespace-{}-{}",std::process::id(),member));
    std::fs::create_dir(&directory).unwrap();
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {fn drop(&mut self){std::fs::remove_dir_all(&self.0).unwrap();}}
    let _fixture=Fixture(directory.clone());
    std::fs::create_dir_all(directory.join(format!("{member}/ns"))).unwrap();
    std::fs::create_dir_all(directory.join("sys/kernel/random")).unwrap();
    symlink(format!("/proc/{member}/stat"),directory.join(format!("{member}/stat"))).unwrap();
    symlink("pid:[unknown]",directory.join(format!("{member}/ns/pid"))).unwrap();
    symlink("/proc/sys/kernel/random/boot_id",directory.join("sys/kernel/random/boot_id")).unwrap();
    child.facts=ProcessReader::new(directory.clone());
    assert!(matches!(child.members(),Err(SupervisionError::Identity(_))));
    std::fs::remove_file(directory.join(format!("{member}/ns/pid"))).unwrap();
    assert!(matches!(child.members(),Err(SupervisionError::Identity(_))),"missing namespace is not proof that a live member exited");
    assert!(child.retained_status().is_none());
    child.facts=ProcessReader::default();
    child.cancel().unwrap();
}

#[test]
fn supervised_child_known_drain_reader_error_returns_instead_of_parking() {
    let mut command=Command::new("/bin/true");
    let mut child=SupervisedChild::spawn(&mut command,DrainFailurePolicy::Park,None).unwrap();
    child.reader=Some(std::thread::spawn(||Err(io::Error::other("injected completed reader fault"))));
    assert!(matches!(child.wait(),Err(SupervisionError::Io(_))));
    assert!(child.retained_status().unwrap().success());
}

#[test]
fn supervised_child_exited_before_admission_retains_leader_until_descendant_drain() {
    for policy in [DrainFailurePolicy::Park,DrainFailurePolicy::Refuse(Duration::from_secs(3))] {
        let facts=ProcessReader::default();
        let namespace=present(facts.observer_namespace()).unwrap();
        present(facts.spawning_namespace(&namespace)).unwrap();
        let parent=present(facts.process(std::process::id())).unwrap();
        assert_eq!(parent.identity.pid_namespace,namespace);
        let mut command=Command::new("/bin/sh");
        command.args(["-c","sleep 60 & printf '%s\\n' \"$!\"; exit 23"]).process_group(0).stdout(Stdio::piped());
        let mut child=command.spawn().unwrap();
        let leader=child.id();
        // Deliberately prevent parent admission until the leader has exited.
        // This is a kernel WNOWAIT barrier, independent of scheduler timing.
        rustix::process::waitid(rustix::process::WaitId::Pid(pid(leader).unwrap()),rustix::process::WaitIdOptions::EXITED|rustix::process::WaitIdOptions::NOWAIT).unwrap().unwrap();
        let mut line=String::new();BufReader::new(child.stdout.take().unwrap()).read_line(&mut line).unwrap();
        let member_pid=line.trim().parse().unwrap();
        let member=rustix::process::pidfd_open(pid(member_pid).unwrap(),rustix::process::PidfdFlags::empty()).unwrap();
        // Some kernels retain zombie namespace links; remove that metadata in a
        // deterministic reader image while keeping real kernel stat/pidfds.
        let image=ProcImage::new(&[(leader,false),(member_pid,true)]);
        let mut owner=SupervisedChild {child,process_fd:None,identity:None,finished:None,reader:None,stderr:None,policy:DrainFailurePolicy::Refuse(Duration::from_secs(3)),facts:image.reader()};
        assert!(matches!(owner.facts.process(leader),ProcessRead::Unavailable(_)),"fixture must inject missing namespace metadata for an exited leader");
        owner.admit(&parent.identity).unwrap();
        owner.policy=policy;
        assert!(exited(owner.process_fd.as_ref().unwrap()).unwrap());
        assert!(!exited(&member).unwrap());
        assert_eq!(owner.members().unwrap().len(),1);
        assert!(owner.retained_status().is_none());
        assert!(rustix::process::waitid(rustix::process::WaitId::PidFd(owner.process_fd.as_ref().unwrap().as_fd()),rustix::process::WaitIdOptions::EXITED|rustix::process::WaitIdOptions::NOWAIT|rustix::process::WaitIdOptions::NOHANG).unwrap().is_some());
        assert_eq!(owner.wait().unwrap().code(),Some(23));
        assert!(exited(&member).unwrap());
        assert_eq!(rustix::process::waitid(rustix::process::WaitId::Pid(pid(leader).unwrap()),rustix::process::WaitIdOptions::EXITED|rustix::process::WaitIdOptions::NOHANG).unwrap_err(),rustix::io::Errno::CHILD);
    }
}

#[test]
fn supervised_child_already_exited_member_needs_pidfd_exit_not_zombie_namespace() {
    let mut leader=spawn("exec sleep 60");
    let mut command=Command::new("/bin/sh");
    command.args(["-c","exit 29"]).process_group(pid(leader.pid()).unwrap().as_raw_pid());
    let mut member=command.spawn().unwrap();
    let member_pid=pid(member.id()).unwrap();
    rustix::process::waitid(rustix::process::WaitId::Pid(member_pid),rustix::process::WaitIdOptions::EXITED|rustix::process::WaitIdOptions::NOWAIT).unwrap().unwrap();
    let image=ProcImage::new(&[(member.id(),false)]);
    assert!(matches!(image.reader().process(member.id()),ProcessRead::Unavailable(_)),"fixture must inject missing namespace metadata for an exited member");
    leader.facts=image.reader();
    // The dead member is deliberately left unreaped throughout both inventory
    // and drain, so it cannot disappear before either read by scheduling luck.
    let inventory=leader.members();
    let drain=leader.cancel();
    leader.facts=ProcessReader::default();
    let status=member.wait().unwrap();
    assert!(inventory.unwrap().is_empty());
    drain.unwrap();
    assert_eq!(status.code(),Some(29));
}
