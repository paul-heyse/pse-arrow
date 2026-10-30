// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Parser projection into the one test/check selection owner.
use crate::{
    correctness::CorrectnessError,
    selection::{self, Features, Operation, PackageIntent, Request, Scope, TargetIntent},
};
use clap::Args;

#[derive(Debug, Args)]
#[command(group(clap::ArgGroup::new("workspace_selection").args(["workspace", "all"]).multiple(false)))]
#[command(group(clap::ArgGroup::new("package_selection").args(["targets", "package"]).multiple(true).conflicts_with("workspace_selection")))]
#[command(mut_arg("exclude", |argument| argument.requires("workspace_selection")))]
pub(super) struct SelectionArgs {
    /// Conservatively select changed source owners and reverse dependencies.
    #[arg(long, conflicts_with_all = ["targets", "package", "workspace", "all", "target_kind"])]
    changed: bool,
    /// Git base for --changed, including observed index/worktree/untracked changes.
    #[arg(long, requires = "changed", default_value = "HEAD")]
    base: String,
    /// Workspace package names, optionally followed by :TARGET. Use -p for opaque package IDs.
    #[arg(value_name = "PACKAGE[:TARGET]")]
    targets: Vec<String>,
    #[command(flatten)]
    workspace: clap_cargo::Workspace,
    #[command(flatten)]
    features: clap_cargo::Features,
    /// Select actual library declarations, without guessing their names.
    #[arg(long, group = "cargo_targets", conflicts_with = "target_kind")]
    lib: bool,
    /// Preserve native Cargo all-targets selection without enabling all features.
    #[arg(long, group = "cargo_targets", conflicts_with = "target_kind")]
    all_targets: bool,
    /// Select one integration-test target.
    #[arg(long = "test", group = "cargo_targets", conflicts_with = "target_kind")]
    test_target: Option<String>,
    /// Select one binary target.
    #[arg(long = "bin", group = "cargo_targets", conflicts_with = "target_kind")]
    bin_target: Option<String>,
    /// Features that the actual selected compiler roots must keep absent.
    #[arg(long = "without-feature", value_delimiter = ',')]
    absent: Vec<String>,
    /// Explicit Cargo target platform; host dependency contexts remain independent.
    #[arg(long = "target")]
    platform: Option<String>,
    /// Disambiguate duplicate target names by their Cargo kind, for example test or bin.
    #[arg(long)]
    target_kind: Option<String>,
}
impl SelectionArgs {
    fn cargo_targets(&self) -> Option<selection::CargoTargets> {
        if self.lib {
            Some(selection::CargoTargets::Library)
        } else if self.all_targets {
            Some(selection::CargoTargets::AllTargets)
        } else if let Some(name) = &self.test_target {
            Some(selection::CargoTargets::Test(name.clone()))
        } else {
            self.bin_target
                .as_ref()
                .map(|name| selection::CargoTargets::Bin(name.clone()))
        }
    }
    pub(super) fn request(&self, operation: Operation) -> Result<Request, CorrectnessError> {
        let features = Features {
            defaults: !self.features.no_default_features,
            all: self.features.all_features,
            enabled: self
                .features
                .features
                .iter()
                .flat_map(|features| features.split(','))
                .map(str::to_owned)
                .collect(),
            absent: self.absent.iter().cloned().collect(),
        };
        let scope = if self.changed {
            Scope::Changed {
                base: self.base.clone(),
                features,
            }
        } else if self.workspace.workspace || self.workspace.all {
            Scope::Workspace {
                exclude: self.workspace.exclude.clone(),
                features,
            }
        } else if self.targets.is_empty() && self.workspace.package.is_empty() {
            Scope::DefaultMembers { features }
        } else {
            let mut packages = Vec::new();
            for input in &self.targets {
                let (package, target) = match input.split_once(':') {
                    Some((package, target)) if !package.is_empty() && !target.is_empty() => (
                        package,
                        Some(TargetIntent {
                            name: target.into(),
                            kind: self.target_kind.clone(),
                        }),
                    ),
                    Some(_) => return Err(CorrectnessError::Invalid(
                        "target intent requires nonempty PACKAGE:TARGET; opaque IDs belong to -p"
                            .into(),
                    )),
                    None => (input.as_str(), None),
                };
                packages.push(PackageIntent {
                    package: package.into(),
                    target,
                    features: features.clone(),
                });
            }
            packages.extend(self.workspace.package.iter().map(|package| PackageIntent {
                package: package.clone(),
                target: None,
                features: features.clone(),
            }));
            Scope::Packages(packages)
        };
        let selector = self.cargo_targets();
        if selector.is_some() && self.targets.iter().any(|target| target.contains(':')) {
            return Err(CorrectnessError::Invalid(
                "native target flags and PACKAGE:TARGET are alternative selectors".into(),
            ));
        }
        let scope = match selector {
            Some(selector) => Scope::Targets {
                scope: Box::new(scope),
                selector,
            },
            None => scope,
        };
        Ok(Request {
            operation,
            scope,
            platform: self.platform.clone(),
            filters: selection::Filters::default(),
            ignored: selection::IgnoredPolicy::Default,
        })
    }
}

