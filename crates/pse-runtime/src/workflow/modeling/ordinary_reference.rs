// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent ordinary IDAES correspondence over the source-generated binary contract.
#[path = "../../../tests/reference_support/environment.rs"]
mod reference_environment;
#[path = "../../../tests/reference_support/process.rs"]
mod reference_process;

use super::*;
use crate::reference_exchange::{
    ReferenceCaseContract, ReferenceFixedInput, ReferenceInputArtifact, ReferenceObservableContract,
};
use pse_admission::authoring_driver::document::package_document_paths;
use pse_model::reference_exchange::{
    ComparisonAllowance, ReferenceEnvelope, ReferenceMeaning, ReferenceOutcome, compare,
};
use sha2::Digest as _;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read as _,
    path::{Component, Path, PathBuf},
    process::Command,
    sync::Arc,
    time::Duration,
};

const LIMIT: usize = 16 << 20;
const SOURCE_LIMIT: usize = 64 << 20;
const SOURCE_FILE_LIMIT: usize = 1024;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
struct ScalarComparison {
    role: String,
    output: SemanticId,
    native: f64,
    reference: f64,
    absolute_allowance: f64,
    relative_allowance: f64,
    absolute_error: f64,
    relative_error: Option<f64>,
}

#[derive(Debug, Default)]
struct ComparisonDiagnostics(Vec<ScalarComparison>);

impl std::fmt::Display for ComparisonDiagnostics {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for sample in &self.0 {
            write!(
                formatter,
                " [role={} output={} native={:?} reference={:?} absolute_allowance={:?} relative_allowance={:?} absolute_error={:?} relative_error={:?}]",
                sample.role,
                sample.output,
                sample.native,
                sample.reference,
                sample.absolute_allowance,
                sample.relative_allowance,
                sample.absolute_error,
                sample.relative_error
            )?;
        }
        Ok(())
    }
}

impl ComparisonDiagnostics {
    /// Report every already-held scalar pair; the shared comparator alone decides acceptance.
    fn capture(
        roles: Vec<String>,
        candidate: &ReferenceEnvelope,
        reference: &ReferenceEnvelope,
        allowances: &[ComparisonAllowance],
    ) -> Self {
        let (
            ReferenceOutcome::Completed {
                outputs: native, ..
            },
            ReferenceOutcome::Completed {
                outputs: foreign, ..
            },
        ) = (&candidate.outcome, &reference.outcome)
        else {
            return Self::default();
        };
        let mut samples = Vec::with_capacity(roles.len());
        for (((role, native), reference), allowance) in
            roles.into_iter().zip(native).zip(foreign).zip(allowances)
        {
            let absolute_error = (native.value - reference.value).abs();
            samples.push(ScalarComparison {
                role,
                output: allowance.output,
                native: native.value,
                reference: reference.value,
                absolute_allowance: allowance.absolute,
                relative_allowance: allowance.relative,
                absolute_error,
                relative_error: (reference.value != 0.)
                    .then(|| absolute_error / reference.value.abs()),
            });
        }
        Self(samples)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("independent reference {phase}: {cause}; {diagnostics}{comparison}")]
struct ReferenceDiagnosticFailure {
    phase: &'static str,
    #[source]
    cause: Box<dyn std::error::Error + Send + Sync>,
    diagnostics: String,
    comparison: ComparisonDiagnostics,
    // Diagnostic/error payloads settle before both preparation and response owners.
    _owners: [Option<Arc<pse_columnar::AllocationLease>>; 3],
}

fn reference_failure(
    phase: &'static str,
    cause: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    diagnostics: String,
    owners: &[Arc<pse_columnar::AllocationLease>; 3],
) -> Box<dyn std::error::Error + Send + Sync> {
    reference_failure_with_comparison(
        phase,
        cause,
        diagnostics,
        ComparisonDiagnostics::default(),
        owners,
    )
}

fn reference_failure_with_comparison(
    phase: &'static str,
    cause: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    diagnostics: String,
    comparison: ComparisonDiagnostics,
    owners: &[Arc<pse_columnar::AllocationLease>; 3],
) -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(ReferenceDiagnosticFailure {
        phase,
        cause: cause.into(),
        diagnostics,
        comparison,
        _owners: [
            Some(owners[0].clone()),
            Some(owners[1].clone()),
            Some(owners[2].clone()),
        ],
    })
}

fn read_prefix(path: &Path, limit: usize) -> TestResult<(Vec<u8>, bool)> {
    use rustix::fs::{Mode, OFlags, open};
    let fd = open(
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )?;
    let mut file = std::fs::File::from(fd);
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err("reference exchange input must be a regular file".into());
    }
    let maximum = limit
        .checked_add(1)
        .ok_or("reference input extent overflow")?;
    let extent = usize::try_from(metadata.len().min(u64::try_from(maximum)?))?
        .checked_add(1)
        .ok_or("reference input extent overflow")?
        .min(maximum);
    // Fixed storage, sized from the bounded original file extent. A changing file
    // cannot force amortizing growth or reserve the entire aggregate source cap
    // for every small source document.
    let mut bytes = vec![0; extent].into_boxed_slice();
    let mut used = 0;
    while used < extent {
        match file.read(&mut bytes[used..]) {
            Ok(0) => break,
            Ok(count) => used += count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error.into()),
        }
    }
    if used == extent && extent < maximum && file.read(&mut [0])? != 0 {
        return Err("reference file grew during its bounded read".into());
    }
    let exceeded = used > limit;
    let mut bytes = bytes.into_vec();
    bytes.truncate(used.min(limit));
    Ok((bytes, exceeded))
}

fn read_bounded(path: &Path, limit: usize) -> TestResult<Vec<u8>> {
    let (bytes, exceeded) = read_prefix(path, limit)?;
    if exceeded {
        return Err(format!(
            "reference file exceeds its {limit}-byte extent: {}",
            path.display()
        )
        .into());
    }
    Ok(bytes)
}

fn verify_environment(
    root: &Path,
    python: &Path,
    interpreter_digest: &str,
    solver: &reference_environment::PreparedSolver,
    case: &str,
    diagnostics: &str,
) -> TestResult {
    const PREFIX: &str = "ordinary_reference_environment ";
    let mut lines = diagnostics
        .lines()
        .filter_map(|line| line.strip_prefix(PREFIX));
    let encoded = lines
        .next()
        .ok_or("performed reference environment absent")?;
    if lines.next().is_some() {
        return Err("multiple performed reference environment records".into());
    }
    let evidence: BTreeMap<String, String> = serde_json::from_str(encoded)?;
    reference_environment::verify(
        &evidence,
        python,
        interpreter_digest,
        &reference_environment::pins(root)?,
    )?;
    let stages: &[(&str, usize)] = match case {
        "exchanger_rating" => &[
            ("exchanger.initializer", 14),
            ("exchanger.design", 1),
            ("exchanger.rating", 1),
            ("exchanger.costing", 1),
        ],
        "exchanger_design" | "exchanger_arithmetic" => &[
            ("exchanger.initializer", 14),
            ("exchanger.design", 1),
            ("exchanger.costing", 1),
        ],
        "ideal_recycle" => &[("recycle.tear", 1), ("recycle.original", 1)],
        "overheated_recycle_failure" => &[("recycle.overheated_tear", 1)],
        _ => return Err("unknown ordinary reference stage population".into()),
    };
    for &(stage, count) in stages {
        reference_environment::verify_stage(&evidence, stage, solver, count..=count)?;
    }
    Ok(())
}

