// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owned, offline headers for the fixed native packed profile.
use crate::MathError;
use std::path::{Component, Path, PathBuf};
include!(concat!(env!("OUT_DIR"), "/xsimd-header-data.rs"));

pub(super) const fn fingerprint() -> &'static [u8; 32] {
    &FINGERPRINT
}
pub(super) const fn version() -> &'static str {
    VERSION
}

/// Complete fixed archive/files/name backing and bounded decoder/path overlap.
/// The native module owner reserves this amount before calling `prepare`.
pub(super) fn required_backing_bytes(directory: &Path) -> Result<usize, MathError> {
    let fail = || MathError::Limit("native packed header backing");
    let entries = FILE_COUNT.checked_add(DIRECTORY_COUNT).ok_or_else(fail)?;
    let prefix = directory
        .as_os_str()
        .len()
        .checked_add(1)
        .ok_or_else(fail)?;
    let prefixes = entries.checked_mul(prefix).ok_or_else(fail)?;
    let paths = MAX_NAME.checked_mul(4).ok_or_else(fail)?;
    ARCHIVE
        .len()
        .checked_add(FILE_BYTES)
        .and_then(|n| n.checked_add(NAME_BYTES))
        .and_then(|n| n.checked_add(prefixes))
        .and_then(|n| n.checked_add(128 << 10))
        .and_then(|n| n.checked_add(paths))
        .ok_or_else(fail)
}

