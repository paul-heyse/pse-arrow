// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Separately interpreted numerical products under the existing canonical owner.
//! A checked execution product never grants symbolic or class-proof eligibility.
use super::*;
use pse_compiler::workspace::NumericalObservationRequest;
use pse_math::transfer::{CapturedExecution, ExecutionSnapshot, RestoredExecutionTemplate};

const NUMERICAL_INTERPRETATION: &str = "pse.runtime.numerical-observation.v1";
const MAX_NUMERICAL_BYTES: usize = pse_ids::scientific_replay::MAX_ADMITTED_RECIPE_BYTES;
// Leave envelope headroom below serde_json's fixed recursion ceiling. This is
// a wire-profile bound, distinct from the mathematical stage-depth bound.
const MAX_NUMERICAL_JSON_DEPTH: usize = 120;

#[cfg(test)]
mod tests;

#[derive(Serialize)]
struct Outgoing<'a> {
    interpretation: &'static str,
    request: &'a [u8],
    executions: Vec<&'a ExecutionSnapshot>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Incoming {
    interpretation: String,
    request: Vec<u8>,
    executions: Vec<ExecutionSnapshot>,
}

/// Checked immutable numerical templates and original compiler-issued custody.
/// No `AdmittedBody`, scalar family, source recipe or producer control is retained.
#[derive(Debug)]
pub struct AcquiredExecution {
    pub(crate) request: Arc<NumericalObservationRequest>,
    pub(crate) templates: Vec<Arc<RestoredExecutionTemplate>>,
    pub(crate) encoded: Arc<Vec<u8>>,
    owner: Arc<pse_columnar::AllocationLease>,
}
impl AcquiredExecution {
    /// Original compiler-issued ordered observation demand and physical binding.
    pub fn request(&self) -> &Arc<NumericalObservationRequest> {
        &self.request
    }
    /// Checked numerical programs, in original instance order.
    pub fn templates(&self) -> &[Arc<RestoredExecutionTemplate>] {
        &self.templates
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.encoded.capacity()
            + self.request.retained_bytes()
            + self
                .templates
                .iter()
                .map(|template| template.extents().template_bytes)
                .sum::<usize>()
    }
}

/// A freshly restored numerical product, or an identical already admitted local
/// realization whose canonical bytes were just qualified under the current read.
pub(crate) enum ExecutionAcquisition {
    Restored(AcquiredExecution),
    Retained(Arc<super::super::NumericalArtifact>),
}

fn transfer_error<E: std::fmt::Debug + std::fmt::Display>(
    error: pse_math::transfer::TransferError<E>,
    admission: impl FnOnce(E) -> PortableError,
) -> PortableError {
    use pse_math::transfer::TransferError as E;
    match error {
        E::Math(error) => error.into(),
        E::Admission(error) => admission(error),
        E::Overflow(_) | E::Allocation(_) => {
            pse_math::MathError::Limit("numerical transfer backing").into()
        }
        other => PortableError::Qualification(other.to_string()),
    }
}
fn native_transfer_error(
    error: pse_math::transfer::TransferError<super::super::MathRuntimeError>,
) -> PortableError {
    transfer_error(error, |cause| {
        let diagnostic = pse_model::diagnostic::project_typed(
            &cause,
            pse_diagnostics::DiagnosticStage::ModelingAdmission,
        );
        PortableError::Math(pse_math::MathError::Typed {
            retained: size_of_val(&diagnostic) + pse_model::HeapUsage::heap_bytes(&diagnostic),
            cause: pse_model::diagnostic::DiagnosticCause::new(diagnostic),
        })
    })
}
fn passive_transfer_error(
    error: pse_math::transfer::TransferError<std::convert::Infallible>,
) -> PortableError {
    transfer_error(error, |never| match never {})
}