fn retained_destination() -> TestResult<Option<PathBuf>> {
    std::env::var_os("PSE_ORDINARY_REFERENCE_CONTRACTS")
        .map(|destination| {
            let destination = PathBuf::from(destination);
            if !destination.is_absolute() || !destination.is_dir() {
                return Err(
                    "contract evidence destination must already exist and be absolute".into(),
                );
            }
            Ok(destination)
        })
        .transpose()
}

/// Retain the existing factual owner, including native coordinates and original quality.
pub(super) async fn retain_native_context(
    name: &str,
    package: &ModelingPackage,
    services: &crate::workflow::PreparationServices,
    declared: &DeclaredExecution,
    outcome: &DeclaredAlgebraicOutcome,
    cancel: &crate::CancelSource,
) -> TestResult {
    let Some(destination) = retained_destination()? else {
        return Ok(());
    };
    let context = package.declared_review_context(
        services,
        declared,
        Some(outcome),
        &crate::workflow::ReviewContextRequest {
            // This selected full-source capture uses the owner's supported finite maximum.
            maximum_bytes: 256 << 20,
            ..Default::default()
        },
        None,
    )?;
    serde_json::to_writer(
        std::fs::File::create(destination.join(format!("{name}.native-context.json")))?,
        context.document(),
    )?;
    // The declared capture retains observations, not a specialized symbol table.
    // Obtain qualified member lineage from the existing diagnostic owner using
    // this exact declaration; do not infer a physical role from quantity or value.
    let prepared = package
        .prepare_declared_diagnostics(services, declared, cancel)
        .await?;
    let source_map = package
        .modeling_diagnostic_source_map(
            services,
            &prepared,
            16 << 20,
            Duration::from_secs(60),
            cancel,
        )
        .await?;
    serde_json::to_writer(
        std::fs::File::create(destination.join(format!("{name}.native-source-map.json")))?,
        source_map.document(),
    )?;
    Ok(())
}

fn source_path(root: &Path, relative: &str) -> TestResult<PathBuf> {
    let relative_path = Path::new(relative);
    if relative.is_empty()
        || relative.contains('\\')
        || relative_path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("scientific artifact path must be an exact relative source path".into());
    }
    if relative.contains("//")
        || relative.contains("/./")
        || relative.ends_with("/.")
        || relative.ends_with('/')
    {
        return Err("scientific artifact path must retain its exact source spelling".into());
    }
    let mut path = root.to_owned();
    for part in relative_path.components() {
        path.push(part);
        if std::fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err("scientific artifact paths cannot traverse symlinks".into());
        }
    }
    if path.canonicalize()? != root.join(relative) {
        return Err("scientific artifact escaped its immutable source root".into());
    }
    Ok(path)
}

/// Original bytes used by native admission and the independent reference contract.
#[derive(Debug)]
pub(super) struct OriginalInputs {
    pub(super) manifest: crate::workflow::ConformanceManifest,
    pub(super) files: BTreeMap<String, Vec<u8>>,
    pub(super) package_paths: pr_jacobian_tests::PackageDocumentPaths,
    artifacts: Vec<ReferenceInputArtifact>,
    root: PathBuf,
    // Original source bytes and their attestation metadata settle before the lease.
    pub(super) owner: Option<Arc<pse_columnar::AllocationLease>>,
}

impl OriginalInputs {
    fn verification_bound(&self) -> TestResult<usize> {
        let payload = self
            .files
            .values()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("source verification extent overflow")?;
        let path = self
            .files
            .keys()
            .map(String::len)
            .max()
            .unwrap_or(0)
            .checked_add(self.root.as_os_str().len())
            .ok_or("source verification extent overflow")?;
        // Original path, canonical result and comparison join coexist. At a
        // join reallocation: two retained paths plus old/new storage cost at
        // most six complete path lengths; spelling admission allocates no list.
        // Membership verification retains at most one package's selected paths.
        // The declared globs add a fixed directory to one Linux file name.
        let membership = self
            .package_paths
            .values()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .checked_mul(1024)
            .ok_or("source verification membership extent overflow")?;
        payload
            .checked_add(
                path.checked_mul(6)
                    .ok_or("source verification extent overflow")?,
            )
            .and_then(|bytes| bytes.checked_add(membership))
            .and_then(|bytes| bytes.checked_add(size_of::<sha2::Sha256>()))
            .ok_or_else(|| "source verification extent overflow".into())
    }

    pub(super) fn capture(root: &Path) -> TestResult<Self> {
        Self::capture_with_limits(root, SOURCE_LIMIT, SOURCE_FILE_LIMIT)
    }

    fn capture_with_limits(
        root: &Path,
        maximum_bytes: usize,
        maximum_files: usize,
    ) -> TestResult<Self> {
        if maximum_bytes > SOURCE_LIMIT || maximum_files == 0 || maximum_files > SOURCE_FILE_LIMIT {
            return Err("original input capture extent exceeds its fixture policy".into());
        }
        let root = root.canonicalize()?;
        let manifest_name = "packages/reference/conformance.toml";
        let manifest_path = source_path(&root, manifest_name)?;
        let manifest_bytes = read_bounded(&manifest_path, LIMIT.min(maximum_bytes))?;
        let manifest =
            crate::workflow::ConformanceManifest::from_bytes(&manifest_bytes, &manifest_path)?;
        let source = pr_jacobian_tests::ordinary_delivery_source(&manifest)?;
        let mut packages = source.dependencies.into_iter().collect::<BTreeSet<_>>();
        packages.extend([source.package, source.physical]);
        let registry = pse_schema::shared_registry()?;
        let mut paths = BTreeSet::new();
        let mut package_paths = BTreeMap::new();
        for package in packages {
            let prefix = format!("packages/reference/{package}");
            let directory = source_path(&root, &prefix)?;
            let selected =
                package_document_paths(&directory, &registry, maximum_files - 1 - paths.len())?;
            paths.extend(
                selected
                    .iter()
                    .map(|relative| format!("{prefix}/{relative}")),
            );
            package_paths.insert(package, selected);
        }
        paths.extend(
            [
                "packages/reference/data/ordinary-bt/source-artifacts/yaws-critical.tsv",
                "packages/reference/data/ordinary-bt/source-artifacts/crc-vaporization.tsv",
                "packages/reference/data/ordinary-bt/source-artifacts/poling-wagner.tsv",
                "packages/reference/data/ordinary-bt/source-artifacts/feos-poling2000.json",
                "packages/reference/data/perry7/data/perry_liquid.parquet",
            ]
            .map(str::to_owned),
        );
        if paths.len() >= maximum_files {
            return Err(format!(
                "original scientific artifact population exceeds {maximum_files} files"
            )
            .into());
        }
        let mut remaining = maximum_bytes
            .checked_sub(manifest_bytes.len())
            .ok_or("original source extent")?;
        let mut files = BTreeMap::from([(manifest_name.to_owned(), manifest_bytes)]);
        for relative in paths {
            let path = source_path(&root, &relative)?;
            let bytes = read_bounded(&path, remaining)?;
            remaining = remaining
                .checked_sub(bytes.len())
                .ok_or("original source extent")?;
            files.insert(relative, bytes);
        }
        let artifacts = files
            .iter()
            .map(|(path, bytes)| ReferenceInputArtifact {
                path: path.clone(),
                sha256: sha2::Sha256::digest(bytes).to_vec().into(),
            })
            .collect();
        Ok(Self {
            manifest,
            files,
            package_paths,
            artifacts,
            root,
            owner: None,
        })
    }

