// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Canonical, qualified admitted-body publication and strict numerical reconstruction.
//! A missing eligible product remains distinct from successful reconstruction.
use futures_util::FutureExt;
use pse_compiler::{
    typed_math::{AdmittedBody, portable_payload_hash, reconstruct_portable},
    workspace::{CompilerContext, physical_identity},
};
use pse_ids::{ContentHash, Frame, FramedHasher, roles::SemanticBodyHash};
use pse_operations::{
    canonical::{CanonicalError, CanonicalStore},
    canonical_selection::SelectedRead,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, atomic::AtomicBool};

const INTERPRETATION: &str = "pse.runtime.admitted-body-product.v4";
const OBSERVATION_WORK_BYTES: usize = pse_buildinfo::identity::OBSERVATION_WORK_BYTES;
/// Refusals preserve scientific reconstruction and storage failures distinctly.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum PortableError {
    /// Concrete canonical storage failure.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Store(#[from] CanonicalError),
    /// Incomplete or incompatible scientific construction receipts.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Math(#[from] pse_math::MathError),
    /// Deployment/product interpretation cannot establish persistent reuse.
    #[error("portable admitted product: {0}")]
    #[diagnostic(code(runtime::infrastructure))]
    Qualification(String),
}
impl pse_diagnostics::TypedDiagnostic for PortableError {
    fn diagnostic_code(&self) -> Option<pse_diagnostics::DiagnosticCode> {
        match self {
            Self::Store(error) => error.diagnostic_code(),
            Self::Math(error) => error.diagnostic_code(),
            Self::Qualification(_) => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
        }
    }
    fn diagnostic_facts(&self) -> pse_diagnostics::DiagnosticFacts {
        match self {
            Self::Store(error) => error.diagnostic_facts(),
            Self::Math(error) => error.diagnostic_facts(),
            Self::Qualification(reason) => {
                let mut facts = pse_diagnostics::DiagnosticFacts {
                    rule: Some(pse_diagnostics::DiagnosticRule::MathLibrary),
                    ..Default::default()
                };
                facts.observations.insert(
                    "portable_refusal".into(),
                    pse_diagnostics::DiagnosticObservation::Text(reason.clone()),
                );
                facts
            }
        }
    }
    fn diagnostic_children(
        &self,
    ) -> Option<Box<dyn Iterator<Item = &dyn pse_diagnostics::TypedDiagnostic> + '_>> {
        let child: &dyn pse_diagnostics::TypedDiagnostic = match self {
            Self::Store(error) => error,
            Self::Math(error) => error,
            Self::Qualification(_) => return None,
        };
        Some(Box::new(std::iter::once(child)))
    }
}
/// The executable boundary that owns a producer capture. Dependency units cannot
/// qualify a different executable, even when their outer attestations coincide.
#[derive(Clone, Copy, Debug)]
pub struct ExpectedProducerTarget {
    /// Cargo package selected by the capture.
    pub package: &'static str,
    /// Cargo target selected as the production graph root.
    pub target: &'static str,
    /// Cargo target kind (`lib`, `bin`, or `cdylib`).
    pub kind: &'static str,
}
impl ExpectedProducerTarget {
    /// The native extension imported by Python.
    pub const PYTHON: Self = Self {
        package: "pse-py",
        target: "_native",
        kind: "cdylib",
    };
    /// The supervised native worker executable.
    pub const WORKER: Self = Self {
        package: "xtask",
        target: "pse-worker",
        kind: "bin",
    };
}
/// Relevant producer identity from an operator-owned, verified deployment receipt.
/// It is separate from a whole-build attestation and has no deserializer.
#[derive(Clone, Debug)]
pub struct QualifiedProducer {
    identity: ContentHash,
}
impl QualifiedProducer {
    /// Read the output of the production-unit identity tool at the deployment boundary.
    /// The operator owns the receipt's source/input qualification. Incomplete receipts
    /// return `None`, allowing normal execution without persistent reuse.
    /// # Safety
    /// The bytes must be the actual current capture from the controlled deployment
    /// identity tool, with its reviewed completeness declarations. The caller must
    /// supply the independently observed actual loaded artifact digest, after current
    /// consumed inputs and role contract have been checked, not hashes chosen
    /// to make an arbitrary JSON claim match. This asserts deployment provenance;
    /// matching JSON fields alone cannot establish a qualified producer.
    ///
    /// ```compile_fail
    /// use pse_runtime::math::portable::QualifiedProducer;
    /// let id=pse_ids::ContentHash::from_bytes([0;32]);
    /// let _=QualifiedProducer::from_deployment_receipt(b"{}",&id.to_hex(),pse_runtime::math::portable::ExpectedProducerTarget::WORKER);
    /// ```
    /// # Errors
    /// Wrong interpretation, malformed identity or contradictory eligibility evidence.
    #[allow(
        unsafe_code,
        reason = "ADR-0164 controlled deployment producer qualification mint"
    )]
    pub unsafe fn from_deployment_receipt(
        bytes: &[u8],
        actual_artifact_sha256: &str,
        expected_target: ExpectedProducerTarget,
    ) -> Result<Option<Self>, PortableError> {
        #[derive(Deserialize)]
        struct Receipt {
            frame: String,
            persistent_reuse_eligible: bool,
            reasons: Vec<String>,
            deployment: Option<Association>,
            native_abi: Option<String>,
            #[serde(default)]
            package: Option<String>,
            #[serde(default)]
            selected_root: Option<String>,
            #[serde(default)]
            units: Vec<Unit>,
        }
        #[derive(Deserialize)]
        struct Unit {
            key: String,
            target_name: String,
            target_kind: Vec<String>,
            mode: String,
            #[serde(default)]
            profile: serde_json::Value,
            #[serde(default)]
            features: Vec<String>,
        }
        #[derive(Deserialize)]
        struct Association {
            artifact: Artifact,
            producer_identity: String,
            selected_root: String,
        }
        #[derive(Deserialize)]
        struct Artifact {
            sha256: String,
        }
        let header: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|error| PortableError::Qualification(error.to_string()))?;
        if header
            .get("receipt_version")
            .and_then(serde_json::Value::as_u64)
            != Some(u64::from(pse_buildinfo::DEPLOYMENT_RECEIPT_VERSION))
        {
            return Err(PortableError::Qualification(
                "unsupported deployment receipt interpretation".into(),
            ));
        }
        let receipt: Receipt = serde_json::from_value(header.clone())
            .map_err(|error| PortableError::Qualification(error.to_string()))?;
        if receipt.frame != Frame::ProducerV1.as_str() {
            return Err(PortableError::Qualification(
                "unsupported producer interpretation".into(),
            ));
        }
        if !receipt.persistent_reuse_eligible {
            return Ok(None);
        }
        if !receipt.reasons.is_empty() {
            return Err(PortableError::Qualification(
                "eligible producer retains unresolved input evidence".into(),
            ));
        }
        let Some(association) = receipt.deployment else {
            return Ok(None);
        };
        if association.artifact.sha256 != actual_artifact_sha256
            || actual_artifact_sha256.len() != 64
            || !actual_artifact_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Ok(None);
        }
        let Some(selected_root) = receipt.selected_root else {
            return Ok(None);
        };
        if association.selected_root != selected_root {
            return Err(PortableError::Qualification(
                "actual artifact association names another selected root".into(),
            ));
        }
        let mut roots = receipt
            .units
            .iter()
            .filter(|unit| unit.key == selected_root);
        let Some(root) = roots.next() else {
            return Err(PortableError::Qualification(
                "selected producer root is absent from captured units".into(),
            ));
        };
        if roots.next().is_some() || root.mode != "build" {
            return Err(PortableError::Qualification(
                "contradictory selected producer root".into(),
            ));
        }
        if receipt.package.as_deref() != Some(expected_target.package)
            || root.target_name != expected_target.target
            || !root
                .target_kind
                .iter()
                .any(|kind| kind == expected_target.kind)
        {
            return Ok(None);
        }
        if receipt
            .native_abi
            .as_ref()
            .is_none_or(|abi| abi.trim().is_empty())
        {
            return Ok(None);
        }
        if matches!(expected_target.package, "xtask" | "pse-py")
            && (root.profile.get("name").and_then(serde_json::Value::as_str) != Some("producer")
                || !root
                    .features
                    .iter()
                    .any(|feature| feature == "native-solvers"))
        {
            return Ok(None);
        }
        let identity = pse_buildinfo::identity::verify_scientific_producer_identity(&header)
            .map_err(|error| PortableError::Qualification(error.to_string()))?;
        if association.producer_identity != identity.to_hex() {
            return Err(PortableError::Qualification(
                "actual artifact association names another scientific producer".into(),
            ));
        }
        Ok(Some(Self { identity }))
    }
    /// Exact relevant producer identity; eligibility is not a property of global builds.
    pub fn identity(&self) -> ContentHash {
        self.identity
    }
    fn key(&self) -> String {
        format!(
            "{}{}",
            pse_operations::canonical_selection::SCIENTIFIC_PRODUCER_PREFIX,
            self.identity.to_hex()
        )
    }
}
/// Additional effective configuration observed by the actual composition root.
/// It is called outside the loader scope, so Python can acquire the GIL without
/// holding glibc's loader lock. The controlled root excludes concurrent mutation.
/// The root bounds construction, not just the returned length, by
/// `pse_buildinfo::identity::MAX_CONFIGURATION_BYTES` and observes the complete
/// supported configuration. An opaque/unbounded embedding must omit local replay.
pub type RuntimeConfigurationObserver =
    Arc<dyn Fn() -> std::io::Result<RuntimeConfigurationObservation> + Send + Sync>;

/// Complete bounded configuration and owned environment observed by the actual
/// composition root outside the loader scope. JSON bytes retain their old framing.
#[derive(Debug, PartialEq, Eq)]
pub struct RuntimeConfigurationObservation {
    bytes: Vec<u8>,
    environment: pse_buildinfo::identity::CurrentEnvironmentSnapshot,
}
impl RuntimeConfigurationObservation {
    /// Bind original configuration bytes to an independently owned environment
    /// snapshot. Producers bound their allocations before calling this operation.
    /// # Errors
    /// Configuration generation exceeded the supported finite extent.
    pub fn new(
        bytes: Vec<u8>,
        environment: pse_buildinfo::identity::CurrentEnvironmentSnapshot,
    ) -> std::io::Result<Self> {
        if bytes.capacity() > pse_buildinfo::identity::MAX_CONFIGURATION_BYTES {
            return Err(std::io::Error::other("runtime configuration byte limit"));
        }
        Ok(Self { bytes, environment })
    }
}

/// Checkout context observed during startup, separately from loaded-image assurance.
#[derive(Debug)]
pub struct StartupDeploymentObservation {
    /// Complete bytes observed now, or explicit installed source-unavailable context.
    pub outer: pse_buildinfo::OuterObservation,
    /// Strict operator-reviewed producer, when the selected receipt qualifies it.
    pub producer: Option<QualifiedProducer>,
}

/// Opaque exact local deployment compatibility, distinct from producer qualification.
pub struct LocalReplay {
    observation: pse_buildinfo::LocalRuntimeObservation,
    anchor: usize,
    role: pse_buildinfo::LocalRuntimeRole,
    configuration: RuntimeConfigurationObserver,
    configuration_bytes: Vec<u8>,
    configuration_environment: pse_buildinfo::identity::CurrentEnvironmentSnapshot,
    identity: ContentHash,
    _owner: Option<Arc<pse_columnar::AllocationLease>>,
}

// Weak evidence slots retain neither metadata copies nor allocation owners. Every
// hit compares the complete supported configuration; no digest or timestamp is
// permission. Bounded replacement changes cache residency only.
struct ReplayCandidates {
    slots: [std::sync::Weak<LocalReplay>; 8],
    next: usize,
}
fn replay_candidates() -> &'static Mutex<ReplayCandidates> {
    static CANDIDATES: std::sync::OnceLock<Mutex<ReplayCandidates>> = std::sync::OnceLock::new();
    CANDIDATES.get_or_init(|| {
        Mutex::new(ReplayCandidates {
            slots: std::array::from_fn(|_| std::sync::Weak::new()),
            next: 0,
        })
    })
}
fn candidate(
    role: pse_buildinfo::LocalRuntimeRole,
    anchor: usize,
    configuration: &RuntimeConfigurationObservation,
) -> Option<Arc<LocalReplay>> {
    let mut candidates = replay_candidates()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for slot in &mut candidates.slots {
        let Some(local) = slot.upgrade() else {
            *slot = std::sync::Weak::new();
            continue;
        };
        if local.role == role
            && local.anchor == anchor
            && local.configuration_bytes == configuration.bytes
            && local.configuration_environment == configuration.environment
        {
            return Some(local);
        }
    }
    None
}
fn retain_candidate(local: &Arc<LocalReplay>) {
    let mut candidates = replay_candidates()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let next = candidates.next;
    candidates.slots[next] = Arc::downgrade(local);
    candidates.next = (next + 1) % candidates.slots.len();
}
fn local_role(
    target: ExpectedProducerTarget,
) -> Result<pse_buildinfo::LocalRuntimeRole, PortableError> {
    match (target.package, target.target, target.kind) {
        ("xtask", "pse-worker", "bin") => Ok(pse_buildinfo::LocalRuntimeRole::Worker),
        ("pse-py", "_native", "cdylib") => Ok(pse_buildinfo::LocalRuntimeRole::Python),
        _ => Err(PortableError::Qualification(
            "unsupported local reconstruction role".into(),
        )),
    }
}
fn observe_configuration(
    configuration: &RuntimeConfigurationObserver,
) -> Result<RuntimeConfigurationObservation, PortableError> {
    let bytes = configuration().map_err(observation_error)?;
    if bytes.bytes.capacity() > pse_buildinfo::identity::MAX_CONFIGURATION_BYTES {
        return Err(PortableError::Qualification(
            "runtime configuration byte limit".into(),
        ));
    }
    Ok(bytes)
}
#[cfg(test)]
#[allow(
    unsafe_code,
    reason = "controlled native test roots supply their genuine no-environment-writers lifetime"
)]
unsafe fn observed_test_configuration(
    bytes: Vec<u8>,
) -> std::io::Result<RuntimeConfigurationObservation> {
    // SAFETY: caller is the actual controlled test root, not a public safe wrapper.
    let environment =
        unsafe { pse_buildinfo::identity::CurrentEnvironmentSnapshot::capture_scoped(&|| Ok(())) }?;
    RuntimeConfigurationObservation::new(bytes, environment)
}
impl std::fmt::Debug for LocalReplay {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LocalReplay")
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}
/// Versioned replay authority. Both guarantees use the same protected selection,
/// concrete receipts and owning scientific reconstruction; neither decodes old
/// unqualified publications into eligible products.
#[derive(Clone, Debug)]
pub enum ReplayAdmission {
    /// Controlled receiving-root context; qualified only at actual trust transitions.
    Local(Arc<LocalReplay>),
    /// Existing broader reviewed selected-producer qualification.
    QualifiedProducer(QualifiedProducer),
}
/// Current admission for one receiving/publication operation. It is neither a
/// persisted capability nor the validity owner of already admitted mathematics.
#[derive(Clone, Debug)]
pub(crate) struct ValidatedReplayNamespace {
    producer: String,
    producer_class: pse_model::generated::enums::CanonicalProducerClass,
    deadline: Option<std::time::Instant>,
}
impl ValidatedReplayNamespace {
    pub(crate) fn producer_key(&self) -> &str {
        &self.producer
    }
    pub(crate) fn producer_class(&self) -> pse_model::generated::enums::CanonicalProducerClass {
        self.producer_class
    }
}
impl From<QualifiedProducer> for ReplayAdmission {
    fn from(value: QualifiedProducer) -> Self {
        Self::QualifiedProducer(value)
    }
}
impl ReplayAdmission {
    pub(crate) fn producer_class(&self) -> pse_model::generated::enums::CanonicalProducerClass {
        use pse_model::generated::enums::CanonicalProducerClass as C;
        match self {
            Self::Local(_) => C::LocalScientificReplay,
            Self::QualifiedProducer(_) => C::LibraryMathematics,
        }
    }
    /// Original finite observation clock used by the worker and Python startup roots.
    pub const STARTUP_OBSERVATION_LIMIT: std::time::Duration = std::time::Duration::from_secs(60);