/// Extract only authenticated regular headers/license into the caller's module directory.
/// That owner retains the directory until every compiler process and module has drained.
pub(super) fn prepare(
    directory: &Path,
    scope: &pse_kernels::ExecutionScope,
    allowance: usize,
) -> Result<PathBuf, MathError> {
    scope.check().map_err(crate::error::scope_error)?;
    if required_backing_bytes(directory)? > allowance {
        return Err(MathError::Limit("native packed header backing"));
    }
    let root = directory.join(ROOT);
    if root.try_exists().map_err(library)? {
        return Err(MathError::Contract(
            "packed header directory already exists".into(),
        ));
    }
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(ARCHIVE));
    let mut count = 0usize;
    let mut bytes = 0usize;
    for entry in archive.entries().map_err(library)? {
        scope.check().map_err(crate::error::scope_error)?;
        let mut entry = entry.map_err(library)?;
        if entry.header().entry_type().is_pax_global_extensions()
            || entry.header().entry_type().is_pax_local_extensions()
        {
            continue;
        }
        let selected = {
            let path = entry.path().map_err(library)?;
            if path
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
            {
                return Err(MathError::Contract("unsafe packed header path".into()));
            }
            let relative = path.strip_prefix(ROOT).map_err(library)?;
            relative.starts_with("include") || relative == Path::new("LICENSE")
        };
        if !selected || entry.header().entry_type().is_dir() {
            continue;
        }
        if !entry.header().entry_type().is_file() {
            return Err(MathError::Contract(
                "packed header is not a regular file".into(),
            ));
        }
        count = count
            .checked_add(1)
            .ok_or(MathError::Limit("packed header count"))?;
        bytes = bytes
            .checked_add(usize::try_from(entry.size()).map_err(library)?)
            .ok_or(MathError::Limit("packed header file bytes"))?;
        if count > FILE_COUNT || bytes > FILE_BYTES {
            return Err(MathError::Contract(
                "packed header archive extent differs".into(),
            ));
        }
        if !entry.unpack_in(directory).map_err(library)? {
            return Err(MathError::Contract(
                "packed header extraction refused".into(),
            ));
        }
    }
    if count != FILE_COUNT || bytes != FILE_BYTES {
        return Err(MathError::Contract(
            "packed header archive extent differs".into(),
        ));
    }
    scope.check().map_err(crate::error::scope_error)?;
    Ok(root.join("include"))
}
fn library(error: impl std::fmt::Display) -> MathError {
    MathError::Library(format!("packed headers ({SOURCE}): {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_packed_headers_admit_before_effects_and_retain_original_scope() {
        let directory = tempfile::tempdir().unwrap();
        let scope = pse_kernels::ExecutionScope::new(Default::default(), None);
        let backing = required_backing_bytes(directory.path()).unwrap();
        assert!(
            backing > FILE_BYTES
                && !version().is_empty()
                && fingerprint().iter().any(|byte| *byte != 0)
        );
        assert!(matches!(
            prepare(directory.path(), &scope, backing - 1),
            Err(MathError::Limit("native packed header backing"))
        ));
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
        let canceled =
            pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::new(true), None);
        assert!(prepare(directory.path(), &canceled, backing).is_err());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
        let include = prepare(directory.path(), &scope, backing).unwrap();
        assert!(include.join("xsimd/xsimd.hpp").is_file());
        assert!(directory.path().join(ROOT).join("LICENSE").is_file());
        assert!(prepare(directory.path(), &scope, backing).is_err());
    }
    #[test]
    fn packed_stable_heads_match_independent_long_double_values_and_partials() {
        use symbolica::atom::{FunctionBuilder, Symbol};
        use symbolica::evaluate::{CppProfile, ExportSettings, InlineASM};
        crate::initialize().unwrap();
        let x = crate::library::formal(0).unwrap();
        let y = crate::library::formal(1).unwrap();
        let heads = crate::stable::cpp_heads().unwrap();
        let mut expressions = heads
            .iter()
            .map(|head| {
                let mut call = FunctionBuilder::new(head.symbol).add_arg(&x);
                if head.arity == 2 {
                    call = call.add_arg(&y);
                }
                call.finish()
            })
            .collect::<Vec<_>>();
        expressions.push(FunctionBuilder::new(Symbol::LOG).add_arg(&x).finish());
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut cache = crate::library::ScalarCache::default();
        let evaluator = crate::library::bounded_scalar_evaluator(
            &expressions,
            &[x, y],
            crate::library::Optimization::default(),
            &cancel,
            crate::jets::EvaluationLimits::default(),
            &mut cache,
        )
        .unwrap();
        let source = evaluator
            .evaluator
            .try_export_cpp_with(
                "stable_probe",
                ExportSettings::new().inline_asm(InlineASM::None),
                CppProfile::X4F64Avx2,
                heads,
                &mut |_| Ok::<_, MathError>(()),
            )
            .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let scope = pse_kernels::ExecutionScope::new(Default::default(), None);
        let include = prepare(
            directory.path(),
            &scope,
            required_backing_bytes(directory.path()).unwrap(),
        )
        .unwrap();
        let cpp = directory.path().join("probe.cpp");
        let binary = directory.path().join("probe");
        let driver = r#"
#include <cstdio>
#include <memory>
#include <limits>
long double expected(unsigned int tag,double x,double y) {
    const long double nan=std::numeric_limits<long double>::quiet_NaN();
    long double a=x,b=y;
    if(tag==17)return std::log(a);
    if(!std::isfinite(x)||(tag>3&&!std::isfinite(y)))return nan;
    if(tag<=3){if(x<=-1.0)return nan;if(tag==1)return std::log1p(a);if(tag==2)return 1.0L/(1.0L+a);return -1.0L/((1.0L+a)*(1.0L+a));}
    // Neighboring 300 K uses Decimal(100): even an 80-bit ratio loses increment digits.
    if(tag>=11){if(x<=0.0||y<=0.0)return nan;switch(tag){case 11:if(x==std::nextafter(300.0,301.0)&&y==300.0)return 1.8947806286936003e-16L;return std::log(a/b);case 12:return 1.0L/a;case 13:return -1.0L/b;case 14:return -1.0L/(a*a);case 15:return 0.0L;default:return 1.0L/(b*b);}}
    long double h=std::hypot(a,b);
    if(tag==4)return h;
    if(tag==10)return h==0.0L?0.0L:1.0L;
    if(h==0.0L)return nan;
    switch(tag){case 5:return a/h;case 6:return b/h;case 7:return (b/h)*(b/h)/h;case 8:return -(a/h)*(b/h)/h;default:return (a/h)*(a/h)/h;}
}
int main(){
    const double tiny=std::numeric_limits<double>::denorm_min(),large=std::numeric_limits<double>::max();
    const double xs[][4]={{1e-30,-1e-30,0.0,-0.0},{3.0,1e-320,large,tiny},{std::nextafter(300.0,301.0),1.0,large,tiny},{-1.0,-2.0,INFINITY,NAN},{-1.0+std::numeric_limits<double>::epsilon()/2,3e200,1e160,1e20}};
    const double ys[][4]={{1.0,1.0,0.0,1.0},{4.0,1e-160,large,0.0},{300.0,1e20,tiny,large},{1.0,4.0,2.0,3.0},{2.0,4e200,1.0,1e20}};
    std::unique_ptr<simd[]> stack(new simd[stable_probe_simd_realf64_get_buffer_len()]);
    for(unsigned int row=0;row<5;++row){
        simd input[2]={simd::load_unaligned(xs[row]),simd::load_unaligned(ys[row])};simd output[17];
        stable_probe_simd_realf64(input,stack.get(),output);
        for(unsigned int column=0;column<17;++column){
            alignas(32)double actual[4];output[column].store_aligned(actual);
            for(unsigned int lane=0;lane<4;++lane){
                double reference=static_cast<double>(expected(column+1,xs[row][lane],ys[row][lane]));
                bool good=std::isnan(reference)?std::isnan(actual[lane]):std::isinf(reference)?actual[lane]==reference:reference==0.0?actual[lane]==0.0:std::isfinite(actual[lane])&&std::abs(actual[lane]-reference)<=8*std::numeric_limits<double>::epsilon()*std::abs(reference)+4*tiny;
                if(column==0&&xs[row][lane]==0.0)good=good&&(std::signbit(actual[lane])==std::signbit(xs[row][lane]));
                if(!good){std::fprintf(stderr,"packed head %u row %u lane %u: %.17g != %.17g\n",column+1,row,lane,actual[lane],reference);return 1;}
            }
        }
    }
}
"#;
        std::fs::write(&cpp, format!("{}\n{driver}", source.source)).unwrap();
        let mut compiler = std::process::Command::new(super::super::COMPILER);
        compiler
            .args(super::super::FLAGS)
            .args(["-mavx2", "-mno-fma", "-I"])
            .arg(include)
            .arg(&cpp)
            .arg("-o")
            .arg(&binary);
        super::super::run_compiler(compiler, &scope).unwrap();
        super::super::run_compiler(std::process::Command::new(binary), &scope).unwrap();
    }
}
