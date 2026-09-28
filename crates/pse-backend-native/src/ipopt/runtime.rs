// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "scalar oneMKL, OpenMP and Ipopt runtime queries with caller-owned storage"
)]
//! The linked runtime of the Ipopt profile (ADR-0108): which linear solvers the library was
//! built with, the OpenMP and oneMKL state SPRAL and Pardiso depend on, native thread counts
//! scoped to the owning worker, and the build identity recorded in every profile key.
use crate::ProblemError;
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
    fn MKL_Set_Num_Threads_Local(threads: c_int) -> c_int;
    fn omp_get_cancellation() -> c_int;
    fn omp_get_max_threads() -> c_int;
    fn omp_get_proc_bind() -> c_int;
    fn omp_set_num_threads(threads: c_int);
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

/// Process state that linear-solver admission depends on, observed on the calling thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Runtime {
    /// Linear solvers the linked Ipopt was built with (`IpoptGetAvailableLinearSolvers`);
    /// the runtime library loader is excluded.
    pub linked: u32,
    /// OpenMP cancellation (`OMP_CANCELLATION`), fixed when the OpenMP runtime started.
    pub cancellation: bool,
    /// OpenMP thread-binding policy (`OMP_PROC_BIND`); `0` is unbound.
    pub proc_bind: i32,
    /// oneMKL conditional-numerical-reproducibility branch in force.
    pub cbwr: i32,
    /// oneMKL dynamic thread adjustment (`MKL_DYNAMIC`).
    pub mkl_dynamic: bool,
}
impl Runtime {
    /// Observe the linked library and process state now.
    pub fn observe() -> Self {
        // SAFETY: argument-only queries of linked native runtimes; none retains state.
        unsafe {
            Self {
                linked: pse_ipopt_sys::IpoptGetAvailableLinearSolvers(1),
                cancellation: omp_get_cancellation() != 0,
                proc_bind: omp_get_proc_bind(),
                cbwr: MKL_CBWR_Get(MKL_CBWR_BRANCH) & !MKL_CBWR_STRICT,
                mkl_dynamic: MKL_Get_Dynamic() != 0,
            }
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
        // SAFETY: each call writes only caller-owned storage of the stated length.
        unsafe {
            pse_ipopt_sys::GetIpoptVersion(&mut major, &mut minor, &mut release);
            MKL_Get_Version_String(mkl.as_mut_ptr(), mkl.len() as c_int);
        }
        mkl[mkl.len() - 1] = 0;
        // SAFETY: the buffer is NUL-terminated above.
        let mkl = unsafe { CStr::from_ptr(mkl.as_ptr()) }
            .to_string_lossy()
            .trim()
            .to_owned();
        let runtime = Runtime::observe();
        let ipopt = format!("Ipopt {major}.{minor}.{release}");
        let mut h = FramedHasher::new("pse.native.ipopt.build.v1");
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

/// OpenMP and oneMKL thread counts of the owning worker for one solve, restored on drop
/// (ADR-0108 item 12). SPRAL uses OpenMP threads and Pardiso MKL threads; MUMPS runs at one.
#[derive(Debug)]
pub(crate) struct Threads {
    omp: c_int,
    mkl: c_int,
}
impl Threads {
    pub(crate) fn enter(threads: usize) -> Result<Self, ProblemError> {
        let n = c_int::try_from(threads)
            .ok()
            .filter(|n| *n > 0)
            .ok_or_else(|| ProblemError::Contract("native thread count".into()))?;
        // SAFETY: per-thread ICV and MKL-local settings of the calling worker.
        unsafe {
            let omp = omp_get_max_threads();
            omp_set_num_threads(n);
            let mkl = MKL_Set_Num_Threads_Local(n);
            Ok(Self { omp, mkl })
        }
    }
}
impl Drop for Threads {
    fn drop(&mut self) {
        // SAFETY: restores the calling worker's own previous settings.
        unsafe {
            omp_set_num_threads(self.omp);
            MKL_Set_Num_Threads_Local(self.mkl);
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::ffi::c_void;

    #[repr(C)]
    struct DlInfo {
        fname: *const c_char,
        fbase: *mut c_void,
        sname: *const c_char,
        saddr: *mut c_void,
    }
    unsafe extern "C" {
        fn dladdr(address: *const c_void, info: *mut DlInfo) -> c_int;
    }
    #[cfg(feature = "sdp")]
    unsafe extern "C" {
        fn dgemm_();
    }
    /// The file that defines the code at `address`, by the dynamic linker.
    fn object_of(address: *const c_void) -> Option<String> {
        let mut info = DlInfo {
            fname: std::ptr::null(),
            fbase: std::ptr::null_mut(),
            sname: std::ptr::null(),
            saddr: std::ptr::null_mut(),
        };
        // SAFETY: `info` is caller-owned and `dladdr` only reads the address.
        let found = unsafe { dladdr(address, &mut info) } != 0 && !info.fname.is_null();
        // SAFETY: a successful lookup returns a NUL-terminated loader-owned path.
        found.then(|| {
            unsafe { CStr::from_ptr(info.fname) }
                .to_string_lossy()
                .into_owned()
        })
    }
    /// Shared objects mapped into this process, by file name.
    pub(crate) fn mapped_libraries() -> std::collections::BTreeSet<String> {
        std::fs::read_to_string("/proc/self/maps")
            .unwrap()
            .lines()
            .filter_map(|l| l.split_whitespace().nth(5))
            .filter_map(|p| p.rsplit('/').next())
            .filter(|n| n.contains(".so"))
            .map(str::to_owned)
            .collect()
    }
    /// One BLAS/LAPACK provider (oneMKL, LP64, GNU threading) and one OpenMP runtime
    /// (libgomp) are loaded after an Ipopt solve, and the Fortran BLAS symbols every other
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
    #[test]
    fn worker_threads_are_scoped_and_restored() {
        // SAFETY: per-thread queries of this test's own worker.
        let before = unsafe { (omp_get_max_threads(), MKL_Set_Num_Threads_Local(0)) };
        // SAFETY: restore the MKL-local value the probe above cleared.
        unsafe { MKL_Set_Num_Threads_Local(before.1) };
        {
            let _scope = Threads::enter(3).unwrap();
            // SAFETY: as above.
            assert_eq!(unsafe { omp_get_max_threads() }, 3);
        }
        // SAFETY: as above.
        assert_eq!(unsafe { omp_get_max_threads() }, before.0);
        assert!(Threads::enter(0).is_err());
    }
}