    /// Independently observe the receiving executable/imported module and mint the
    /// local guarantee only within the supported bounded glibc reconstruction owner.
    /// # Safety
    /// The actual worker/Python composition root supplies its own code anchor and
    /// reads actual effective configuration. The supported reconstruction path must
    /// contain only immutable native mathematical construction: no Python imports,
    /// provider/plugin callbacks, JIT/direct executable mappings, runtime mutation,
    /// or concurrent current-environment mutation, including foreign libc writers,
    /// or work which waits for another thread to acquire the loader lock. Existing
    /// native generation-use ownership protects managed artifacts; privileged or
    /// external mutations are outside this controlled lifetime. Opaque contexts must
    /// omit local admission and use fresh scientific preparation.
    /// The configuration observer obeys its preallocation bound and observes the
    /// actual process-global role configuration for this anchor throughout replay.
    /// # Errors
    /// Unsupported role/platform/loader, unidentified mappings or unavailable context.
    #[allow(
        unsafe_code,
        reason = "ADR-0164 actual controlled composition-root local replay admission"
    )]
    pub unsafe fn observe_local(
        target: ExpectedProducerTarget,
        anchor: usize,
        configuration: RuntimeConfigurationObserver,
    ) -> Result<Self, PortableError> {
        // SAFETY: the convenience caller supplies the same receiving-root premises.
        // Production roots supply their original stop and clock to the scoped mint.
        unsafe {
            Self::observe_local_scoped(target, anchor, configuration, &AtomicBool::new(false), None)
        }
    }
    /// Observe the receiving root under the original operation cancellation and clock.
    /// File capture checks between chunks; opaque initialization/configuration calls and
    /// waiting to enter the loader scope can only be checked before and after they return.
    /// # Safety
    /// The receiving-root requirements of [`Self::observe_local`] apply unchanged.
    /// # Errors
    /// Typed cancellation/deadline, incompatible registration, or unavailable observation.
    #[allow(
        unsafe_code,
        reason = "ADR-0164 scoped controlled composition-root local replay admission"
    )]
    pub unsafe fn observe_local_scoped(
        target: ExpectedProducerTarget,
        anchor: usize,
        configuration: RuntimeConfigurationObserver,
        cancelled: &AtomicBool,
        deadline: Option<std::time::Instant>,
    ) -> Result<Self, PortableError> {
        receiving_checkpoint(cancelled, deadline)?;
        let role = local_role(target)?;
        // Establish the process-global mathematical runtime before observing its
        // loaded closure. License activation can resolve a hostname and load NSS
        // modules; capturing first would make the first preparation invalidate
        // its own admission. This effect must remain outside the loader scope.
        receiving_checkpoint(cancelled, deadline)?;
        let initialized = pse_math::initialize();
        receiving_checkpoint(cancelled, deadline)?;
        initialized?;
        let configuration_bytes = observe_configuration(&configuration);
        receiving_checkpoint(cancelled, deadline)?;
        let configuration_bytes = configuration_bytes?;
        let checkpoint = || observation_checkpoint(cancelled, deadline);
        let observation = pse_buildinfo::with_local_runtime_scope(|| {
            // SAFETY: the actual root captured the owned environment outside the
            // loader scope and retains controlled native execution premises.
            unsafe {
                pse_buildinfo::LocalRuntimeObservation::capture_scoped(
                    role,
                    anchor,
                    &configuration_bytes.environment,
                    &checkpoint,
                )
            }
        });
        receiving_checkpoint(cancelled, deadline)?;
        let observation = observation.map_err(observation_error)?;
        let current_configuration = observe_configuration(&configuration);
        receiving_checkpoint(cancelled, deadline)?;
        if current_configuration? != configuration_bytes {
            return Err(PortableError::Qualification(
                "effective runtime configuration changed during observation".into(),
            ));
        }
        let mut hash = FramedHasher::new(Frame::BuildInputsV1);
        hash.str("pse.local-runtime-replay.v2")
            .hash(&observation.identity().map_err(observation_error)?)
            .part(&configuration_bytes.bytes);
        receiving_checkpoint(cancelled, deadline)?;
        Ok(Self::Local(Arc::new(LocalReplay {
            observation,
            anchor,
            role,
            configuration,
            configuration_bytes: configuration_bytes.bytes,
            configuration_environment: configuration_bytes.environment,
            identity: hash.finish_hash(),
            _owner: None,
        })))
    }
    /// The relevant key's identity; the guarantee namespace remains explicit.
    pub fn identity(&self) -> ContentHash {
        match self {
            Self::Local(local) => local.identity,
            Self::QualifiedProducer(producer) => producer.identity(),
        }
    }
    pub(super) fn key(&self) -> String {
        match self {
            Self::Local(local) => format!(
                "{}{}",
                pse_operations::canonical_selection::LOCAL_RUNTIME_PRODUCER_PREFIX,
                local.identity.to_hex()
            ),
            Self::QualifiedProducer(producer) => producer.key(),
        }
    }
    /// Independently check the receiving root now. Pure retained products do not
    /// invoke this diagnostic/transition check.
    pub fn is_current(&self) -> bool {
        self.qualify_receiving(&AtomicBool::new(false), None)
            .is_ok_and(|namespace| namespace.is_some())
    }
    #[allow(
        unsafe_code,
        reason = "LocalReplay retains the actual controlled-root observation contract"
    )]
    pub(super) fn qualify_receiving(
        &self,
        cancelled: &AtomicBool,
        deadline: Option<std::time::Instant>,
    ) -> Result<Option<ValidatedReplayNamespace>, PortableError> {
        receiving_checkpoint(cancelled, deadline)?;
        if let Self::Local(local) = self {
            let before = observe_configuration(&local.configuration)?;
            if before.bytes != local.configuration_bytes
                || before.environment != local.configuration_environment
            {
                return Ok(None);
            }
            let checkpoint = || observation_checkpoint(cancelled, deadline);
            let result = pse_buildinfo::with_local_runtime_scope(|| {
                // SAFETY: LocalReplay comes from the actual controlled root. Its
                // newly observed environment is owned, so ordinary Python writers
                // cannot race a borrowed libc vector inside this loader scope.
                unsafe {
                    local
                        .observation
                        .verify_scoped(local.anchor, &before.environment, &checkpoint)
                }
            });
            if let Err(error) = result {
                return match error.kind() {
                    std::io::ErrorKind::Interrupted | std::io::ErrorKind::TimedOut => {
                        Err(observation_error(error))
                    }
                    _ => Ok(None),
                };
            }
            receiving_checkpoint(cancelled, deadline)?;
            let after = observe_configuration(&local.configuration)?;
            if after.bytes != local.configuration_bytes
                || after.environment != local.configuration_environment
            {
                return Ok(None);
            }
        }
        Ok(Some(ValidatedReplayNamespace {
            producer: self.key(),
            producer_class: self.producer_class(),
            deadline,
        }))
    }
}
fn observation_checkpoint(
    cancelled: &AtomicBool,
    deadline: Option<std::time::Instant>,
) -> std::io::Result<()> {
    if cancelled.load(std::sync::atomic::Ordering::Acquire) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Interrupted,
            "receiving operation cancelled",
        ));
    }
    if deadline.is_some_and(|at| std::time::Instant::now() >= at) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "receiving operation deadline",
        ));
    }
    Ok(())
}
fn observation_error(error: std::io::Error) -> PortableError {
    match error.kind() {
        std::io::ErrorKind::Interrupted => PortableError::Math(pse_math::MathError::Cancelled),
        std::io::ErrorKind::TimedOut => PortableError::Store(CanonicalError::Timeout),
        _ => PortableError::Qualification(error.to_string()),
    }
}
pub(super) fn receiving_checkpoint(
    cancelled: &AtomicBool,
    deadline: Option<std::time::Instant>,
) -> Result<(), PortableError> {
    observation_checkpoint(cancelled, deadline).map_err(observation_error)
}
impl ValidatedReplayNamespace {
    fn reconstruct<T>(
        &self,
        cancelled: &AtomicBool,
        operation: impl FnOnce() -> Result<T, PortableError>,
    ) -> Result<T, PortableError> {
        receiving_checkpoint(cancelled, self.deadline)?;
        if self.producer_class
            == pse_model::generated::enums::CanonicalProducerClass::LocalScientificReplay
        {
            pse_buildinfo::with_local_runtime_scope(|| Ok(operation()))
                .map_err(observation_error)?
        } else {
            operation()
        }
    }
}

pub(crate) fn runtime_error(error: PortableError) -> super::MathRuntimeError {
    match error {
        PortableError::Math(error) => error.into(),
        error => {
            let diagnostic = pse_model::diagnostic::project_typed(
                &error,
                pse_diagnostics::DiagnosticStage::ModelingAdmission,
            );
            super::MathRuntimeError::Math(pse_math::MathError::Typed {
                retained: size_of_val(&diagnostic) + pse_model::HeapUsage::heap_bytes(&diagnostic),
                cause: pse_model::diagnostic::DiagnosticCause::new(diagnostic),
            })
        }
    }
}

/// Receiving checks run under the same completion-owned native job as construction.
/// A departing caller cancels and joins its observation; it never leaves an unowned
/// loader callback or file read on an asynchronous executor thread.
pub(crate) async fn qualify_replay(
    service: &Arc<super::MathService>,
    producer: Option<ReplayAdmission>,
    deadline: std::time::Instant,
    driver: &crate::CancelSource,
) -> Result<Option<ValidatedReplayNamespace>, super::MathRuntimeError> {
    let Some(producer) = producer else {
        return Ok(None);
    };
    let control = pse_diagnostics::Cancellation::default();
    let operation = service.job_scoped(
        1,
        OBSERVATION_WORK_BYTES,
        control.clone(),
        Some(deadline),
        move |flag| {
            producer
                .qualify_receiving(&flag, Some(deadline))
                .map_err(runtime_error)
        },
    );
    tokio::pin!(operation);
    tokio::select! {
        biased;
        () = driver.cancelled() => {
            control.cancel();
            let _ = operation.await;
            Err(super::MathRuntimeError::Cancelled)
        },
        result = &mut operation => result,
    }
}

impl super::MathService {
    /// Capture startup checkout context and an optional strict receipt on a native
    /// owner. Cancellation joins the actual worker before returning its allocation.
    /// This does not refresh later run context or qualify a replay candidate.
    /// # Safety
    /// A selected receipt must satisfy [`QualifiedProducer::from_deployment_receipt`]'s
    /// reviewed deployment provenance contract; when selecting one, `artifact` is
    /// the actual loaded role. An unqualified outer observation mints no permission.
    /// # Errors
    /// Original deadline, cancellation, admission, source or strict receipt refusal.
    #[allow(
        unsafe_code,
        reason = "actual composition root supplies reviewed strict receipt provenance"
    )]
    pub async unsafe fn observe_deployment_runtime(
        self: &Arc<Self>,
        artifact: std::path::PathBuf,
        receipt: Option<std::path::PathBuf>,
        target: ExpectedProducerTarget,
        deadline: std::time::Instant,
        driver: &crate::CancelSource,
    ) -> Result<Arc<pse_columnar::Leased<StartupDeploymentObservation>>, PortableError> {
        // A size observation is only allocation preflight, never content evidence.
        // Its native owner drains before the complete independently admitted capture.
        let receipt_bytes = if let Some(path) = receipt.clone() {
            let (size, owner) = startup_observation_job(
                self,
                OBSERVATION_WORK_BYTES,
                deadline,
                driver,
                move |flag| {
                    receiving_checkpoint(&flag, Some(deadline))?;
                    let metadata = std::fs::metadata(path).map_err(observation_error)?;
                    if !metadata.is_file() {
                        return Err(PortableError::Qualification(
                            "startup receipt is not a regular file".into(),
                        ));
                    }
                    let size = metadata.len();
                    receiving_checkpoint(&flag, Some(deadline))?;
                    usize::try_from(size)
                        .map(|size| (size, 0))
                        .map_err(|_| PortableError::Qualification("receipt extent overflow".into()))
                },
            )
            .await?;
            drop(owner);
            size
        } else {
            0
        };
        let allowance = receipt_bytes
            .checked_mul(16)
            .and_then(|n| n.checked_add(OBSERVATION_WORK_BYTES))
            .and_then(|n| {
                n.checked_add(if receipt.is_some() {
                    pse_buildinfo::identity::DEPLOYMENT_NAMESPACE_WORK_BYTES
                } else {
                    0
                })
            })
            .ok_or_else(|| {
                PortableError::Qualification("startup observation extent overflow".into())
            })?;
        let (observation, owner) =
            startup_observation_job(self, allowance, deadline, driver, move |flag| {
                let checkpoint = || observation_checkpoint(&flag, Some(deadline));
                let outer =
                    pse_buildinfo::identity::observe_deployment_scoped(&artifact, &checkpoint)
                        .map_err(observation_error)?;
                let producer = match receipt {
                    None => None,
                    Some(path) => {
                        let bytes = read_startup_receipt(&path, receipt_bytes, &checkpoint)
                            .map_err(observation_error)?;
                        let actual = pse_buildinfo::identity::verify_receipt_artifact_scoped(
                            &bytes,
                            &artifact,
                            &checkpoint,
                        )
                        .map_err(observation_error)?;
                        checkpoint().map_err(observation_error)?;
                        // SAFETY: the actual composition root retains the operator-owned
                        // receipt contract; the same worker just verified current inputs.
                        let qualified = unsafe {
                            QualifiedProducer::from_deployment_receipt(
                                &bytes,
                                &actual.sha256,
                                target,
                            )
                        }?;
                        checkpoint().map_err(observation_error)?;
                        qualified
                    }
                };
                checkpoint().map_err(observation_error)?;
                let observation = StartupDeploymentObservation { outer, producer };
                // The receipt, namespace and parsing scratch have died on this native
                // owner. Only this observation and its two shared payload allocations
                // escape; the inline hashes/producer are included in the value size.
                let retained = size_of::<StartupDeploymentObservation>()
                    + observation.outer.git_sha.capacity()
                    + size_of::<pse_columnar::Leased<StartupDeploymentObservation>>()
                    + 4 * size_of::<usize>();
                Ok((observation, retained))
            })
            .await?;
        Ok(Arc::new(pse_columnar::Leased::new(
            Arc::new(observation),
            owner,
        )))
    }
    /// Observe the actual local composition root with owned admission and native drain.
    /// # Safety
    /// The caller must satisfy [`ReplayAdmission::observe_local_scoped`]'s controlled
    /// root and mathematical-state contract. This entry does not qualify arbitrary
    /// embedding, callbacks, interposition or executable-map mutation.
    /// # Errors
    /// Receiving refusal, cancellation, original deadline or resource admission failure.
    #[allow(
        unsafe_code,
        reason = "ADR-0164 actual composition roots mint local receiving admission through a completion-owned job"
    )]
    pub async unsafe fn observe_local_runtime(
        self: &Arc<Self>,
        role: ExpectedProducerTarget,
        anchor: usize,
        configuration: RuntimeConfigurationObserver,
        deadline: std::time::Instant,
        driver: &crate::CancelSource,
    ) -> Result<ReplayAdmission, PortableError> {
        let control = pse_diagnostics::Cancellation::default();
        let operation = self.job_retained_scoped(
            1,
            OBSERVATION_WORK_BYTES,
            control.clone(),
            Some(deadline),
            move |flag| {
                let observed_role = match receiving_checkpoint(&flag, Some(deadline))
                    .and_then(|()| local_role(role))
                {
                    Ok(role) => role,
                    Err(error) => return Ok((Err(error), 0)),
                };
                // The callback itself is bounded by the unsafe composition-root
                // contract and runs on this completion-owned native worker.
                let bytes = match observe_configuration(&configuration) {
                    Ok(bytes) => bytes,
                    Err(error) => return Ok((Err(error), 0)),
                };
                if let Err(error) = receiving_checkpoint(&flag, Some(deadline)) {
                    return Ok((Err(error), 0));
                }
                if let Some(local) = candidate(observed_role, anchor, &bytes) {
                    // Evidence only. No observation or receiving permission is
                    // asserted here; acquisition/publication must still qualify.
                    return Ok((Ok((ReplayAdmission::Local(local), false)), 0));
                }
                drop(bytes);
                // SAFETY: the actual root supplies the same controlled contract as
                // the synchronous mint; the job owns stop, accounting and final join.
                let observed = unsafe {
                    ReplayAdmission::observe_local_scoped(
                        role,
                        anchor,
                        configuration,
                        &flag,
                        Some(deadline),
                    )
                };
                let bytes = observed.as_ref().map_or(0, |admission| match admission {
                    ReplayAdmission::Local(local) => {
                        size_of::<LocalReplay>()
                            + 256
                            + local.observation.retained_bytes()
                            + local.configuration_bytes.capacity()
                            + local.configuration_environment.retained_bytes()
                    }
                    ReplayAdmission::QualifiedProducer(_) => size_of::<ReplayAdmission>(),
                });
                Ok((observed.map(|admission| (admission, true)), bytes))
            },
        );
        tokio::pin!(operation);
        let result = tokio::select! {
            biased;
            () = driver.cancelled() => {
                control.cancel(); let _ = operation.await;
                return Err(pse_math::MathError::Cancelled.into());
            },
            result = &mut operation => result,
        };
        let (admission, owner) = result.map_err(|error| {
            PortableError::Math(pse_math::MathError::Typed {
                retained: size_of_val(&error),
                cause: pse_model::diagnostic::DiagnosticCause::new(error),
            })
        })?;
        let (mut admission, fresh) = admission?;
        if fresh && let ReplayAdmission::Local(local) = &mut admission {
            Arc::get_mut(local)
                .ok_or_else(|| {
                    PortableError::Qualification(
                        "startup observation ownership escaped before admission".into(),
                    )
                })?
                ._owner = Some(owner);
            // Publish only after attaching the actual retained allocation. Hits
            // retain this original owner, even across another Runtime's pool.
            retain_candidate(local);
        }
        Ok(admission)
    }
}

