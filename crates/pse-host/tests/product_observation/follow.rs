// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use std::io::{BufRead as _, BufReader};

struct Producer {
    directory: tempfile::TempDir,
    owner: SupervisedChild,
    source: Value,
    reader: String,
}
impl Producer {
    fn start() -> Self {
        Self::with_telemetry(None)
    }
    fn with_telemetry(settings: Option<Value>) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let reader = std::env::var("PSE_NATIVE_FIXTURE_PRODUCT_READER").unwrap();
        let helper = std::env::var_os("PSE_NATIVE_FIXTURE_OBSERVATION_HELPER").unwrap();
        let mut command = Command::new(helper);
        command
            .arg(directory.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(settings) = settings {
            command.env("PSE_NATIVE_TELEMETRY_SETTINGS", settings.to_string());
        }
        let owner = SupervisedChild::spawn(&mut command, DrainFailurePolicy::Park, None).unwrap();
        wait(&directory.path().join("active.ready"));
        let source =
            serde_json::from_slice(&std::fs::read(directory.path().join("source.json")).unwrap())
                .unwrap();
        Self {
            directory,
            owner,
            source,
            reader,
        }
    }
    fn follow(&self, samples: u64, timeout: u64) -> Command {
        let request = json!({"operation":"follow","source":{"selection":"retained","checkpoint":self.source["checkpoint"],"expected":null},"request":{"version":2,"operation":self.source["operation"],"window":{"secs":30,"nanos":0},"include_related":false,"related_limit":1},"invocation":null,"resume":null});
        let path = self.directory.path().join("follow.json");
        std::fs::write(&path, request.to_string()).unwrap();
        let mut command = pse();
        command
            .args([
                "--format",
                "ndjson",
                "observe",
                "follow",
                "--reader",
                &self.reader,
                "--request",
            ])
            .arg(path)
            .args([
                "--samples",
                &samples.to_string(),
                "--interval-ms",
                "1",
                "--timeout",
                &timeout.to_string(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        command
    }
    fn advance(&mut self) {
        std::fs::write(self.directory.path().join("advance"), []).unwrap();
        wait(&self.directory.path().join("advanced.ready"));
        assert!(self.owner.try_wait().unwrap().is_none());
        let request = json!({"operation":"page","source":{"selection":"retained","checkpoint":self.source["checkpoint"],"expected":null},"operation_id":self.source["operation"],"section":"records","producer":null,"limit":1,"cursor":null});
        let response = query(self.directory.path(), &self.reader, &request, false);
        assert_eq!(
            response["payload"]["product"]["page"]["records"]["records"][0]["sequence"], 4,
            "{response}"
        );
    }
    fn close(&mut self) {
        for (request, ready) in [("finish", "terminal.ready"), ("expire", "expired.ready")] {
            std::fs::write(self.directory.path().join(request), []).unwrap();
            wait(&self.directory.path().join(ready));
        }
        std::fs::write(self.directory.path().join("close"), []).unwrap();
        self.owner
            .wait_ready(None, Some(Instant::now() + Duration::from_secs(5)))
            .unwrap();
        assert!(self.owner.wait().unwrap().success());
    }
}

#[test]
fn actual_slow_follow_pipe_preserves_native_progress_and_complete_ndjson() {
    let mut producer = Producer::start();
    let mut observer =
        SupervisedChild::spawn(&mut producer.follow(8, 15), DrainFailurePolicy::Park, None)
            .unwrap();
    let pipe = observer.take_stdout().unwrap();
    let capacity = rustix::pipe::fcntl_setpipe_size(&pipe, 4096).unwrap();
    let mut output = BufReader::new(pipe);
    let mut first = String::new();
    output.read_line(&mut first).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&first).unwrap()["payload"]["observer_state"],
        "observing"
    );
    // Leave the pipe unread while the original performing owner publishes independently.
    let deadline = Instant::now() + Duration::from_secs(1);
    while rustix::io::ioctl_fionread(output.get_ref()).unwrap() < capacity as u64 {
        assert!(
            Instant::now() < deadline,
            "owned slow-reader output pipe did not fill"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    producer.advance();
    let mut bytes = first.into_bytes();
    let mut buffer = [0_u8; 512];
    loop {
        let read = output.read(&mut buffer).unwrap();
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..read]);
        std::thread::sleep(Duration::from_millis(1));
    }
    observer
        .wait_ready(None, Some(Instant::now() + Duration::from_secs(5)))
        .unwrap();
    assert!(observer.wait().unwrap().success());
    let frames: Vec<Value> = bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect();
    assert_eq!(frames.len(), 9);
    assert_eq!(frames.last().unwrap()["payload"]["summary"]["samples"], 8);
    assert!(frames[..8].iter().any(|frame| {
        frame["payload"]["product"]["sample"]["cursor"]["producers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|producer| producer["sequence"] == 4)
    }));
    producer.close();
}

#[test]
fn unavailable_optional_otlp_backend_leaves_actual_shell_observation_and_native_progress_usable() {
    // Select an actually refused loopback endpoint, not another managed collector/service.
    let reservation = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = reservation.local_addr().unwrap();
    drop(reservation);
    assert!(std::net::TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_err());
    let settings = json!({
        "schema_version":1,"disabled":false,
        "traces":{"enabled":true,"endpoint":format!("http://{address}/v1/traces"),"headers":{},"timeout_ms":100},
        "logs":{"enabled":false,"endpoint":null,"headers":{},"timeout_ms":100},
        "metrics":{"enabled":false,"endpoint":null,"headers":{},"timeout_ms":100},
        "resource":{"service.name":"observation-unavailable-backend"},
        "process_incarnation":"fixture-unavailable-backend","max_queue_size":8,"max_export_batch_size":1,
        "schedule_delay_ms":10,"export_timeout_ms":200,"metric_export_interval_ms":60000,
        "sampler":"always_on","sampler_argument":1.0
    });
    let mut producer = Producer::with_telemetry(Some(settings));
    std::thread::sleep(Duration::from_millis(150));
    producer.advance();
    let request = json!({"operation":"inspect","source":{"selection":"retained","checkpoint":producer.source["checkpoint"],"expected":null},"request":{"version":2,"operation":producer.source["operation"],"window":{"secs":30,"nanos":0},"include_related":false,"related_limit":1},"invocation":null});
    let observed = query(producer.directory.path(), &producer.reader, &request, false);
    assert_eq!(observed["payload"]["category"], "success", "{observed}");
    assert_eq!(
        observed["payload"]["product"]["document"]["operation"]["id"],
        producer.source["operation"]
    );
    assert!(producer.owner.try_wait().unwrap().is_none());
    producer.close();
    let settled: Value = serde_json::from_slice(
        &std::fs::read(producer.directory.path().join("telemetry.settled")).unwrap(),
    )
    .unwrap();
    assert_eq!(settled["health"]["delivery"], "not_observed");
    assert!(
        matches!(
            settled["health"]["shutdown"].as_str(),
            Some("succeeded" | "failed")
        ),
        "{settled}"
    );
    assert!(std::net::TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_err());
}

#[test]
fn actual_broken_pipe_full_renderer_and_stalled_follow_cancellation_leave_native_owner_alive() {
    for mode in ["broken_pipe", "full_renderer", "cancel", "deadline"] {
        let mut producer = Producer::start();
        let mut command = producer.follow(1000, if mode == "deadline" { 2 } else { 15 });
        if mode == "full_renderer" {
            command.stdout(
                std::fs::OpenOptions::new()
                    .write(true)
                    .open("/dev/full")
                    .unwrap(),
            );
        }
        let mut observer =
            SupervisedChild::spawn(&mut command, DrainFailurePolicy::Park, None).unwrap();
        let mut retained_output = None;
        if mode != "full_renderer" {
            let pipe = observer.take_stdout().unwrap();
            let capacity = rustix::pipe::fcntl_setpipe_size(&pipe, 4096).unwrap();
            let mut output = BufReader::new(pipe);
            if mode == "broken_pipe" {
                drop(output);
            } else {
                let mut first = String::new();
                output.read_line(&mut first).unwrap();
                assert_eq!(
                    serde_json::from_str::<Value>(&first).unwrap()["payload"]["observer_state"],
                    "observing"
                );
                retained_output = Some(output);
                let deadline = Instant::now() + Duration::from_secs(1);
                while rustix::io::ioctl_fionread(retained_output.as_ref().unwrap().get_ref())
                    .unwrap()
                    < capacity as u64
                {
                    assert!(
                        Instant::now() < deadline,
                        "{mode}: owned output pipe did not fill"
                    );
                    std::thread::sleep(Duration::from_millis(5));
                }
                if mode == "cancel" {
                    // The held SupervisedChild still owns this exact unreaped native leader.
                    rustix::process::kill_process(
                        rustix::process::Pid::from_raw(observer.pid() as i32).unwrap(),
                        rustix::process::Signal::TERM,
                    )
                    .unwrap();
                }
            }
        }
        // Hold the unread output through original leader/group settlement. Cancellation
        // and the observer deadline must not require a consumer to unblock stdout.
        observer
            .wait_ready(None, Some(Instant::now() + Duration::from_secs(5)))
            .unwrap();
        let status = observer.wait().unwrap();
        assert!(
            !status.success(),
            "{mode}: incomplete required delivery cannot be success"
        );
        if mode == "cancel" {
            assert_eq!(status.code(), Some(130));
        }
        if mode == "deadline" {
            assert_eq!(status.code(), Some(124));
        }
        drop(retained_output);
        producer.advance();
        producer.close();
    }
}