    pub(super) fn verify_checkout(&self) -> TestResult {
        let registry = pse_schema::shared_registry()?;
        for (package, expected) in &self.package_paths {
            let directory = source_path(&self.root, &format!("packages/reference/{package}"))?;
            let changed = || format!("checkout document membership differs for {package}");
            let current = package_document_paths(&directory, &registry, expected.len())
                .map_err(|error| format!("{}: {error}", changed()))?;
            if &current != expected {
                return Err(changed().into());
            }
        }
        for artifact in &self.artifacts {
            let path = source_path(&self.root, &artifact.path)?;
            let original = self
                .files
                .get(&artifact.path)
                .ok_or("original artifact bytes absent")?;
            let actual = read_bounded(&path, original.len())?;
            let actual_hash: [u8; 32] = sha2::Sha256::digest(&actual).into();
            if actual_hash.as_slice() != artifact.sha256.as_slice() {
                return Err(format!(
                    "checkout differs from the original admitted source snapshot: {}",
                    artifact.path
                )
                .into());
            }
        }
        Ok(())
    }
}

fn physical(
    package: &ModelingPackage,
    quantity: SemanticId,
    unit: Option<SemanticId>,
    owners: &mut Vec<Arc<pse_columnar::MemoryReservation>>,
) -> Result<crate::workflow::PhysicalQuantityDescription, Box<dyn std::error::Error + Send + Sync>>
{
    let description =
        package
            .physical_context()
            .describe(&crate::workflow::PhysicalInspectionRequest {
                query: crate::workflow::PhysicalInspectionQuery::Quantity {
                    id: quantity.into(),
                    unit: unit.map(Into::into),
                },
                offset: 0,
                limit: 1,
            })?;
    let crate::workflow::PhysicalInspectionDocument::Page {
        mut quantities,
        owner,
        ..
    } = description
    else {
        return Err("complete original physical correspondence unavailable".into());
    };
    if quantities.len() != 1 {
        return Err("physical coordinate is ambiguous".into());
    }
    owners.push(owner.ok_or("original physical description lost its allocation owner")?);
    Ok(quantities.remove(0))
}