async fn startup_observation_job<T: Send + 'static>(
    service: &Arc<super::MathService>,
    bytes: usize,
    deadline: std::time::Instant,
    driver: &crate::CancelSource,
    work: impl FnOnce(Arc<AtomicBool>) -> Result<(T, usize), PortableError> + Send + 'static,
) -> Result<(T, Arc<pse_columnar::AllocationLease>), PortableError> {
    let control = pse_diagnostics::Cancellation::default();
    let operation =
        service.job_retained_scoped(1, bytes, control.clone(), Some(deadline), move |flag| {
            // Keep the complete work envelope through native join, then transfer
            // only the result's escaped extent without release/reacquisition.
            match work(flag.flag()) {
                Ok((value, retained)) => Ok((Ok(value), retained)),
                Err(error) => Ok((Err(error), 0)),
            }
        });
    tokio::pin!(operation);
    let result = tokio::select! {
        biased;
        () = driver.cancelled() => {
            control.cancel();
            let _ = operation.await;
            return Err(pse_math::MathError::Cancelled.into());
        },
        result = &mut operation => result,
    };
    let (result, owner) = result.map_err(|error| {
        PortableError::Math(pse_math::MathError::Typed {
            retained: size_of_val(&error),
            cause: pse_model::diagnostic::DiagnosticCause::new(error),
        })
    })?;
    Ok((result?, owner))
}

fn read_startup_receipt(
    path: &std::path::Path,
    extent: usize,
    checkpoint: &dyn Fn() -> std::io::Result<()>,
) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    checkpoint()?;
    let mut file = std::fs::File::open(path)?;
    let before = file.metadata()?;
    let canonical = path.canonicalize()?;
    if !before.is_file() || before.len() != extent as u64 {
        return Err(std::io::Error::other(
            "startup receipt changed after preflight",
        ));
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(extent)
        .map_err(|_| std::io::Error::other("startup receipt allocation refused"))?;
    let mut buffer = [0_u8; 65_536];
    loop {
        checkpoint()?;
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        if count > extent.saturating_sub(bytes.len()) {
            return Err(std::io::Error::other(
                "startup receipt grew after preflight",
            ));
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    checkpoint()?;
    if bytes.len() != extent {
        return Err(std::io::Error::other(
            "startup receipt changed after preflight",
        ));
    }
    let after = file.metadata()?;
    let current = std::fs::metadata(path)?;
    if before.len() != after.len()
        || before.modified()? != after.modified()?
        || canonical != path.canonicalize()?
    {
        return Err(std::io::Error::other(
            "startup receipt changed during capture",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let identity =
            |m: &std::fs::Metadata| (m.dev(), m.ino(), m.mode(), m.ctime(), m.ctime_nsec());
        if identity(&before) != identity(&after) || identity(&before) != identity(&current) {
            return Err(std::io::Error::other(
                "startup receipt replaced during capture",
            ));
        }
    }
    #[cfg(not(unix))]
    let _ = current;
    checkpoint()?;
    Ok(bytes)
}

async fn native_receiving<T: Send + 'static>(
    service: &Arc<super::MathService>,
    cancelled: &Arc<AtomicBool>,
    deadline: Option<std::time::Instant>,
    bytes: usize,
    operation: impl FnOnce(Arc<AtomicBool>) -> Result<T, PortableError> + Send + 'static,
) -> Result<T, PortableError> {
    let control = pse_diagnostics::Cancellation::default();
    let work = service.job_scoped(1, bytes, control.clone(), deadline, move |flag| {
        operation(flag.flag()).map_err(runtime_error)
    });
    let stop = async {
        loop {
            if cancelled.load(std::sync::atomic::Ordering::Acquire) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    };
    tokio::pin!(work);
    let result = tokio::select! {
        biased;
        () = stop => { control.cancel(); let _ = work.await; return Err(pse_math::MathError::Cancelled.into()); },
        result = &mut work => result,
    };
    result.map_err(|error| {
        PortableError::Math(pse_math::MathError::Typed {
            retained: size_of_val(&error),
            cause: pse_model::diagnostic::DiagnosticCause::new(error),
        })
    })
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    interpretation: String,
    semantic_identity: SemanticBodyHash,
    // Complete receiving deployment context; spec.physical seals consumed premises.
    physical: ContentHash,
    spec: pse_math::binding::BodySpec,
    payload_hash: ContentHash,
}
// Keep the compiler payload as opaque bytes. Nesting it as a JSON number array
// would duplicate and expand the same semantic description at the transport boundary.
const ENVELOPE_MAGIC: &[u8] = b"pse-admitted-body-product-v4\0";
// Framing metadata is bounded independently of the scientific recipe and storage wire blocks.
const MAX_METADATA_BYTES: usize = 1024 * 1024;
const MAX_PRODUCT_BYTES: usize = ENVELOPE_MAGIC.len()
    + 4
    + MAX_METADATA_BYTES
    + pse_ids::scientific_replay::MAX_ADMITTED_RECIPE_BYTES;
fn pack(envelope: &Envelope, payload: &[u8]) -> Result<Vec<u8>, PortableError> {
    let metadata = serde_json::to_vec(envelope)
        .map_err(|error| PortableError::Qualification(error.to_string()))?;
    let length = u32::try_from(metadata.len()).map_err(|_| {
        PortableError::Qualification("portable metadata exceeds frame bound".into())
    })?;
    let total = ENVELOPE_MAGIC
        .len()
        .saturating_add(4)
        .saturating_add(metadata.len())
        .saturating_add(payload.len());
    if metadata.len() > MAX_METADATA_BYTES
        || payload.len() > pse_ids::scientific_replay::MAX_ADMITTED_RECIPE_BYTES
    {
        return Err(PortableError::Qualification(
            "portable envelope exceeds its logical extent".into(),
        ));
    }
    let mut bytes = Vec::with_capacity(total);
    bytes.extend_from_slice(ENVELOPE_MAGIC);
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(&metadata);
    bytes.extend_from_slice(payload);
    Ok(bytes)
}
// Validate framing and exact bounded slices before parsing metadata or constructing record authority.
fn split_frame(bytes: &[u8]) -> Result<(&[u8], &[u8]), PortableError> {
    if bytes.len() > MAX_PRODUCT_BYTES {
        return Err(PortableError::Qualification(
            "portable envelope exceeds its logical extent".into(),
        ));
    }
    let bytes = bytes.strip_prefix(ENVELOPE_MAGIC).ok_or_else(|| {
        PortableError::Qualification("unsupported portable envelope framing".into())
    })?;
    let length = bytes
        .get(..4)
        .ok_or_else(|| PortableError::Qualification("truncated portable metadata length".into()))?;
    let length = u32::from_be_bytes(
        length
            .try_into()
            .map_err(|_| PortableError::Qualification("invalid portable metadata length".into()))?,
    ) as usize;
    if length > MAX_METADATA_BYTES {
        return Err(PortableError::Qualification(
            "portable metadata exceeds frame bound".into(),
        ));
    }
    let data = &bytes[4..];
    let metadata = data
        .get(..length)
        .ok_or_else(|| PortableError::Qualification("truncated portable metadata".into()))?;
    let payload = &data[length..];
    pse_ids::scientific_replay::QualifiedScientificRecipe::scratch_bytes(payload.len())
        .map_err(|error| PortableError::Qualification(error.to_string()))?;
    Ok((metadata, payload))
}
fn unpack(bytes: &[u8]) -> Result<(Envelope, &[u8]), PortableError> {
    let (metadata, payload) = split_frame(bytes)?;
    let envelope = serde_json::from_slice(metadata)
        .map_err(|error| PortableError::Qualification(error.to_string()))?;
    Ok((envelope, payload))
}
fn reserve_portable(
    service: &super::MathService,
    label: &'static str,
    bytes: usize,
) -> Result<Arc<pse_columnar::AllocationLease>, PortableError> {
    service.reserve(label, bytes).map_err(|error| {
        let (retained, cause) = match error.retained_bytes() {
            Ok(retained) => (retained, pse_model::diagnostic::DiagnosticCause::new(error)),
            Err(refusal) => (
                refusal.retained_bytes(),
                pse_model::diagnostic::DiagnosticCause::new(refusal),
            ),
        };
        PortableError::Math(pse_math::MathError::Typed { retained, cause })
    })
}
/// Exact portable product key admitted under the selected revision's durable root.
#[derive(Clone, Debug)]
pub struct PublishedBody {
    key: String,
    semantic_identity: SemanticBodyHash,
}
impl PublishedBody {
    /// Immutable canonical product identity.
    pub fn key(&self) -> &str {
        &self.key
    }
    /// Compiler-issued body identity authenticated by the published product.
    pub fn semantic_identity(&self) -> SemanticBodyHash {
        self.semantic_identity
    }
}
fn request(identity: SemanticBodyHash, physical: ContentHash) -> Vec<u8> {
    let mut hash = FramedHasher::new(Frame::MathAdmittedRecipeV1);
    hash.str(INTERPRETATION)
        .str("request")
        .hash(&identity.as_id())
        .hash(&physical);
    hash.finish_hash().as_bytes().to_vec()
}
fn encode(
    body: &AdmittedBody,
    deployment_physical: ContentHash,
) -> Result<(Envelope, Vec<u8>), PortableError> {
    let semantic_identity = body.semantic_identity().ok_or_else(|| {
        PortableError::Qualification("body lacks compiler-issued selected semantic identity".into())
    })?;
    let payload = body.portable_payload()?;
    Ok((
        Envelope {
            interpretation: INTERPRETATION.into(),
            semantic_identity,
            physical: deployment_physical,
            spec: body.spec().clone(),
            payload_hash: portable_payload_hash(&payload),
        },
        payload,
    ))
}
/// Publish one admitted body and the selected read premises in the canonical substrate.
/// The storage owner establishes the immutable product/root. Selection protection stays
/// live until the caller explicitly releases it after publishing the complete selection.
/// # Errors
/// Unsealed body, incomplete receipts, changed physical context or guarded store refusal.
pub async fn publish_body(
    service: &Arc<super::MathService>,
    store: &CanonicalStore,
    read: &SelectedRead,
    producer: &ReplayAdmission,
    body: &AdmittedBody,
    context: &CompilerContext,
) -> Result<PublishedBody, PortableError> {
    let deadline = std::time::Instant::now() + pse_operations::canonical::REQUEST_TIMEOUT;
    let namespace = qualify_replay(
        service,
        Some(producer.clone()),
        deadline,
        &crate::CancelSource::new(),
    )
    .await
    .map_err(|error| {
        PortableError::Math(pse_math::MathError::Typed {
            retained: size_of::<super::MathRuntimeError>(),
            cause: pse_model::diagnostic::DiagnosticCause::new(error),
        })
    })?
    .ok_or_else(|| PortableError::Qualification("receiving deployment changed".into()))?;
    let description = describe_body(
        service,
        read,
        selection_material(service, read)?,
        namespace.producer_key().to_owned(),
        namespace.producer_class(),
        body,
        context,
    )?;
    let key = publish_description(service, store, read, &description, deadline.into()).await?;
    Ok(PublishedBody {
        key,
        semantic_identity: description.semantic_identity,
    })
}
/// Persist admitted meaning when producer eligibility is incomplete. The outer build
/// is recorded as unqualified provenance and never used as a relevant producer key.
pub async fn publish_unqualified_body(
    service: &super::MathService,
    store: &CanonicalStore,
    read: &SelectedRead,
    outer_build: ContentHash,
    body: &AdmittedBody,
    context: &CompilerContext,
) -> Result<PublishedBody, PortableError> {
    publish_with_producer_key(
        service,
        store,
        read,
        format!("pse.unqualified-producer.v1:{}", outer_build.to_hex()),
        body,
        context,
    )
    .await
}
#[allow(
    unsafe_code,
    reason = "ADR-0164 compiler-issued scientific description publication in reserved namespace"
)]
async fn publish_with_producer_key(
    service: &super::MathService,
    store: &CanonicalStore,
    read: &SelectedRead,
    producer: String,
    body: &AdmittedBody,
    context: &CompilerContext,
) -> Result<PublishedBody, PortableError> {
    let description = describe_body(
        service,
        read,
        selection_material(service, read)?,
        producer,
        pse_model::generated::enums::CanonicalProducerClass::General,
        body,
        context,
    )?;
    let key = publish_description(
        service,
        store,
        read,
        &description,
        tokio::time::Instant::now() + pse_operations::canonical::REQUEST_TIMEOUT,
    )
    .await?;
    Ok(PublishedBody {
        key,
        semantic_identity: description.semantic_identity,
    })
}
/// Owned exact recipe/envelope encoding. This carries scientific meaning, not permission.
#[derive(Debug)]
pub(super) struct EncodedBody {
    semantic_identity: SemanticBodyHash,
    deployment_physical: ContentHash,
    request: Vec<u8>,
    payload: Arc<Vec<u8>>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl EncodedBody {
    pub(super) fn retained_bytes(&self) -> usize {
        size_of::<Self>() + self.request.capacity() + self.payload.capacity()
    }
}
#[derive(Debug)]
pub(crate) struct BodyDescription {
    pub(crate) description: pse_operations::canonical_selection::ProductDescription,
    pub(crate) semantic_identity: SemanticBodyHash,
    _content_owner: Arc<pse_columnar::AllocationLease>,
    _owner: Arc<pse_columnar::AllocationLease>,
}
/// Precharge and capture the complete post-merge receipt once for a body inventory.
pub(crate) fn selection_material(
    service: &super::MathService,
    read: &SelectedRead,
) -> Result<Arc<pse_operations::canonical_selection::SelectionMaterial>, PortableError> {
    let owner = reserve_portable(
        service,
        "math:canonical-selection-material",
        pse_operations::canonical_selection::SelectionMaterial::allocation_bytes(read)?,
    )?;
    Ok(pse_operations::canonical_selection::SelectionMaterial::capture(read, owner)?)
}
fn encoded_body(
    service: &super::MathService,
    body: &AdmittedBody,
    context: &CompilerContext,
) -> Result<Arc<EncodedBody>, PortableError> {
    let identity = body.semantic_identity().ok_or_else(|| {
        PortableError::Qualification(
            "portable encoding requires a compiler-sealed semantic body".into(),
        )
    })?;
    let physical = physical_identity(&context.quantities, &context.preconditions);
    if let Some(encoded) = service.modeling_cache.encoded_body(identity)
        && encoded.deployment_physical == physical
    {
        return Ok(encoded);
    }
    let generation = service.modeling_cache.generation();
    let extent = body.portable_payload_bytes()?;
    let _scratch = reserve_portable(
        service,
        "math:qualified-recipe-encode",
        extent
            .saturating_mul(3)
            .saturating_add(MAX_METADATA_BYTES.saturating_mul(3)),
    )?;
    let (envelope, body_payload) = encode(body, physical)?;
    let request = request(envelope.semantic_identity, physical);
    let payload = pack(&envelope, &body_payload)?;
    let owner = reserve_portable(
        service,
        "math:retained-recipe-encoding",
        size_of::<EncodedBody>() + request.capacity() + payload.capacity(),
    )?;
    let encoded = Arc::new(EncodedBody {
        semantic_identity: identity,
        deployment_physical: physical,
        request,
        payload: Arc::new(payload),
        _owner: owner,
    });
    service
        .modeling_cache
        .retain_encoded_body(generation, identity, encoded.clone());
    Ok(encoded)
}
pub(crate) fn describe_body(
    service: &super::MathService,
    read: &SelectedRead,
    selection: Arc<pse_operations::canonical_selection::SelectionMaterial>,
    producer: String,
    producer_class: pse_model::generated::enums::CanonicalProducerClass,
    body: &AdmittedBody,
    context: &CompilerContext,
) -> Result<Arc<BodyDescription>, PortableError> {
    let encoded = encoded_body(service, body, context)?;
    let _scratch = reserve_portable(
        service,
        "math:canonical-description-encode",
        MAX_METADATA_BYTES.saturating_add(encoded.request.len()),
    )?;
    let mut hash = FramedHasher::new(Frame::MathAdmittedRecipeV1);
    hash.str(INTERPRETATION)
        .str("product")
        .part(&encoded.request)
        .str(&producer)
        .str(producer_class.as_str())
        .part(&encoded.payload);
    let revision = read.selection().revision();
    let product = pse_model::generated::runtime::canonical_products::Row {
        key: hash.finish_hash().to_hex(),
        problem: revision.problem.clone(),
        revision: revision.key.clone(),
        request: encoded.request.clone().into(),
        payload: Vec::new().into(),
        dependencies: Vec::new().into(),
        producer: producer.clone(),
        producer_class,
        interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
    };
    let description =
        pse_operations::canonical_selection::ProductDescription::prepare_with_shared_payload(
            read,
            product,
            selection,
            encoded.payload.clone(),
            encoded.clone(),
        )?;
    let content_owner = reserve_portable(
        service,
        "math:retained-canonical-description",
        description.descriptor_allocation_bytes() + producer.capacity() + 128,
    )?;
    let owner = reserve_portable(
        service,
        "math:canonical-description-binding",
        size_of::<BodyDescription>() + 128,
    )?;
    Ok(Arc::new(BodyDescription {
        description,
        semantic_identity: encoded.semantic_identity,
        _owner: owner,
        _content_owner: content_owner,
    }))
}
#[allow(
    unsafe_code,
    reason = "ADR-0164 compiler-sealed description and actual receiving admission are required by this private publication owner"
)]
pub(crate) async fn publish_description(
    service: &super::MathService,
    store: &CanonicalStore,
    read: &SelectedRead,
    description: &BodyDescription,
    deadline: tokio::time::Instant,
) -> Result<String, PortableError> {
    // Callers may use original provenance only for acknowledgment. New eligible
    // publication must establish a current namespace before constructing this value.
    let publication = || async {
        if description.description.producer_class()
            != pse_model::generated::enums::CanonicalProducerClass::General
        {
            // SAFETY: this private path is reached only after current receiving admission;
            // descriptions are compiler-sealed and bound to the consumer's exact read.
            unsafe {
                store.publish_scientific_description_until(read, &description.description, deadline)
            }
            .await
        } else {
            store
                .publish_description_until(read, &description.description, deadline)
                .await
        }
    };
    Ok(charged_publication(service, publication).await??)
}

// Publication carries nested clock/activity and staging futures. Keep that
// phase off its caller's preparation stack while charging its complete heap
// state to the same scientific runtime until the original operation settles.
// The callback remains uncalled until admission, including on allocation refusal.
async fn charged_publication<F, Fut>(
    service: &super::MathService,
    publication: F,
) -> Result<Fut::Output, PortableError>
where
    F: FnOnce() -> Fut,
    Fut: Future + Send,
{
    let _owner = reserve_portable(
        service,
        "math:canonical-publication-future",
        size_of::<Fut>(),
    )?;
    Ok(publication().boxed().await)
}
/// Return only a dependency-qualified persisted product, reconstructed from its actual
/// concrete receipts. A hit with corrupt or incomplete receipts is an error; it never
/// runs normal admission and is never reported as a reconstruction success.
/// # Errors
/// Storage/protection failure, incompatible envelope, missing receipts or native error.
pub async fn reuse_body(
    service: &Arc<super::MathService>,
    store: &CanonicalStore,
    read: &mut SelectedRead,
    producer: &ReplayAdmission,
    identity: SemanticBodyHash,
    context: &CompilerContext,
    cancelled: &Arc<AtomicBool>,
) -> Result<Option<AdmittedBody>, PortableError> {
    if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(pse_math::MathError::Cancelled.into());
    }
    let after = String::new();
    if !product_candidate_presence(store, read, producer, identity, context, &after).await? {
        return Ok(None);
    }
    if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(pse_math::MathError::Cancelled.into());
    }
    let deadline = std::time::Instant::now() + pse_operations::canonical::REQUEST_TIMEOUT;
    let admission = producer.clone();
    let namespace = native_receiving(
        service,
        cancelled,
        Some(deadline),
        OBSERVATION_WORK_BYTES,
        move |flag| admission.qualify_receiving(&flag, Some(deadline)),
    )
    .await?;
    let Some(namespace) = namespace else {
        return Ok(None);
    };
    if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(pse_math::MathError::Cancelled.into());
    }
    reuse_body_current(
        service, store, read, &namespace, identity, context, cancelled,
    )
    .await
}

