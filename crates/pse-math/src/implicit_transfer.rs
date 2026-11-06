// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checked numerical Root capture and passive metadata restoration. Origin belongs
//! to the private runtime issuer and authenticated attempt, never these raw records.
use super::*;
use crate::{
    AllocationOwner, AttributionTransferError, ExecutionAttributionSnapshot,
    construction::ConstructionAdmission,
    transfer::{CapturedExecution, CheckedExecution, TransferExtents},
};
use pse_model::diagnostic::SourceLocation;
use std::{collections::TryReserveError, convert::Infallible};

pse_contract::projection_struct! {
/// Original numerical Root definition. A matching attribution checksum detects
/// inconsistent data; it cannot authenticate a caller-controlled replacement.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootDefinitionSnapshot {
    projection: { name: "MathRootDefinitionSnapshot", rust_path: crate::implicit::transfer::RootDefinitionSnapshot, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Fixed private numerical Root profile, independent of wire recursion allowance.
    pub version: u32 => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::UnsignedRange { min: 1, max: 1 }] };
    /// Original implicit system identity.
    pub id: SemanticId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original residual content identity, including original residual offsets.
    pub identity: ContentHash => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Unknowns in original leading-formal order; this profile requires finite intervals.
    pub unknowns: Vec<Unknown> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original equation identities in selected output order.
    pub rows: Vec<SemanticId> => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original independent parameter population after the leading unknown formals.
    pub inputs: usize => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original finite offsets subtracted from residuals in equation order, preserving bits.
    pub residual_offsets: Vec<f64> => { tag: 7, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Float64Domain(pse_contract::boundary_projection::Float64Domain::Finite)] };
    /// Original resolved derivative facts; these do not grant later source preparation.
    pub requirements: pse_kernels::DerivativeRequirements => { tag: 8, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// Original native unknown product order; the fixed transfer profile is First.
    pub native_order: DerivativeOrder => { tag: 9, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Compact CSC column pointers, starting zero and ending at row_idx.len().
    pub col_ptr: Vec<usize> => { tag: 10, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Strictly increasing row indices within every original CSC column.
    pub row_idx: Vec<usize> => { tag: 11, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Checksum of the complete original attribution framing, sealed by the parent issuer.
    pub attribution_commitment: ContentHash => { tag: 12, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Complete passive capture of an actual held RootWorker and its original metadata.
/// Decoding or consistently replacing these records establishes no attempt origin.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootExecutionSnapshot {
    projection: { name: "MathRootExecutionSnapshot", rust_path: crate::implicit::transfer::RootExecutionSnapshot, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Original numerical Root signature, CSC and attribution checksum.
    pub definition: RootDefinitionSnapshot => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// Exact complete original maps, not a filtered token subset.
    pub attribution: ExecutionAttributionSnapshot => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Required] };
    /// Exact cold numerical template from the actual held RootWorker.
    pub numerical: crate::transfer::ExecutionSnapshot => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both,
        rules: [pse_contract::boundary_projection::LocalRule::Required] };
}
}

/// Populations admitted before original metadata capture. Backing is a checked Rust
/// upper bound, not allocator usable size, RSS or a wire/stack allowance.
#[derive(Clone, Debug)]
pub struct RootCaptureExtents {
    /// Actual numerical-template populations and requested backing.
    pub numerical: TransferExtents,
    /// Requested Root definition and full original attribution backing upper bound.
    pub metadata_bytes: usize,
    /// Combined passive output backing upper bound before capture effects.
    pub capture_bytes: usize,
    /// Actual original attribution entries across all three maps.
    pub attribution_entries: usize,
    /// Actual original UTF-8 string bytes to clone.
    pub attribution_text_bytes: usize,
}
/// Derived Root metadata/frame construction demand; input ownership is already
/// admitted by the boundary and must remain live while moved into the Root.
#[derive(Clone, Debug)]
pub struct RootRestoreExtents {
    /// Checked numerical demand from actual owned input, including retained capacities.
    pub numerical: TransferExtents,
    /// Existing owned metadata backing upper bound retained across construction.
    pub metadata_input_bytes: usize,
    /// New Root Arc, owner anchors and freshly allocated argument/cache backing.
    pub construction_bytes: usize,
    /// Admitted identity validation scratch released before final publication.
    pub validation_bytes: usize,
    /// Combined requested backing bound including still-live metadata/numerical input.
    pub restore_peak_bytes: usize,
    /// Actual square unknown/equation population.
    pub unknowns: usize,
    /// Actual compact numerical input population.
    pub numerical_inputs: usize,
    /// Actual original sparse CSC entries, including conservative extras.
    pub csc_entries: usize,
}

/// Original admission, numerical, metadata and control causes remain typed.
#[derive(Debug, thiserror::Error)]
pub enum RootTransferError<E: std::fmt::Debug + std::fmt::Display> {
    /// A well-defined original Root outside this private delegation capability.
    #[error("unsupported numerical Root profile: {0}")]
    Unsupported(&'static str),
    /// Inconsistent passive definition or malformed structural data.
    #[error("numerical Root transfer: {0}")]
    Invalid(&'static str),
    /// Original checked numerical-template refusal.
    #[error("numerical Root template: {0}")]
    Numerical(#[source] crate::transfer::TransferError<E>),
    /// Original attribution shape, full-map checksum or Closure coverage refusal.
    #[error("numerical Root attribution: {0}")]
    Attribution(#[from] AttributionTransferError),
    /// Original derivative capability/requirement refusal.
    #[error("numerical Root requirements: {0}")]
    Requirements(#[source] pse_kernels::ProviderError),
    /// Original mathematical limit, cancellation or deadline.
    #[error("{0}")]
    Math(#[from] MathError),
    /// Actual fallible reservation failure.
    #[error("numerical Root reservation: {0}")]
    Allocation(#[source] TryReserveError),
    /// Unchanged caller admission/cancellation error.
    #[error("numerical Root admission: {0}")]
    Admission(#[source] E),
}
impl<E: std::fmt::Debug + std::fmt::Display + std::error::Error + 'static> miette::Diagnostic
    for RootTransferError<E>
{
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            Self::Math(error) => miette::Diagnostic::code(error),
            _ => Some(Box::new("math.root_transfer")),
        }
    }
}
type Result<T> = std::result::Result<T, RootTransferError<Infallible>>;
fn add(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b)
        .ok_or_else(|| RootTransferError::Math(MathError::Limit("Root transfer extent")))
}
fn backing<T>(entries: usize) -> Result<usize> {
    std::alloc::Layout::array::<T>(entries)
        .map(|layout| layout.size())
        .map_err(|_| RootTransferError::Math(MathError::Limit("Root transfer backing")))
}
fn arc<T>() -> Result<usize> {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map(|(layout, _)| layout.pad_to_align().size())
        .map_err(|_| RootTransferError::Math(MathError::Limit("Root transfer Arc")))
}
fn reserve<T>(entries: usize) -> Result<Vec<T>> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(entries)
        .map_err(RootTransferError::<Infallible>::Allocation)?;
    Ok(result)
}
fn copy<T: Copy>(values: &[T]) -> Result<Vec<T>> {
    let mut result = reserve(values.len())?;
    result.extend_from_slice(values);
    Ok(result)
}
fn widen<E: std::fmt::Debug + std::fmt::Display>(
    error: RootTransferError<Infallible>,
) -> RootTransferError<E> {
    match error {
        RootTransferError::Unsupported(e) => RootTransferError::Unsupported(e),
        RootTransferError::Invalid(e) => RootTransferError::Invalid(e),
        RootTransferError::Attribution(e) => RootTransferError::Attribution(e),
        RootTransferError::Requirements(e) => RootTransferError::Requirements(e),
        RootTransferError::Math(e) => RootTransferError::Math(e),
        RootTransferError::Allocation(e) => RootTransferError::Allocation(e),
        RootTransferError::Admission(e) => match e {},
        RootTransferError::Numerical(e) => RootTransferError::Numerical(match e {
            crate::transfer::TransferError::Unsupported(e) => {
                crate::transfer::TransferError::Unsupported(e)
            }
            crate::transfer::TransferError::Invalid(e) => {
                crate::transfer::TransferError::Invalid(e)
            }
            crate::transfer::TransferError::Overflow(e) => {
                crate::transfer::TransferError::Overflow(e)
            }
            crate::transfer::TransferError::Supplier(e) => {
                crate::transfer::TransferError::Supplier(e)
            }
            crate::transfer::TransferError::Layout(e) => crate::transfer::TransferError::Layout(e),
            crate::transfer::TransferError::Math(e) => crate::transfer::TransferError::Math(e),
            crate::transfer::TransferError::Allocation(e) => {
                crate::transfer::TransferError::Allocation(e)
            }
            crate::transfer::TransferError::Admission(e) => match e {},
        }),
    }
}

#[derive(Clone, Copy)]
struct Definition<'a> {
    version: u32,
    id: SemanticId,
    identity: ContentHash,
    unknowns: &'a [Unknown],
    rows: &'a [SemanticId],
    inputs: usize,
    offsets: &'a [f64],
    requirements: pse_kernels::DerivativeRequirements,
    native_order: DerivativeOrder,
    col_ptr: &'a [usize],
    row_idx: &'a [usize],
    commitment: ContentHash,
}
impl<'a> Definition<'a> {
    fn original(root: &'a RootExecution, commitment: ContentHash) -> Result<Self> {
        let pattern = root.pattern();
        if pattern.nrows() != root.unknowns.len()
            || pattern.ncols() != root.unknowns.len()
            || pattern.col_nnz().is_some()
        {
            return Err(RootTransferError::Invalid("compact square CSC profile"));
        }
        Ok(Self {
            version: 1,
            id: root.id,
            identity: root.identity,
            unknowns: &root.unknowns,
            rows: &root.rows,
            inputs: root.inputs,
            offsets: &root.residual_offsets,
            requirements: root.requirements,
            native_order: root.native_order,
            col_ptr: root.pattern.col_ptr(),
            row_idx: root.pattern.row_idx(),
            commitment,
        })
    }
    fn passive(value: &'a RootDefinitionSnapshot) -> Self {
        Self {
            version: value.version,
            id: value.id,
            identity: value.identity,
            unknowns: &value.unknowns,
            rows: &value.rows,
            inputs: value.inputs,
            offsets: &value.residual_offsets,
            requirements: value.requirements,
            native_order: value.native_order,
            col_ptr: &value.col_ptr,
            row_idx: &value.row_idx,
            commitment: value.attribution_commitment,
        }
    }
    fn validate(self, numeric: Signature<'_>) -> Result<()> {
        let n = self.unknowns.len();
        let width = add(n, self.inputs)?;
        self.requirements
            .validate()
            .map_err(RootTransferError::Requirements)?;
        if self.version != 1
            || self.native_order != DerivativeOrder::First
            || self.native_order < self.requirements.inner_minimum
        {
            return Err(RootTransferError::Unsupported("Root version/native order"));
        }
        if self.id == SemanticId::NIL
            || n == 0
            || n > isize::MAX as usize
            || self.rows.len() != n
            || self.offsets.len() != n
            || numeric.outputs() != n
        {
            return Err(RootTransferError::Invalid("Root signature/order profile"));
        }
        if self.unknowns.iter().any(|u| {
            u.id == SemanticId::NIL
                || u.lower.is_nan()
                || u.upper.is_nan()
                || u.lower > u.upper
                || u.lower == f64::INFINITY
                || u.upper == f64::NEG_INFINITY
        }) || self.rows.contains(&SemanticId::NIL)
            || self.offsets.iter().any(|value| !value.is_finite())
        {
            return Err(RootTransferError::Invalid(
                "original identities/intervals/offsets",
            ));
        }
        if self
            .unknowns
            .iter()
            .any(|u| !u.lower.is_finite() || !u.upper.is_finite())
        {
            return Err(RootTransferError::Unsupported(
                "finite Root interval profile",
            ));
        }
        let (formals, coordinates) = numeric.inputs();
        if formals.iter().any(|&formal| formal >= width) || coordinates.len() != n {
            return Err(RootTransferError::Invalid(
                "Root numerical input/coordinate extent",
            ));
        }
        for (axis, &offset) in coordinates.iter().enumerate() {
            if formals.get(offset) != Some(&axis) {
                return Err(RootTransferError::Invalid(
                    "Root unknown authored coordinate order",
                ));
            }
        }
        if self.col_ptr.len() != add(n, 1)?
            || self.col_ptr.first() != Some(&0)
            || self.col_ptr.last() != Some(&self.row_idx.len())
        {
            return Err(RootTransferError::Invalid("compact CSC pointer extent"));
        }
        for pair in self.col_ptr.windows(2) {
            let rows = self
                .row_idx
                .get(pair[0]..pair[1])
                .ok_or(RootTransferError::Invalid("CSC pointer range"))?;
            if rows.iter().any(|&row| row >= n) || rows.windows(2).any(|pair| pair[0] >= pair[1]) {
                return Err(RootTransferError::Invalid("sorted unique CSC row indices"));
            }
        }
        for row in 0..n {
            for &axis in numeric
                .axes(row)
                .ok_or(RootTransferError::Invalid("First row extent"))?
            {
                let pair = self
                    .col_ptr
                    .get(axis..add(axis, 2)?)
                    .ok_or(RootTransferError::Invalid("First axis CSC column"))?;
                if self.row_idx[pair[0]..pair[1]].binary_search(&row).is_err() {
                    return Err(RootTransferError::Invalid(
                        "CSC omits actual First derivative position",
                    ));
                }
            }
        }
        Ok(())
    }
    fn clone_bytes(self) -> Result<usize> {
        add(
            add(
                add(
                    size_of::<RootDefinitionSnapshot>(),
                    backing::<Unknown>(self.unknowns.len())?,
                )?,
                backing::<SemanticId>(self.rows.len())?,
            )?,
            add(
                backing::<f64>(self.offsets.len())?,
                backing::<usize>(add(self.col_ptr.len(), self.row_idx.len())?)?,
            )?,
        )
    }
    fn capture(self) -> Result<RootDefinitionSnapshot> {
        let mut unknowns = reserve(self.unknowns.len())?;
        unknowns.extend_from_slice(self.unknowns);
        Ok(RootDefinitionSnapshot {
            version: self.version,
            id: self.id,
            identity: self.identity,
            unknowns,
            rows: copy(self.rows)?,
            inputs: self.inputs,
            residual_offsets: copy(self.offsets)?,
            requirements: self.requirements,
            native_order: self.native_order,
            col_ptr: copy(self.col_ptr)?,
            row_idx: copy(self.row_idx)?,
            attribution_commitment: self.commitment,
        })
    }
}
#[derive(Clone, Copy)]
enum Signature<'a> {
    Passive(&'a crate::transfer::ExecutionSnapshot),
    Checked(&'a CheckedExecution),
    Original(&'a NumericFrame),
}
impl<'a> Signature<'a> {
    fn inputs(self) -> (&'a [usize], &'a [usize]) {
        match self {
            Self::Passive(s) => (&s.input_formals, &s.coordinate_offsets),
            Self::Checked(s) => (s.input_formals(), s.coordinate_offsets()),
            Self::Original(s) => (s.input_formals(), s.transfer_coordinates()),
        }
    }
    fn outputs(self) -> usize {
        match self {
            Self::Passive(s) => s.outputs.len(),
            Self::Checked(s) => s.output_count(),
            Self::Original(s) => s.transfer_outputs(),
        }
    }
    fn axes(self, row: usize) -> Option<&'a [usize]> {
        match self {
            Self::Passive(s) => s
                .first_rows
                .get(row)
                .and_then(|&(start, end)| s.first_axes.get(start..end)),
            Self::Checked(s) => s.first_axis_slice(row),
            Self::Original(s) => s.transfer_first_axes(row),
        }
    }
}

/// Captured passive output retains its actual allocation owner through publication.
#[derive(Debug)]
pub struct CapturedRootExecution {
    snapshot: RootExecutionSnapshot,
    owner: Arc<dyn AllocationOwner>,
}
impl CapturedRootExecution {
    fn check_scope(self, scope: &pse_kernels::ExecutionScope) -> Result<Self> {
        // Keep the complete snapshot and its lease in this owning value across
        // the last refusal: struct fields drop snapshot first, then owner.
        scope.check().map_err(crate::error::scope_error)?;
        Ok(self)
    }
    /// Borrow records while preserving their actual allocation lifetime.
    pub fn snapshot(&self) -> &RootExecutionSnapshot {
        &self.snapshot
    }
    /// The publisher retains the returned actual owner through encoding/launch/reap.
    pub fn into_parts(self) -> (RootExecutionSnapshot, Arc<dyn AllocationOwner>) {
        (self.snapshot, self.owner)
    }
}
/// Owned passive definition/attribution, with the boundary's original allocation owner.
#[derive(Debug)]
pub struct RetainedRootMetadata {
    definition: RootDefinitionSnapshot,
    attribution: ExecutionAttributionSnapshot,
    owner: Arc<dyn AllocationOwner>,
}
impl RetainedRootMetadata {
    /// Ownership-only binding after already-admitted decode/conversion. This grants
    /// no source, Root or attempt authority and validates nothing.
    pub fn from_admitted_parts(
        definition: RootDefinitionSnapshot,
        attribution: ExecutionAttributionSnapshot,
        owner: Arc<dyn AllocationOwner>,
    ) -> Self {
        Self {
            definition,
            attribution,
            owner,
        }
    }
}
impl RootExecutionSnapshot {
    /// Split already-admitted passive backing; the same original owner follows
    /// numerical input and metadata. Decode/conversion/path scratch is pre-admitted
    /// separately by the runtime, never retroactively by restoration callbacks.
    pub fn into_restore_parts(
        self,
        owner: Arc<dyn AllocationOwner>,
    ) -> (CapturedExecution, RetainedRootMetadata) {
        (
            CapturedExecution::from_admitted_snapshot(self.numerical, owner.clone()),
            RetainedRootMetadata::from_admitted_parts(self.definition, self.attribution, owner),
        )
    }
    /// Allocation-free checked planning over actual owned decoded populations.
    /// Full admitted identity/Closure/dataflow validation is still required.
    pub fn restore_extents(&self) -> Result<RootRestoreExtents> {
        let numerical = self
            .numerical
            .inspect()
            .map_err(RootTransferError::Numerical)?;
        metadata_facts(
            &self.definition,
            &self.attribution,
            Signature::Passive(&self.numerical),
            numerical,
        )
    }
}
fn metadata_facts(
    definition: &RootDefinitionSnapshot,
    attribution: &ExecutionAttributionSnapshot,
    signature: Signature<'_>,
    numerical: TransferExtents,
) -> Result<RootRestoreExtents> {
    let view = Definition::passive(definition);
    view.validate(signature)?;
    attribution.check_commitment(view.commitment)?;
    let attributed = attribution.extents()?;
    let metadata_input_bytes = add(
        add(
            add(
                size_of::<RootDefinitionSnapshot>(),
                backing::<Unknown>(definition.unknowns.capacity())?,
            )?,
            backing::<SemanticId>(definition.rows.capacity())?,
        )?,
        add(
            backing::<f64>(definition.residual_offsets.capacity())?,
            backing::<usize>(add(
                definition.col_ptr.capacity(),
                definition.row_idx.capacity(),
            )?)?,
        )?,
    )?;
    let metadata_input_bytes = add(
        metadata_input_bytes,
        add(
            size_of::<ExecutionAttributionSnapshot>(),
            attributed.owned_bytes,
        )?,
    )?;
    let validation_bytes = backing::<SemanticId>(view.unknowns.len())?;
    let mut construction_bytes = arc::<RootExecution>()?;
    construction_bytes = add(
        construction_bytes,
        backing::<f64>(signature.inputs().0.len())?,
    )?;
    construction_bytes = add(construction_bytes, numerical.evaluation_bytes)?;
    construction_bytes = add(
        construction_bytes,
        add(
            add(
                arc::<BTreeMap<SemanticId, SemanticId>>()?,
                arc::<BTreeMap<SemanticId, SemanticId>>()?,
            )?,
            arc::<BTreeMap<SemanticId, SourceLocation>>()?,
        )?,
    )?;
    construction_bytes = add(
        construction_bytes,
        arc::<(
            Arc<dyn AllocationOwner>,
            Arc<dyn AllocationOwner>,
            Arc<dyn ConstructionAdmission>,
        )>()?,
    )?;
    construction_bytes = add(
        construction_bytes,
        arc::<(Arc<dyn AllocationOwner>, Arc<dyn AllocationOwner>)>()?,
    )?;
    let restore_peak_bytes = add(
        numerical.restore_peak_bytes,
        add(
            metadata_input_bytes,
            add(construction_bytes, validation_bytes)?,
        )?,
    )?;
    Ok(RootRestoreExtents {
        numerical,
        metadata_input_bytes,
        construction_bytes,
        validation_bytes,
        restore_peak_bytes,
        unknowns: view.unknowns.len(),
        numerical_inputs: signature.inputs().0.len(),
        csc_entries: view.row_idx.len(),
    })
}

/// Capture the actual locked original RootWorker; an unrelated body cannot be supplied.
pub fn capture<E: std::fmt::Debug + std::fmt::Display>(
    root: &RootExecution,
    scope: &pse_kernels::ExecutionScope,
    admit: impl FnOnce(&RootCaptureExtents) -> std::result::Result<Arc<dyn AllocationOwner>, E>,
) -> std::result::Result<CapturedRootExecution, RootTransferError<E>> {
    scope.check().map_err(crate::error::scope_error)?;
    let held = root
        .unknown_worker
        .lock()
        .map_err(|_| MathError::PoisonedOwner {
            owner: "Root capture",
        })?;
    let attribution = &held.worker.attribution;
    crate::transfer::check_closure_attribution(
        &held.worker.numeric,
        attribution.checked_members(),
        attribution.checked_sources(),
    )?;
    let attributed = attribution.snapshot_extents()?;
    let view = Definition::original(root, attribution.commitment()).map_err(widen::<E>)?;
    view.validate(Signature::Original(&held.worker.numeric))
        .map_err(widen::<E>)?;
    let metadata_bytes = add(
        view.clone_bytes().map_err(widen::<E>)?,
        add(
            size_of::<ExecutionAttributionSnapshot>(),
            attributed.clone_bytes,
        )
        .map_err(widen::<E>)?,
    )
    .map_err(widen::<E>)?;
    let validation_bytes = backing::<SemanticId>(root.unknowns.len()).map_err(widen::<E>)?;
    let numerical = crate::transfer::capture_frame(&held.worker.numeric, scope, |numerical| {
        let capture_bytes = numerical
            .capture_bytes
            .checked_add(metadata_bytes)
            .and_then(|n| n.checked_add(validation_bytes))
            .ok_or(RootTransferError::Math(MathError::Limit(
                "Root capture extent",
            )))?;
        let facts = RootCaptureExtents {
            numerical: numerical.clone(),
            metadata_bytes,
            capture_bytes,
            attribution_entries: attributed.entries,
            attribution_text_bytes: attributed.clone_string_bytes,
        };
        admit(&facts).map_err(RootTransferError::Admission)
    })
    .map_err(|error| match error {
        crate::transfer::TransferError::Admission(error) => error,
        crate::transfer::TransferError::Unsupported(e) => {
            RootTransferError::Numerical(crate::transfer::TransferError::Unsupported(e))
        }
        crate::transfer::TransferError::Invalid(e) => {
            RootTransferError::Numerical(crate::transfer::TransferError::Invalid(e))
        }
        crate::transfer::TransferError::Overflow(e) => {
            RootTransferError::Numerical(crate::transfer::TransferError::Overflow(e))
        }
        crate::transfer::TransferError::Supplier(e) => {
            RootTransferError::Numerical(crate::transfer::TransferError::Supplier(e))
        }
        crate::transfer::TransferError::Layout(e) => {
            RootTransferError::Numerical(crate::transfer::TransferError::Layout(e))
        }
        crate::transfer::TransferError::Math(e) => {
            RootTransferError::Numerical(crate::transfer::TransferError::Math(e))
        }
        crate::transfer::TransferError::Allocation(e) => {
            RootTransferError::Numerical(crate::transfer::TransferError::Allocation(e))
        }
    })?;
    let (definition, attribution_data) = (|| -> Result<_> {
        validate_identities(view.unknowns, view.rows)?;
        Ok((view.capture()?, attribution.snapshot_after_admission()))
    })()
    .map_err(widen::<E>)?;
    let (numerical, owner) = numerical.into_parts();
    CapturedRootExecution {
        snapshot: RootExecutionSnapshot {
            definition,
            attribution: attribution_data,
            numerical,
        },
        owner,
    }
    .check_scope(scope)
    .map_err(widen::<E>)
}
fn validate_identities(unknowns: &[Unknown], rows: &[SemanticId]) -> Result<()> {
    let mut ids = reserve(unknowns.len())?;
    ids.extend(unknowns.iter().map(|unknown| unknown.id));
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(RootTransferError::Invalid(
            "duplicate Root unknown identities",
        ));
    }
    ids.clear();
    ids.extend_from_slice(rows);
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(RootTransferError::Invalid(
            "duplicate Root equation identities",
        ));
    }
    Ok(())
}

/// Construct only the source-independent Root core after actual metadata admission.
/// No source body, symbolic analysis, factor or later derivative preparation is restored.
pub fn restore<E: std::fmt::Debug + std::fmt::Display>(
    mut checked: CheckedExecution,
    input: RetainedRootMetadata,
    scope: pse_kernels::ExecutionScope,
    construction: Arc<dyn ConstructionAdmission>,
    admit: impl FnOnce(&RootRestoreExtents) -> std::result::Result<Arc<dyn AllocationOwner>, E>,
) -> std::result::Result<Arc<RootExecution>, RootTransferError<E>> {
    let input_owner = input.owner;
    let definition = input.definition;
    let attribution = input.attribution;
    scope.check().map_err(crate::error::scope_error)?;
    let facts = metadata_facts(
        &definition,
        &attribution,
        Signature::Checked(&checked),
        checked.extents().clone(),
    )
    .map_err(widen::<E>)?;
    checked
        .check_closure_attribution(&attribution.checked_members, &attribution.checked_sources)?;
    let owner = admit(&facts).map_err(RootTransferError::Admission)?;
    scope.check().map_err(crate::error::scope_error)?;
    validate_identities(&definition.unknowns, &definition.rows).map_err(widen::<E>)?;
    let anchor: Arc<dyn AllocationOwner> = Arc::new((input_owner, owner, construction.clone()));
    checked.retain_owner(anchor.clone());
    checked.frame.set_scope(scope.clone());
    let mut inputs = reserve(facts.numerical_inputs).map_err(widen::<E>)?;
    inputs.resize(facts.numerical_inputs, f64::NAN);
    let RootDefinitionSnapshot {
        id,
        identity,
        unknowns,
        rows,
        inputs: parameters,
        residual_offsets,
        requirements,
        native_order,
        col_ptr,
        row_idx,
        ..
    } = definition;
    let pattern =
        SymbolicSparseColMat::new_checked(unknowns.len(), unknowns.len(), col_ptr, None, row_idx);
    let root = Arc::new(RootExecution {
        id,
        identity,
        unknowns,
        rows,
        inputs: parameters,
        requirements,
        residual_offsets,
        unknown_worker: Mutex::new(ResidualWorker {
            worker: RootWorker {
                numeric: checked.frame,
                attribution: attribution.into_attribution(),
            },
            inputs,
            evaluation: crate::execution::TrialEvaluation::default(),
        }),
        providers: Mutex::new(pse_kernels::ProviderBindings::new()),
        pattern,
        native_order,
        _construction_owner: Some(construction),
        _metadata_owner: Some(anchor),
    });
    scope.check().map_err(crate::error::scope_error)?;
    Ok(root)
}

#[cfg(test)]
#[path = "implicit_transfer_tests.rs"]
mod tests;
#[cfg(test)]
mod declaration_tests {
    #[test]
    fn root_transfer_records_are_passive_and_reuse_actual_native_owner_records() {
        use super::*;
        use pse_contract::boundary_projection::{
            ConstructionRole, LocalRule, ProjectionGraph, Shape,
        };
        let graph = ProjectionGraph::of::<RootExecutionSnapshot>().unwrap();
        for name in [
            "MathRootExecutionSnapshot",
            "MathRootDefinitionSnapshot",
            "MathExecutionAttributionSnapshot",
            "KernelDerivativeRequirements",
            "ImplicitUnknown",
            "MathExecutionSnapshot",
        ] {
            assert_eq!(
                graph.definitions()[name].construction,
                ConstructionRole::Data
            );
        }
        let Shape::Record(fields) = &graph.definitions()["MathRootDefinitionSnapshot"].shape else {
            panic!("definition record");
        };
        assert_eq!(
            fields
                .iter()
                .find(|field| field.name == "version")
                .unwrap()
                .rules,
            [LocalRule::UnsignedRange { min: 1, max: 1 }]
        );
        assert!(
            fields
                .iter()
                .any(|field| field.name == "attribution_commitment")
        );
    }
}
