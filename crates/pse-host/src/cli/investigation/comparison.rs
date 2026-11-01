// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected original pairs. Source, artifact, admitted-unit and native meanings stay separate.
use crate::{
    HostError,
    envelope::{Completeness, Envelope, OutputOptions},
    investigation::{self as query, Locator, ResultReference},
};
use clap::Args as ClapArgs;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
mod context;
mod semantic;
mod source;
pub(in crate::cli) use context::Side as ContextExpansion;
pub(in crate::cli) use context::{ContextInspection, context_inspection};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub(in crate::cli) enum View {
    Source,
    Api,
    Context,
    Semantic,
}
#[derive(Debug, ClapArgs)]
#[command(group(clap::ArgGroup::new("comparison_input").args(["request","before_result"]).required(true).multiple(false)))]
pub(in crate::cli) struct Args {
    /// Complete versioned pair request. Exclusive with all pair flags.
    #[arg(long, conflicts_with_all=["before_result","after_result","view","before_checkout","after_checkout"])]
    request: Option<PathBuf>,
    /// Source selector or original opaque address; supplied guards are enforced unchanged.
    #[arg(long, requires_all=["after_result","view"])]
    before_result: Option<String>,
    /// Source selector or original opaque address for the after observation.
    #[arg(long, requires_all=["before_result","view"])]
    after_result: Option<String>,
    #[arg(long, value_enum, requires_all=["before_result","after_result"])]
    view: Option<View>,
    /// Explicit original invocation owner for the before native/context address.
    #[arg(long, requires = "before_result")]
    before_checkout: Option<PathBuf>,
    /// Explicit original invocation owner for the after native/context address.
    #[arg(long, requires = "after_result")]
    after_checkout: Option<PathBuf>,
}
fn allowance() -> usize {
    100
}
/// Both flag and JSON forms reach this validator before any read/provider operation.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(in crate::cli) struct Request {
    version: u32,
    view: View,
    before: ResultReference,
    after: ResultReference,
    #[serde(default)]
    before_checkout: Option<PathBuf>,
    #[serde(default)]
    after_checkout: Option<PathBuf>,
    /// Whole records per original side and aligned result; omitted records remain explicit.
    #[serde(default = "allowance")]
    max_records: usize,
    /// Optional selected world enriching retained-unit comparison; never prepares it.
    #[serde(default)]
    before_world: Option<ResultReference>,
    #[serde(default)]
    after_world: Option<ResultReference>,
}
fn invalid(message: impl Into<String>) -> HostError {
    query::Error::Invalid(message.into()).into()
}
impl Request {
    fn validate(&self) -> Result<(), HostError> {
        if self.version != 1 || self.max_records == 0 {
            return Err(invalid(
                "comparison requires version 1 and positive max_records",
            ));
        }
        for (reference, root) in [
            (&self.before, &self.before_checkout),
            (&self.after, &self.after_checkout),
        ] {
            reference.validate()?;
            let suitable = matches!(
                (self.view, &reference.locator),
                (View::Source, Locator::Source { .. })
                    | (View::Api, Locator::Api { .. })
                    | (View::Context, Locator::Context { .. })
                    | (View::Semantic, Locator::Semantic { .. })
            );
            if !suitable {
                return Err(invalid(
                    "comparison view and original address kind disagree",
                ));
            }
            if matches!(self.view, View::Semantic | View::Context) && root.is_none() {
                return Err(invalid(
                    "semantic/context comparison requires both explicit owning checkouts",
                ));
            }
            if root.as_ref().is_some_and(|root| !root.is_absolute()) {
                return Err(invalid("comparison checkout must be absolute"));
            }
            if let Locator::Semantic {
                session_invocation,
                generation,
                source_sha256,
                ..
            } = &reference.locator
            {
                if session_invocation.is_none() || generation.is_none() || source_sha256.is_none() {
                    return Err(invalid(
                        "semantic comparison requires original session, generation and exact source guards",
                    ));
                }
            }
            if matches!(
                &reference.locator,
                Locator::Api {
                    artifact_sha256: None,
                    ..
                } | Locator::Api {
                    context_key: None,
                    ..
                }
            ) {
                return Err(invalid(
                    "API comparison requires original artifact and extraction-context guards",
                ));
            }
        }
        if self.view != View::Context && (self.before_world.is_some() || self.after_world.is_some())
        {
            return Err(invalid("optional world enrichment requires context view"));
        }
        for world in [&self.before_world, &self.after_world]
            .into_iter()
            .flatten()
        {
            world.validate()?;
            if !matches!(
                world.locator,
                Locator::Semantic {
                    session_invocation: Some(_),
                    generation: Some(_),
                    source_sha256: Some(_),
                    ..
                }
            ) {
                return Err(invalid(
                    "context world requires an original guarded semantic address",
                ));
            }
        }
        if self.view == View::Semantic
            && self.before_checkout == self.after_checkout
            && same_session(&self.before, &self.after)
        {
            return Err(invalid(
                "two-world comparison requires distinct original native world owners",
            ));
        }
        Ok(())
    }
}
impl Args {
    fn lower(self) -> Result<Request, HostError> {
        let request = if let Some(path) = self.request {
            serde_json::from_slice(&std::fs::read(path)?).map_err(query::Error::from)?
        } else {
            Request {
                version: 1,
                view: self
                    .view
                    .ok_or_else(|| invalid("comparison view is required"))?,
                before: ResultReference::decode(
                    &self
                        .before_result
                        .ok_or_else(|| invalid("before-result is required"))?,
                )?,
                after: ResultReference::decode(
                    &self
                        .after_result
                        .ok_or_else(|| invalid("after-result is required"))?,
                )?,
                before_checkout: self.before_checkout,
                after_checkout: self.after_checkout,
                max_records: allowance(),
                before_world: None,
                after_world: None,
            }
        };
        request.validate()?;
        Ok(request)
    }
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "availability", rename_all = "snake_case")]
pub(in crate::cli) enum Observation<T> {
    Available {
        value: T,
    },
    Unavailable {
        diagnostic: crate::envelope::ErrorRecord,
    },
}
impl<T> Observation<T> {
    fn from_result(result: Result<T, HostError>) -> Self {
        match result {
            Ok(value) => Self::Available { value },
            Err(error) => Self::Unavailable {
                diagnostic: crate::envelope::ErrorRecord::from_typed(
                    &error,
                    &crate::envelope::RedactionPolicy::from_environment(),
                ),
            },
        }
    }
    fn value(&self) -> Option<&T> {
        match self {
            Self::Available { value } => Some(value),
            _ => None,
        }
    }
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "view", rename_all = "snake_case")]
pub(in crate::cli) enum Report {
    Source {
        request: Request,
        comparison: source::Comparison,
    },
    Api {
        request: Request,
        before: Observation<crate::api::comparison::Side>,
        after: Observation<crate::api::comparison::Side>,
        comparison: Observation<crate::api::comparison::Page>,
    },
    Context {
        request: Request,
        comparison: context::Comparison,
    },
    Semantic {
        request: Request,
        comparison: semantic::Comparison,
    },
}
fn side_root(selected: &Path, explicit: Option<&PathBuf>, reference: &ResultReference) -> PathBuf {
    explicit
        .cloned()
        .or_else(|| match &reference.locator {
            Locator::Source {
                guard: Some(guard), ..
            } => guard.root.as_ref().map(query::NativePath::to_path_buf),
            _ => None,
        })
        .unwrap_or_else(|| selected.to_owned())
}
fn api(root: &Path, reference: &ResultReference) -> Result<crate::api::Artifact, HostError> {
    // Reuse the canonical artifact hydration, including reexport definition guards, once.
    let artifact = query::rust::checked_api(root, &reference.locator)?;
    let Locator::Api { item, .. } = &reference.locator else {
        return Err(invalid("API comparison requires acquired item addresses"));
    };
    let id = item
        .parse::<u32>()
        .map_err(|_| invalid("invalid original artifact-local item address"))?;
    if !artifact.document.index.contains_key(&rustdoc_types::Id(id)) {
        return Err(
            query::Error::Stale("original API item is absent from its artifact".into()).into(),
        );
    }
    Ok(artifact)
}
fn same_session(before: &ResultReference, after: &ResultReference) -> bool {
    matches!((&before.locator,&after.locator),(Locator::Semantic{session_invocation:Some(a),..},Locator::Semantic{session_invocation:Some(b),..}) if a==b)
}
fn api_side(artifact: &crate::api::Artifact) -> Result<crate::api::comparison::Side, HostError> {
    Ok(crate::api::comparison::Side {
        package_id: artifact.context.package_id.clone(),
        artifact: artifact.path.clone(),
        sha256: artifact.sha256.clone(),
        context_key: artifact.context.key().map_err(query::Error::from)?,
    })
}
pub(in crate::cli) fn dispatch(
    args: Args,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    output.validate_for::<Report>("inspect comparison")?;
    let request = args.lower()?;
    let root = selected.map_or_else(std::env::current_dir, std::path::absolute)?;
    let before_root = side_root(&root, request.before_checkout.as_ref(), &request.before);
    let after_root = side_root(&root, request.after_checkout.as_ref(), &request.after);
    if request.view == View::Semantic
        && same_session(&request.before, &request.after)
        && matches!((before_root.canonicalize(),after_root.canonicalize()),(Ok(a),Ok(b)) if a==b)
    {
        return Err(invalid(
            "two-world comparison cannot reuse one original native world through checkout aliases",
        ));
    }
    let report = match request.view {
        View::Source => {
            let comparison = source::inspect(&request, &before_root, &after_root);
            Report::Source {
                request,
                comparison,
            }
        }
        View::Api => {
            let before = api(&before_root, &request.before);
            let after = api(&after_root, &request.after);
            let comparison = match (&before, &after) {
                (Ok(before), Ok(after)) => {
                    crate::api::comparison::inspect(before, after, request.max_records, None)
                        .map_err(|error| query::Error::from(error).into())
                }
                _ => Err(query::Error::Unavailable(
                    "both original API artifacts are required; neither was acquired by this read"
                        .into(),
                )
                .into()),
            };
            Report::Api {
                request,
                before: Observation::from_result(before.and_then(|a| api_side(&a))),
                after: Observation::from_result(after.and_then(|a| api_side(&a))),
                comparison: Observation::from_result(comparison),
            }
        }
        View::Context => {
            let comparison = cancel_owned(|token| {
                let (request, before, after) = (&request, &before_root, &after_root);
                async move { context::inspect(request, before, after, &token).await }
            })?;
            Report::Context {
                request,
                comparison,
            }
        }
        View::Semantic => {
            let comparison = cancel_owned(|token| {
                let (request, before, after) = (&request, &before_root, &after_root);
                async move { semantic::inspect(request, before, after, &token).await }
            })?;
            Report::Semantic {
                request,
                comparison,
            }
        }
    };
    // Each side retains its original refusal. Unknown sides cannot produce a negative verdict.
    let partial = match &report {
        Report::Source { comparison, .. } => comparison.partial(),
        Report::Context { comparison, .. } => comparison.partial(),
        Report::Semantic { comparison, .. } => comparison.partial(),
        Report::Api { comparison, .. } => comparison
            .value()
            .is_none_or(|page| page.omitted_differences > 0 || page.missing_items != [0, 0]),
    };
    let mut response = Envelope::response(
        "inspect comparison",
        "selected original observations with qualified side correspondence",
        report,
    );
    if partial {
        response.observation.completeness = Completeness::Partial;
    }
    response.observation.limitations.push("This read does not prepare, acquire, compile or execute. Correspondence candidates are not cross-revision identity, compiler equivalence, behavioral compatibility or validation coverage.".into());
    output
        .emit_page(response, crate::envelope::DetailView::default())
        .map_err(HostError::from)
}