struct Count<'a> {
    bytes: usize,
    scope: &'a pse_kernels::ExecutionScope,
}
impl std::io::Write for Count<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.scope
            .check()
            .map_err(|cause| std::io::Error::other(cause.to_string()))?;
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_NUMERICAL_BYTES)
            .ok_or_else(|| std::io::Error::other("numerical payload extent"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[derive(Default)]
struct BoundedNesting {
    depth: usize,
}
impl BoundedNesting {
    fn enter(&mut self) -> std::io::Result<()> {
        self.depth += 1;
        if self.depth > MAX_NUMERICAL_JSON_DEPTH {
            return Err(std::io::Error::other("numerical wire nesting extent"));
        }
        Ok(())
    }
}
impl serde_json::ser::Formatter for BoundedNesting {
    fn begin_array<W: std::io::Write + ?Sized>(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.enter()?;
        writer.write_all(b"[")
    }
    fn end_array<W: std::io::Write + ?Sized>(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.depth -= 1;
        writer.write_all(b"]")
    }
    fn begin_object<W: std::io::Write + ?Sized>(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.enter()?;
        writer.write_all(b"{")
    }
    fn end_object<W: std::io::Write + ?Sized>(&mut self, writer: &mut W) -> std::io::Result<()> {
        self.depth -= 1;
        writer.write_all(b"}")
    }
}
struct BoundedEncoding<'a> {
    bytes: Vec<u8>,
    limit: usize,
    scope: &'a pse_kernels::ExecutionScope,
}
impl std::io::Write for BoundedEncoding<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.scope
            .check()
            .map_err(|cause| std::io::Error::other(cause.to_string()))?;
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > self.limit)
        {
            return Err(std::io::Error::other(
                "numerical payload changed during encoding",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Capture only the actual post-preparation programs chosen by the sealed warm recipe.
/// The caller preserves the corresponding semantic seals before normalized grouping.
pub(crate) fn capture_execution(
    service: &super::super::MathService,
    request: Arc<NumericalObservationRequest>,
    programs: &[Arc<super::super::Artifact>],
    scope: &pse_kernels::ExecutionScope,
) -> Result<AcquiredExecution, PortableError> {
    if programs.len() != request.instances().len() {
        return Err(PortableError::Qualification(
            "numerical producer instance population".into(),
        ));
    }
    let _metadata = reserve_portable(
        service,
        "math:numerical-capture-descriptors",
        programs
            .len()
            .checked_mul(size_of::<CapturedExecution>() + size_of::<&ExecutionSnapshot>() + 64)
            .ok_or_else(|| PortableError::Qualification("numerical capture extent".into()))?,
    )?;
    let mut captures = Vec::with_capacity(programs.len());
    for program in programs {
        captures.push(
            pse_math::transfer::capture(&program.program, scope, |facts| {
                service
                    .reserve("math:numerical-capture", facts.capture_bytes)
                    .map(|owner| -> Arc<dyn pse_math::AllocationOwner> { owner })
            })
            .map_err(native_transfer_error)?,
        );
    }
    for (capture, instance) in captures.iter().zip(request.instances()) {
        capture
            .snapshot()
            .check_source_attribution(instance.attribution())
            .map_err(|cause| PortableError::Qualification(cause.to_string()))?;
    }
    let outgoing = Outgoing {
        interpretation: NUMERICAL_INTERPRETATION,
        request: request.request_bytes(),
        executions: captures.iter().map(CapturedExecution::snapshot).collect(),
    };
    let mut count = Count { bytes: 0, scope };
    outgoing
        .serialize(&mut serde_json::Serializer::with_formatter(
            &mut count,
            BoundedNesting::default(),
        ))
        .map_err(|cause| {
            receiving_checkpoint(scope.cancellation(), scope.deadline())
                .err()
                .unwrap_or_else(|| PortableError::Qualification(cause.to_string()))
        })?;
    receiving_checkpoint(scope.cancellation(), scope.deadline())?;
    let shell = programs
        .len()
        .checked_mul(size_of::<Arc<RestoredExecutionTemplate>>())
        .and_then(|n| {
            n.checked_add(
                size_of::<AcquiredExecution>() + size_of::<Vec<u8>>() + 4 * size_of::<usize>(),
            )
        })
        .and_then(|n| n.checked_add(count.bytes))
        .ok_or_else(|| PortableError::Qualification("numerical encoding shell extent".into()))?;
    let owner = reserve_portable(service, "math:numerical-encoding", shell)?;
    let mut encoded = BoundedEncoding {
        bytes: Vec::new(),
        limit: count.bytes,
        scope,
    };
    encoded
        .bytes
        .try_reserve_exact(count.bytes)
        .map_err(|cause| PortableError::Qualification(cause.to_string()))?;
    serde_json::to_writer(&mut encoded, &outgoing).map_err(|cause| {
        receiving_checkpoint(scope.cancellation(), scope.deadline())
            .err()
            .unwrap_or_else(|| PortableError::Qualification(cause.to_string()))
    })?;
    if encoded.bytes.len() != count.bytes {
        return Err(PortableError::Qualification(
            "numerical encoded extent changed".into(),
        ));
    }
    let templates = restore_captures(service, &request, captures, scope)?;
    receiving_checkpoint(scope.cancellation(), scope.deadline())?;
    Ok(AcquiredExecution {
        request,
        templates,
        encoded: Arc::new(encoded.bytes),
        owner,
    })
}

fn restore_captures(
    service: &super::super::MathService,
    request: &Arc<NumericalObservationRequest>,
    captures: Vec<CapturedExecution>,
    scope: &pse_kernels::ExecutionScope,
) -> Result<Vec<Arc<RestoredExecutionTemplate>>, PortableError> {
    let construction = pse_math::construction::current().ok_or_else(|| {
        PortableError::Qualification("numerical restore lacks original construction owner".into())
    })?;
    let mut templates = Vec::with_capacity(captures.len());
    for capture in captures {
        let checked =
            pse_math::transfer::restore(capture, scope.clone(), construction.clone(), |facts| {
                service
                    .reserve("math:numerical-restored-template", facts.restore_peak_bytes)
                    .map(|owner| -> Arc<dyn pse_math::AllocationOwner> { owner })
            })
            .map_err(native_transfer_error)?;
        templates.push(Arc::new(checked.into_template()));
    }
    // Exact request correspondence and original closure custody are checked by
    // the execution-only mathematical binding before any template is published.
    request.binding().validate_templates(&templates)?;
    Ok(templates)
}

fn product_key(
    request: &NumericalObservationRequest,
    namespace: &ValidatedReplayNamespace,
    payload: &[u8],
) -> String {
    let mut hash = FramedHasher::new(Frame::NumericalObservationProductV1);
    hash.str(NUMERICAL_INTERPRETATION)
        .part(request.request_bytes())
        .str(namespace.producer_key())
        .str(namespace.producer_class().as_str())
        .part(payload);
    hash.finish_hash().to_hex()
}

/// Publish the distinct numerical interpretation under the exact protected source read.
#[allow(
    unsafe_code,
    reason = "ADR-0264 numerical descriptions require compiler custody and a currently qualified scientific producer namespace"
)]
pub(crate) async fn publish_execution_current(
    service: &super::super::MathService,
    store: &CanonicalStore,
    read: &SelectedRead,
    namespace: &ValidatedReplayNamespace,
    execution: &AcquiredExecution,
    deadline: tokio::time::Instant,
) -> Result<String, PortableError> {
    let key = product_key(&execution.request, namespace, &execution.encoded);
    let selection = selection_material(service, read)?;
    let revision = read.selection().revision();
    let _descriptor = reserve_portable(
        service,
        "math:numerical-product-description",
        execution.request.request_bytes().len() + 64 * 1024,
    )?;
    let row = pse_model::generated::runtime::canonical_products::Row {
        key: key.clone(),
        problem: revision.problem.clone(),
        revision: revision.key.clone(),
        request: execution.request.request_bytes().to_vec().into(),
        payload: Vec::new().into(),
        dependencies: Vec::new().into(),
        producer: namespace.producer_key().into(),
        producer_class: namespace.producer_class(),
        interpretation: pse_operations::generated::surreal::INTERPRETATION.into(),
    };
    let description =
        pse_operations::canonical_selection::ProductDescription::prepare_with_shared_payload(
            read,
            row,
            selection,
            execution.encoded.clone(),
            execution.owner.clone(),
        )?;
    Ok(charged_publication(service, || async {
        // SAFETY: only the actual prepared numerical program plus its original
        // compiler-issued request reaches this scientific publication owner.
        unsafe { store.publish_scientific_description_until(read, &description, deadline) }.await
    })
    .await??)
}

/// Acquire an execution-only product. A present incompatible/corrupt product is
/// an error; source replay is only a distinct cold-miss producer operation.
pub(crate) async fn reuse_execution_current(
    service: &Arc<super::super::MathService>,
    store: &CanonicalStore,
    read: &mut SelectedRead,
    namespace: &ValidatedReplayNamespace,
    request: Arc<NumericalObservationRequest>,
    cancelled: &Arc<AtomicBool>,
) -> Result<Option<ExecutionAcquisition>, PortableError> {
    let mut after = String::new();
    loop {
        receiving_checkpoint(cancelled, namespace.deadline)?;
        let Some(candidate) = store
            .product_candidate(
                read.selection(),
                request.request_bytes(),
                namespace.producer_key(),
                namespace.producer_class(),
                &after,
            )
            .await?
        else {
            return Ok(None);
        };
        let (product, payload_owner) = qualify_candidate(service, store, read, &candidate).await?;
        let Some(product) = product else {
            after = candidate.key().into();
            continue;
        };
        // The canonical owner wraps the scientific content key in its original
        // selection/receipt association. Its qualified key is that association,
        // not the inner mathematical content key supplied at publication.
        if product.payload().len() > MAX_NUMERICAL_BYTES {
            return Err(PortableError::Qualification(
                "numerical product extent".into(),
            ));
        }
        if let Some(retained) = service.retained_numerical_artifact(&request)
            && retained.execution.encoded.as_slice() == product.payload()
        {
            // Qualification above still hydrates and authenticates the current
            // protected candidate, so a warm entry cannot hide corrupt storage.
            // Identical bytes have already passed the full passive/custody checks;
            // keep the existing realized template instead of restoring it again.
            receiving_checkpoint(cancelled, namespace.deadline)?;
            return Ok(Some(ExecutionAcquisition::Retained(retained)));
        }
        let scratch = product
            .payload()
            .len()
            .checked_mul(256)
            .and_then(|n| n.checked_add(request.retained_bytes()))
            .ok_or_else(|| PortableError::Qualification("numerical decode extent".into()))?;
        let decode_owner = reserve_portable(service, "math:numerical-passive-decode", scratch)?;
        let shell = request
            .instances()
            .len()
            .checked_mul(size_of::<Arc<RestoredExecutionTemplate>>())
            .and_then(|n| {
                n.checked_add(
                    size_of::<AcquiredExecution>() + size_of::<Vec<u8>>() + 4 * size_of::<usize>(),
                )
            })
            .and_then(|n| n.checked_add(product.payload().len()))
            .ok_or_else(|| {
                PortableError::Qualification("numerical receiving shell extent".into())
            })?;
        let encoded_owner = reserve_portable(service, "math:numerical-retained-encoding", shell)?;
        let service_copy = service.clone();
        let current = namespace.clone();
        return native_receiving_scoped(
            service,
            cancelled,
            namespace.deadline,
            super::super::WITHIN_WORKSPACE,
            move |control| {
                let _payload_owner = payload_owner;
                let flag = control.flag();
                let scope = pse_kernels::ExecutionScope::new(control, current.deadline);
                receiving_checkpoint(&flag, current.deadline)?;
                current.reconstruct(&flag, || {
                    let incoming: Incoming = serde_json::from_slice(product.payload())
                        .map_err(|cause| PortableError::Qualification(cause.to_string()))?;
                    if incoming.interpretation != NUMERICAL_INTERPRETATION
                        || incoming.request != request.request_bytes()
                        || incoming.executions.len() != request.instances().len()
                    {
                        return Err(PortableError::Qualification(
                            "numerical request or original custody differs".into(),
                        ));
                    }
                    let mut passive_bytes = 0_usize;
                    for (snapshot, instance) in incoming.executions.iter().zip(request.instances())
                    {
                        let facts = snapshot.inspect().map_err(passive_transfer_error)?;
                        passive_bytes =
                            passive_bytes
                                .checked_add(facts.input_bytes)
                                .ok_or_else(|| {
                                    PortableError::Qualification(
                                        "numerical passive snapshot extent".into(),
                                    )
                                })?;
                        snapshot
                            .check_source_attribution(instance.attribution())
                            .map_err(|cause| PortableError::Qualification(cause.to_string()))?;
                    }
                    let temporary =
                        decode_owner
                            .size()
                            .checked_sub(passive_bytes)
                            .ok_or_else(|| {
                                PortableError::Qualification(
                                    "numerical decoded backing exceeds admission".into(),
                                )
                            })?;
                    let mut owners = decode_owner
                        .partition(&[passive_bytes, temporary])
                        .map_err(|_| {
                            PortableError::Qualification(
                                "numerical decode owner is shared or insufficient".into(),
                            )
                        })?
                        .into_iter();
                    let passive_owner = owners.next().ok_or_else(|| {
                        PortableError::Qualification("numerical passive decode lease absent".into())
                    })?;
                    let _temporary_owner = owners.next().ok_or_else(|| {
                        PortableError::Qualification(
                            "numerical temporary decode lease absent".into(),
                        )
                    })?;
                    let captures = incoming
                        .executions
                        .into_iter()
                        .map(|snapshot| {
                            CapturedExecution::from_admitted_snapshot(
                                snapshot,
                                passive_owner.clone(),
                            )
                        })
                        .collect();
                    let templates = restore_captures(&service_copy, &request, captures, &scope)?;
                    receiving_checkpoint(&flag, current.deadline)?;
                    Ok(AcquiredExecution {
                        request,
                        templates,
                        encoded: Arc::new(product.payload().to_vec()),
                        owner: encoded_owner,
                    })
                })
            },
        )
        .await
        .map(|execution| Some(ExecutionAcquisition::Restored(execution)));
    }
}