/// Probe only for a bounded protected candidate row without decoding its key or payload.
/// A cold miss needs no local runtime observation; a present candidate still requires full
/// current-context admission.
pub(crate) async fn product_candidate_presence(
    store: &CanonicalStore,
    read: &SelectedRead,
    producer: &ReplayAdmission,
    identity: SemanticBodyHash,
    context: &CompilerContext,
    after: &str,
) -> Result<bool, PortableError> {
    let physical = physical_identity(&context.quantities, &context.preconditions);
    Ok(store
        .product_candidate_presence(
            read.selection(),
            &request(identity, physical),
            &producer.key(),
            producer.producer_class(),
            after,
        )
        .await?)
}

#[allow(
    unsafe_code,
    reason = "ADR-0164 sole production storage-to-science authority mint after canonical qualification"
)]
pub(crate) async fn reuse_body_current(
    service: &Arc<super::MathService>,
    store: &CanonicalStore,
    read: &mut SelectedRead,
    namespace: &ValidatedReplayNamespace,
    identity: SemanticBodyHash,
    context: &CompilerContext,
    cancelled: &Arc<AtomicBool>,
) -> Result<Option<AdmittedBody>, PortableError> {
    let physical = physical_identity(&context.quantities, &context.preconditions);
    let mut after = String::new();
    loop {
        receiving_checkpoint(cancelled, namespace.deadline)?;
        let Some(candidate) = store
            .product_candidate(
                read.selection(),
                &request(identity, physical),
                namespace.producer_key(),
                namespace.producer_class(),
                &after,
            )
            .await?
        else {
            receiving_checkpoint(cancelled, namespace.deadline)?;
            return Ok(None);
        };
        receiving_checkpoint(cancelled, namespace.deadline)?;
        let (product, _product_scratch) =
            qualify_candidate(service, store, read, &candidate).await?;
        receiving_checkpoint(cancelled, namespace.deadline)?;
        let Some(product) = product else {
            after = candidate.key().to_owned();
            continue;
        };
        // Reserve before envelope/recipe parsing or record-index allocation. The lease
        // remains live through strict reconstruction; decoded records never outlive it.
        let (metadata, payload) = split_frame(product.payload())?;
        let scratch =
            pse_ids::scientific_replay::QualifiedScientificRecipe::scratch_bytes(payload.len())
                .map_err(|e| PortableError::Qualification(e.to_string()))?
                .saturating_add(metadata.len().saturating_mul(256));
        let scratch_owner = reserve_portable(service, "math:qualified-recipe-replay", scratch)?;
        let (envelope, body_payload) = unpack(product.payload())?;
        if envelope.interpretation != INTERPRETATION
            || envelope.semantic_identity != identity
            || envelope.physical != physical
        {
            return Err(PortableError::Qualification(
                "persisted body differs from qualified selected request".into(),
            ));
        }
        let qualification = ContentHash::parse_hex(product.key())
            .map_err(|e| PortableError::Qualification(e.to_string()))?;
        let selected_request = ContentHash::try_from_slice(&request(identity, physical))
            .map_err(|e| PortableError::Qualification(e.to_string()))?;
        // SAFETY: ReusableProduct is issued only by the protected canonical selection
        // after complete dependency/eligible-producer/interpretation qualification.
        // This controlled scientific publication role emits compiler-sealed descriptions;
        // arbitrary generic product writers must not impersonate that deployment role.
        let permit = unsafe {
            pse_ids::scientific_replay::QualifiedScientificRecipe::from_canonical_product(
                body_payload.to_vec(),
                envelope.payload_hash,
                qualification,
                selected_request,
            )
        }
        .map_err(|e| PortableError::Qualification(e.to_string()))?;
        // Only immutable owned mathematical values leave this bounded loader scope.
        // Provider workers/evaluators remain downstream fresh execution state.
        let current = namespace.clone();
        let quantities = context.quantities.clone();
        let product_owner = service.clone();
        let body = native_receiving(
            service,
            cancelled,
            namespace.deadline,
            super::WITHIN_WORKSPACE,
            move |flag| {
                let _scratch = scratch_owner;
                let _payload = _product_scratch;
                receiving_checkpoint(&flag, current.deadline)?;
                pse_math::initialize()?;
                receiving_checkpoint(&flag, current.deadline)?;
                let body = current.reconstruct(&flag, || {
                    let body = reconstruct_portable(&permit, &quantities, &flag)?;
                    if body.spec() != &envelope.spec {
                        return Err(PortableError::Qualification(
                            "reconstructed body differs from qualified envelope specification"
                                .into(),
                        ));
                    }
                    if body.semantic_identity() != Some(identity) {
                        return Err(PortableError::Qualification(
                            "reconstructed body lacks exact sealed semantic identity".into(),
                        ));
                    }
                    Ok(body)
                })?;
                let owned = product_owner
                    .own_semantic_body(Arc::new(body))
                    .map_err(|error| {
                        PortableError::Math(pse_math::MathError::Typed {
                            retained: size_of_val(&error),
                            cause: pse_model::diagnostic::DiagnosticCause::new(error),
                        })
                    })?;
                Ok(owned.as_ref().clone())
            },
        )
        .await?;
        return Ok(Some(body));
    }
}

// A store-issued descriptor is the only path into body hydration. Keep this lease
// together with the qualified payload until its scientific reconstruction finishes.
async fn qualify_candidate(
    service: &super::MathService,
    store: &CanonicalStore,
    read: &mut SelectedRead,
    candidate: &pse_operations::canonical_selection::ProductCandidate,
) -> Result<
    (
        Option<pse_operations::canonical_selection::ReusableProduct>,
        Arc<pse_columnar::AllocationLease>,
    ),
    PortableError,
> {
    let lease = reserve_portable(
        service,
        "math:qualified-product-hydration",
        candidate.retained_bytes(),
    )?;
    let product = store.qualify_product(read, candidate).await?;
    Ok((product, lease))
}

