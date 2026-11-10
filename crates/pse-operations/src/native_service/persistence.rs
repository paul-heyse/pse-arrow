// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive lookup of original owner-issued startup records; no adoption or stop authority.
use super::*;

/// Read the selected private native service's original authenticated MX metrics.
/// This observes owner-issued startup records and the actual process/listener. It never adopts
/// a unit, reconnects a stopped service, starts a process or grants maintenance authority.
/// # Errors
/// Foreign/private-state mismatch, absent/ambiguous live incarnation, denied or unavailable
/// native metrics, changed original producer, bounded extent or original deadline failure.
pub async fn inspect_owned_persistence(
    state: &Path,
    deadline: Instant,
) -> Result<NativePersistenceObservation, NativeServiceError> {
    let reader = PersistenceReader::lookup(state, deadline).await?;
    reader.inspect(deadline).await
}
struct PersistenceReader {
    state: PathBuf,
    device: u64,
    inode: u64,
    nonce: String,
    records: RetainedRecords,
    startup_digest: String,
    payload: StartupPayload,
    claim: StartupClaim,
    manager: HostManager,
}
impl PersistenceReader {
    async fn lookup(state: &Path, deadline: Instant) -> Result<Self, NativeServiceError> {
        deadline_left(deadline)?;
        private_path(state, true)?;
        let metadata = fs::metadata(state)?;
        let identity: DisposableIdentity = private_read(&state.join("disposable.json"))?;
        if identity.version != 1 {
            return Err(NativeServiceError::Refused(
                "native metric state interpretation differs",
            ));
        }
        let records = RetainedRecords::read(state)?;
        let manager = host_observation(deadline, HostManager::connect_user(None)).await?;
        let runs = state.join("runs");
        private_path(&runs, true)?;
        let mut selected = None;
        for (index, entry) in fs::read_dir(&runs)?.enumerate() {
            if index >= 128 {
                return Err(NativeServiceError::Refused(
                    "native metric startup lookup exceeds bound",
                ));
            }
            deadline_left(deadline)?;
            let run = entry?.path();
            verify_run(state, &run)?;
            if !run.join("started.json").try_exists()? {
                continue;
            }
            let payload: StartupPayload = private_read(&run.join("startup.json"))?;
            let claim: StartupClaim = private_read(&run.join("started.json"))?;
            if payload.version != 1
                || payload.state != state
                || payload.run != run
                || payload.nonce != identity.nonce
                || claim.nonce != identity.nonce
                || payload.bind.ip() != Ipv4Addr::LOCALHOST
                || payload.bind.port() == 0
                || !payload.unit.starts_with("pse-host-")
                || !payload.unit.ends_with(".service")
            {
                return Err(NativeServiceError::Refused(
                    "native metric startup selection differs",
                ));
            }
            // Historical drained launches need no live unit. This read grants no mutation.
            if run.join("stopped.json").try_exists()? {
                continue;
            }
            let observed = match host_observation(deadline, manager.observe(&payload.unit)).await {
                Ok(observed) => observed,
                Err(error) => {
                    deadline_left(deadline)?;
                    if run.join("stopped.json").try_exists()? {
                        continue;
                    }
                    return Err(error);
                }
            };
            if matches!(observed.active_state.as_str(), "inactive" | "failed") {
                continue;
            }
            if claim.pid != observed.main_pid || claim.invocation != hex_bytes(&observed.invocation)
            {
                return Err(NativeServiceError::Refused(
                    "native metric claim differs from actual original invocation",
                ));
            }
            if selected.is_some() {
                return Err(NativeServiceError::Refused(
                    "native metric state has ambiguous active producers",
                ));
            }
            selected = Some((payload, claim));
        }
        let (payload, claim) = selected.ok_or(NativeServiceError::Refused(
            "selected native service has no original live metrics producer",
        ))?;
        let startup_digest = hex::encode(Sha256::digest(json(&payload)?));
        let reader = Self {
            state: state.into(),
            device: metadata.dev(),
            inode: metadata.ino(),
            nonce: identity.nonce,
            records,
            startup_digest,
            payload,
            claim,
            manager,
        };
        reader.verify()?;
        Ok(reader)
    }
    fn verify(&self) -> Result<(), NativeServiceError> {
        private_path(&self.state, true)?;
        let metadata = fs::metadata(&self.state)?;
        let identity: DisposableIdentity = private_read(&self.state.join("disposable.json"))?;
        if metadata.dev() != self.device
            || metadata.ino() != self.inode
            || identity.version != 1
            || identity.nonce != self.nonce
        {
            return Err(NativeServiceError::Refused(
                "native metric state identity changed",
            ));
        }
        self.records.verify(&self.state)?;
        verify_run(&self.state, &self.payload.run)?;
        let payload: StartupPayload = private_read(&self.payload.run.join("startup.json"))?;
        let claim: StartupClaim = private_read(&self.payload.run.join("started.json"))?;
        if self.startup_digest != hex::encode(Sha256::digest(json(&payload)?))
            || claim != self.claim
        {
            return Err(NativeServiceError::Refused(
                "native metric original startup record changed",
            ));
        }
        Ok(())
    }
    async fn observe(&self, deadline: Instant) -> Result<UnitObservation, NativeServiceError> {
        self.verify()?;
        let observed = host_observation(deadline, self.manager.observe(&self.payload.unit)).await?;
        if observed.main_pid == 0
            || matches!(observed.active_state.as_str(), "inactive" | "failed")
            || observed.main_pid != self.claim.pid
            || hex_bytes(&observed.invocation) != self.claim.invocation
        {
            return Err(NativeServiceError::Refused(
                "native metric original producer is unavailable or changed",
            ));
        }
        let exact = server::reopen_generation(&self.payload.generation, &self.payload.recipe)?;
        if fs::read_link(format!("/proc/{}/exe", observed.main_pid))? != exact.executable()
            || !owns_listener(observed.main_pid, self.payload.bind)?
        {
            return Err(NativeServiceError::Refused(
                "native metric listener is not the original prepared producer",
            ));
        }
        deadline_left(deadline)?;
        Ok(observed)
    }
    async fn inspect(
        &self,
        deadline: Instant,
    ) -> Result<NativePersistenceObservation, NativeServiceError> {
        let before = self.observe(deadline).await?;
        let credentials: Credentials = private_read(&self.state.join("credentials.json"))?;
        let bytes = read_metrics(self.payload.bind, credentials, deadline).await?;
        let after = self.observe(deadline).await?;
        if before.main_pid != after.main_pid || before.invocation != after.invocation {
            return Err(NativeServiceError::Refused(
                "native metric producer changed during original read",
            ));
        }
        let observed_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|value| u64::try_from(value.as_millis()).ok())
            .ok_or(NativeServiceError::Refused(
                "native observation clock is unavailable",
            ))?;
        NativePersistenceObservation::from_prometheus(
            &bytes,
            self.claim.invocation.clone(),
            observed_at_ms,
        )
    }
}
async fn read_metrics(
    bind: SocketAddr,
    credentials: Credentials,
    deadline: Instant,
) -> Result<Vec<u8>, NativeServiceError> {
    use base64::Engine as _;
    use http_body_util::BodyExt as _;
    let authorization = base64::engine::general_purpose::STANDARD
        .encode(format!("{}:{}", credentials.username, credentials.password));
    let exchange = async {
        let stream = tokio::net::TcpStream::connect(bind).await?;
        let (mut sender, connection) = hyper::client::conn::http2::handshake(
            hyper_util::rt::TokioExecutor::new(),
            hyper_util::rt::TokioIo::new(stream),
        )
        .await
        .map_err(|_| NativeServiceError::Refused("native metric connection failed"))?;
        let read = async {
            let request = http::Request::builder()
                .uri(format!("http://{bind}/metrics"))
                .header(
                    http::header::AUTHORIZATION,
                    format!("Basic {authorization}"),
                )
                .body(http_body_util::Full::new(bytes::Bytes::new()))
                .map_err(|_| NativeServiceError::Refused("invalid native metric request"))?;
            let response = sender
                .send_request(request)
                .await
                .map_err(|_| NativeServiceError::Refused("native metric request failed"))?;
            if response.status() != http::StatusCode::OK {
                return Err(NativeServiceError::Refused(
                    "native metrics are unavailable or denied",
                ));
            }
            let mut body = response.into_body();
            let mut bytes = Vec::new();
            while let Some(frame) = body.frame().await {
                let frame = frame.map_err(|_| {
                    NativeServiceError::Refused("incomplete native metric response")
                })?;
                if let Ok(data) = frame.into_data() {
                    if bytes.len().saturating_add(data.len()) > 4 * 1024 * 1024 {
                        return Err(NativeServiceError::Refused(
                            "native metric response exceeds admission",
                        ));
                    }
                    bytes.extend_from_slice(&data);
                }
            }
            Ok(bytes)
        };
        tokio::pin!(read, connection);
        tokio::select! {biased;result=&mut read=>result,_=&mut connection=>Err(NativeServiceError::Refused("native metric connection ended before completion"))}
    };
    tokio::time::timeout_at(deadline.into(), exchange)
        .await
        .map_err(|_| NativeServiceError::Refused("original native metric deadline expired"))?
}