/// Expand the same opaque retained-context address without preparing or reacquiring anything.
pub(in crate::cli) fn inspect_context(
    root: &Path,
    locator: &Locator,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    output.validate_for::<ContextExpansion>("inspect context")?;
    let reference = ResultReference::new(locator.clone());
    reference.validate()?;
    let report = super::super::runtime()?.block_on(context::inspect_reference(
        root,
        &reference,
        output.presentation.budget.unwrap_or_else(allowance),
    ))?;
    let partial = report.partial();
    let detail = report.detail(output.presentation.budget);
    let mut response = Envelope::response(
        "inspect context",
        "original independently verified metadata and admitted compiler-unit evidence",
        report,
    );
    if partial {
        response.observation.completeness = Completeness::Partial;
    }
    response.observation.reference = Some(reference.encode()?);
    output.emit_page(response, detail).map_err(HostError::from)
}
fn cancel_owned<T, F, Fut>(operation: F) -> Result<T, HostError>
where
    F: FnOnce(tokio_util::sync::CancellationToken) -> Fut,
    Fut: Future<Output = Result<T, HostError>>,
{
    super::super::runtime()?.block_on(async {
        let token=tokio_util::sync::CancellationToken::new();
        let work=operation(token.clone());tokio::pin!(work);
        tokio::select! { result=&mut work=>result, signal=tokio::signal::ctrl_c()=>{signal?;token.cancel();work.await} }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;
    #[derive(clap::Parser)]
    struct Parsed {
        #[command(flatten)]
        args: Args,
    }
    fn source() -> ResultReference {
        ResultReference::new(Locator::Source {
            path: PathBuf::from("src/lib.rs").into(),
            source: query::SourceKind::Working,
            range: None,
            guard: None,
        })
    }
    #[test]
    fn comparison_public_flags_and_json_use_the_same_validated_pair() {
        let reference = source().encode().unwrap();
        assert!(
            crate::cli::Cli::try_parse_from([
                "pse",
                "inspect",
                "comparison",
                "--before-result",
                &reference,
                "--after-result",
                &reference,
                "--view",
                "source"
            ])
            .is_ok()
        );
        let flags = Parsed::try_parse_from([
            "pair",
            "--before-result",
            &reference,
            "--after-result",
            &reference,
            "--view",
            "source",
        ])
        .unwrap()
        .args
        .lower()
        .unwrap();
        let t = tempfile::tempdir().unwrap();
        let path = t.path().join("pair.json");
        std::fs::write(&path, serde_json::to_vec(&flags).unwrap()).unwrap();
        let json = Parsed::try_parse_from([
            std::ffi::OsStr::new("pair"),
            std::ffi::OsStr::new("--request"),
            path.as_os_str(),
        ])
        .unwrap()
        .args
        .lower()
        .unwrap();
        assert_eq!(
            serde_json::to_value(flags).unwrap(),
            serde_json::to_value(json).unwrap()
        );
        assert!(
            Parsed::try_parse_from(["pair", "--request", "pair.json", "--view", "source"]).is_err()
        );
        assert!(
            Parsed::try_parse_from(["pair", "--before-result", &reference, "--view", "source"])
                .is_err()
        );
    }
    #[test]
    fn comparison_context_guards_and_original_worlds_are_required_before_reads() {
        let mut request = Request {
            version: 1,
            view: View::Context,
            before: source(),
            after: source(),
            before_checkout: None,
            after_checkout: None,
            max_records: 10,
            before_world: None,
            after_world: None,
        };
        assert!(request.validate().is_err());
        let invocation = crate::envelope::CliInvocationId::new().to_string();
        request.before = ResultReference::new(Locator::Context {
            metadata: PathBuf::from("/retained/metadata.json").into(),
            metadata_sha256: "a".repeat(64),
            compiler_invocation: invocation.clone(),
            compiler_context_sha256: "b".repeat(64),
        });
        request.after = request.before.clone();
        assert!(request.validate().is_err());
        request.before_checkout = Some("/before".into());
        request.after_checkout = Some("/after".into());
        assert!(request.validate().is_ok());
        if let Locator::Context {
            metadata_sha256, ..
        } = &mut request.before.locator
        {
            *metadata_sha256 = "wrong".into();
        }
        assert!(request.validate().is_err());
        request.view = View::Semantic;
        request.before = ResultReference::new(Locator::Semantic {
            project: "/project".into(),
            path: PathBuf::from("/project/src/lib.rs").into(),
            line: 1,
            column: 1,
            generation: Some("1".into()),
            source_sha256: Some("c".repeat(64)),
            candidate_fingerprint: None,
            session_invocation: Some(invocation),
        });
        request.after = request.before.clone();
        request.after_checkout = request.before_checkout.clone();
        assert!(request.validate().is_err());
    }
    #[test]
    fn comparison_missing_original_api_never_prepares_an_artifact() {
        let t = tempfile::tempdir().unwrap();
        let reference = ResultReference::new(Locator::Api {
            artifact: t.path().join("missing/document.json"),
            item: "0".into(),
            artifact_sha256: Some("a".repeat(64)),
            context_key: Some("b".repeat(64)),
            public_path: None,
            definition: None,
        });
        assert!(api(t.path(), &reference).is_err());
        assert_eq!(std::fs::read_dir(t.path()).unwrap().count(), 0);
    }
}
