// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed application entry for the original Study coordinator; children own native work.
use crate::{HostError, HostManager, discovery::BuildCapture, environment::Environment};
use std::{os::unix::process::CommandExt as _, path::PathBuf, process::Command};

/// Original source-built images and selected inputs for the three-phase Study journey.
/// Parsing these paths creates neither a service nor a native allocation.
#[derive(Debug, clap::Parser)]
#[command(about = "Qualify the original Study using prepared captures; prepare nothing implicitly")]
pub struct Args {
    /// Original checkout whose native manager authenticates the selected work.
    #[arg(long)]
    pub root: PathBuf,
    /// Completed native BuildCapture containing keeper, frontend and pse-worker.
    #[arg(long)]
    pub runtime_build: PathBuf,
    /// Completed native BuildCapture containing pse-canonical-service.
    #[arg(long)]
    pub canonical_build: PathBuf,
    /// Existing explicitly prepared production module receipt.
    #[arg(long)]
    pub module_receipt: PathBuf,
    /// Original validated native scientific allocation; the keeper revalidates it.
    #[arg(long)]
    pub worker_allocation: PathBuf,
    /// Fresh absolute private diagnostic directory, created only by the coordinator.
    #[arg(long)]
    pub work_directory: PathBuf,
    /// Startup observation allowance in milliseconds.
    #[arg(long, default_value_t = 120_000)]
    pub startup_ms: u64,
    /// Observation allowance for each original client phase, including source build.
    #[arg(long, default_value_t = 600_000)]
    pub phase_ms: u64,
    /// Original independent shutdown/drain observation allowance.
    #[arg(long, default_value_t = 90_000)]
    pub shutdown_ms: u64,
}

/// Host-only activation selection; it is not forwarded as a coordinator protocol field.
#[derive(Debug, clap::Args)]
pub(crate) struct LaunchArgs {
    #[command(flatten)]
    input: Args,
    /// Select the same compiler-cache context as the explicitly prepared native generation.
    #[arg(long, value_enum, default_value = "auto")]
    cache: crate::cli::CacheArg,
}

impl Args {
    fn command(&self, image: &std::path::Path, environment: Environment) -> Command {
        let mut command = Command::new(image);
        command
            .current_dir(&self.root)
            .env_clear()
            .envs(environment)
            .arg("qualify-study");
        for (name, value) in [
            ("--root", &self.root),
            ("--runtime-build", &self.runtime_build),
            ("--canonical-build", &self.canonical_build),
            ("--module-receipt", &self.module_receipt),
            ("--worker-allocation", &self.worker_allocation),
            ("--work-directory", &self.work_directory),
        ] {
            command.arg(name).arg(value);
        }
        for (name, value) in [
            ("--startup-ms", self.startup_ms),
            ("--phase-ms", self.phase_ms),
            ("--shutdown-ms", self.shutdown_ms),
        ] {
            command.arg(name).arg(value.to_string());
        }
        command
    }
}

/// Select prepared libraries and replace this fixed application entry with its captured owner.
/// No workload, allocation, service or exported environment mapping is created here.
/// # Errors
/// Foreign/incomplete images, invalid application placement or unavailable native providers.
pub(crate) fn launch(
    launch: LaunchArgs,
    selected_root: Option<&std::path::Path>,
) -> Result<i32, HostError> {
    let mut args = launch.input;
    args.root = args.root.canonicalize()?;
    if let Some(selected) = selected_root
        && selected.canonicalize()? != args.root
    {
        return Err(HostError::Ownership(
            "qualification checkout differs from selected checkout".into(),
        ));
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let manager = HostManager::connect_user(None).await?;
        let membership = manager.current_membership().await?;
        if manager.native_workload(membership.root()).await? {
            return Err(HostError::Ownership("Study qualification requires the original application scope, outside a native workload".into()));
        }
        let (_, _, tasks) = manager.foreground_placement(membership.root()).await?;
        crate::foreground::preflight_service(&membership.root().control_group, tasks)?;
        Ok::<_, HostError>(())
    })?;
    drop(runtime);
    let capture = BuildCapture::read(&args.runtime_build)?;
    let image = capture.require_binary(
        &args.root,
        "pse-runtime",
        "pse-engineering-fixture",
        "crates/pse-runtime/src/bin/pse-engineering-fixture.rs",
        &["engineering-service", "native-service", "product-reader"],
    )?;
    let caller = std::env::vars_os().collect();
    let environment = crate::environment::activate_with_options(
        &args.root,
        &caller,
        &Environment::new(),
        launch.cache.into(),
        false,
    )?
    .values;
    let environment =
        crate::native::select_capabilities(&args.root, &environment, "compiler,klu,solver")?;
    image.verify()?;
    let error = args.command(&image.captured, environment).exec();
    match error.kind() {
        std::io::ErrorKind::NotFound => Ok(127),
        std::io::ErrorKind::PermissionDenied => Ok(126),
        _ => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;
    use std::ffi::OsString;
    #[test]
    fn qualification_launch_keeps_literal_paths_deadlines_and_fixed_entry() {
        #[derive(clap::Parser)]
        struct Entry {
            #[command(flatten)]
            launch: LaunchArgs,
        }
        let entry = Entry::parse_from([
            "qualify-study",
            "--root",
            "/original/root",
            "--runtime-build",
            "/capture/runtime.json",
            "--canonical-build",
            "/capture/canonical.json",
            "--module-receipt",
            "/module/receipt.json",
            "--worker-allocation",
            "/allocation.json",
            "--work-directory",
            "/diagnostics/space ; literal",
            "--phase-ms",
            "1800000",
            "--cache",
            "off",
        ]);
        assert!(matches!(entry.launch.cache, crate::cli::CacheArg::Off));
        let args = entry.launch.input;
        let command = args.command(
            std::path::Path::new("/captured/coordinator"),
            [(
                OsString::from("FIXTURE_SECRET"),
                OsString::from("native-value"),
            )]
            .into(),
        );
        assert_eq!(command.get_program(), "/captured/coordinator");
        let arguments: Vec<_> = command.get_args().collect();
        assert_eq!(arguments[0], "qualify-study");
        assert!(!arguments.iter().any(|argument| *argument == "--cache"));
        assert_eq!(arguments[12], "/diagnostics/space ; literal");
        assert!(
            arguments
                .windows(2)
                .any(|pair| pair == ["--phase-ms", "1800000"])
        );
        assert_eq!(
            command.get_current_dir(),
            Some(std::path::Path::new("/original/root"))
        );
        assert_eq!(
            command.get_envs().collect::<Vec<_>>(),
            vec![(
                std::ffi::OsStr::new("FIXTURE_SECRET"),
                Some(std::ffi::OsStr::new("native-value"))
            )]
        );
    }
}
