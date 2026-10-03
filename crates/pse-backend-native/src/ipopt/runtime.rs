// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "scalar oneMKL, OpenMP and Ipopt runtime queries with caller-owned storage"
)]
//! The linked runtime of the Ipopt profile (ADR-0108): which linear solvers the library was
//! built with, the OpenMP and oneMKL state SPRAL and Pardiso depend on, and the build
//! identity recorded in every profile key. Native thread counts are scoped by `crate::mkl`.
use pse_ids::{ContentHash, FramedHasher};
use std::{
    ffi::{CStr, c_char, c_int},
    sync::OnceLock,
};

// oneMKL service layer (`mkl_service.h`) and the GNU OpenMP runtime, both linked by the
// build script from the solver image's `mkl-dynamic-lp64-gomp` description.
unsafe extern "C" {
    fn MKL_CBWR_Get(option: c_int) -> c_int;
    fn MKL_Get_Dynamic() -> c_int;
    fn MKL_Get_Version_String(buffer: *mut c_char, len: c_int);
    fn omp_get_cancellation() -> c_int;
    fn omp_get_proc_bind() -> c_int;
}
/// `MKL_CBWR_BRANCH` (`mkl_types.h`): query the branch in force.
const MKL_CBWR_BRANCH: c_int = 1;
/// `MKL_CBWR_STRICT` flag, which qualifies but does not change a branch.
const MKL_CBWR_STRICT: c_int = 0x10000;
/// `MKL_CBWR_COMPATIBLE`, the image's pinned branch (execution decision 9).
const MKL_CBWR_COMPATIBLE: c_int = 3;
/// `omp_proc_bind_false`.
pub(crate) const OMP_PROC_BIND_FALSE: i32 = 0;
/// oneMKL CBWR branch spellings (`mkl_types.h`).
const CBWR: [(&str, i32); 10] = [
    ("OFF", 0),
    ("BRANCH_OFF", 1),
    ("AUTO", 2),
    ("COMPATIBLE", 3),
    ("SSE2", 4),
    ("SSE4_2", 8),
    ("AVX2", 10),
    ("AVX512", 12),
    ("AVX512_E1", 14),
    ("AVX10", 15),
];
/// The solver image manifest embedded at build time; empty outside the image.
const MANIFEST: &str = include_str!(concat!(env!("OUT_DIR"), "/solver-manifest.txt"));

pub use crate::settings::ipopt::Runtime;
impl Runtime {
    /// Observe the linked library and process state now.
    pub fn observe() -> Self {
        // SAFETY: an argument-only query of the linked Ipopt's build; it retains nothing.
        let linked = unsafe { pse_ipopt_sys::IpoptGetAvailableLinearSolvers(1) };
        // SAFETY: an argument-free query of the OpenMP runtime's fixed cancellation ICV.
        let cancellation = unsafe { omp_get_cancellation() } != 0;
        // SAFETY: an argument-free query of the calling thread's binding ICV.
        let proc_bind = unsafe { omp_get_proc_bind() };
        // SAFETY: a by-value query of oneMKL's process CBWR setting; it retains nothing.
        let cbwr = unsafe { MKL_CBWR_Get(MKL_CBWR_BRANCH) } & !MKL_CBWR_STRICT;
        // SAFETY: an argument-free query of oneMKL's dynamic-threads setting.
        let mkl_dynamic = unsafe { MKL_Get_Dynamic() } != 0;
        Self {
            linked,
            cancellation,
            proc_bind,
            cbwr,
            mkl_dynamic,
        }
    }
}
/// Spelling of a oneMKL CBWR branch code.
pub(crate) fn cbwr_name(code: i32) -> String {
    CBWR.iter()
        .find(|(_, c)| *c == code)
        .map_or_else(|| format!("code {code}"), |(n, _)| (*n).to_owned())
}
/// The CBWR branch the image pins: its manifest's `mkl_cbwr` entry, or `COMPATIBLE` (the only
/// branch honoured on every supported host) for a build outside the image.
pub(crate) fn pinned_cbwr() -> i32 {
    manifest_value("mkl_cbwr")
        .and_then(|name| CBWR.iter().find(|(n, _)| *n == name).map(|(_, c)| *c))
        .unwrap_or(MKL_CBWR_COMPATIBLE)
}
fn manifest_value(key: &str) -> Option<&'static str> {
    MANIFEST.lines().find_map(|line| {
        line.strip_prefix(key)
            .and_then(|rest| rest.strip_prefix(' '))
            .map(str::trim)
    })
}