#[cfg(test)]
mod portable_frame_tests {
    use super::*;
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    #[tokio::test]
    #[allow(
        unsafe_code,
        reason = "actual native test root excludes current-environment writers during its owned snapshots"
    )]
    async fn shared_replay_candidate_retains_original_charge_and_requires_receiving_qualification()
    {
        fn anchor() {}
        let anchor = anchor as *const () as usize;
        let original = super::super::tests::service();
        let other = super::super::tests::service();
        let original_pool = original.pool.clone();
        let other_baseline = other.pool.reserved();
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counted = calls.clone();
        let observe: RuntimeConfigurationObserver = Arc::new(move || {
            counted.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            // SAFETY: this actual controlled test root has no environment writers.
            unsafe { observed_test_configuration(b"charged-candidate-control".to_vec()) }
        });
        let driver = crate::CancelSource::new();
        let deadline = std::time::Instant::now() + ReplayAdmission::STARTUP_OBSERVATION_LIMIT;
        // SAFETY: genuine test executable anchor and complete controlled configuration.
        let first = unsafe {
            original.observe_local_runtime(
                ExpectedProducerTarget::WORKER,
                anchor,
                observe.clone(),
                deadline,
                &driver,
            )
        }
        .await
        .unwrap();
        let retained = original_pool.reserved();
        assert!(retained > 0);
        let captured = calls.load(std::sync::atomic::Ordering::Relaxed);
        // SAFETY: same actual root, with a separate service and allocation pool.
        let second = unsafe {
            other.observe_local_runtime(
                ExpectedProducerTarget::WORKER,
                anchor,
                observe.clone(),
                deadline,
                &driver,
            )
        }
        .await
        .unwrap();
        let (ReplayAdmission::Local(first_local), ReplayAdmission::Local(second_local)) =
            (&first, &second)
        else {
            panic!("actual local observations")
        };
        assert!(Arc::ptr_eq(first_local, second_local));
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::Relaxed),
            captured + 1
        );
        assert_eq!(original_pool.reserved(), retained);
        assert_eq!(other.pool.reserved(), other_baseline);
        // Evidence sharing does not replace the full completion-owned receiving check.
        assert!(
            qualify_replay(&other, Some(second.clone()), deadline, &driver)
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            calls.load(std::sync::atomic::Ordering::Relaxed),
            captured + 3
        );
        // SAFETY: actual owned snapshot under the same no-writers test lifetime.
        let complete =
            unsafe { observed_test_configuration(b"charged-candidate-control".to_vec()) }.unwrap();
        assert!(candidate(pse_buildinfo::LocalRuntimeRole::Python, anchor, &complete).is_none());
        assert!(
            candidate(
                pse_buildinfo::LocalRuntimeRole::Worker,
                anchor + 1,
                &complete
            )
            .is_none()
        );
        // SAFETY: complete but different supported configuration, not a new permission.
        let changed =
            unsafe { observed_test_configuration(b"different-candidate-control".to_vec()) }
                .unwrap();
        assert!(candidate(pse_buildinfo::LocalRuntimeRole::Worker, anchor, &changed).is_none());
        let weak = Arc::downgrade(second_local);
        drop(first);
        drop(original);
        assert_eq!(original_pool.reserved(), retained);
        drop(second);
        assert!(weak.upgrade().is_none());
        assert_eq!(original_pool.reserved(), 0);
        assert!(candidate(pse_buildinfo::LocalRuntimeRole::Worker, anchor, &complete).is_none());
    }
    #[tokio::test]
    #[allow(
        unsafe_code,
        reason = "outer startup fixtures deliberately select no qualifying receipt"
    )]
    async fn deployment_startup_retains_its_observation_owner_and_truthful_receipt_outcomes() {
        let directory = tempfile::tempdir().unwrap();
        let artifact = directory.path().join("installed-native");
        std::fs::write(&artifact, b"actual isolated artifact").unwrap();
        let receipt = directory.path().join("receipt.json");
        std::fs::write(
            &receipt,
            serde_json::to_vec(&serde_json::json!({
                "receipt_version":2,"frame":Frame::ProducerV1.as_str(),"native_abi":null,
                "deployment":null,"identity":ContentHash::from_bytes([1;32]).to_hex(),
                "persistent_reuse_eligible":false,"reasons":["unqualified fixture"]
            }))
            .unwrap(),
        )
        .unwrap();
        let local_service = super::super::tests::service();
        let local_baseline = local_service.pool.reserved();
        let driver = crate::CancelSource::new();
        let deadline = std::time::Instant::now() + ReplayAdmission::STARTUP_OBSERVATION_LIMIT;
        // Receipt-free construction keeps its small observation reservation. The
        // same pool must truthfully refuse the larger strict namespace envelope.
        // SAFETY: this isolated artifact and absent receipt cannot grant a producer.
        let local = unsafe {
            local_service.observe_deployment_runtime(
                artifact.clone(),
                None,
                ExpectedProducerTarget::PYTHON,
                deadline,
                &driver,
            )
        }
        .await
        .unwrap();
        assert!(local.producer.is_none());
        let local_retained = local_service.pool.reserved() - local_baseline;
        assert!(local_retained > local.outer.git_sha.capacity());
        assert!(local_retained < OBSERVATION_WORK_BYTES);
        drop(local);
        assert_eq!(local_service.pool.reserved(), local_baseline);
        assert!(
            // SAFETY: this controlled receipt explicitly refuses producer eligibility.
            unsafe {
                local_service.observe_deployment_runtime(
                    artifact.clone(),
                    Some(receipt.clone()),
                    ExpectedProducerTarget::PYTHON,
                    deadline,
                    &driver,
                )
            }
            .await
            .is_err()
        );
        assert_eq!(local_service.pool.reserved(), local_baseline);
        let (service, _cache) =
            super::super::tests::service_with_policy(1 << 30, local_service.policy.clone());
        let baseline = service.pool.reserved();
        // SAFETY: the controlled fixture's receipt explicitly refuses eligibility;
        // no scientific permission is minted from its artificial artifact path.
        let observed = unsafe {
            service.observe_deployment_runtime(
                artifact.clone(),
                Some(receipt.clone()),
                ExpectedProducerTarget::PYTHON,
                deadline,
                &driver,
            )
        }
        .await
        .unwrap();
        assert!(observed.outer.source.is_none());
        assert_eq!(observed.outer.git_sha, "source-unavailable");
        assert!(observed.producer.is_none());
        assert_eq!(service.pool.reserved() - baseline, local_retained);
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
        assert_eq!(service.cpu.available_permits(), service.cores);
        let escaped = observed.clone();
        drop(observed);
        assert!(service.pool.reserved() > baseline);
        drop(escaped);
        assert_eq!(service.pool.reserved(), baseline);
        std::fs::write(&receipt, b"not a receipt").unwrap();
        assert!(
            // SAFETY: invalid controlled fixture bytes cannot grant a producer.
            unsafe {
                service.observe_deployment_runtime(
                    artifact,
                    Some(receipt),
                    ExpectedProducerTarget::PYTHON,
                    deadline,
                    &driver,
                )
            }
            .await
            .is_err()
        );
        assert_eq!(service.pool.reserved(), baseline);
    }

    #[tokio::test]
    async fn deployment_startup_transfers_only_escaped_result_after_native_join() {
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let driver = crate::CancelSource::new();
        let deadline = std::time::Instant::now() + ReplayAdmission::STARTUP_OBSERVATION_LIMIT;
        let (entered, witness) = tokio::sync::oneshot::channel();
        let (release, gate) = std::sync::mpsc::channel();
        let mut observation = Box::pin(startup_observation_job(
            &service,
            OBSERVATION_WORK_BYTES,
            deadline,
            &driver,
            move |_| {
                let escaped = Vec::<u8>::with_capacity(127);
                let retained = escaped.capacity();
                entered.send(()).unwrap();
                gate.recv().unwrap();
                Ok((escaped, retained))
            },
        ));
        assert!(futures_util::poll!(observation.as_mut()).is_pending());
        witness.await.unwrap();
        assert!(service.pool.reserved() - baseline >= OBSERVATION_WORK_BYTES);
        release.send(()).unwrap();
        let (escaped, owner) = observation.await.unwrap();
        assert_eq!(service.pool.reserved() - baseline, escaped.capacity());
        assert_eq!(owner.size(), escaped.capacity());
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
        assert_eq!(service.cpu.available_permits(), service.cores);
        let consumer = owner.clone();
        drop(owner);
        drop(escaped);
        assert_eq!(service.pool.reserved() - baseline, consumer.size());
        drop(consumer);
        assert_eq!(service.pool.reserved(), baseline);

        // The receipt-size preflight returns a scalar, so none of its complete
        // native allowance remains reserved during the next admitted operation.
        let (extent, owner) =
            startup_observation_job(&service, OBSERVATION_WORK_BYTES, deadline, &driver, |_| {
                Ok((172_usize, 0))
            })
            .await
            .unwrap();
        assert_eq!(extent, 172);
        assert_eq!(owner.size(), 0);
        assert_eq!(service.pool.reserved(), baseline);
    }

    #[tokio::test]
    async fn deployment_startup_preserves_original_expiry_before_native_work() {
        let service = super::super::tests::service();
        let driver = crate::CancelSource::new();
        let entered = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let calls = entered.clone();
        let baseline = service.pool.reserved();
        assert!(
            startup_observation_job(
                &service,
                OBSERVATION_WORK_BYTES,
                std::time::Instant::now(),
                &driver,
                move |_| {
                    calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    Ok(((), 0))
                }
            )
            .await
            .is_err()
        );
        assert_eq!(entered.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert!(!driver.token().is_cancelled());
        assert_eq!(service.pool.reserved(), baseline);
    }

    #[tokio::test]
    async fn deployment_startup_abandonment_retains_native_owner_until_join() {
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let driver = crate::CancelSource::new();
        let (entered, witness) = tokio::sync::oneshot::channel();
        let (release, gate) = std::sync::mpsc::channel();
        let deadline = std::time::Instant::now() + ReplayAdmission::STARTUP_OBSERVATION_LIMIT;
        let mut observe = Box::pin(startup_observation_job(
            &service,
            OBSERVATION_WORK_BYTES,
            deadline,
            &driver,
            move |flag| {
                entered.send(()).unwrap();
                gate.recv().unwrap();
                receiving_checkpoint(&flag, Some(deadline))?;
                Ok(((), 0))
            },
        ));
        assert!(futures_util::poll!(observe.as_mut()).is_pending());
        witness.await.unwrap();
        let charged = service.pool.reserved();
        drop(observe);
        assert!(!driver.token().is_cancelled());
        assert_eq!(service.pool.reserved(), charged);
        assert_eq!(service.jobs.available_permits(), service.policy.jobs - 1);
        assert_eq!(service.cpu.available_permits(), service.cores - 1);
        release.send(()).unwrap();
        let drained = service
            .jobs
            .clone()
            .acquire_many_owned(service.policy.jobs as u32)
            .await
            .unwrap();
        assert_eq!(service.pool.reserved(), baseline);
        assert_eq!(service.cpu.available_permits(), service.cores);
        drop(drained);
    }

    #[tokio::test]
    async fn deployment_startup_cancel_and_deadline_keep_native_charge_until_actual_drain() {
        for cancel in [true, false] {
            let service = super::super::tests::service();
            let baseline = service.pool.reserved();
            let driver = crate::CancelSource::new();
            let (entered, witness) = tokio::sync::oneshot::channel();
            let (release, gate) = std::sync::mpsc::channel();
            let deadline = std::time::Instant::now()
                + if cancel {
                    std::time::Duration::from_secs(10)
                } else {
                    std::time::Duration::from_millis(50)
                };
            let mut observe = Box::pin(startup_observation_job(
                &service,
                OBSERVATION_WORK_BYTES,
                deadline,
                &driver,
                move |flag| {
                    entered.send(()).unwrap();
                    gate.recv().unwrap();
                    receiving_checkpoint(&flag, Some(deadline))?;
                    Ok(((), 0))
                },
            ));
            assert!(futures_util::poll!(observe.as_mut()).is_pending());
            witness.await.unwrap();
            let charged = service.pool.reserved();
            assert!(charged > baseline);
            if cancel {
                driver.cancel();
            } else {
                tokio::time::sleep_until(deadline.into()).await;
            }
            assert!(
                futures_util::poll!(observe.as_mut()).is_pending(),
                "startup must join its actual worker"
            );
            assert_eq!(service.pool.reserved(), charged);
            assert_eq!(service.jobs.available_permits(), service.policy.jobs - 1);
            assert_eq!(service.cpu.available_permits(), service.cores - 1);
            release.send(()).unwrap();
            assert!(observe.await.is_err());
            assert_eq!(service.pool.reserved(), baseline);
            assert_eq!(service.jobs.available_permits(), service.policy.jobs);
            assert_eq!(service.cpu.available_permits(), service.cores);
            assert_eq!(driver.token().is_cancelled(), cancel);
        }
    }

    #[test]
    fn deployment_startup_receipt_checks_chunks_extent_and_original_stop() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("receipt");
        let bytes = vec![31; 3 * 65_536];
        std::fs::write(&path, &bytes).unwrap();
        assert_eq!(
            read_startup_receipt(&path, bytes.len(), &|| Ok(())).unwrap(),
            bytes
        );
        assert!(read_startup_receipt(&path, bytes.len() - 1, &|| Ok(())).is_err());
        let polls = std::sync::atomic::AtomicUsize::new(0);
        let error = read_startup_receipt(&path, bytes.len(), &|| {
            if polls.fetch_add(1, std::sync::atomic::Ordering::Relaxed) == 2 {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "startup stop",
                ))
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::Interrupted);
        assert_eq!(polls.load(std::sync::atomic::Ordering::Relaxed), 3);
    }

    #[allow(
        unsafe_code,
        reason = "controlled native test root excludes every environment writer during owned snapshot capture"
    )]
    fn gated_startup_configuration() -> (
        RuntimeConfigurationObserver,
        tokio::sync::oneshot::Receiver<()>,
        std::sync::mpsc::Sender<()>,
        Arc<std::sync::atomic::AtomicUsize>,
    ) {
        let (entered, witness) = tokio::sync::oneshot::channel();
        let (release, gate) = std::sync::mpsc::channel();
        let entered = Mutex::new(Some(entered));
        let gate = Mutex::new(gate);
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed_calls = calls.clone();
        let configuration: RuntimeConfigurationObserver = Arc::new(move || {
            observed_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if let Some(entered) = entered.lock().unwrap().take() {
                let _ = entered.send(());
                gate.lock().unwrap().recv().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::Interrupted, "startup gate abandoned")
                })?;
            }
            // SAFETY: this controlled test root has no environment writers during capture.
            unsafe { observed_test_configuration(Vec::new()) }
        });
        (configuration, witness, release, calls)
    }
    #[tokio::test]
    #[allow(
        unsafe_code,
        reason = "owned startup cancellation control supplies the actual test root"
    )]
    async fn async_local_replay_cancellation_joins_native_configuration_before_releasing_charge() {
        fn anchor() {}
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let driver = crate::CancelSource::new();
        let (configuration, entered, release, calls) = gated_startup_configuration();
        let deadline = std::time::Instant::now() + ReplayAdmission::STARTUP_OBSERVATION_LIMIT;
        // SAFETY: this actual test root supplies controlled configuration that only waits for its gate.
        let mut observe = Box::pin(unsafe {
            service.observe_local_runtime(
                ExpectedProducerTarget::WORKER,
                anchor as *const () as usize,
                configuration,
                deadline,
                &driver,
            )
        });
        assert!(futures_util::poll!(observe.as_mut()).is_pending());
        entered.await.unwrap();
        let charged = service.pool.reserved();
        assert!(charged > baseline);
        assert_eq!(service.jobs.available_permits(), service.policy.jobs - 1);
        assert_eq!(service.cpu.available_permits(), service.cores - 1);
        driver.cancel();
        assert!(
            futures_util::poll!(observe.as_mut()).is_pending(),
            "cancellation must join the gated native call"
        );
        assert_eq!(service.pool.reserved(), charged);
        assert_eq!(service.jobs.available_permits(), service.policy.jobs - 1);
        assert_eq!(service.cpu.available_permits(), service.cores - 1);
        release.send(()).unwrap();
        assert!(matches!(
            observe.await,
            Err(PortableError::Math(pse_math::MathError::Cancelled))
        ));
        assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(service.pool.reserved(), baseline);
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
        assert_eq!(service.cpu.available_permits(), service.cores);
    }
    #[tokio::test]
    #[allow(
        unsafe_code,
        reason = "abandoned startup control supplies the actual test root"
    )]
    async fn async_local_replay_abandoned_waiter_retains_job_and_charge_until_native_drain() {
        fn anchor() {}
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let driver = crate::CancelSource::new();
        let (configuration, entered, release, calls) = gated_startup_configuration();
        let deadline = std::time::Instant::now() + ReplayAdmission::STARTUP_OBSERVATION_LIMIT;
        // SAFETY: this actual test root supplies controlled gated configuration, with no loader callback.
        let mut observe = Box::pin(unsafe {
            service.observe_local_runtime(
                ExpectedProducerTarget::WORKER,
                anchor as *const () as usize,
                configuration,
                deadline,
                &driver,
            )
        });
        assert!(futures_util::poll!(observe.as_mut()).is_pending());
        entered.await.unwrap();
        let charged = service.pool.reserved();
        assert!(charged > baseline);
        drop(observe);
        assert!(
            !driver.token().is_cancelled(),
            "abandonment stops the owned job without cancelling the driver"
        );
        assert_eq!(service.pool.reserved(), charged);
        assert_eq!(service.jobs.available_permits(), service.policy.jobs - 1);
        assert_eq!(service.cpu.available_permits(), service.cores - 1);
        release.send(()).unwrap();
        // Acquiring every slot witnesses completion of the supervisor's native join,
        // including teardown and release of the pool and compute owners.
        let drained = service
            .jobs
            .clone()
            .acquire_many_owned(service.policy.jobs as u32)
            .await
            .unwrap();
        assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(service.pool.reserved(), baseline);
        assert_eq!(service.cpu.available_permits(), service.cores);
        drop(drained);
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
    }
    #[tokio::test]
    #[allow(
        unsafe_code,
        reason = "expired startup clock control supplies the actual test root"
    )]
    async fn async_local_replay_expired_original_clock_refuses_before_configuration() {
        fn anchor() {}
        let service = super::super::tests::service();
        let baseline = service.pool.reserved();
        let driver = crate::CancelSource::new();
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed_calls = calls.clone();
        let configuration: RuntimeConfigurationObserver = Arc::new(move || {
            observed_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            // SAFETY: this controlled test root has no environment writers during capture.
            unsafe { observed_test_configuration(Vec::new()) }
        });
        // SAFETY: this controlled test root must be refused before its observer is invoked.
        let error = unsafe {
            service.observe_local_runtime(
                ExpectedProducerTarget::WORKER,
                anchor as *const () as usize,
                configuration,
                std::time::Instant::now(),
                &driver,
            )
        }
        .await
        .unwrap_err();
        let PortableError::Math(pse_math::MathError::Typed { cause, .. }) = error else {
            panic!("expired startup must retain the native admission timeout: {error:?}");
        };
        assert!(matches!(
            cause
                .as_error()
                .downcast_ref::<super::super::MathRuntimeError>(),
            Some(super::super::MathRuntimeError::Solve(
                pse_backend_native::ProblemError::Limit {
                    kind: pse_backend_native::LimitKind::Time,
                    ..
                }
            ))
        ));
        assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert!(!driver.token().is_cancelled());
        assert_eq!(service.pool.reserved(), baseline);
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
        assert_eq!(service.cpu.available_permits(), service.cores);
    }
    #[test]
    #[allow(
        unsafe_code,
        reason = "scoped mint stop controls use the actual test code anchor"
    )]
    fn scoped_local_replay_precancellation_and_expiry_skip_configuration() {
        fn anchor() {}
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed_calls = calls.clone();
        let configuration: RuntimeConfigurationObserver = Arc::new(move || {
            observed_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            // SAFETY: this controlled test root has no environment writers during capture.
            unsafe { observed_test_configuration(Vec::new()) }
        });
        let cancelled = AtomicBool::new(true);
        // SAFETY: the actual test root supplies its own anchor and controlled configuration.
        let error = unsafe {
            ReplayAdmission::observe_local_scoped(
                ExpectedProducerTarget::WORKER,
                anchor as *const () as usize,
                configuration.clone(),
                &cancelled,
                None,
            )
        }
        .unwrap_err();
        assert!(matches!(
            error,
            PortableError::Math(pse_math::MathError::Cancelled)
        ));
        cancelled.store(false, std::sync::atomic::Ordering::Release);
        // SAFETY: same controlled root; an already expired clock must stop before observation.
        let error = unsafe {
            ReplayAdmission::observe_local_scoped(
                ExpectedProducerTarget::WORKER,
                anchor as *const () as usize,
                configuration,
                &cancelled,
                Some(std::time::Instant::now()),
            )
        }
        .unwrap_err();
        assert!(matches!(
            error,
            PortableError::Store(CanonicalError::Timeout)
        ));
        assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 0);
    }
    #[test]
    #[allow(
        unsafe_code,
        reason = "scoped mint cancellation control uses the actual test code anchor"
    )]
    fn scoped_local_replay_configuration_cancellation_cannot_mint() {
        fn anchor() {}
        let cancelled = pse_diagnostics::Cancellation::new(false);
        let stop = cancelled.clone();
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed_calls = calls.clone();
        let configuration: RuntimeConfigurationObserver = Arc::new(move || {
            observed_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            stop.store(true, std::sync::atomic::Ordering::Release);
            // SAFETY: this controlled test root has no environment writers during capture.
            unsafe { observed_test_configuration(Vec::new()) }
        });
        // SAFETY: this test supplies its own controlled root; configuration only requests stop.
        let error = unsafe {
            ReplayAdmission::observe_local_scoped(
                ExpectedProducerTarget::WORKER,
                anchor as *const () as usize,
                configuration,
                &cancelled,
                None,
            )
        }
        .unwrap_err();
        assert!(matches!(
            error,
            PortableError::Math(pse_math::MathError::Cancelled)
        ));
        assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 1);
    }
    #[test]
    #[allow(
        unsafe_code,
        reason = "scoped mint IO stop controls use the actual test code anchor"
    )]
    fn scoped_local_replay_configuration_io_stops_remain_typed() {
        fn anchor() {}
        for (kind, expects_cancellation) in [
            (std::io::ErrorKind::Interrupted, true),
            (std::io::ErrorKind::TimedOut, false),
        ] {
            let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let observed_calls = calls.clone();
            let configuration: RuntimeConfigurationObserver = Arc::new(move || {
                observed_calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Err(std::io::Error::new(kind, "controlled configuration stop"))
            });
            // SAFETY: the actual test root uses a controlled, immediately refusing observer.
            let error = unsafe {
                ReplayAdmission::observe_local_scoped(
                    ExpectedProducerTarget::WORKER,
                    anchor as *const () as usize,
                    configuration,
                    &AtomicBool::new(false),
                    None,
                )
            }
            .unwrap_err();
            if expects_cancellation {
                assert!(matches!(
                    error,
                    PortableError::Math(pse_math::MathError::Cancelled)
                ));
            } else {
                assert!(matches!(
                    error,
                    PortableError::Store(CanonicalError::Timeout)
                ));
            }
            assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 1);
        }
    }
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    #[test]
    #[allow(
        unsafe_code,
        reason = "isolated actual native composition-root local admission controls"
    )]
    fn local_replay_admission_is_opaque_role_bound_and_revalidates_effective_configuration() {
        fn anchor() {}
        let anchor = anchor as *const () as usize;
        let effective = Arc::new(Mutex::new(b"controlled-runtime-configuration".to_vec()));
        let current = effective.clone();
        let observe: RuntimeConfigurationObserver = Arc::new(move || {
            // SAFETY: this controlled test root has no environment writers during capture.
            unsafe { observed_test_configuration(current.lock().unwrap().clone()) }
        });
        // SAFETY: isolated test binary supplies its actual code anchor/configuration;
        // this control invokes no imports, JIT or provider callback within the scope.
        let local = unsafe {
            ReplayAdmission::observe_local(ExpectedProducerTarget::WORKER, anchor, observe.clone())
        }
        .unwrap();
        assert!(matches!(local, ReplayAdmission::Local(_)));
        assert!(local.is_current());
        assert!(
            local
                .key()
                .starts_with(pse_operations::canonical_selection::LOCAL_RUNTIME_PRODUCER_PREFIX)
        );
        let strict: ReplayAdmission = QualifiedProducer {
            identity: local.identity(),
        }
        .into();
        assert_ne!(local.key(), strict.key());
        let old = effective.lock().unwrap().clone();
        *effective.lock().unwrap() = b"changed-runtime-configuration".to_vec();
        assert!(!local.is_current());
        assert!(
            local
                .qualify_receiving(&AtomicBool::new(false), None)
                .unwrap()
                .is_none()
        );
        // SAFETY: same controlled actual root, now independently observing changed configuration.
        let changed = unsafe {
            ReplayAdmission::observe_local(ExpectedProducerTarget::WORKER, anchor, observe.clone())
        }
        .unwrap();
        assert_ne!(changed.key(), local.key());
        *effective.lock().unwrap() = old;
        assert!(local.is_current());
        // SAFETY: actual observer input is unchanged; unsupported claimed role must refuse.
        assert!(
            // SAFETY: the same controlled observer is used to check role refusal.
            unsafe {
                ReplayAdmission::observe_local(
                    ExpectedProducerTarget {
                        package: "opaque-plugin",
                        target: "foreign",
                        kind: "lib",
                    },
                    anchor,
                    observe,
                )
            }
            .is_err()
        );
    }
    #[test]
    #[allow(
        unsafe_code,
        reason = "isolated current receipt decoder controls, no production qualification exposed"
    )]
    fn deployment_receipt_current_shape_binds_actual_artifact_role_and_native_contract() {
        let artifact = "a".repeat(64);
        let identity = "b".repeat(64);
        let mut value = serde_json::json!({
            "receipt_version": pse_buildinfo::DEPLOYMENT_RECEIPT_VERSION,
            "frame": Frame::ProducerV1.as_str(), "identity": identity,
            "persistent_reuse_eligible": true, "reasons": [],
            "selected_lock_records":[],"consumed_inputs":{},"declared_environment":{},
            "deployment": {"artifact": {"sha256": artifact},"producer_identity":"pending-fixture-key","selected_root":"root"},
            "native_abi": "actual reviewed native ABI", "package": "xtask", "selected_root": "root",
            "units": [{"key":"root", "target_name":"pse-worker", "target_kind":["bin"], "crate_types":["bin"],"package_id":"selected-fixture","edition":"2024","platform":null,"dependencies":{}, "mode":"build", "profile":{"name":"producer"}, "features":["native-solvers"]}]
        });
        let units: Vec<pse_buildinfo::identity::ProductionUnit> =
            serde_json::from_value(value["units"].clone()).unwrap();
        value["identity"] = pse_buildinfo::identity::scientific_producer_identity(
            "xtask",
            &units,
            &[],
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
            &Some("actual reviewed native ABI".into()),
        )
        .unwrap()
        .to_hex()
        .into();
        value["deployment"]["producer_identity"] = value["identity"].clone();
        // SAFETY: isolated decoder controls use artificial observations; this fixture
        // exposes no native composition root or persisted eligibility.
        let mint = |value: &serde_json::Value, observed: &str, role| unsafe {
            // Isolated decoder controls supply artificial observations; no native
            // composition root or persisted eligibility is exposed by this fixture.
            QualifiedProducer::from_deployment_receipt(
                &serde_json::to_vec(value).unwrap(),
                observed,
                role,
            )
        };
        assert!(
            mint(&value, &artifact, ExpectedProducerTarget::WORKER)
                .unwrap()
                .is_some()
        );
        let mut substituted = value.clone();
        substituted["identity"] = "c".repeat(64).into();
        assert!(mint(&substituted, &artifact, ExpectedProducerTarget::WORKER).is_err());
        assert!(
            mint(&value, &"c".repeat(64), ExpectedProducerTarget::WORKER)
                .unwrap()
                .is_none()
        );
        assert!(
            mint(&value, &artifact, ExpectedProducerTarget::PYTHON)
                .unwrap()
                .is_none()
        );
        for changed in [
            {
                let mut v = value.clone();
                v["native_abi"] = serde_json::Value::Null;
                v
            },
            {
                let mut v = value.clone();
                v["units"][0]["profile"]["name"] = "dev".into();
                v
            },
            {
                let mut v = value.clone();
                v["units"][0]["features"] = serde_json::json!([]);
                v
            },
        ] {
            assert!(
                mint(&changed, &artifact, ExpectedProducerTarget::WORKER)
                    .unwrap()
                    .is_none()
            );
        }
        let mut inconsistent = value.clone();
        inconsistent["reasons"] = serde_json::json!(["unresolved"]);
        assert!(mint(&inconsistent, &artifact, ExpectedProducerTarget::WORKER).is_err());
        let historical = serde_json::json!({"receipt_version":1,"deployment":"old shape"});
        assert!(
            matches!(mint(&historical,&artifact,ExpectedProducerTarget::WORKER),Err(PortableError::Qualification(reason)) if reason=="unsupported deployment receipt interpretation")
        );
    }
    #[test]
    fn portable_body_relative_envelope_refuses_historical_v3_frame() {
        assert_eq!(INTERPRETATION, "pse.runtime.admitted-body-product.v4");
        let mut historical = b"pse-admitted-body-product-v3\0".to_vec();
        historical.extend_from_slice(&0u32.to_be_bytes());
        assert!(split_frame(&historical).is_err());
    }
    #[test]
    fn portable_envelope_over_wire_block_preserves_hash_and_refuses_forged_extent() {
        let payload = vec![b'x'; 3 * 1024 * 1024 + 1];
        let id = ContentHash::from_bytes([1; 32]);
        let envelope = Envelope {
            interpretation: INTERPRETATION.into(),
            semantic_identity: SemanticBodyHash::from_id(id),
            physical: id,
            spec: pse_math::binding::BodySpec {
                definition: id,
                structure: id,
                physical: id,
                admissions: id,
                providers: Vec::new(),
                policy: id,
            },
            payload_hash: portable_payload_hash(&payload),
        };
        let frame = pack(&envelope, &payload).unwrap();
        assert!(frame.len() > 3 * 1024 * 1024);
        let (restored, bytes) = unpack(&frame).unwrap();
        assert_eq!(bytes, payload);
        assert_eq!(restored.payload_hash, portable_payload_hash(bytes));
        let mut forged = frame;
        let offset = ENVELOPE_MAGIC.len();
        forged[offset..offset + 4]
            .copy_from_slice(&((MAX_METADATA_BYTES + 1) as u32).to_be_bytes());
        assert!(split_frame(&forged).is_err());
        assert!(split_frame(&forged[..offset + 3]).is_err());
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_deployment_tests {
    use super::*;

    #[test]
    #[ignore = "operator-invoked strict deployment capture campaign"]
    #[allow(
        unsafe_code,
        reason = "actual controlled deployment captures qualified against independently observed role artifacts"
    )]
    fn canonical_deployment_actual_receipts_enforce_selected_role() {
        for (path, target, other) in [
            (
                "PSE_PYTHON_PRODUCER_RECEIPT",
                ExpectedProducerTarget::PYTHON,
                ExpectedProducerTarget::WORKER,
            ),
            (
                "PSE_WORKER_PRODUCER_RECEIPT",
                ExpectedProducerTarget::WORKER,
                ExpectedProducerTarget::PYTHON,
            ),
        ] {
            let bytes =
                std::fs::read(std::env::var_os(path).expect("actual producer capture")).unwrap();
            let receipt: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(receipt["persistent_reuse_eligible"], true);
            assert!(receipt["reasons"].as_array().unwrap().is_empty());
            // Each role is observed independently; no outer-context equality
            // participates in scientific producer qualification.
            let role = if target.package == "pse-py" {
                "python"
            } else {
                "worker"
            };
            let observation: serde_json::Value = serde_json::from_slice(
                &std::fs::read(
                    std::env::var_os("PSE_DEPLOYMENT_ARTIFACT_OBSERVATIONS")
                        .expect("independent role observations"),
                )
                .unwrap(),
            )
            .unwrap();
            let artifact = observation[role]["sha256"].as_str().unwrap();
            // SAFETY: the qualification runner supplies current actual controlled
            // captures and independently records actual worker/imported Python
            // artifact bytes, with native loaded association checked at each
            // composition root. No receipt chooses its own expected artifact.
            let qualified =
                unsafe { QualifiedProducer::from_deployment_receipt(&bytes, artifact, target) }
                    .unwrap()
                    .expect("actual capture qualifies for its selected deployment role");
            assert_eq!(qualified.identity().to_hex(), receipt["identity"]);
            assert!(
                // SAFETY: these actual controlled captures and independently observed
                // artifact bytes differ only in the expected receiving executable role.
                unsafe { QualifiedProducer::from_deployment_receipt(&bytes, artifact, other) }
                    .unwrap()
                    .is_none()
            );
        }
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_portable_body_tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "portable replay fixtures fail immediately on invalid setup, poisoned capture locks or missing expected bodies"
    )]
    use super::*;
    use pse_compiler::workspace::{CompilerWorkspace, ModelingBodyRetention, WorkspaceLimits};
    use pse_modeling::{Bindings, InstanceId, Limits, PhysicalScope};
    use pse_operations::canonical::{CanonicalOptions, ObjectEdit};
    use std::{collections::BTreeMap, path::Path, sync::Mutex, time::Duration};

    #[tokio::test]
    async fn charged_publication_refuses_before_constructing_operation() {
        let service = crate::math::tests::limited_service(1024);
        let baseline = service.pool.reserved();
        let constructed = AtomicBool::new(false);
        let result = charged_publication(&service, || {
            constructed.store(true, std::sync::atomic::Ordering::SeqCst);
            async {
                let payload = [3u8; 2048];
                std::future::pending::<()>().await;
                std::hint::black_box(payload)
            }
        })
        .await;
        assert!(result.is_err());
        assert!(!constructed.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(service.pool.reserved(), baseline);
    }

    #[tokio::test]
    async fn charged_publication_holds_allocation_until_operation_returns() {
        let service = crate::math::tests::limited_service(8192);
        let baseline = service.pool.reserved();
        let (complete, completion) = tokio::sync::oneshot::channel();
        let operation = charged_publication(&service, || {
            assert!(service.pool.reserved() > baseline);
            async {
                let payload = [7u8; 2048];
                completion.await.unwrap();
                assert!(service.pool.reserved() > baseline);
                Err::<u8, _>(std::hint::black_box(payload)[0])
            }
        });
        tokio::pin!(operation);
        assert!(futures_util::poll!(operation.as_mut()).is_pending());
        assert!(service.pool.reserved() > baseline);
        complete.send(()).unwrap();
        assert_eq!(operation.await.unwrap(), Err(7));
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[allow(
        unsafe_code,
        reason = "controlled deployment fixtures test refusal and outer-binding semantics"
    )]
    fn deployment_fixture(
        bytes: &[u8],
        artifact: &str,
    ) -> Result<Option<ReplayAdmission>, PortableError> {
        // SAFETY: isolated fixture supplies a complete controlled receipt and exact
        // fixture attestations; no production completeness is asserted by these tests.
        unsafe {
            QualifiedProducer::from_deployment_receipt(
                bytes,
                artifact,
                ExpectedProducerTarget {
                    package: "producer-fixture",
                    target: "producer_fixture",
                    kind: "lib",
                },
            )
        }
        .map(|producer| producer.map(Into::into))
    }
    #[test]
    fn canonical_portable_body_errors_preserve_typed_identity() {
        use pse_diagnostics::{DiagnosticCode, TypedDiagnostic, diagnostic_leaves};
        let store = PortableError::Store(CanonicalError::PayloadLimit);
        assert_eq!(
            store.diagnostic_code(),
            Some(DiagnosticCode::RuntimeInfrastructure)
        );
        assert_eq!(diagnostic_leaves(&store).len(), 1);
        let math = pse_math::MathError::Contract("receipt mismatch".into());
        let expected = math.diagnostic_code();
        let error = PortableError::Math(math);
        assert_eq!(error.diagnostic_code(), expected);
        assert_eq!(diagnostic_leaves(&error)[0].diagnostic_code(), expected);
        assert_eq!(
            PortableError::Qualification("producer ineligible".into()).diagnostic_code(),
            Some(DiagnosticCode::RuntimeInfrastructure)
        );
    }
    #[tokio::test]
    async fn canonical_portable_body_large_scientific_recipe_persists_and_reconnects() {
        let state = std::env::var("PSE_SURREAL_STATE")
            .expect("canonical portable unit supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_portable_large_{}",
            pse_operations::mint_id::<pse_ids::SemanticId>().to_hex()
        );
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        // One ordinary semantic body retains a balanced consumed arithmetic tree and
        // its physical/proof receipts. Independent equations produce separate bodies.
        let mut terms = vec![String::from("(x-x)"); 1024];
        while terms.len() > 1 {
            terms = terms
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| format!("({}+{})", pair[0], pair[1]))
                .collect();
        }
        let text = format!(
            "package p {{def Root {{var x:Scalar;eq residual:x+{}==2;}}}}",
            terms.pop().unwrap()
        );
        let revision = store
            .edit(
                "problem",
                None,
                "authored-initial",
                &[ObjectEdit {
                    logical: "root".into(),
                    scope: "p".into(),
                    name: "Root".into(),
                    version: Some(version("root-v1", &text)),
                    references: Vec::new(),
                }],
            )
            .await
            .unwrap();
        let mut read = SelectedRead::new(
            store
                .protect(revision.clone(), Duration::from_secs(3600))
                .await
                .unwrap(),
        );
        store
            .resolve_names(&mut read, "p", &["Root".into()])
            .await
            .unwrap();
        let context = context();
        let body = admitted_with_limits(
            context.clone(),
            &text,
            Limits {
                body_occurrences: Some(262_144),
                body_slots: Some(65_536),
                ..Limits::default()
            },
        );
        let identity = body.semantic_identity().unwrap();
        let spec = body.spec().clone();
        let payload_bytes = body.portable_payload_bytes().unwrap();
        assert!(payload_bytes > 3 * 1024 * 1024);
        let service = crate::math::tests::limited_service(8usize << 30);
        let producer = producer(68);
        publish_body(&service, &store, &read, &producer, &body, &context)
            .await
            .unwrap();
        store.release(read.selection()).await.unwrap();
        drop(body);
        drop(text);
        drop(read);
        drop(store);
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.open().await.unwrap();
        let mut read = SelectedRead::new(
            store
                .protect(revision, Duration::from_secs(3600))
                .await
                .unwrap(),
        );
        let cancel = pse_diagnostics::Cancellation::new(false);
        let restored = reuse_body(
            &service, &store, &mut read, &producer, identity, &context, &cancel,
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(restored.spec(), &spec);
        let value_support = restored
            .math()
            .compile(
                &[0],
                &[],
                pse_kernels::DerivativeOrder::Value,
                pse_math::library::Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap();
        assert_eq!(value_support.input_formals().len(), 1);
        let coordinate = value_support.input_formals()[0];
        let compiled = restored
            .math()
            .compile(
                &[0],
                &[coordinate],
                pse_kernels::DerivativeOrder::First,
                pse_math::library::Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap();
        let values = vec![9.0; compiled.input_formals().len()];
        let jet = compiled
            .worker()
            .evaluate(
                &values,
                pse_kernels::DerivativeOrder::First,
                &mut pse_kernels::ProviderBindings::new(),
                &cancel,
            )
            .unwrap();
        assert_eq!(jet.values, vec![7.0]);
        assert_eq!(jet.jacobian, vec![1.0]);
        store.release(read.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_portable_body_candidate_reserves_before_protected_hydration() {
        use pse_diagnostics::{DiagnosticCode, TypedDiagnostic};
        let state = std::env::var("PSE_SURREAL_STATE")
            .expect("canonical portable unit supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_portable_budget_{}",
            pse_operations::mint_id::<pse_ids::SemanticId>().to_hex()
        );
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        let text = "package p {def Root {var x:Scalar;eq residual:x==2;}}";
        let revision = store
            .edit(
                "problem",
                None,
                "authored-initial",
                &[ObjectEdit {
                    logical: "root".into(),
                    scope: "p".into(),
                    name: "Root".into(),
                    version: Some(version("root-v1", text)),
                    references: Vec::new(),
                }],
            )
            .await
            .unwrap();
        let mut read = SelectedRead::new(
            store
                .protect(revision, Duration::from_secs(60))
                .await
                .unwrap(),
        );
        store
            .resolve_names(&mut read, "p", &["Root".into()])
            .await
            .unwrap();
        let context = context();
        let body = admitted(context.clone(), text);
        let service = crate::math::tests::service();
        let producer = producer(67);
        let published = publish_body(&service, &store, &read, &producer, &body, &context)
            .await
            .unwrap();
        let candidate = store
            .product_candidate(
                read.selection(),
                &request(
                    body.semantic_identity().unwrap(),
                    physical_identity(&context.quantities, &context.preconditions),
                ),
                &producer.key(),
                producer.producer_class(),
                "",
            )
            .await
            .unwrap()
            .unwrap();
        assert!(candidate.retained_bytes() > 1024);
        // Header discovery stays valid, while protected payload hydration will now
        // find corruption. A tiny budget must refuse before reading that native block.
        store.corrupt_product_block(published.key()).await.unwrap();
        let tiny = crate::math::tests::limited_service(1024);
        let identity = body.semantic_identity().unwrap();
        let cancel = pse_diagnostics::Cancellation::new(false);
        let error = reuse_body(
            &tiny, &store, &mut read, &producer, identity, &context, &cancel,
        )
        .await
        .unwrap_err();
        assert_eq!(
            error.diagnostic_code(),
            Some(DiagnosticCode::RuntimeResourceLimit)
        );
        assert_eq!(
            pse_model::diagnostic::project_typed(
                &error,
                pse_diagnostics::DiagnosticStage::ModelingAdmission
            )
            .class,
            pse_model::diagnostic::BoundaryClass::ResourceLimit
        );
        let error = reuse_body(
            &service, &store, &mut read, &producer, identity, &context, &cancel,
        )
        .await
        .unwrap_err();
        assert!(
            matches!(error, PortableError::Store(CanonicalError::Configuration(ref detail))
            if detail.contains("block identity/length/digest mismatch"))
        );
        store.release(read.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[derive(Debug, Default)]
    struct Capture(Mutex<Vec<Arc<AdmittedBody>>>);
    impl ModelingBodyRetention for Capture {
        fn generation(&self) -> u64 {
            0
        }
        fn get(
            &self,
            _: SemanticBodyHash,
        ) -> Result<Option<Arc<AdmittedBody>>, pse_math::MathError> {
            Ok(None)
        }
        fn retain(
            &self,
            _: u64,
            _: SemanticBodyHash,
            body: Arc<AdmittedBody>,
        ) -> Result<Arc<AdmittedBody>, pse_math::MathError> {
            self.0.lock().unwrap().push(body.clone());
            Ok(body)
        }
    }
    fn context() -> CompilerContext {
        CompilerContext {
            quantities: Arc::new(pse_quantity::standard::standard_registry().unwrap()),
            preconditions: Arc::new(
                pse_quantity::PhysicalPreconditions::new(
                    pse_quantity::generated::standard_preconditions(),
                )
                .unwrap(),
            ),
            providers: BTreeMap::new(),
        }
    }
    #[tokio::test]
    async fn canonical_portable_shared_selection_material_precharges_and_retains_one_real_lease() {
        let state = std::env::var("PSE_SURREAL_STATE")
            .expect("canonical portable unit supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_shared_material_{}",
            uuid::Uuid::new_v4().simple()
        );
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        let revision = store
            .edit("problem", None, "shared-material", &[])
            .await
            .unwrap();
        let read = SelectedRead::new(
            store
                .protect(revision, Duration::from_secs(60))
                .await
                .unwrap(),
        );
        let extent =
            pse_operations::canonical_selection::SelectionMaterial::allocation_bytes(&read)
                .unwrap();
        let refused = crate::math::tests::limited_service(extent - 1);
        let refused_baseline = refused.pool.reserved();
        assert!(
            selection_material(&refused, &read).is_err(),
            "allocation refusal precedes material allocation"
        );
        assert_eq!(refused.pool.reserved(), refused_baseline);
        let service = crate::math::tests::service();
        let baseline = service.pool.reserved();
        let material = selection_material(&service, &read).unwrap();
        assert_eq!(service.pool.reserved(), baseline + extent);
        let lease_lifetime = Arc::downgrade(&material);
        let descriptions = [b"one".as_slice(), b"two".as_slice()].map(|payload| {
            pse_operations::canonical_selection::ProductDescription::prepare_with_selection(
                &read,
                pse_model::generated::runtime::canonical_products::Row {
                    key: "offer".into(),
                    problem: "problem".into(),
                    revision: read.selection().revision().key.clone(),
                    request: b"request".to_vec().into(),
                    payload: payload.to_vec().into(),
                    dependencies: Vec::new().into(),
                    producer: "fixture".into(),
                    producer_class: pse_model::generated::enums::CanonicalProducerClass::General,
                    interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
                },
                material.clone(),
            )
            .unwrap()
        });
        assert_eq!(
            service.pool.reserved(),
            baseline + extent,
            "body aliases charge the captured premise allocation once"
        );
        drop(material);
        service.clear_program_cache();
        assert_eq!(service.pool.reserved(), baseline + extent);
        let escaped = descriptions[1].clone();
        drop(descriptions);
        assert!(lease_lifetime.upgrade().is_some());
        drop(escaped);
        assert!(lease_lifetime.upgrade().is_none());
        assert_eq!(service.pool.reserved(), baseline);
        read.finish().await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    fn admitted(context: CompilerContext, text: &str) -> Arc<AdmittedBody> {
        admitted_with_limits(context, text, Limits::default())
    }
    fn admitted_with_limits(
        context: CompilerContext,
        text: &str,
        limits: Limits,
    ) -> Arc<AdmittedBody> {
        let declarations = pse_authoring::language::parse(
            text,
            pse_ids::SemanticId::from_bytes([88; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|r| r.name == "Root")
            .unwrap()
            .declaration_id;
        let capture = Arc::new(Capture::default());
        let mut compiler = CompilerWorkspace::new(context, WorkspaceLimits::default()).unwrap();
        compiler.attach_body_retention(capture.clone()).unwrap();
        compiler
            .publish_modeling(declarations, PhysicalScope::default())
            .unwrap();
        let prepared = compiler
            .prepare_modeling_cancellable(
                root,
                InstanceId::from_id(pse_ids::SemanticId::NIL),
                Bindings::default(),
                limits,
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        let body = capture
            .0
            .lock()
            .unwrap()
            .iter()
            .find(|body| body.has_portable_payload())
            .unwrap()
            .clone();
        drop(prepared);
        drop(compiler);
        drop(capture);
        body
    }
    // A named fixture qualification exercises consumer mechanics only. It makes no
    // claim that the real deployment's producer inputs have been completely qualified.
    fn producer(byte: u8) -> ReplayAdmission {
        QualifiedProducer {
            identity: ContentHash::from_bytes([byte; 32]),
        }
        .into()
    }
    #[test]
    fn canonical_portable_body_encoding_keeps_consumed_and_deployment_contexts_distinct() {
        let context = context();
        let text = "package p {def Root {var x:Scalar;eq residual:x==2;}}";
        let body = admitted(context.clone(), text);
        let deployment = physical_identity(&context.quantities, &context.preconditions);
        assert_ne!(body.spec().physical, deployment);
        let mut changed_context = context.clone();
        changed_context.preconditions =
            Arc::new(pse_quantity::PhysicalPreconditions::new(vec![]).unwrap());
        let changed_body = admitted(changed_context.clone(), text);
        assert_eq!(changed_body.spec(), body.spec());
        assert_eq!(changed_body.semantic_identity(), body.semantic_identity());
        let changed_deployment =
            physical_identity(&changed_context.quantities, &changed_context.preconditions);
        assert_ne!(deployment, changed_deployment);

        let service = crate::math::tests::service();
        let baseline = service.pool.reserved();
        let encoded = encoded_body(&service, &body, &context).unwrap();
        let changed = encoded_body(&service, &changed_body, &changed_context).unwrap();
        assert!(!Arc::ptr_eq(&encoded, &changed));
        assert_ne!(encoded.request, changed.request);
        let warm = encoded_body(&service, &changed_body, &changed_context).unwrap();
        assert!(Arc::ptr_eq(&changed, &warm));
        let (original, original_payload) = unpack(&encoded.payload).unwrap();
        let (current, current_payload) = unpack(&changed.payload).unwrap();
        assert_eq!(original.physical, deployment);
        assert_eq!(current.physical, changed_deployment);
        assert_eq!(original.spec, current.spec);
        assert_eq!(current.spec, *changed_body.spec());
        assert_eq!(original_payload, current_payload);
        assert_eq!(portable_payload_hash(current_payload), current.payload_hash);
        service.clear_program_cache();
        drop(warm);
        drop(changed);
        drop(encoded);
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[test]
    fn canonical_portable_body_pure_retention_and_encoding_share_owned_material_after_clear() {
        use crate::math::retention::BodyRetention;
        let context = context();
        let body = admitted(
            context.clone(),
            "package p {def Root {var x:Scalar;eq residual:x==2;}}",
        );
        let identity = body.semantic_identity().unwrap();
        let service = crate::math::tests::service();
        let baseline = service.pool.reserved();
        let memory = BodyRetention(Arc::downgrade(&service));
        let retained = memory
            .retain(memory.generation(), identity, body.clone())
            .unwrap();
        assert!(Arc::ptr_eq(
            &retained,
            &memory.get(identity).unwrap().unwrap()
        ));
        let encoded = encoded_body(&service, &retained, &context).unwrap();
        let warm = encoded_body(&service, &retained, &context).unwrap();
        assert!(
            Arc::ptr_eq(&encoded, &warm),
            "pure hits reuse exact encoding allocation"
        );
        assert_eq!(service.modeling_cache.report(0).entries, 2);
        assert!(service.modeling_cache.report(0).retained_bytes > encoded.retained_bytes());
        let old_generation = memory.generation();
        service.clear_program_cache();
        assert!(memory.get(identity).unwrap().is_none());
        assert!(service.modeling_cache.encoded_body(identity).is_none());
        assert!(
            service.pool.reserved() > baseline,
            "escaped aliases retain their actual allocation leases"
        );
        let late = memory
            .retain(old_generation, identity, retained.clone())
            .unwrap();
        service
            .modeling_cache
            .retain_encoded_body(old_generation, identity, encoded.clone());
        assert!(memory.get(identity).unwrap().is_none());
        assert!(service.modeling_cache.encoded_body(identity).is_none());
        assert!(
            memory
                .retain(
                    memory.generation(),
                    SemanticBodyHash::from(ContentHash::from_bytes([9; 32])),
                    body.clone()
                )
                .is_err(),
            "a caller key cannot change compiler-sealed meaning"
        );
        drop(late);
        drop(warm);
        drop(encoded);
        drop(retained);
        drop(body);
        assert_eq!(service.pool.reserved(), baseline);
    }
    #[test]
    fn canonical_portable_body_oversized_pure_retention_bypasses_without_losing_ownership() {
        use crate::math::retention::BodyRetention;
        let context = context();
        let body = admitted(
            context.clone(),
            "package p {def Root {var x:Scalar;eq residual:x==2;}}",
        );
        let identity = body.semantic_identity().unwrap();
        let (service, _) = crate::math::tests::service_with_policy(
            512 << 20,
            crate::math::MathPolicy {
                artifact_bytes: 1,
                ..crate::math::MathPolicy::default()
            },
        );
        let memory = BodyRetention(Arc::downgrade(&service));
        let retained = memory.retain(memory.generation(), identity, body).unwrap();
        let encoded = encoded_body(&service, &retained, &context).unwrap();
        assert!(memory.get(identity).unwrap().is_none());
        assert!(service.modeling_cache.encoded_body(identity).is_none());
        assert_eq!(service.modeling_cache.report(0).entries, 0);
        assert!(service.modeling_cache.report(0).bypasses >= 2);
        assert!(service.pool.reserved() > 0);
        drop(encoded);
        drop(retained);
        assert_eq!(service.pool.reserved(), 0);
    }
    #[tokio::test]
    async fn canonical_portable_body_explicit_receiving_acquisition_survives_reconnect() {
        explicit_receiving_acquisition_survives_reconnect(false).await;
    }
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    #[tokio::test]
    async fn canonical_portable_body_explicit_local_receiving_acquisition_survives_reconnect() {
        explicit_receiving_acquisition_survives_reconnect(true).await;
    }
    #[allow(
        unsafe_code,
        reason = "controlled actual native reconstruction test observes no-receipt local deployment"
    )]
    fn restarted_admission(local: bool) -> ReplayAdmission {
        if !local {
            return producer(65);
        }
        fn anchor() {}
        let observe = Arc::new(|| {
            // SAFETY: this controlled fixture has no mutable interpreter configuration
            // or provider callbacks beyond the explicit test configuration.
            unsafe { observed_test_configuration(Vec::new()) }
        });
        // SAFETY: this actual native test binary is a controlled immutable mathematical
        // reconstruction root with no additional interpreter configuration or provider callbacks.
        unsafe {
            ReplayAdmission::observe_local(
                ExpectedProducerTarget::WORKER,
                anchor as *const () as usize,
                observe,
            )
        }
        .unwrap()
    }
    async fn explicit_receiving_acquisition_survives_reconnect(local: bool) {
        use crate::math::retention::BodyRetention;
        let state = std::env::var("PSE_SURREAL_STATE").unwrap();
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_explicit_{}",
            pse_operations::mint_id::<pse_ids::SemanticId>().to_hex()
        );
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        let text = "package p {def Root {var x:Scalar;eq residual:x==2;}}";
        let revision = store
            .edit(
                "explicit",
                None,
                "source",
                &[ObjectEdit {
                    logical: "root".into(),
                    scope: "p".into(),
                    name: "Root".into(),
                    version: Some(version("explicit-v1", text)),
                    references: Vec::new(),
                }],
            )
            .await
            .unwrap();
        let context = context();
        let initial_admission = restarted_admission(local);
        let body = admitted(context.clone(), text);
        let identity = body.semantic_identity().unwrap();
        let mut selected = SelectedRead::new(
            store
                .protect(revision.clone(), Duration::from_secs(60))
                .await
                .unwrap(),
        );
        store
            .resolve_names(&mut selected, "p", &["Root".into()])
            .await
            .unwrap();
        selected
            .interpretation(
                "physical".into(),
                physical_identity(&context.quantities, &context.preconditions).to_prefixed(),
            )
            .unwrap();
        let service = crate::math::tests::service();
        publish_body(
            &service,
            &store,
            &selected,
            &initial_admission,
            &body,
            &context,
        )
        .await
        .unwrap();
        let initial_key = initial_admission.key();
        store.release(selected.selection()).await.unwrap();
        drop(body);
        drop(selected);
        drop(service);
        drop(store);
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.open().await.unwrap();
        let mut selected = SelectedRead::new(
            store
                .protect(revision, Duration::from_secs(60))
                .await
                .unwrap(),
        );
        selected
            .interpretation(
                "physical".into(),
                physical_identity(&context.quantities, &context.preconditions).to_prefixed(),
            )
            .unwrap();
        let service = crate::math::tests::service();
        let receiving = restarted_admission(local);
        assert_eq!(receiving.key(), initial_key);
        let cancelled = pse_diagnostics::Cancellation::new(false);
        let rebuilt = reuse_body(
            &service,
            &store,
            &mut selected,
            &receiving,
            identity,
            &context,
            &cancelled,
        )
        .await
        .unwrap()
        .expect("explicit receiving acquisition reconstructs persisted body");
        let memory = BodyRetention(Arc::downgrade(&service));
        assert!(
            memory.get(identity).unwrap().is_none(),
            "receiving acquisition itself performs no retention callback"
        );
        let retained = memory
            .retain(memory.generation(), identity, Arc::new(rebuilt))
            .unwrap();
        assert!(Arc::ptr_eq(
            &retained,
            &memory.get(identity).unwrap().unwrap()
        ));
        let compiled = retained
            .math()
            .compile(
                &[0],
                &[0],
                pse_kernels::DerivativeOrder::First,
                pse_math::library::Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancelled.clone(), None),
            )
            .unwrap();
        let mut worker = compiled.worker();
        let jet = worker
            .evaluate(
                &[9.0],
                pse_kernels::DerivativeOrder::First,
                &mut pse_kernels::ProviderBindings::new(),
                &cancelled,
            )
            .unwrap();
        assert_eq!(9.0 - jet.values[0] / jet.jacobian[0], 2.0);
        store.release(selected.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    #[allow(
        unsafe_code,
        reason = "controlled local admission fixture verifies cold canonical miss avoids loader observation"
    )]
    async fn canonical_portable_body_cold_miss_skips_local_runtime_observation() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let state = std::env::var("PSE_SURREAL_STATE").unwrap();
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_cold_miss_{}",
            pse_operations::mint_id::<pse_ids::SemanticId>().to_hex()
        );
        let store = Arc::new(CanonicalStore::connect(&options).await.unwrap());
        store.create().await.unwrap();
        let context = context();
        let body = admitted(
            context.clone(),
            "package p {def Root {var x:Scalar;eq residual:x==2;}}",
        );
        let identity = body.semantic_identity().unwrap();
        let revision = store.edit("cold-miss", None, "source", &[]).await.unwrap();
        let mut selected = SelectedRead::new(
            store
                .protect(revision.clone(), Duration::from_secs(60))
                .await
                .unwrap(),
        );
        selected
            .interpretation(
                "physical".into(),
                physical_identity(&context.quantities, &context.preconditions).to_prefixed(),
            )
            .unwrap();
        let service = crate::math::tests::service();
        let observations = Arc::new(AtomicUsize::new(0));
        let observed = observations.clone();
        fn anchor() {}
        let observe = Arc::new(move || {
            if observed.fetch_add(1, Ordering::Relaxed) < 2 {
                // SAFETY: the controlled test executable uses only this explicit
                // configuration and has no mutable interpreter or provider callbacks.
                unsafe { observed_test_configuration(vec![7]) }
            } else {
                Err(std::io::Error::other(
                    "cold miss unexpectedly observed producer context",
                ))
            }
        });
        // SAFETY: the unit-test executable is the controlled anchor for this local replay fixture.
        let producer = unsafe {
            ReplayAdmission::observe_local(
                ExpectedProducerTarget::WORKER,
                anchor as *const () as usize,
                observe,
            )
        }
        .unwrap();
        assert_eq!(observations.load(Ordering::Relaxed), 2);
        let absent = reuse_body(
            &service,
            &store,
            &mut selected,
            &producer,
            identity,
            &context,
            &Arc::new(AtomicBool::new(false)),
        )
        .await
        .unwrap();
        assert!(absent.is_none());
        assert_eq!(observations.load(Ordering::Relaxed), 2);
        store.release(selected.selection()).await.unwrap();
        drop(body);
        drop(service);
        store.remove_isolated_fixture().await.unwrap();
    }
    fn version(key: &str, text: &str) -> pse_model::generated::runtime::canonical_versions::Row {
        pse_model::generated::runtime::canonical_versions::Row {
            key: key.into(),
            logical: "root".into(),
            kind: "authored-modeling".into(),
            payload: text.as_bytes().to_vec().into(),
            interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
        }
    }
    #[tokio::test]
    async fn canonical_portable_body_reconnect_reuses_ordinary_scalar_solve_with_actual_tool_fixture_receipt()
     {
        let state = std::env::var("PSE_SURREAL_STATE")
            .expect("canonical portable unit supplies supervised state");
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_portable_{}",
            pse_operations::mint_id::<pse_ids::SemanticId>().to_hex()
        );
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.create().await.unwrap();
        let text = "package p {def Root {var x:Scalar;eq residual:x==2;}}";
        let revision = store
            .edit(
                "problem",
                None,
                "authored-initial",
                &[ObjectEdit {
                    logical: "root".into(),
                    scope: "p".into(),
                    name: "Root".into(),
                    version: Some(version("root-v1", text)),
                    references: Vec::new(),
                }],
            )
            .await
            .unwrap();
        let context = context();
        let body = admitted(context.clone(), text);
        let identity = body.semantic_identity().unwrap();
        // The recipe runs the actual production identity tool over a finite Rust-only
        // fixture. Its eligible receipt is evidence for this integration fixture;
        // it is explicitly not real scientific-kernel deployment qualification.
        let receipt_path = std::env::var("PSE_PRODUCER_FIXTURE_RECEIPT")
            .expect("portable recipe supplies actual tool fixture capture");
        let bytes = std::fs::read(receipt_path).unwrap();
        let receipt: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(receipt["package"], "producer-fixture");
        assert_eq!(receipt["persistent_reuse_eligible"], true);
        assert!(receipt["reasons"].as_array().unwrap().is_empty());
        assert!(!receipt["units"].as_array().unwrap().is_empty());
        let qualified = deployment_fixture(
            &bytes,
            receipt["deployment"]["artifact"]["sha256"]
                .as_str()
                .unwrap(),
        )
        .unwrap()
        .expect("actual finite tool fixture is eligible");
        assert_eq!(
            qualified.identity().to_hex(),
            receipt["identity"].as_str().unwrap()
        );
        let mut read = SelectedRead::new(
            store
                .protect(revision.clone(), Duration::from_secs(60))
                .await
                .unwrap(),
        );
        assert_eq!(
            store
                .resolve_names(&mut read, "p", &["Root".into()])
                .await
                .unwrap()
                .len(),
            1
        );
        read.interpretation(
            "physical".into(),
            physical_identity(&context.quantities, &context.preconditions).to_prefixed(),
        )
        .unwrap();
        let service = crate::math::tests::service();
        let published = publish_body(&service, &store, &read, &qualified, &body, &context)
            .await
            .unwrap();
        assert_eq!(published.semantic_identity(), identity);
        let outer_build = ContentHash::from_bytes([67; 32]);
        let unqualified =
            publish_unqualified_body(&service, &store, &read, outer_build, &body, &context)
                .await
                .unwrap();
        let unqualified_key = format!("pse.unqualified-producer.v1:{}", outer_build.to_hex());
        let candidate = store
            .product_candidate(
                read.selection(),
                &request(
                    identity,
                    physical_identity(&context.quantities, &context.preconditions),
                ),
                &unqualified_key,
                pse_model::generated::enums::CanonicalProducerClass::General,
                "",
            )
            .await
            .unwrap()
            .unwrap();
        let (unqualified_product, _product_lease) =
            qualify_candidate(&service, &store, &mut read, &candidate)
                .await
                .unwrap();
        let unqualified_product = unqualified_product.unwrap();
        assert_eq!(unqualified_product.key(), unqualified.key());
        assert_eq!(unqualified_product.producer(), unqualified_key);
        assert!(
            reuse_body(
                &crate::math::tests::service(),
                &store,
                &mut read,
                &producer(67),
                identity,
                &context,
                &Arc::new(AtomicBool::new(false))
            )
            .await
            .unwrap()
            .is_none()
        );
        store.release(read.selection()).await.unwrap();
        // Drop the native source product and the original authenticated transport.
        drop(body);
        drop(read);
        drop(store);
        let store = CanonicalStore::connect(&options).await.unwrap();
        store.open().await.unwrap();
        let mut read = SelectedRead::new(
            store
                .protect(revision.clone(), Duration::from_secs(60))
                .await
                .unwrap(),
        );
        read.interpretation(
            "physical".into(),
            physical_identity(&context.quantities, &context.preconditions).to_prefixed(),
        )
        .unwrap();
        let cancel = pse_diagnostics::Cancellation::new(false);
        let rebuilt = reuse_body(
            &crate::math::tests::service(),
            &store,
            &mut read,
            &qualified,
            identity,
            &context,
            &cancel,
        )
        .await
        .unwrap()
        .expect("qualified persisted body");
        assert_eq!(rebuilt.semantic_identity(), Some(identity));
        let compiled = rebuilt
            .math()
            .compile(
                &[0],
                &[0],
                pse_kernels::DerivativeOrder::First,
                pse_math::library::Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancel.clone(), None),
            )
            .unwrap();
        let mut x = 9.0;
        let mut worker = compiled.worker();
        for _ in 0..3 {
            let result = worker
                .evaluate(
                    &[x],
                    pse_kernels::DerivativeOrder::First,
                    &mut pse_kernels::ProviderBindings::new(),
                    &cancel,
                )
                .unwrap();
            if result.values[0].abs() < 1e-12 {
                break;
            }
            x -= result.values[0] / result.jacobian[0];
        }
        assert_eq!(x, 2.0);
        // A different relevant producer has no compatible persisted product.
        assert!(
            reuse_body(
                &crate::math::tests::service(),
                &store,
                &mut read,
                &producer(45),
                identity,
                &context,
                &cancel
            )
            .await
            .unwrap()
            .is_none()
        );
        store.release(read.selection()).await.unwrap();
        let changed = store
            .edit(
                "problem",
                Some(&revision.key),
                "authored-change",
                &[ObjectEdit {
                    logical: "root".into(),
                    scope: "p".into(),
                    name: "Root".into(),
                    version: Some(version(
                        "root-v2",
                        "package p {def Root {var x:Scalar;eq residual:x==3;}}",
                    )),
                    references: Vec::new(),
                }],
            )
            .await
            .unwrap();
        let mut changed_read = SelectedRead::new(
            store
                .protect(changed, Duration::from_secs(60))
                .await
                .unwrap(),
        );
        changed_read
            .interpretation(
                "physical".into(),
                physical_identity(&context.quantities, &context.preconditions).to_prefixed(),
            )
            .unwrap();
        assert!(
            reuse_body(
                &crate::math::tests::service(),
                &store,
                &mut changed_read,
                &qualified,
                identity,
                &context,
                &cancel
            )
            .await
            .unwrap()
            .is_none()
        );
        store.release(changed_read.selection()).await.unwrap();
        store.remove_isolated_fixture().await.unwrap();
    }
    #[test]
    fn canonical_portable_body_incomplete_producer_refuses_persistent_reuse() {
        let receipt=serde_json::to_vec(&serde_json::json!({"receipt_version":2,"frame":Frame::ProducerV1.as_str(),"native_abi":null,"deployment":null,"identity":ContentHash::from_bytes([1;32]).to_hex(),"persistent_reuse_eligible":false,"reasons":["undeclared build-script input"]})).unwrap();
        assert!(
            deployment_fixture(&receipt, &"a".repeat(64))
                .unwrap()
                .is_none()
        );
        let contradictory=serde_json::to_vec(&serde_json::json!({"receipt_version":2,"frame":Frame::ProducerV1.as_str(),"native_abi":null,"deployment":null,"identity":ContentHash::from_bytes([1;32]).to_hex(),"persistent_reuse_eligible":true,"reasons":["unknown native bytes"]})).unwrap();
        assert!(deployment_fixture(&contradictory, &"a".repeat(64)).is_err());
    }

    #[test]
    fn canonical_portable_body_producer_requires_selected_root_not_dependency() {
        let artifact = "a".repeat(64);
        let mut receipt = serde_json::json!({
            "frame": Frame::ProducerV1.as_str(), "identity": ContentHash::from_bytes([4; 32]).to_hex(),
            "persistent_reuse_eligible": true, "reasons": [], "package": "producer-fixture",
            "receipt_version":2,"native_abi":"fixture-abi","deployment":{"artifact":{"sha256":artifact},"producer_identity":"pending-fixture-key","selected_root":"worker"}, "selected_root": "worker",
            "selected_lock_records":[],"consumed_inputs":{},"declared_environment":{},
            "units": [
                {"key": "worker", "target_name": "worker", "target_kind": ["bin"], "crate_types":["bin"],"package_id":"fixture","edition":"2024","platform":null,"dependencies":{},"profile":{},"features":[], "mode": "build"},
                {"key": "library", "target_name": "producer_fixture", "target_kind": ["lib"], "crate_types":["lib"],"package_id":"fixture","edition":"2024","platform":null,"dependencies":{},"profile":{},"features":[], "mode": "build"}
            ]
        });
        let units: Vec<pse_buildinfo::identity::ProductionUnit> =
            serde_json::from_value(receipt["units"].clone()).unwrap();
        receipt["identity"] = pse_buildinfo::identity::scientific_producer_identity(
            "producer-fixture",
            &units,
            &[],
            &BTreeMap::new(),
            &BTreeMap::new(),
            &Some("fixture-abi".into()),
        )
        .unwrap()
        .to_hex()
        .into();
        receipt["deployment"]["producer_identity"] = receipt["identity"].clone();
        let check = |receipt: &serde_json::Value| {
            deployment_fixture(&serde_json::to_vec(receipt).unwrap(), &artifact)
        };
        assert!(check(&receipt).unwrap().is_none());
        receipt["selected_root"] = "library".into();
        assert!(
            check(&receipt).is_err(),
            "artifact bound to the actual worker root cannot qualify a dependency root"
        );
        receipt["deployment"]["selected_root"] = "library".into();
        assert!(check(&receipt).unwrap().is_some());
        receipt["selected_root"] = "absent".into();
        assert!(matches!(
            check(&receipt),
            Err(PortableError::Qualification(_))
        ));
        receipt.as_object_mut().unwrap().remove("selected_root");
        assert!(check(&receipt).unwrap().is_none());
        receipt["selected_root"] = "library".into();
        receipt["package"] = "another-package".into();
        assert!(check(&receipt).unwrap().is_none());
    }

    #[test]
    fn canonical_portable_body_envelope_preserves_opaque_payload_and_refuses_truncation() {
        let hash = ContentHash::from_bytes([1; 32]);
        let envelope = Envelope {
            interpretation: INTERPRETATION.into(),
            semantic_identity: SemanticBodyHash::from(hash),
            physical: hash,
            spec: pse_math::binding::BodySpec {
                definition: hash,
                structure: hash,
                physical: hash,
                admissions: hash,
                providers: Vec::new(),
                policy: hash,
            },
            payload_hash: hash,
        };
        let payload = [0, 255, 128, 1, 0, 42];
        let packed = pack(&envelope, &payload).unwrap();
        let (decoded, raw) = unpack(&packed).unwrap();
        assert_eq!(decoded.spec, envelope.spec);
        assert_eq!(raw, payload);
        assert!(unpack(b"unknown-format").is_err());
        assert!(unpack(ENVELOPE_MAGIC).is_err());
        let mut short = ENVELOPE_MAGIC.to_vec();
        short.extend_from_slice(&u32::MAX.to_be_bytes());
        assert!(unpack(&short).is_err());
        assert!(
            pack(
                &envelope,
                &vec![0; pse_ids::scientific_replay::MAX_ADMITTED_RECIPE_BYTES + 1]
            )
            .is_err()
        );
    }
    #[tokio::test]
    #[allow(
        unsafe_code,
        reason = "controlled corrupt-store fixture deliberately crosses publication trust boundary"
    )]
    async fn canonical_portable_body_corrupt_explicit_acquisition_refuses_before_retention() {
        let state = std::env::var("PSE_SURREAL_STATE").unwrap();
        let mut options = CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_corrupt_{}",
            pse_operations::mint_id::<pse_ids::SemanticId>().to_hex()
        );
        let store = Arc::new(CanonicalStore::connect(&options).await.unwrap());
        store.create().await.unwrap();
        let text = "package p {def Root {var x:Scalar;eq residual:x==2;}}";
        let context = context();
        let body = admitted(context.clone(), text);
        let identity = body.semantic_identity().unwrap();
        let revision = store
            .edit(
                "corrupt",
                None,
                "source",
                &[ObjectEdit {
                    logical: "root".into(),
                    scope: "p".into(),
                    name: "Root".into(),
                    version: Some(version("corrupt-v1", text)),
                    references: Vec::new(),
                }],
            )
            .await
            .unwrap();
        let mut selected = SelectedRead::new(
            store
                .protect(revision.clone(), Duration::from_secs(60))
                .await
                .unwrap(),
        );
        store
            .resolve_names(&mut selected, "p", &["Root".into()])
            .await
            .unwrap();
        let physical = physical_identity(&context.quantities, &context.preconditions);
        selected
            .interpretation("physical".into(), physical.to_prefixed())
            .unwrap();
        // Generic substrate admission establishes immutable dependency-qualified bytes;
        // this corrupt fixture deliberately lacks a valid scientific-owner envelope.
        // SAFETY: isolated corrupt-store fixture deliberately tests rejection at scientific decoding.
        unsafe {
            store.publish_scientific_product(
                &selected,
                pse_model::generated::runtime::canonical_products::Row {
                    key: "corrupt-recipe-fixture".into(),
                    problem: revision.problem.clone(),
                    revision: revision.key.clone(),
                    request: request(identity, physical).into(),
                    payload: b"corrupt scientifically typed bytes".to_vec().into(),
                    dependencies: Vec::new().into(),
                    producer: producer(68).key(),
                    producer_class: producer(68).producer_class(),
                    interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
                },
            )
        }
        .await
        .unwrap();
        drop(body);
        let service = crate::math::tests::service();
        let memory = crate::math::retention::BodyRetention(Arc::downgrade(&service));
        let error = reuse_body(
            &service,
            &store,
            &mut selected,
            &producer(68),
            identity,
            &context,
            &Arc::new(AtomicBool::new(false)),
        )
        .await
        .unwrap_err();
        fn has_refusal(diagnostic: &pse_model::diagnostic::BoundaryDiagnostic) -> bool {
            diagnostic.observations.values().any(|value|matches!(value,pse_model::diagnostic::Observation::Text(detail) if detail.contains("unsupported portable envelope framing"))) || diagnostic.causes.iter().any(has_refusal)
        }
        let diagnostic = pse_model::diagnostic::project_typed(
            &error,
            pse_diagnostics::DiagnosticStage::ModelingAdmission,
        );
        assert!(has_refusal(&diagnostic), "{diagnostic:?}");
        assert!(memory.get(identity).unwrap().is_none());
        store.release(selected.selection()).await.unwrap();
        drop(service);
        store.remove_isolated_fixture().await.unwrap();
    }
}
