// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "scalar oneMKL and OpenMP thread controls, dynamic-linker queries with caller-owned storage, and the one environment write that points pardiso-wrapper's loader at the linked oneMKL"
)]
//! The process's one oneMKL (LP64, GNU threading) and GNU OpenMP runtime, linked by the
//! build script from the solver image's `mkl-dynamic-lp64-gomp` description (ADR-0108):
//! native thread counts scoped to the owning worker, and the loading of Clarabel's MKL
//! Pardiso KKT solver from that same library.
use crate::ProblemError;
use std::ffi::c_int;
#[cfg(any(test, feature = "sdp"))]
use std::{
    collections::BTreeSet,
    ffi::{CStr, c_char, c_void},
};

unsafe extern "C" {
    fn MKL_Set_Num_Threads_Local(threads: c_int) -> c_int;
    fn omp_get_max_threads() -> c_int;
    fn omp_set_num_threads(threads: c_int);
    #[cfg(any(test, feature = "sdp"))]
    fn dladdr(address: *const c_void, info: *mut DlInfo) -> c_int;
}

#[cfg(any(test, feature = "sdp"))]
#[repr(C)]
struct DlInfo {
    fname: *const c_char,
    fbase: *mut c_void,
    sname: *const c_char,
    saddr: *mut c_void,
}
/// The file that defines the code at `address`, by the dynamic linker.
#[cfg(any(test, feature = "sdp"))]
pub(crate) fn object_of(address: *const c_void) -> Option<String> {
    let mut info = DlInfo {
        fname: std::ptr::null(),
        fbase: std::ptr::null_mut(),
        sname: std::ptr::null(),
        saddr: std::ptr::null_mut(),
    };
    // SAFETY: `info` is caller-owned and `dladdr` only reads the address.
    let found = unsafe { dladdr(address, &mut info) } != 0 && !info.fname.is_null();
    found.then(|| {
        // SAFETY: a successful lookup returns a non-null, NUL-terminated path owned by
        // the loader for as long as the object stays mapped; it is copied at once.
        unsafe { CStr::from_ptr(info.fname) }
            .to_string_lossy()
            .into_owned()
    })
}
/// Shared objects mapped into this process, by file name.
#[cfg(any(test, feature = "sdp"))]
pub(crate) fn mapped_libraries() -> BTreeSet<String> {
    std::fs::read_to_string("/proc/self/maps")
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.split_whitespace().nth(5))
        .filter_map(|p| p.rsplit('/').next())
        .filter(|n| n.contains(".so"))
        .map(str::to_owned)
        .collect()
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
        // SAFETY: a scalar query of the calling worker's OpenMP thread-count ICV.
        let omp = unsafe { omp_get_max_threads() };
        // SAFETY: sets the calling worker's own ICV to a positive count.
        unsafe { omp_set_num_threads(n) };
        // SAFETY: sets the calling thread's MKL-local count to a positive value and
        // returns the previous one.
        let mkl = unsafe { MKL_Set_Num_Threads_Local(n) };
        Ok(Self { omp, mkl })
    }
}
impl Drop for Threads {
    fn drop(&mut self) {
        // SAFETY: restores the calling worker's own previous OpenMP ICV.
        unsafe { omp_set_num_threads(self.omp) };
        // SAFETY: restores the calling thread's previous MKL-local count (0 clears it).
        unsafe { MKL_Set_Num_Threads_Local(self.mkl) };
    }
}

#[cfg(feature = "clarabel-pardiso")]
pub(crate) use pardiso::pardiso;
#[cfg(feature = "clarabel-pardiso")]
mod pardiso {
    //! Clarabel's `pardiso-mkl` KKT solver goes through pardiso-wrapper, which loads a
    //! file named `libmkl_rt.so` at run time, searching `LD_LIBRARY_PATH`, `$MKLROOT/lib`,
    //! `$MKLROOT`, `$MKL_PARDISO_PATH`, `/opt/intel/oneapi/mkl/latest/lib` and `./` in that
    //! order. The solver image ships no `libmkl_rt` (ADR-0108): its interface library
    //! `libmkl_intel_lp64` exports every symbol pardiso-wrapper resolves (`pardiso_`,
    //! `pardisoinit_` and the thread controls). The loader is therefore pointed at an alias
    //! of that already-linked library, and loading it must return the object already in
    //! the process: never a second oneMKL or another OpenMP runtime.
    use super::{mapped_libraries, object_of};
    use crate::ProblemError;
    use std::{
        ffi::{CString, c_char, c_int, c_void},
        path::{Path, PathBuf},
        sync::OnceLock,
    };
    unsafe extern "C" {
        fn pardiso_();
        fn dlopen(file: *const c_char, flags: c_int) -> *mut c_void;
        fn dlclose(handle: *mut c_void) -> c_int;
    }
    const RTLD_LAZY: c_int = 0x1;
    const RTLD_NOLOAD: c_int = 0x4;
    const LIBRARY: &str = "libmkl_rt.so";
    const VARIABLE: &str = "MKL_PARDISO_PATH";

