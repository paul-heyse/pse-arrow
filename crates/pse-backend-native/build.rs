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

#[path = "build-link.rs"]
mod build_link;
mod build_receipt;
#[path = "build-source.rs"]
mod build_source;

const MKL: &str = "mkl-dynamic-lp64-gomp";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build-link.rs");
    println!("cargo:rerun-if-changed=build_receipt.rs");
    println!("cargo:rerun-if-changed=build-source.rs");
    let isolation = env::var_os("CARGO_FEATURE_ROOT_ISOLATION").is_some();
    if isolation {
        println!("cargo:rerun-if-env-changed=PSE_ROOT_ISOLATION_DIR");
        root_isolation()?;
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR unset")?);
    let ipopt = env::var_os("CARGO_FEATURE_IPOPT").is_some();
    let sdp = env::var_os("CARGO_FEATURE_SDP").is_some();
    let pardiso = env::var_os("CARGO_FEATURE_CLARABEL_PARDISO").is_some();
    let pipeline =
        env::var_os("CARGO_FEATURE_UNO").is_some() || env::var_os("CARGO_FEATURE_PETSC").is_some();
    let linked = ipopt || sdp || pardiso || pipeline || env::var_os("CARGO_FEATURE_SCIP").is_some();
    if linked || isolation {
        println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    }
    let prefix = if linked {
        println!("cargo:rerun-if-env-changed=IPOPT_DIR");
        env::var_os("IPOPT_DIR").map(PathBuf::from)
    } else {
        None
    };
    let mut manifest = String::new();
    let mut dynamic_link = String::new();
    if linked {
        match &prefix {
            Some(prefix) => {
                let pc = prefix.join("lib/pkgconfig").join(format!("{MKL}.pc"));
                println!("cargo:rerun-if-changed={}", pc.display());
                dynamic_link = link_from_pc(&pc, &prefix.join("lib"), "dylib")?;
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
    build_receipt::write_if_changed(&out.join("provider-link.rs"), dynamic_link.as_bytes())?;
    if ipopt && let Some(prefix) = &prefix {
        let info = prefix.join("share/pse-solvers/build-info.txt");
        println!("cargo:rerun-if-changed={}", info.display());
        if info.is_file() {
            manifest = fs::read_to_string(&info)?;
        }
    }
    build_receipt::write_if_changed(&out.join("solver-manifest.txt"), manifest.as_bytes())?;
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
    build_receipt::write_if_changed(
        &out.join("pounce-source.txt"),
        format!("{git}@{revision}").as_bytes(),
    )?;
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("CARGO_MANIFEST_DIR unset")?)
        .join("../..");
    supplier_sources(&out, &root, &workspace_manifest)?;
    kinsol_profile(&out)?;
    Ok(())
}

/// Embed only consumed suppliers. The dependency itself reports the final SUNDIALS
/// source selection, including an external-header failure's bundled fallback.
fn supplier_sources(
    out: &Path,
    root: &Path,
    workspace: &toml::Value,
) -> Result<(), Box<dyn Error>> {
    let mut arithmetic = String::from("faer-f64-nonfused-scalar-generic.v1\n");
    for name in ["faer", "faer-traits"] {
        let declared = workspace["patch"]["crates-io"][name]["path"]
            .as_str()
            .ok_or("missing selected arithmetic source path")?;
        if root.join(declared).canonicalize()? != root.join("vendor").join(name).canonicalize()? {
            return Err(
                "selected arithmetic source has no checked maintained-source receipt".into(),
            );
        }
        arithmetic.push_str(&build_source::authenticated(root, name)?);
        arithmetic.push('\n');
    }
    build_receipt::write_if_changed(&out.join("arithmetic-source.txt"), arithmetic.as_bytes())?;
    let selected = env::var_os("CARGO_FEATURE_KINSOL").is_some()
        || env::var_os("CARGO_FEATURE_IDAS").is_some();
    let source = if selected {
        let actual = PathBuf::from(
            env::var_os("DEP_SUNDIALS_SOURCE_ROOT")
                .ok_or("missing selected SUNDIALS wrapper source")?,
        )
        .canonicalize()?;
        if actual != root.join("vendor/sundials-sys").canonicalize()? {
            return Err(
                "selected SUNDIALS wrapper has no checked maintained-source receipt".into(),
            );
        }
        println!("cargo:rerun-if-env-changed=PSE_SUNDIALS_PROVIDER_RECEIPT");
        let receipt = env::var_os("PSE_SUNDIALS_PROVIDER_RECEIPT").map(PathBuf::from);
        build_source::selected_sundials(
            root,
            &env::var("DEP_SUNDIALS_SOURCE_KIND")?,
            receipt.as_deref(),
        )?
    } else {
        String::new()
    };
    build_receipt::write_if_changed(&out.join("sundials-source.txt"), source.as_bytes())?;
    // Diffsol is consumed by the pure callback/linearization contracts as well as
    // the linked dynamics adapter. Its maintained changes must never be invisible.
    let mut diffsol = String::new();
    for name in ["diffsol", "diffsol-la"] {
        let declared = workspace["workspace"]["dependencies"][name]["path"]
            .as_str()
            .ok_or("selected Diffsol source has no maintained path")?;
        if root.join(declared).canonicalize()? != root.join("vendor").join(name).canonicalize()? {
            return Err("selected Diffsol source has no checked maintained-source receipt".into());
        }
    }
    for name in ["diffsol", "diffsol-la", "diffsol-nl"] {
        diffsol.push_str(&build_source::authenticated(root, name)?);
        diffsol.push('\n');
    }
    diffsol.push_str(&workspace["workspace"]["dependencies"]["diffsol"].to_string());
    build_receipt::write_if_changed(&out.join("diffsol-source.txt"), diffsol.as_bytes())?;
    Ok(())
}

/// Capture the admitted KLU provider for every linked consumer, including IDAS-only
/// and Diffsol configurations without SUNDIALS.
fn kinsol_profile(out: &Path) -> Result<(), Box<dyn Error>> {
    let output = out.join("kinsol-klu-provider.json");
    if env::var_os("CARGO_FEATURE_KINSOL").is_none()
        && env::var_os("CARGO_FEATURE_IDAS").is_none()
        && env::var_os("CARGO_FEATURE_DIFFSOL").is_none()
    {
        build_receipt::write_if_changed(&output, b"")?;
        return Ok(());
    }
    for key in [
        "SUITESPARSE_INCLUDE_DIR",
        "SUITESPARSE_LIBRARY_DIR",
        "DEP_SUITESPARSE_SUITESPARSE_INCLUDE",
        "DEP_SUITESPARSE_SUITESPARSE_LIB",
        "PSE_KLU_PROVIDER_RECEIPT",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }
    let directory = |explicit, metadata| {
        env::var_os(explicit)
            .or_else(|| env::var_os(metadata))
            .map(PathBuf::from)
            .and_then(|path| path.canonicalize().ok())
    };
    let include = directory(
        "SUITESPARSE_INCLUDE_DIR",
        "DEP_SUITESPARSE_SUITESPARSE_INCLUDE",
    );
    let library = directory("SUITESPARSE_LIBRARY_DIR", "DEP_SUITESPARSE_SUITESPARSE_LIB");
    let receipt = include
        .zip(library)
        .and_then(|(include, library)| {
            let prefix = include.parent()?.parent()?;
            if include != prefix.join("include/suitesparse") || library != prefix.join("lib") {
                return None;
            }
            Some(prefix.join(".complete.json"))
        })
        .filter(|receipt| {
            // Mutable external prefixes are admitted by current file observations, not
            // any old receipt they happen to contain. Only the existing qualifier may
            // select a managed provider receipt for portable-child profile evidence.
            env::var_os("PSE_KLU_PROVIDER_RECEIPT")
                .filter(|value| !value.is_empty())
                .and_then(|path| PathBuf::from(path).canonicalize().ok())
                .zip(receipt.canonicalize().ok())
                .is_some_and(|(admitted, actual)| admitted == actual)
        });
    if let Some(receipt) = receipt {
        println!("cargo:rerun-if-changed={}", receipt.display());
        if receipt.is_file() {
            build_receipt::materialize(&receipt, &output)?;
            return Ok(());
        }
    }
    // Unavailable profile evidence is explicit. Ordinary direct native calls retain
    // their API contract; authenticated portable-child execution refuses this state.
    build_receipt::write_if_changed(&output, b"")?;
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
    build_receipt::materialize(&manifest, &out.join("root-isolation-manifest.json"))?;
    let pc = prefix.join("share/pkgconfig/ibex.pc");
    println!("cargo:rerun-if-changed={}", pc.display());
    // The transport consumes installed headers and these static providers. A
    // receipt path alone cannot notice relevant bytes replaced at the same prefix.
    for input in [
        "include",
        "lib/libibex.a",
        "lib/ibex/3rd/libprim.a",
        "lib/ibex/3rd/libsoplex.a",
    ] {
        println!("cargo:rerun-if-changed={}", prefix.join(input).display());
    }
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
/// links in the described order, searched in the image library directory. Dynamic
/// retention scopes are translated to transitive Rust native-library attributes;
/// discarding the provider's no-as-needed flag can drop MKL's core/threading layers.
fn link_from_pc(pc: &Path, libdir: &Path, linkage: &str) -> Result<String, Box<dyn Error>> {
    let text = fs::read_to_string(pc)
        .map_err(|e| format!("pse-backend-native: unreadable {}: {e}", pc.display()))?;
    let libs = text
        .lines()
        .find_map(|l| l.strip_prefix("Libs:"))
        .ok_or_else(|| format!("{} has no Libs line", pc.display()))?;
    println!("cargo:rustc-link-search=native={}", libdir.display());
    let libraries = build_link::libraries(libs, linkage)?;
    if libraries.is_empty() {
        return Err(format!("{} names no libraries", pc.display()).into());
    }
    if linkage == "dylib" {
        return Ok(build_link::attributes(&libraries)?);
    }
    for library in libraries {
        println!("cargo:rustc-link-lib={library}");
    }
    Ok(String::new())
}
