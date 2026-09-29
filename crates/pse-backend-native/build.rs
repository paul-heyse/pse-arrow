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

const MKL: &str = "mkl-dynamic-lp64-gomp";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-env-changed=IPOPT_DIR");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo:rerun-if-changed=build.rs");
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR unset")?);
    let ipopt = env::var_os("CARGO_FEATURE_IPOPT").is_some();
    let sdp = env::var_os("CARGO_FEATURE_SDP").is_some();
    let pardiso = env::var_os("CARGO_FEATURE_CLARABEL_PARDISO").is_some();
    let prefix = env::var_os("IPOPT_DIR").map(PathBuf::from);
    let mut manifest = String::new();
    if ipopt || sdp || pardiso {
        match &prefix {
            Some(prefix) => {
                let pc = prefix.join("lib/pkgconfig").join(format!("{MKL}.pc"));
                println!("cargo:rerun-if-changed={}", pc.display());
                link_from_pc(&pc, &prefix.join("lib"))?;
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
    Ok(())
}

/// Emit the `Libs:` line of the image's MKL link description: its `-l` libraries as dynamic
/// links in the described order, searched in the image library directory. The interface
/// layer comes first, so under the linker's `--as-needed` the threading and core layers stay
/// linked for the symbols it leaves undefined.
fn link_from_pc(pc: &Path, libdir: &Path) -> Result<(), Box<dyn Error>> {
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
            println!("cargo:rustc-link-lib=dylib={name}");
            linked += 1;
        }
    }
    if linked == 0 {
        return Err(format!("{} names no libraries", pc.display()).into());
    }
    Ok(())
}
