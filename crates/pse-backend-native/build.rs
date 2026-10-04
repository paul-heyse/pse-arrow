// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::print_stdout,
    reason = "a build script talks to cargo on stdout"
)]

//! The native profile's single BLAS/LAPACK and OpenMP provider (ADR-0108 items 5 and 14).
//!
//! When Ipopt, Clarabel's SDP cones or Clarabel's MKL Pardiso are linked, every native
//! routine in the process runs on the solver image's oneMKL (LP64, GNU threading layer) and
//! libgomp. The link line is
//! the image's own `mkl-dynamic-lp64-gomp.pc`, so the image stays the one authority for it.
//! The image manifest (`share/pse-solvers/build-info.txt`) is embedded for the profile key;
//! a build outside the image records it as unavailable.

use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const MKL: &str = "mkl-dynamic-lp64-gomp";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-env-changed=IPOPT_DIR");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PSE_ROOT_ISOLATION_DIR");
    if env::var_os("CARGO_FEATURE_ROOT_ISOLATION").is_some() {
        root_isolation()?;
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR unset")?);
    let ipopt = env::var_os("CARGO_FEATURE_IPOPT").is_some();
    let sdp = env::var_os("CARGO_FEATURE_SDP").is_some();
    let pardiso = env::var_os("CARGO_FEATURE_CLARABEL_PARDISO").is_some();
    let pipeline =
        env::var_os("CARGO_FEATURE_UNO").is_some() || env::var_os("CARGO_FEATURE_PETSC").is_some();
    let prefix = env::var_os("IPOPT_DIR").map(PathBuf::from);
    let mut manifest = String::new();
    if ipopt || sdp || pardiso || pipeline {
        match &prefix {
            Some(prefix) => {
                let pc = prefix.join("lib/pkgconfig").join(format!("{MKL}.pc"));
                println!("cargo:rerun-if-changed={}", pc.display());
                link_from_pc(&pc, &prefix.join("lib"), "dylib")?;
            }
            None => {
                pkg_config::Config::new().probe(MKL).map_err(|e| {
                    format!(
                        "pse-backend-native: the native profile links the solver image's oneMKL \
                         ({MKL}); set IPOPT_DIR to the image prefix or expose {MKL}.pc: {e}"
                    )
                })?;
            }
        }
    }
    if ipopt && let Some(prefix) = &prefix {
        let info = prefix.join("share/pse-solvers/build-info.txt");
        println!("cargo:rerun-if-changed={}", info.display());
        if info.is_file() {
            manifest = fs::read_to_string(&info)?;
        }
    }
    fs::write(out.join("solver-manifest.txt"), manifest)?;
    // The dependency source is declared once by the workspace. Embed the immutable
    // requested revision rather than treating an unchanged crate version as provenance.
    let workspace =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("CARGO_MANIFEST_DIR unset")?)
            .join("../../Cargo.toml");
    println!("cargo:rerun-if-changed={}", workspace.display());
    let workspace_manifest = fs::read_to_string(workspace)?;
    let workspace_manifest: toml::Value = toml::from_str(&workspace_manifest)?;
    let source = &workspace_manifest["workspace"]["dependencies"]["pounce-rs"];
    let git = source["git"]
        .as_str()
        .ok_or("missing immutable POUNCE source")?;
    let revision = source["rev"]
        .as_str()
        .ok_or("missing immutable POUNCE revision")?;
    fs::write(out.join("pounce-source.txt"), format!("{git}@{revision}"))?;
    Ok(())
}

/// Compile the small synchronous native transport against the prepared immutable prefix.
fn root_isolation() -> Result<(), Box<dyn Error>> {
    let prefix = PathBuf::from(
        env::var_os("PSE_ROOT_ISOLATION_DIR")
            .ok_or("root isolation requires native-math-env.sh (or native-isolation-prepare)")?,
    );
    let manifest = prefix.join(".complete.json");
    println!("cargo:rerun-if-changed={}", manifest.display());
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR unset")?);
    fs::copy(manifest, out.join("root-isolation-manifest.json"))?;
    let pc = prefix.join("share/pkgconfig/ibex.pc");
    println!("cargo:rerun-if-changed={}", pc.display());
    println!("cargo:rerun-if-changed=native/root_isolation.cpp");
    let output = Command::new("pkg-config")
        .arg("--cflags")
        .arg(&pc)
        .output()?;
    if !output.status.success() {
        return Err("pkg-config failed for the prepared ibex.pc".into());
    }
    let flags = String::from_utf8(output.stdout)?;
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .file("native/root_isolation.cpp")
        .flag("-frounding-math")
        .flag("-ffp-contract=off")
        .flag("-fno-fast-math")
        .flag("-fvisibility=hidden")
        .flag("-fvisibility-inlines-hidden");
    for flag in flags.split_whitespace() {
        if let Some(path) = flag.strip_prefix("-I") {
            build.flag("-isystem").flag(path);
        } else {
            build.flag(flag);
        }
    }
    build.try_compile("pse_root_isolation")?;
    println!(
        "cargo:rustc-link-search=native={}",
        prefix.join("lib/ibex/3rd").display()
    );
    link_from_pc(&pc, &prefix.join("lib"), "static")?;
    Ok(())
}

/// Emit the `Libs:` line of the image's MKL link description: its `-l` libraries as dynamic
/// links in the described order, searched in the image library directory. The interface
/// layer comes first, so under the linker's `--as-needed` the threading and core layers stay
/// linked for the symbols it leaves undefined.
fn link_from_pc(pc: &Path, libdir: &Path, linkage: &str) -> Result<(), Box<dyn Error>> {
    let text = fs::read_to_string(pc)
        .map_err(|e| format!("pse-backend-native: unreadable {}: {e}", pc.display()))?;
    let libs = text
        .lines()
        .find_map(|l| l.strip_prefix("Libs:"))
        .ok_or_else(|| format!("{} has no Libs line", pc.display()))?;
    println!("cargo:rustc-link-search=native={}", libdir.display());
    let mut linked = 0;
    for token in libs.split_whitespace() {
        if let Some(name) = token.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={linkage}={name}");
            linked += 1;
        }
    }
    if linked == 0 {
        return Err(format!("{} names no libraries", pc.display()).into());
    }
    Ok(())
}