/// The linked native build: library versions and the numerical contract of this process.
#[derive(Clone, Debug)]
pub struct Build {
    /// Ipopt version reported by the linked library.
    pub ipopt: String,
    /// oneMKL version string reported by the linked library.
    pub mkl: String,
    /// Linked linear solvers.
    pub linked: u32,
    /// CBWR branch in force when first observed.
    pub cbwr: i32,
    /// Image manifest: component versions, linear-solver inventory and the pinned branch.
    pub manifest: &'static str,
    /// Identity of every field above.
    pub identity: ContentHash,
}
/// The linked native build, observed once per process.
pub fn build() -> &'static Build {
    static BUILD: OnceLock<Build> = OnceLock::new();
    BUILD.get_or_init(|| {
        let (mut major, mut minor, mut release) = (0, 0, 0);
        let mut mkl: [c_char; 256] = [0; 256];
        // SAFETY: writes only the three caller-owned integers passed by reference.
        unsafe { pse_ipopt_sys::GetIpoptVersion(&mut major, &mut minor, &mut release) };
        // SAFETY: writes at most `mkl.len()` bytes into the caller-owned buffer.
        unsafe { MKL_Get_Version_String(mkl.as_mut_ptr(), mkl.len() as c_int) };
        mkl[mkl.len() - 1] = 0;
        // SAFETY: the buffer is NUL-terminated above.
        let mkl = unsafe { CStr::from_ptr(mkl.as_ptr()) }
            .to_string_lossy()
            .trim()
            .to_owned();
        let runtime = Runtime::observe();
        let ipopt = format!("Ipopt {major}.{minor}.{release}");
        let mut h = FramedHasher::new(pse_ids::Frame::NativeIpoptBuildV1);
        h.str(&ipopt)
            .str(&mkl)
            .u64(u64::from(runtime.linked))
            .str(&cbwr_name(runtime.cbwr))
            .str(MANIFEST);
        Build {
            ipopt,
            mkl,
            linked: runtime.linked,
            cbwr: runtime.cbwr,
            manifest: MANIFEST,
            identity: h.finish_hash(),
        }
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::mkl::{mapped_libraries, object_of};
    use std::ffi::c_void;

    #[cfg(feature = "sdp")]
    unsafe extern "C" {
        fn dgemm_();
    }
    /// One BLAS/LAPACK provider (oneMKL, LP64, GNU threading) and one OpenMP runtime
    /// (libgomp) are loaded after an Ipopt solve (and a Clarabel MKL Pardiso solve, whose
    /// runtime loader resolves the same library), and the Fortran BLAS symbols every other
    /// native component calls resolve into oneMKL (ADR-0108 item 5).
    pub(crate) fn assert_single_provider() {
        let libraries = mapped_libraries();
        let has = |prefix: &str| libraries.iter().any(|l| l.starts_with(prefix));
        for required in [
            "libmkl_intel_lp64.so",
            "libmkl_gnu_thread.so",
            "libmkl_core.so",
            "libgomp.so",
        ] {
            assert!(has(required), "{required} not loaded: {libraries:?}");
        }
        for foreign in [
            "libblas.",
            "liblapack.",
            "libcblas.",
            "libopenblas",
            "libblis",
            "libflexiblas",
            "libatlas",
            "libsatlas",
            "libtatlas",
            "libmkl_rt.",
            "libmkl_sequential",
            "libmkl_intel_thread",
            "libmkl_tbb_thread",
            "libmkl_intel_ilp64",
            "libiomp5",
            "libomp.",
            "libomp5",
        ] {
            assert!(
                !has(foreign),
                "second provider {foreign} loaded: {libraries:?}"
            );
        }
        let service = object_of(MKL_CBWR_Get as *const c_void).unwrap();
        assert!(service.contains("libmkl_intel_lp64"), "{service}");
        #[cfg(feature = "sdp")]
        {
            let blas = object_of(dgemm_ as *const c_void).unwrap();
            assert!(
                blas.contains("libmkl_intel_lp64"),
                "dgemm_ resolves into {blas}"
            );
        }
    }
    #[test]
    fn manifest_pins_the_cbwr_branch_the_process_runs() {
        let build = build();
        assert!(build.ipopt.starts_with("Ipopt 3.14"), "{}", build.ipopt);
        assert!(
            build.mkl.contains("oneMKL") || build.mkl.contains("Math Kernel"),
            "{}",
            build.mkl
        );
        if !MANIFEST.is_empty() {
            assert_eq!(manifest_value("mkl_cbwr"), Some("COMPATIBLE"));
            assert_eq!(
                manifest_value("ipopt_linear_solvers"),
                Some("mumps spral pardisomkl")
            );
        }
        assert_eq!(pinned_cbwr(), MKL_CBWR_COMPATIBLE);
        assert_eq!(cbwr_name(3), "COMPATIBLE");
        assert_eq!(cbwr_name(99), "code 99");
    }
}