fn fixed_keys(name: &str, recycle: bool) -> Vec<String> {
    if recycle {
        return [
            "feed.state.flow",
            "feed.state.T",
            "feed.state.pressure",
            "feed.state.z[benzene]",
            "feed.state.z[toluene]",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
    }
    let mut keys = Vec::new();
    for side in ["hot", "cold"] {
        for key in ["flow", "T", "pressure", "z[benzene]", "z[toluene]"] {
            keys.push(format!("exchanger.{side}.inlet.{key}"));
        }
    }
    keys.extend([
        "exchanger.hot.delta_pressure".into(),
        "exchanger.cold.delta_pressure".into(),
        "exchanger.coefficient".into(),
    ]);
    keys.push(
        if name == "ordinary_rating" {
            "exchanger.area"
        } else {
            "exchanger.cold.outlet.T"
        }
        .into(),
    );
    keys
}

fn source_hash_scratch(
    artifacts: &[ReferenceInputArtifact],
    keys: &[String],
    case_name: &str,
) -> TestResult<usize> {
    let add = |a: usize, b: usize| a.checked_add(b).ok_or("source hash extent overflow");
    let mul = |a: usize, b: usize| a.checked_mul(b).ok_or("source hash extent overflow");
    // JSON is an internal hash preimage here. Escaping costs at most six bytes
    // per input byte; u8 array entries cost at most three digits plus a comma.
    let artifact_json =
        artifacts
            .iter()
            .try_fold(2usize, |bytes, artifact| -> TestResult<usize> {
                Ok(add(
                    bytes,
                    add(
                        add(
                            mul(6, artifact.path.len())?,
                            mul(4, artifact.sha256.as_slice().len())?,
                        )?,
                        8,
                    )?,
                )?)
            })?;
    let formulation_json = add(add(mul(4, artifact_json)?, mul(6, case_name.len())?)?, 9)?;
    // A u64 uses at most usize::BITS decimal digits (also bounds the fixed
    // f64-bit u64 on this supported64-bit target); identity strings use at most
    // their canonical hexadecimal width plus separators and JSON punctuation.
    let specification_json = keys
        .iter()
        .try_fold(2usize, |bytes, key| -> TestResult<usize> {
            Ok(add(
                bytes,
                add(
                    mul(6, key.len())?,
                    2 * SemanticId::WIDTH + u64::BITS as usize + 9,
                )?,
            )?)
        })?;
    let tuples = add(
        mul(artifacts.len(), size_of::<(&String, &[u8])>())?,
        mul(keys.len(), size_of::<(&String, u64, SemanticId)>())?,
    )?;
    Ok(add(
        mul(
            3,
            add(add(artifact_json, formulation_json)?, specification_json)?,
        )?,
        tuples,
    )?)
}

fn output_context(
    physical: &PhysicalContext,
    numerics: &pse_model::numerics::ResolvedNumericalPolicy,
    target: &pse_math::numerics::TargetSpec,
) -> TestResult<pse_model::numerics::EngineeringContext> {
    let original = pse_math::numerics::TargetAccess::new(numerics)?.find(target.kind, target.id);
    if original.is_some_and(|original| {
        original.quantity != target.quantity.as_id() || original.unit != target.unit.as_id()
    }) {
        return Err("reference output differs from its original numerical quantity or unit".into());
    }
    let context = match original.and_then(|original| original.engineering.as_ref()) {
        Some(context) => context.clone(),
        None => pse_math::numerics::operational_output_context(
            physical.quantities(),
            physical.preconditions(),
            target,
            &numerics.policy,
            &numerics.targets,
        )?,
    };
    if context.canonical_fallback || !context.budget.is_finite() || context.budget <= 0. {
        return Err(format!(
            "reference output {} lacks a finite positive original engineering context",
            target.id
        )
        .into());
    }
    Ok(context)
}

fn coordinates(recycle: bool) -> Vec<(String, String, bool)> {
    if recycle {
        return [
            ("fresh.flow", "ordinary feed flow"),
            ("vapor.flow", "vapor product"),
            ("purge.flow", "ordinary purge flow"),
            ("recycle.flow", "liquid recycle"),
            ("fresh.energy", "ordinary feed energy"),
            ("heater.duty", "ordinary heater duty"),
            ("vapor.energy", "ordinary vapor energy"),
            ("purge.energy", "ordinary purge energy"),
            ("fresh.benzene", "ordinary feed benzene"),
            ("vapor.benzene", "ordinary vapor benzene"),
            ("purge.benzene", "ordinary purge benzene"),
            ("fresh.toluene", "ordinary feed toluene"),
            ("vapor.toluene", "ordinary vapor toluene"),
            ("purge.toluene", "ordinary purge toluene"),
        ]
        .into_iter()
        .map(|(key, label)| (key.into(), label.into(), false))
        .collect();
    }
    let mut roles = Vec::new();
    for side in ["hot", "cold"] {
        for position in ["inlet", "outlet"] {
            for coordinate in ["flow", "T", "h", "energy_flow"] {
                let path = format!("exchanger.{side}.{position}.{coordinate}");
                roles.push((path.clone(), path, true));
            }
        }
    }
    roles.extend(
        [
            ("exchanger.hot.duty", "ordinary hot duty"),
            ("exchanger.cold.duty", "ordinary cold duty"),
            (
                "exchanger.coefficient",
                "ordinary heat transfer coefficient",
            ),
            ("exchanger.area", "solved heat-transfer area"),
            ("cost.area_count_ft2", "area in square feet"),
            ("cost.base", "base purchase cost"),
            ("cost.material_factor", "ordinary material factor"),
            ("cost.pressure_factor", "ordinary pressure factor"),
            ("cost.capital", "capital cost at CEPCI 500"),
        ]
        .into_iter()
        .map(|(key, label)| (key.into(), label.into(), false)),
    );
    roles
}

pub(super) fn check(
    name: &str,
    package: &ModelingPackage,
    analysis: &ModelingAnalysis,
    result: &ModelingResult,
    failed: bool,
    services: &crate::workflow::PreparationServices,
    inputs: &OriginalInputs,
) -> TestResult {
    // Declared before every response/contract payload so final guards outlive them
    // even though each phase can only be admitted after its original inputs exist.
    let retained_owners: [Arc<pse_columnar::AllocationLease>; 3];
    let verification_owner = services.math().reserve(
        "ordinary-reference:source-verification",
        inputs.verification_bound()?,
    )?;
    if inputs.owner.is_none() {
        return Err("original reference inputs lost their allocation owner".into());
    }
    if let Err(cause) = inputs.verify_checkout() {
        return Err(Box::new(ReferenceDiagnosticFailure {
            phase: "original source snapshot refused",
            cause,
            diagnostics: String::new(),
            comparison: ComparisonDiagnostics::default(),
            _owners: [Some(verification_owner.clone()), None, None],
        }));
    }
    let root = &inputs.root;
    let recycle = matches!(name, "ordinary_recycle" | "ordinary_overheated");
    let case_name = match name {
        "ordinary_recycle" => "ordinary_delivery.ideal_recycle",
        "ordinary_overheated" => "ordinary_delivery.overheated_recycle_failure",
        "ordinary_exchanger" => "ordinary_delivery.exchanger_design",
        "ordinary_rating" => "ordinary_delivery.exchanger_rating",
        "ordinary_arithmetic_closure" => "ordinary_delivery.exchanger_arithmetic",
        _ => return Err("no independent constructor for selected original case".into()),
    };
    let roles = coordinates(recycle);
    let keys = fixed_keys(name, recycle);
    let mapping_storage = roles
        .iter()
        .try_fold(
            roles
                .capacity()
                .checked_mul(size_of::<(String, String, bool)>())
                .ok_or("mapping extent overflow")?,
            |bytes, (key, matching, _)| {
                bytes
                    .checked_add(key.capacity())?
                    .checked_add(matching.capacity())
            },
        )
        .ok_or("mapping extent overflow")?;
    let mapping_storage = keys
        .iter()
        .try_fold(
            mapping_storage
                .checked_add(
                    keys.capacity()
                        .checked_mul(size_of::<String>())
                        .ok_or("mapping extent overflow")?,
                )
                .ok_or("mapping extent overflow")?,
            |bytes, key| bytes.checked_add(key.capacity()),
        )
        .ok_or("mapping extent overflow")?;
    let container_bytes = crate::reference_exchange::case_contract_container_bound(
        roles.len(),
        keys.len(),
        &inputs.artifacts,
        mapping_storage
            .checked_add(case_name.len())
            .ok_or("mapping extent overflow")?,
    )?
    .checked_add(source_hash_scratch(&inputs.artifacts, &keys, case_name)?)
    .ok_or("contract preparation extent overflow")?;
    let preparation_owner = services
        .math()
        .reserve("ordinary-reference:contract-containers", container_bytes)?;
    // Rebind the finite source-role intake under its admitted container owner.
    let (roles, mut keys) = (roles, keys);
    let mut physical_owners = Vec::new();
    let mut observables = Vec::new();
    let mut selection = Vec::new();
    let mut allowances = Vec::new();
    let product = result.prepared.model.model.compiled();
    let model = &product.model;
    let targets =
        product.numerical_annotations(pse_compiler::workspace::NumericalAnnotationMode::Steady)?;
    let numerics = result.prepared.solve.numerics();
    for (key, matching, path) in roles {
        // A failed attempt still has the entire original authored output population.
        // Metadata comes from its admitted declarations, never from a rejected
        // candidate's availability or an opportunistically shortened report list.
        let mut found = model.annotations.iter().filter(|annotation| {
            let pse_modeling::annotation::AnnotationValue::Report(label) = &annotation.value else {
                return false;
            };
            if path {
                model
                    .symbols
                    .get(&annotation.target)
                    .is_some_and(|symbol| symbol.lineage.path.ends_with(&matching))
            } else {
                label == &matching
            }
        });
        let annotation = found
            .next()
            .ok_or_else(|| format!("original reference coordinate absent: {key}"))?;
        if found.next().is_some() {
            return Err(format!("ambiguous original reference coordinate: {key}").into());
        }
        let symbol = model
            .symbols
            .get(&annotation.target)
            .ok_or("reported original scalar absent")?;
        let quantity = symbol
            .ty
            .quantity_scheme()
            .ok_or("reported original physical scheme absent")?
            .resolve(package.physical_context().quantities(), &Default::default())?;
        let unit = package
            .physical_context()
            .quantities()
            .quantity_type(quantity)?
            .canonical_unit;
        let target = targets
            .target(annotation.target)
            .ok_or_else(|| format!("original output numerical owner absent: {key}"))?;
        if target.quantity != quantity || target.unit != unit {
            return Err(format!(
                "original output physical correspondence differs from its numerical owner: {key}"
            )
            .into());
        }
        // This is a frozen physical output-resolution criterion, independently
        // tested against the foreign value. Native termination does not establish
        // this error bound. The original rules, tagged scales and caloric consuming
        // requirements supply its magnitude without reading either result value.
        let context = output_context(
            package.physical_context(),
            numerics,
            &target.specification(),
        )
        .map_err(|cause| format!("original reference output context absent for {key}: {cause}"))?;
        allowances.push(ComparisonAllowance {
            output: annotation.target,
            absolute: context.budget,
            relative: 0.,
        });
        selection.push(ReferenceReportSelection {
            output: annotation.target,
            source: annotation.lineage.declaration,
            target: annotation.target,
        });
        observables.push(ReferenceObservableContract {
            output: annotation.target,
            key,
            physical: physical(
                package,
                quantity.as_id(),
                Some(unit.as_id()),
                &mut physical_owners,
            )?,
        });
    }
    let fixture = model
        .fixtures
        .get(&analysis.instance)
        .ok_or("original fixture specification absent")?;
    let mut specification = Vec::new();
    for key in keys.drain(..) {
        let authored = format!(
            "root.{}",
            key.replace("[benzene]", "[chem.benzene]")
                .replace("[toluene]", "[chem.toluene]")
        );
        let value = fixture.specifications.get(&authored).ok_or_else(|| {
            format!(
                "original fixed input absent: {authored}; actual={:?}",
                fixture.specifications.keys()
            )
        })?;
        if value.fixed != Some(true) {
            return Err(format!("reference input is not fixed: {key}").into());
        }
        let magnitude = value
            .value
            .ok_or("original fixed input has no selected magnitude")?;
        let symbol = model
            .symbols
            .get(&value.target)
            .ok_or("fixed input symbol absent")?;
        let scheme = symbol
            .ty
            .quantity_scheme()
            .ok_or("fixed input physical scheme absent")?;
        let quantity =
            scheme.resolve(package.physical_context().quantities(), &Default::default())?;
        specification.push(ReferenceFixedInput {
            key,
            value: magnitude,
            physical: physical(package, quantity.as_id(), None, &mut physical_owners)?,
        });
    }
    let input_artifacts = inputs.artifacts.clone();
    let artifact_bytes = serde_json::to_vec(
        &input_artifacts
            .iter()
            .map(|input| (&input.path, input.sha256.as_slice()))
            .collect::<Vec<_>>(),
    )?;
    let source = pse_ids::preimage::hash(&artifact_bytes);
    let formulation =
        pse_ids::preimage::hash(&serde_json::to_vec(&(1_u32, case_name, &artifact_bytes))?);
    let configuration = pse_ids::preimage::hash(&serde_json::to_vec(
        &specification
            .iter()
            .map(|input| {
                (
                    &input.key,
                    input.value.to_bits(),
                    input.physical.quantity.id.as_id(),
                )
            })
            .collect::<Vec<_>>(),
    )?);
    drop(artifact_bytes);
    let meaning = ReferenceMeaning {
        case: analysis.root.as_id(),
        source,
        formulation,
        physical: package.physical_context().identity(),
        configuration,
    };
    let required = observables
        .iter()
        .map(|coordinate| coordinate.output)
        .collect::<Vec<_>>();
    // Only the original source metadata selects admission. These unused numeric
    // placeholders are neither candidate values nor independent expected answers.
    let original_shape = ReferenceEnvelope {
        version: 1,
        meaning: meaning.clone(),
        required: required.clone(),
        outcome: if failed {
            ReferenceOutcome::Failed {
                code: "trial_rejected".into(),
                stage: "overheated_tear".into(),
            }
        } else {
            ReferenceOutcome::Completed {
                outputs: observables
                    .iter()
                    .map(
                        |observable| pse_model::reference_exchange::ReferenceObservable {
                            id: observable.output,
                            quantity: observable.physical.quantity.id.as_id(),
                            unit: observable.physical.representation_unit.id.as_id(),
                            basis: observable.physical.quantity.key.basis.map(|id| id.as_id()),
                            datum: observable
                                .physical
                                .quantity
                                .key
                                .reference_state
                                .map(|id| id.as_id()),
                            value: 0.,
                        },
                    )
                    .collect(),
                blocks: Vec::new(),
            }
        },
    };
    let admission = crate::reference_exchange::ScalarAdmission::new(&original_shape, LIMIT)?;
    let diagnostic_backing = observables
        .len()
        .checked_mul(size_of::<ScalarComparison>() + size_of::<String>())
        .ok_or("comparison diagnostic extent overflow")?;
    let diagnostic_bytes = observables
        .iter()
        .try_fold(diagnostic_backing, |bytes, observable| {
            bytes.checked_add(observable.key.capacity())
        })
        .ok_or("comparison diagnostic extent overflow")?;
    let response_bytes = admission
        .custody_bound(reference_process::DIAGNOSTIC_BYTES + 256)?
        .checked_add(
            admission
                .native_envelope_bound()?
                .checked_mul(4)
                .ok_or("native comparison custody overflow")?,
        )
        .and_then(|bytes| bytes.checked_add(size_of::<ReferenceDiagnosticFailure>()))
        .and_then(|bytes| bytes.checked_add(diagnostic_bytes))
        .ok_or("complete scalar custody overflow")?;
    let response_owner = services
        .math()
        .reserve("ordinary-reference:scalar-response-custody", response_bytes)?;
    let diagnostic_roles = observables
        .iter()
        .map(|observable| observable.key.clone())
        .collect();
    let contract = ReferenceCaseContract {
        version: Default::default(),
        case_name: case_name.into(),
        meaning: meaning.clone(),
        observables,
        specification,
        artifacts: input_artifacts,
    };
    let encoding_owner = services.math().reserve(
        "ordinary-reference:contract-encoding",
        crate::reference_exchange::case_contract_encoding_bound(&contract, LIMIT)?,
    )?;
    retained_owners = [preparation_owner, encoding_owner, response_owner];
    let owners = &retained_owners;
    let bytes = match crate::reference_exchange::encode_case_contract(contract, LIMIT) {
        Ok(bytes) => bytes,
        Err(cause) => {
            return Err(reference_failure(
                "request encoding refused",
                cause,
                String::new(),
                owners,
            ));
        }
    };
    drop(physical_owners);
    let scratch = tempfile::tempdir()?;
    let request = scratch.path().join("case.bin");
    let response = scratch.path().join("reference.bin");
    std::fs::write(&request, &bytes)?;
    // An explicitly selected retained destination permits the independent producer's
    // own full contract mutation controls to reuse these actual native descriptions.
    let short = case_name
        .strip_prefix("ordinary_delivery.")
        .ok_or("ordinary case namespace absent")?;
    let evidence = retained_destination()?;
    if let Some(destination) = &evidence {
        std::fs::write(destination.join(format!("{short}.contract.bin")), &bytes)?;
    }
    // Source/native and generated wire validation already occur in encode. The
    // generic codec's roundtrip/unknown preservation is tested by its own controls;
    // a redundant local decode would keep a second metadata forest through solve.
    drop(bytes);
    let python = PathBuf::from(
        std::env::var_os("PSE_ORDINARY_REFERENCE_PYTHON")
            .ok_or("explicit locked ordinary reference interpreter absent")?,
    );
    if !python.is_absolute() || !python.is_file() {
        return Err("reference interpreter must be an existing absolute file".into());
    }
    let interpreter_digest = reference_environment::image_digest(&python)?;
    let solver = reference_environment::prepared("IPOPT_DIR", "ipopt")?;
    let mut command = Command::new(&python);
    command
        .arg("-I")
        .arg(root.join("tests/reference/ordinary_reference_driver.py"))
        .arg("--descriptors")
        .arg(root.join("crates/pse-runtime/src/generated/reference-exchange/descriptor.bin"))
        .arg("--contract")
        .arg(&request)
        .arg("--output")
        .arg(&response)
        .current_dir(scratch.path())
        .env_remove("PYTHONPATH")
        .env_remove("PYTHONHOME");
    let execution =
        reference_process::execute(&mut command, scratch, reference_process::DIAGNOSTIC_BYTES);
    let diagnostic = execution.diagnostics;
    if let Some(destination) = &evidence {
        std::fs::write(
            destination.join(format!("{short}.reference.log")),
            &diagnostic,
        )?;
    }
    if let Some((phase, cause)) = execution.refusal {
        return Err(reference_failure(phase, cause, diagnostic, owners));
    }
    if let Err(cause) = verify_environment(
        root,
        &python,
        &interpreter_digest,
        &solver,
        short,
        &diagnostic,
    ) {
        return Err(reference_failure(
            "environment refused",
            cause,
            diagnostic,
            owners,
        ));
    }
    if let Err(cause) = inputs.verify_checkout() {
        return Err(reference_failure(
            "source snapshot changed",
            cause,
            diagnostic,
            owners,
        ));
    }
    let reference = match (|| -> TestResult<_> {
        let payload = reference_process::read_response(&response, LIMIT)?;
        if let Some(destination) = &evidence {
            std::fs::write(destination.join(format!("{short}.reference.bin")), &payload)?;
        }
        Ok(admission.decode(&payload)?)
    })() {
        Ok(reference) => reference,
        Err(cause) => {
            return Err(reference_failure(
                "response refused",
                cause,
                diagnostic,
                owners,
            ));
        }
    };
    let lease = if failed {
        None
    } else {
        Some(result.reference_observations(&selection)?)
    };
    let candidate = ReferenceEnvelope {
        version: 1,
        meaning,
        required: required.clone(),
        outcome: if failed {
            ReferenceOutcome::Failed {
                code: "trial_rejected".into(),
                stage: "overheated_tear".into(),
            }
        } else {
            ReferenceOutcome::Completed {
                outputs: lease
                    .as_ref()
                    .ok_or("native scalar allocation absent")?
                    .iter()
                    .cloned()
                    .collect(),
                blocks: Vec::new(),
            }
        },
    };
    if let Err(cause) = compare(&candidate, reference.known(), &allowances) {
        return Err(reference_failure_with_comparison(
            "scalar comparison failed",
            cause,
            diagnostic,
            ComparisonDiagnostics::capture(
                diagnostic_roles,
                &candidate,
                reference.known(),
                &allowances,
            ),
            owners,
        ));
    }
    if let ReferenceOutcome::Completed { outputs, .. } = &candidate.outcome {
        let mut wrong = candidate.clone();
        let ReferenceOutcome::Completed {
            outputs: values, ..
        } = &mut wrong.outcome
        else {
            return Err("completed population absent".into());
        };
        values[0].value += 1000.;
        assert!(compare(&wrong, reference.known(), &allowances).is_err());
        let mut missing = candidate.clone();
        if let ReferenceOutcome::Completed { outputs, .. } = &mut missing.outcome {
            outputs.pop();
        }
        assert!(compare(&missing, reference.known(), &allowances).is_err());
        assert_eq!(outputs.len(), required.len());
    }
    let mut wrong = candidate.clone();
    wrong.meaning.formulation = pse_ids::ContentHash::NIL;
    assert!(compare(&wrong, reference.known(), &allowances).is_err());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_model::{
        generated::enums::{EngineeringScaleKind, NumericalSource, NumericalTarget},
        numerics::{EngineeringScale, NumericalPolicy, ResolvedNumericalPolicy},
    };
    use pse_quantity::standard::ids;

    const ORIGINAL_MODEL: &[u8] = b"package q { def D { var x: Scalar; } }";

    #[test]
    fn comparison_diagnostics_preserve_original_pairs_without_deciding_acceptance() {
        use pse_model::reference_exchange::ReferenceObservable;
        // Mechanical diagnostic coordinates, not authored scientific expectations.
        let ids = [1, 2].map(|byte| SemanticId::from_bytes([byte; 16]));
        let envelope = |values: [f64; 2]| ReferenceEnvelope {
            version: 1,
            meaning: ReferenceMeaning {
                case: SemanticId::NIL,
                source: pse_ids::ContentHash::NIL,
                formulation: pse_ids::ContentHash::NIL,
                physical: pse_ids::ContentHash::NIL,
                configuration: pse_ids::ContentHash::NIL,
            },
            required: ids.to_vec(),
            outcome: ReferenceOutcome::Completed {
                outputs: ids
                    .into_iter()
                    .zip(values)
                    .map(|(id, value)| ReferenceObservable {
                        id,
                        quantity: SemanticId::NIL,
                        unit: SemanticId::NIL,
                        basis: None,
                        datum: None,
                        value,
                    })
                    .collect(),
                blocks: Vec::new(),
            },
        };
        let candidate = envelope([3., 0.]);
        let reference = envelope([2., 0.]);
        let allowances = ids.map(|output| ComparisonAllowance {
            output,
            absolute: 0.5,
            relative: 0.1,
        });
        let original = compare(&candidate, &reference, &allowances).unwrap_err();
        let detail = ComparisonDiagnostics::capture(
            vec!["different".into(), "same".into()],
            &candidate,
            &reference,
            &allowances,
        );
        assert_eq!(
            detail.0.len(),
            2,
            "diagnostics retain all pairs, not a second verdict"
        );
        let first = &detail.0[0];
        assert_eq!(
            (first.output, first.native, first.reference),
            (ids[0], 3., 2.)
        );
        assert_eq!(
            (first.absolute_allowance, first.relative_allowance),
            (0.5, 0.1)
        );
        assert_eq!(
            (first.absolute_error, first.relative_error),
            (1., Some(0.5))
        );
        assert_eq!(
            detail.0[1].relative_error, None,
            "zero-reference relative error is undefined"
        );
        assert!(detail.to_string().contains("native=3.0 reference=2.0"));
        assert_eq!(
            compare(&candidate, &reference, &allowances).unwrap_err(),
            original
        );
    }

    fn original_tree() -> TestResult<tempfile::TempDir> {
        let directory = tempfile::tempdir()?;
        let root = directory.path().join("packages/reference");
        for name in [
            "campaign",
            "physical",
            "ordinary-delivery",
            "data/ordinary-bt",
        ] {
            let package = root.join(name);
            std::fs::create_dir_all(&package)?;
            std::fs::write(
                package.join("package.toml"),
                include_bytes!("../../../../../tests/fixtures/packages/minimal_named/package.toml"),
            )?;
        }
        std::fs::create_dir_all(root.join("ordinary-delivery/models"))?;
        std::fs::write(
            root.join("ordinary-delivery/models/root.pse"),
            ORIGINAL_MODEL,
        )?;
        std::fs::write(
            root.join("conformance.toml"),
            "version = 1\n[settings]\nversion = 1\nmemory_limit_bytes = 1048576\n[[runs]]\nversion = 1\nname = 'seed'\npackage = 'campaign'\nphysical = 'physical'\n",
        )?;
        // Mechanical capture controls use opaque input bytes; no property or solver
        // execution is asserted by this small document-admission fixture.
        for relative in [
            "data/ordinary-bt/source-artifacts/yaws-critical.tsv",
            "data/ordinary-bt/source-artifacts/crc-vaporization.tsv",
            "data/ordinary-bt/source-artifacts/poling-wagner.tsv",
            "data/ordinary-bt/source-artifacts/feos-poling2000.json",
            "data/perry7/data/perry_liquid.parquet",
        ] {
            let path = root.join(relative);
            std::fs::create_dir_all(path.parent().ok_or("artifact parent absent")?)?;
            std::fs::write(path, b"original scientific input")?;
        }
        Ok(directory)
    }

    #[tokio::test]
    async fn original_inputs_admit_captured_native_documents_and_refuse_checkout_drift()
    -> TestResult {
        let directory = original_tree()?;
        let inputs = OriginalInputs::capture(directory.path())?;
        inputs.verify_checkout()?;
        for name in [
            "campaign",
            "physical",
            "ordinary-delivery",
            "data/ordinary-bt",
        ] {
            let captured = pr_jacobian_tests::captured_package_sources(
                &inputs.files,
                &inputs.package_paths,
                name,
            )
            .map(|(path, bytes)| (path.to_owned(), bytes.to_vec()))
            .collect::<BTreeMap<_, _>>();
            assert_eq!(
                captured,
                pr_jacobian_tests::documents(
                    &directory.path().join("packages/reference").join(name)
                )?,
                "captured native admission must preserve the original authored document population"
            );
        }
        for relative in [
            "packages/reference/data/ordinary-bt/source-artifacts/yaws-critical.tsv",
            "packages/reference/data/ordinary-bt/source-artifacts/crc-vaporization.tsv",
            "packages/reference/data/ordinary-bt/source-artifacts/poling-wagner.tsv",
            "packages/reference/data/ordinary-bt/source-artifacts/feos-poling2000.json",
        ] {
            assert!(inputs.files.contains_key(relative));
            assert!(inputs.artifacts.iter().any(|input| input.path == relative));
        }
        assert!(
            pr_jacobian_tests::captured_package_sources(
                &inputs.files,
                &inputs.package_paths,
                "data/ordinary-bt",
            )
            .all(|(path, _)| !path.starts_with("source-artifacts/")),
            "raw scientific inputs retain attestation custody without becoming authored documents"
        );
        let relative = "packages/reference/ordinary-delivery/models/root.pse";
        let added = directory
            .path()
            .join("packages/reference/ordinary-delivery/models/added.pse");
        std::fs::write(&added, b"package added {}\n")?;
        assert!(
            inputs
                .verify_checkout()
                .unwrap_err()
                .to_string()
                .contains("membership differs")
        );
        std::fs::remove_file(added)?;
        let original_hash = inputs
            .artifacts
            .iter()
            .find(|input| input.path == relative)
            .ok_or("original model attestation absent")?
            .sha256
            .clone();
        // Neither a later live model nor a changed campaign header can alter the
        // document bytes selected for the already captured native admission.
        std::fs::write(directory.path().join(relative), [0xff])?;
        let manifest = directory.path().join("packages/reference/conformance.toml");
        std::fs::write(&manifest, "version = 2\n")?;
        assert!(crate::workflow::ConformanceManifest::load(&manifest).is_err());
        assert_eq!(inputs.manifest.runs[0].name, "seed");
        assert!(inputs.verify_checkout().is_err());

        let runtime = crate::workflow::tests::runtime();
        let pool = runtime.shared.pool();
        let token = pse_columnar::CancellationToken::new();
        let validation = runtime.validation_context();
        let admitted = pse_admission::authoring_driver::document::load_package_sources_owned(
            pr_jacobian_tests::captured_package_sources(
                &inputs.files,
                &inputs.package_paths,
                "ordinary-delivery",
            ),
            runtime.registry(),
            Default::default(),
            &pool,
            &token,
            &validation,
        )?;
        let document = admitted
            .bundle()
            .documents
            .iter()
            .find(|document| document.path == "models/root.pse")
            .ok_or("captured native document absent")?;
        assert_eq!(document.bytes(), ORIGINAL_MODEL);
        assert!(
            document
                .modeling_rows()
                .is_some_and(|rows| !rows.is_empty())
        );
        assert_eq!(
            inputs
                .artifacts
                .iter()
                .find(|input| input.path == relative)
                .ok_or("original model attestation lost")?
                .sha256,
            original_hash
        );
        let changed = pr_jacobian_tests::documents(
            &directory
                .path()
                .join("packages/reference/ordinary-delivery"),
        )?;
        assert!(
            pse_admission::authoring_driver::document::load_package_documents_owned(
                &changed,
                runtime.registry(),
                Default::default(),
                &pool,
                &token,
                &validation,
            )
            .is_err(),
            "the live corrupted source must refuse through the same native loader"
        );
        Ok(())
    }

    #[test]
    fn reference_file_reads_refuse_extent_before_decoding() -> TestResult {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("response.bin");
        std::fs::write(&path, [7u8; 32])?;
        let original = read_bounded(&path, 32)?;
        assert_eq!(original, vec![7u8; 32]);
        assert!(
            original.capacity() <= 33,
            "fixed storage includes only the extent probe"
        );
        let small = directory.path().join("small.bin");
        std::fs::write(&small, [1])?;
        assert!(
            read_bounded(&small, SOURCE_LIMIT)?.capacity() <= 2,
            "a small file cannot retain the aggregate source budget as spare capacity"
        );
        assert!(read_bounded(&path, 31).is_err());
        std::fs::OpenOptions::new()
            .write(true)
            .open(&path)?
            .set_len((LIMIT as u64) + 1)?;
        let (prefix, exceeded) = read_prefix(&path, 64)?;
        assert_eq!(prefix.len(), 64);
        assert!(exceeded);
        assert!(
            read_bounded(&path, 64)
                .unwrap_err()
                .to_string()
                .contains("64-byte extent")
        );
        assert!(read_bounded(directory.path(), 64).is_err());
        Ok(())
    }

    #[test]
    fn original_inputs_enforce_aggregate_bytes_and_file_population() -> TestResult {
        let directory = original_tree()?;
        let inputs = OriginalInputs::capture(directory.path())?;
        let bytes = inputs.files.values().map(Vec::len).sum::<usize>();
        let files = inputs.files.len();
        let exact = OriginalInputs::capture_with_limits(directory.path(), bytes, files)?;
        assert_eq!(exact.files, inputs.files);
        assert!(
            OriginalInputs::capture_with_limits(directory.path(), bytes - 1, files)
                .unwrap_err()
                .to_string()
                .contains("extent")
        );
        assert!(
            OriginalInputs::capture_with_limits(directory.path(), bytes, files - 1)
                .unwrap_err()
                .to_string()
                .contains("population exceeds")
        );
        assert!(OriginalInputs::capture_with_limits(directory.path(), 1, files).is_err());
        let oversized = directory
            .path()
            .join("packages/reference/ordinary-delivery/models/root.pse");
        std::fs::OpenOptions::new()
            .write(true)
            .open(oversized)?
            .set_len(SOURCE_LIMIT as u64 + 1)?;
        assert!(
            OriginalInputs::capture_with_limits(directory.path(), bytes, files - 1)
                .unwrap_err()
                .to_string()
                .contains("population exceeds"),
            "the selected file count refuses before reading package bodies"
        );
        Ok(())
    }

    #[test]
    fn reference_diagnostics_preserve_bounded_failure_evidence() -> TestResult {
        let mut child = Command::new("/bin/sh");
        child.args(["-c", "printf 'actual stdout '; printf 'actual stderr ' >&2; head -c 256 /dev/zero; head -c 256 /dev/zero >&2"]);
        let output = reference_process::execute(&mut child, tempfile::tempdir()?, 64);
        let (phase, cause) = output.refusal.ok_or("missing bounded capture refusal")?;
        assert_eq!(phase, "diagnostic capture refused");
        assert!(cause.contains("exit=exit status: 0") && cause.contains("settlement=complete"));
        assert!(
            output
                .diagnostics
                .contains("[stdout overflow=true complete=true]")
        );
        assert!(
            output
                .diagnostics
                .contains("[stderr overflow=true complete=true]")
        );
        assert!(output.diagnostics.contains("actual stdout"));
        assert!(output.diagnostics.contains("actual stderr"));
        assert!(output.diagnostics.len() <= 256);
        let directory = tempfile::tempdir()?;
        let response = directory.path().join("response.bin");
        std::fs::write(&response, b"invalid frame")?;
        assert!(reference_process::read_response(&response, 8).is_err());
        assert_eq!(
            reference_process::read_response(&response, 13)?,
            b"invalid frame"
        );
        Ok(())
    }

    #[test]
    fn original_source_hash_preimage_extent_covers_escape_and_byte_arrays() -> TestResult {
        let artifacts = vec![ReferenceInputArtifact {
            path: "source/\"\n\u{0001}é.pse".into(),
            sha256: vec![255; 32].into(),
        }];
        let keys = fixed_keys("ordinary_exchanger", false);
        let case = "ordinary_delivery.exchanger_design";
        let bound = source_hash_scratch(&artifacts, &keys, case)?;
        let artifacts_json = serde_json::to_vec(
            &artifacts
                .iter()
                .map(|input| (&input.path, input.sha256.as_slice()))
                .collect::<Vec<_>>(),
        )?;
        let formulation_json = serde_json::to_vec(&(1u32, case, &artifacts_json))?;
        let specification_json = serde_json::to_vec(
            &keys
                .iter()
                .map(|key| (key, u64::MAX, SemanticId::NIL))
                .collect::<Vec<_>>(),
        )?;
        assert!(
            bound
                >= artifacts_json.capacity()
                    + formulation_json.capacity()
                    + specification_json.capacity()
        );
        Ok(())
    }

    #[tokio::test]
    async fn reference_failure_retains_custody_until_returned_diagnostics_drop() -> TestResult {
        let runtime = crate::workflow::tests::runtime();
        let services = runtime.preparation_services();
        let pool = runtime.shared.pool();
        let baseline = pool.reserved();
        let owners = [
            services
                .math()
                .reserve("reference-control:preparation", 4096)?,
            services
                .math()
                .reserve("reference-control:response", 8192)?,
            services
                .math()
                .reserve("reference-control:encoding", 2048)?,
        ];
        let failure = reference_failure(
            "source metadata refused",
            "original codec cause",
            String::from_utf8_lossy(&[0xff; 128]).into_owned(),
            &owners,
        );
        assert!(failure.to_string().contains("original codec cause"));
        drop(owners);
        assert_eq!(
            pool.reserved() - baseline,
            4096 + 8192 + 2048,
            "returning a diagnostic cannot release its allocation owner"
        );
        drop(failure);
        assert_eq!(pool.reserved(), baseline);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn original_inputs_refuse_escape_and_symlink_members() -> TestResult {
        let directory = original_tree()?;
        let root = directory.path().canonicalize()?;
        for path in [
            "../outside",
            "/etc/passwd",
            "packages//reference/conformance.toml",
            "packages/./reference/conformance.toml",
        ] {
            assert!(source_path(&root, path).is_err(), "{path}");
        }
        let link = root.join("packages/reference/ordinary-delivery/models/linked.pse");
        std::os::unix::fs::symlink(
            root.join("packages/reference/ordinary-delivery/models/root.pse"),
            &link,
        )?;
        assert!(
            source_path(
                &root,
                "packages/reference/ordinary-delivery/models/linked.pse"
            )
            .is_err()
        );
        assert!(read_bounded(&link, LIMIT).is_err());
        assert!(
            OriginalInputs::capture(&root)
                .unwrap_err()
                .to_string()
                .contains("symlink")
        );
        Ok(())
    }

    fn original_output_policy() -> (ResolvedNumericalPolicy, pse_math::numerics::TargetSpec) {
        let id = SemanticId::from_bytes([31; 16]);
        let target = pse_math::numerics::TargetSpec {
            id,
            kind: NumericalTarget::Variable,
            quantity: ids::quantity("temperature.point"),
            unit: ids::unit("K"),
            integer: false,
            declared_tolerance: None,
        };
        let policy = NumericalPolicy {
            engineering_scales: vec![EngineeringScale {
                scale_id: SemanticId::from_bytes([32; 16]).into(),
                model_id: None,
                case_id: Some(SemanticId::from_bytes([33; 16]).into()),
                instance_id: None,
                fit_id: None,
                target_id: id,
                target_kind: target.kind,
                quantity_id: target.quantity.as_id(),
                unit_id: target.unit.as_id(),
                kind: EngineeringScaleKind::RangeWidth,
                value: 100.,
                source: NumericalSource::Case,
                priority: 0,
                provenance: "original fixed 100 K operating span".into(),
            }],
            ..Default::default()
        };
        (
            ResolvedNumericalPolicy {
                policy,
                targets: Vec::new(),
                key: pse_ids::ContentHash::NIL,
            },
            target,
        )
    }

    #[test]
    fn reference_output_resolution_uses_original_context_not_native_stopping() -> TestResult {
        let physical = crate::workflow::tests::physical();
        let (mut original, target) = original_output_policy();
        let context = output_context(&physical, &original, &target)?;
        assert!(!context.canonical_fallback);
        assert_eq!(context.source, NumericalSource::Case);
        assert_eq!(context.scale_kind, Some(EngineeringScaleKind::RangeWidth));
        assert_eq!(
            context.budget.to_bits(),
            (original.policy.engineering_relative_fraction * 100.).to_bits()
        );
        // These govern distinct native numerical decisions, not the independently
        // measured output error permitted by the original physical context.
        original.policy.kkt.stationarity *= 0.01;
        original.policy.kkt.complementarity *= 0.1;
        original.policy.supplier_action_accuracy *= 0.5;
        original.policy.linear_backward_error *= 0.5;
        assert_eq!(output_context(&physical, &original, &target)?, context);
        original.policy.engineering_relative_fraction *= 0.5;
        assert_eq!(
            output_context(&physical, &original, &target)?
                .budget
                .to_bits(),
            (0.5 * context.budget).to_bits()
        );
        Ok(())
    }

    #[test]
    fn reference_output_resolution_preserves_difference_units_and_refuses_fallback() -> TestResult {
        let physical = crate::workflow::tests::physical();
        let (mut original, mut target) = original_output_policy();
        target.unit = ids::unit("degF");
        let context = output_context(&physical, &original, &target)?;
        let conversion = pse_quantity::convert_spec_for_type(
            physical.quantities().unit(ids::unit("K"))?,
            physical.quantities().unit(target.unit)?,
            &physical.quantities().quantity_type(target.quantity)?.key,
        )?;
        assert_eq!(
            context.budget.to_bits(),
            (original.policy.engineering_relative_fraction * (100. * conversion.scale.abs()))
                .to_bits()
        );
        original.policy.engineering_scales.clear();
        assert!(
            output_context(&physical, &original, &target)
                .unwrap_err()
                .to_string()
                .contains("original engineering context")
        );
        Ok(())
    }

    #[test]
    fn reference_output_resolution_refuses_changed_original_physical_owner() -> TestResult {
        let physical = crate::workflow::tests::physical();
        let (original, mut target) = original_output_policy();
        let resolved = pse_math::numerics::resolve(
            physical.quantities(),
            physical.preconditions(),
            &[target.clone()],
            &[],
            &original.policy,
        )?;
        let context = output_context(&physical, &resolved, &target)?;
        assert_eq!(
            context,
            resolved.targets[0]
                .engineering
                .clone()
                .ok_or("original context absent")?
        );
        target.unit = ids::unit("degF");
        assert!(
            output_context(&physical, &resolved, &target)
                .unwrap_err()
                .to_string()
                .contains("original numerical quantity or unit")
        );
        Ok(())
    }

    #[test]
    fn reference_monetary_output_uses_shared_fraction_on_actual_capital_quantity() -> TestResult {
        let physical = crate::workflow::tests::physical();
        let quantity = physical
            .quantities()
            .quantity_types()
            .find(|quantity| quantity.name.as_deref() == Some("CapitalCost"))
            .ok_or("actual CapitalCost quantity absent")?;
        let (mut original, mut target) = original_output_policy();
        target.quantity = quantity.id;
        target.unit = quantity.canonical_unit;
        target.kind = NumericalTarget::Observable;
        assert_eq!(physical.quantities().unit(target.unit)?.symbol, "USD_CE500");
        let scale = &mut original.policy.engineering_scales[0];
        scale.quantity_id = target.quantity.as_id();
        scale.unit_id = target.unit.as_id();
        scale.target_kind = target.kind;
        scale.kind = EngineeringScaleKind::Magnitude;
        scale.value = 1000.;
        scale.provenance = "fixed reference-geometry monetary characteristic".into();
        let context = output_context(&physical, &original, &target)?;
        assert!(!context.canonical_fallback);
        assert!(
            context.physical_allowance.is_none(),
            "no invented absolute monetary floor"
        );
        assert_eq!(context.characteristic, Some(1000.));
        assert_eq!(
            context.budget.to_bits(),
            (original.policy.engineering_relative_fraction * 1000.).to_bits()
        );
        Ok(())
    }
}