#[derive(Debug, Args)]
pub(super) struct TestArgs {
    /// Rebuild current source for exact failed identities and original compiler policy.
    #[arg(long, value_name = "INVOCATION", conflicts_with_all = ["changed", "targets", "package", "workspace", "all", "profile", "expressions", "words", "ignored"])]
    rerun_failed: Option<String>,
    #[command(flatten)]
    pub(super) selection: SelectionArgs,
    /// Bypass the selected nextest profile's default filter.
    #[arg(long)]
    ignore_default_filter: bool,
    /// Native nextest filtersets; nextest owns their grammar.
    #[arg(short = 'E', long = "filterset")]
    expressions: Vec<String>,
    /// Bare test-name filters after --; these are not package or target names.
    #[arg(last = true)]
    words: Vec<String>,
    #[arg(long, value_enum, default_value = "default")]
    ignored: selection::IgnoredPolicy,
    /// Selected nextest policy profile, without running its setup during explain.
    #[arg(long, default_value = "default")]
    pub(super) profile: String,
}
impl TestArgs {
    pub(super) fn request(&self) -> Result<Request, CorrectnessError> {
        let mut request = self.selection.request(Operation::Test)?;
        if let Some(invocation) = &self.rerun_failed {
            if self.selection.cargo_targets().is_some()
                || self.ignore_default_filter
                || self.selection.features.no_default_features
                || self.selection.features.all_features
                || !self.selection.features.features.is_empty()
                || !self.selection.absent.is_empty()
                || self.selection.platform.is_some()
                || self.selection.target_kind.is_some()
            {
                return Err(CorrectnessError::Invalid("rerun preserves recorded compiler policy; choose an explicit new selection to change it".into()));
            }
            request.scope = Scope::RerunFailed {
                invocation: invocation.clone(),
            };
        }
        request.filters = selection::Filters {
            ignore_default_filter: self.ignore_default_filter,
            words: self.words.clone(),
            expressions: self.expressions.clone(),
        };
        request.ignored = self.ignored;
        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    #[derive(Debug, Parser)]
    struct Parsed {
        #[command(flatten)]
        tests: TestArgs,
    }
    #[test]
    fn cargo_workspace_exclusions_and_package_filtersets_remain_legal() {
        let workspace = Parsed::try_parse_from([
            "selection",
            "--workspace",
            "--exclude",
            "pse-runtime",
            "-E",
            "test(case)",
        ])
        .unwrap();
        let request = workspace.tests.request().unwrap();
        assert!(
            matches!(request.scope, Scope::Workspace { exclude, .. } if exclude == ["pse-runtime"])
        );
        assert_eq!(request.filters.expressions, ["test(case)"]);
        let package = Parsed::try_parse_from([
            "selection",
            "-p",
            "pse-host",
            "-E",
            "test(case)",
            "--features",
            "first,second third",
            "--",
            "bare",
        ])
        .unwrap();
        let request = package.tests.request().unwrap();
        assert_eq!(request.filters.words, ["bare"]);
        let Scope::Packages(packages) = request.scope else {
            panic!("explicit package lost")
        };
        assert_eq!(
            packages[0]
                .features
                .enabled
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["first", "second", "third"]
        );
    }
    #[test]
    fn target_identity_absence_platform_and_ignored_policy_survive_projection() {
        let parsed = Parsed::try_parse_from([
            "selection",
            "pse-runtime:native_study",
            "--target-kind",
            "test",
            "--without-feature",
            "forbidden",
            "--target",
            "target-triple",
            "--ignored",
            "all",
            "--ignore-default-filter",
        ])
        .unwrap();
        let request = parsed.tests.request().unwrap();
        assert_eq!(request.platform.as_deref(), Some("target-triple"));
        assert_eq!(request.ignored, selection::IgnoredPolicy::All);
        assert!(request.filters.ignore_default_filter);
        let Scope::Packages(packages) = request.scope else {
            panic!("explicit target lost")
        };
        assert_eq!(packages[0].target.as_ref().unwrap().name, "native_study");
        assert_eq!(
            packages[0].target.as_ref().unwrap().kind.as_deref(),
            Some("test")
        );
        assert!(packages[0].features.absent.contains("forbidden"));
    }
    #[test]
    fn nonsensical_workspace_combinations_refuse_in_the_actual_parser() {
        for args in [
            vec!["selection", "--exclude", "pse-host"],
            vec!["selection", "--workspace", "-p", "pse-host"],
            vec!["selection", "--workspace", "pse-host"],
            vec!["selection", "--workspace", "--all"],
        ] {
            assert!(Parsed::try_parse_from(args).is_err());
        }
    }

    #[test]
    fn native_target_modes_are_exclusive_and_preserve_package_source_intent() {
        for flag in ["--lib", "--all-targets"] {
            let parsed = Parsed::try_parse_from(["selection", "alpha", flag]).unwrap();
            assert!(matches!(
                parsed.tests.request().unwrap().scope,
                Scope::Targets { .. }
            ));
        }
        for args in [
            vec!["selection", "--lib", "--all-targets"],
            vec!["selection", "alpha:target", "--lib"],
            vec!["selection", "--rerun-failed", "abc", "--lib"],
        ] {
            assert!(match Parsed::try_parse_from(args) {
                Err(_) => true,
                Ok(parsed) => parsed.tests.request().is_err(),
            });
        }
    }

    #[test]
    fn source_selection_has_explicit_parser_owned_intent() {
        let changed =
            Parsed::try_parse_from(["selection", "--changed", "--base", "HEAD~1"]).unwrap();
        assert!(
            matches!(changed.tests.request().unwrap().scope,Scope::Changed {base,..} if base=="HEAD~1")
        );
        let rerun = Parsed::try_parse_from(["selection", "--rerun-failed", "01234567"]).unwrap();
        assert!(
            matches!(rerun.tests.request().unwrap().scope,Scope::RerunFailed {invocation} if invocation=="01234567")
        );
        for args in [
            vec!["selection", "--changed", "alpha"],
            vec!["selection", "--rerun-failed", "abc", "--changed"],
            vec!["selection", "--rerun-failed", "abc", "-E", "all()"],
        ] {
            assert!(Parsed::try_parse_from(args).is_err());
        }
    }
}