    /// Resolve Clarabel's MKL Pardiso to the process's linked oneMKL, once per process.
    ///
    /// # Errors
    /// The loader would find another oneMKL, or loading did not return the linked one.
    pub(crate) fn pardiso() -> Result<(), ProblemError> {
        static LOADED: OnceLock<Result<(), String>> = OnceLock::new();
        LOADED
            .get_or_init(load)
            .clone()
            .map_err(|reason| ProblemError::Unsupported(format!("MKL Pardiso: {reason}")))
    }
    /// The directories pardiso-wrapper 0.1.3 (`mkl::loader::get_mkl_lib_path`) searches.
    fn searched() -> Vec<PathBuf> {
        let variable = |name| std::env::var_os(name).unwrap_or_default();
        let mut out: Vec<PathBuf> = std::env::split_paths(&variable("LD_LIBRARY_PATH")).collect();
        let root = PathBuf::from(variable("MKLROOT"));
        if !root.as_os_str().is_empty() {
            out.push(root.join("lib"));
            out.push(root);
        }
        out.push(PathBuf::from(variable(VARIABLE)));
        out.push(PathBuf::from("/opt/intel/oneapi/mkl/latest/lib"));
        out.push(PathBuf::from("./"));
        out.into_iter()
            .filter(|p| !p.as_os_str().is_empty())
            .collect()
    }
    fn same(a: &Path, b: &Path) -> bool {
        a.canonicalize()
            .ok()
            .zip(b.canonicalize().ok())
            .is_some_and(|(a, b)| a == b)
    }
    /// A loader handle for `path` only if it is already loaded; closed again at once.
    fn loaded(path: &Path) -> Result<usize, String> {
        let name = CString::new(path.as_os_str().as_encoded_bytes())
            .map_err(|_| format!("{} is not a C path", path.display()))?;
        // SAFETY: RTLD_NOLOAD never maps or initializes an object; it only returns (and
        // references) one already loaded, which the matching `dlclose` releases.
        let handle = unsafe { dlopen(name.as_ptr(), RTLD_LAZY | RTLD_NOLOAD) };
        if handle.is_null() {
            return Err(format!("{} is not loaded", path.display()));
        }
        // SAFETY: releases the reference taken above.
        unsafe { dlclose(handle) };
        Ok(handle as usize)
    }
    fn load() -> Result<(), String> {
        let interface = PathBuf::from(
            object_of(pardiso_ as *const c_void)
                .ok_or("the linked oneMKL does not define pardiso_")?,
        );
        let name = interface
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if !name.starts_with("libmkl_intel_lp64.so") {
            return Err(format!(
                "pardiso_ resolves into {}, not the linked LP64 interface library",
                interface.display()
            ));
        }
        // The first library the loader would find must be the linked one; with none, an
        // alias of the linked one is published through the loader's own search variable.
        let found = searched()
            .into_iter()
            .map(|d| d.join(LIBRARY))
            .find(|p| p.is_file());
        let alias = match found {
            Some(path) if same(&path, &interface) => path,
            Some(path) => {
                return Err(format!(
                    "the loader would load {}, a second oneMKL runtime beside {}",
                    path.display(),
                    interface.display()
                ));
            }
            None if std::env::var_os(VARIABLE).is_some() => {
                return Err(format!("{VARIABLE} names no directory holding {LIBRARY}"));
            }
            None => {
                let directory = std::env::temp_dir()
                    .join("pse-mkl-pardiso")
                    .join(interface.to_string_lossy().replace('/', "_"));
                std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
                let alias = directory.join(LIBRARY);
                match std::os::unix::fs::symlink(&interface, &alias) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(e) => return Err(e.to_string()),
                }
                if !same(&alias, &interface) {
                    return Err(format!(
                        "{} does not resolve to {}",
                        alias.display(),
                        interface.display()
                    ));
                }
                // SAFETY: the one environment write of this process for pardiso-wrapper,
                // made once under `OnceLock` before its loader first reads the variable.
                // Rust's environment access is serialized by std; the linked native
                // runtimes (oneMKL, libgomp, Ipopt, HiGHS, SCIP) read their environment
                // when they initialize, which precedes any MKL Pardiso admission.
                unsafe { std::env::set_var(VARIABLE, &directory) };
                alias
            }
        };
        let validation = clarabel::solver::DefaultSettings::<f64> {
            direct_solve_method: "mkl".into(),
            ..Default::default()
        };
        clarabel::solver::traits::Settings::validate(&validation)
            .map_err(|e| format!("pardiso-wrapper could not load {}: {e}", alias.display()))?;
        // Loading the alias returned the object already in the process.
        if loaded(&alias)? != loaded(&interface)? {
            return Err(format!(
                "{} loaded as a second object beside {}",
                alias.display(),
                interface.display()
            ));
        }
        let libraries = mapped_libraries();
        if let Some(foreign) = libraries
            .iter()
            .find(|l| l.starts_with("libmkl_rt.") || l.starts_with("libiomp5"))
        {
            return Err(format!("loading mapped a second runtime {foreign}"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn worker_threads_are_scoped_and_restored() {
        // SAFETY: a scalar query of this test worker's own OpenMP ICV.
        let omp = unsafe { omp_get_max_threads() };
        // SAFETY: clears this test thread's MKL-local count and returns the previous one.
        let before = (omp, unsafe { MKL_Set_Num_Threads_Local(0) });
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
