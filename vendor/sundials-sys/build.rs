use std::{
    collections::HashSet,
    env,
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf}
};
use bindgen::{Bindings, BindgenError};

// SUNDIALS has a few non-negative constants that need to be parsed as an i32.
// This is an attempt at doing so generally.
#[derive(Debug)]
struct ParseSignedConstants;

impl bindgen::callbacks::ParseCallbacks for ParseSignedConstants {
    fn int_macro(&self, name: &str, _value: i64) -> Option<bindgen::callbacks::IntKind> {
        let prefix: String = name.chars().take_while(|c| *c != '_').collect();
        match prefix.as_ref() {
            "CV" | "IDA" | "KIN" | "SUN" => Some(bindgen::callbacks::IntKind::Int),
            _ => None,
        }
    }
}

// Ignore some macros (based on https://github.com/rust-lang/rust-bindgen/issues/687#issuecomment-1312298570)
#[derive(Debug)]
struct IgnoreMacros(HashSet<&'static str>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        use bindgen::callbacks::MacroParsingBehavior;
        if self.0.contains(name) {
            MacroParsingBehavior::Ignore
        } else {
            MacroParsingBehavior::Default
        }
    }
}

impl IgnoreMacros {
    const IGNORE_CONSTANTS: [&'static str; 19] = [
        "FE_DIVBYZERO",
        "FE_DOWNWARD",
        "FE_INEXACT",
        "FE_INVALID",
        "FE_OVERFLOW",
        "FE_TONEAREST",
        "FE_TOWARDZERO",
        "FE_UNDERFLOW",
        "FE_UPWARD",
        "FP_INFINITE",
        "FP_INT_DOWNWARD",
        "FP_INT_TONEAREST",
        "FP_INT_TONEARESTFROMZERO",
        "FP_INT_TOWARDZERO",
        "FP_INT_UPWARD",
        "FP_NAN",
        "FP_NORMAL",
        "FP_SUBNORMAL",
        "FP_ZERO",
    ];

    fn new() -> Self {
        Self(Self::IGNORE_CONSTANTS.iter().copied().collect())
    }
}

#[derive(Debug)]
struct Library {
    /// Location if the include files.
    inc: Option<String>,
    /// Location of the library.
    lib: Option<String>,
    /// Headers produced by this build's CMake invocation, rather than a provider.
    generated_include: Option<PathBuf>,
}

#[derive(Debug)]
struct CargoInputs {
    generated_include: Option<PathBuf>,
}

impl bindgen::callbacks::ParseCallbacks for CargoInputs {
    fn include_file(&self, filename: &str) {
        let generated = self.generated_include.as_ref().is_some_and(|include| {
            let Ok(canonical_include) = include.canonicalize() else {
                return false;
            };
            let header = Path::new(filename);
            // Keep external aliases watched, even when their target is generated.
            if !header.starts_with(include) && !header.starts_with(&canonical_include) {
                return false;
            }
            // A symlink or parent component escaping the output namespace is an input.
            header
                .canonicalize()
                .is_ok_and(|path| path.starts_with(&canonical_include))
        });
        if !generated {
            bindgen::callbacks::ParseCallbacks::include_file(&bindgen::CargoCallbacks, filename);
        }
    }

    fn read_env_var(&self, key: &str) {
        bindgen::callbacks::ParseCallbacks::read_env_var(&bindgen::CargoCallbacks, key);
    }
}

/// Build the Sundials code vendor with sundials-sys.
fn build_vendor_sundials(klu: &Library) -> (Library, &'static str) {
    println!("cargo:rerun-if-changed=vendor");
    println!("cargo:source_kind=bundled");
    // SundialsSetupConfig.cmake exports CI metadata or a CMake timestamp through
    // public SUN_JOB_* macros. CMake's timestamp also consumes epoch/timezone.
    for name in ["CI_JOB_ID", "CI_JOB_STARTED_AT", "SOURCE_DATE_EPOCH", "TZ"] {
        println!("cargo:rerun-if-env-changed={}", name);
    }
    // This directory is consumed by CMake only on the bundled SUNDIALS path.
    if let Some(dir) = &klu.lib {
        println!("cargo:rerun-if-changed={}", dir);
    }
    macro_rules! feature {
        ($s:tt) => {
            if cfg!(feature = $s) {
                "ON"
            } else {
                "OFF"
            }
        };
    }

    let static_libraries = feature!("static_libraries");
    let (shared_libraries, library_type) = match static_libraries {
        "ON" => ("OFF", "static"),
        "OFF" => ("ON", "dylib"),
        _ => unreachable!(),
    };

    let mut config = cmake::Config::new("vendor");
    config
        .define("CMAKE_INSTALL_BINDIR", "lib") // v7.1.0
        .define("CMAKE_INSTALL_LIBDIR", "lib") // ≤ v7.0.0
        .define("BUILD_STATIC_LIBS", static_libraries)
        .define("BUILD_SHARED_LIBS", shared_libraries)
        .define("BUILD_TESTING", "OFF")
        .define("EXAMPLES_INSTALL", "OFF")
        .define("EXAMPLES_ENABLE_C", "OFF")
        .define("BUILD_ARKODE", feature!("arkode"))
        .define("BUILD_CVODE", feature!("cvode"))
        .define("BUILD_CVODES", feature!("cvodes"))
        .define("BUILD_IDA", feature!("ida"))
        .define("BUILD_IDAS", feature!("idas"))
        .define("BUILD_KINSOL", feature!("kinsol"))
		.define("ENABLE_KLU", feature!("klu"))
        .define("OPENMP_ENABLE", feature!("nvecopenmp"))
        .define("PTHREAD_ENABLE", feature!("nvecpthreads"));
    if let Some(inc) = &klu.inc {
        config.define("KLU_INCLUDE_DIR", inc);
    }
    if let Some(lib) = &klu.lib {
        config.define("KLU_LIBRARY_DIR", lib);
    }

    let dst = config.build();
    let dst_disp = dst.display();
    let lib_loc = Some(format!("{}/lib", dst_disp));
    let generated_include = dst.join("include");
    let inc_dir = Some(generated_include.display().to_string());
    (
        Library {
            inc: inc_dir,
            lib: lib_loc,
            generated_include: Some(generated_include),
        },
        library_type,
    )
}

fn generate_bindings(
    inc_dirs: &[Option<String>],
    generated_include: Option<PathBuf>,
) -> Result<Bindings, BindgenError> {
    macro_rules! define {
        ($a:tt, $b:tt) => {
            format!(
                "-DUSE_{}={}",
                stringify!($b),
                if cfg!(feature = $a) { 1 } else { 0 }
            )
        };
    }

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        // Parse the header typedefs rather than Clang builtin prototypes, so
        // C size_t remains Rust usize without changing native compilation.
        .clang_arg("-fno-builtin");
    for dir in inc_dirs {
        if let Some(dir) = dir {
            builder = builder.clang_arg(format!("-I{}", dir))
        }
    }
    builder
        .clang_args(&[
            define!("arkode", ARKODE),
            define!("cvode", CVODE),
            define!("cvodes", CVODES),
            define!("ida", IDA),
            define!("idas", IDAS),
            define!("kinsol", KINSOL),
            define!("klu", KLU),
            define!("nvecopenmp", OPENMP),
            define!("nvecpthreads", PTHREADS),
        ])
        // CMake-installed headers are outputs. Watching them would make a
        // timestamp-changing build invalidate itself after Cargo starts it.
        .parse_callbacks(Box::new(CargoInputs { generated_include }))
        .parse_callbacks(Box::new(ParseSignedConstants))
        .parse_callbacks(Box::new(IgnoreMacros::new()))
        .generate()
}

fn write_bindings_if_changed(bindings: &Bindings, path: &Path) -> std::io::Result<()> {
    let mut bytes = Vec::new();
    bindings.write(Box::new(&mut bytes))?;
    if std::fs::read(path).ok().as_deref() != Some(bytes.as_slice()) {
        std::fs::write(path, bytes)?;
    }
    Ok(())
}

fn get_sundials_version_major(bindings: impl AsRef<Path>) -> Option<u32> {
    let b = File::open(bindings).expect("Couldn't read file bindings.rs!");
    let mut b = BufReader::new(b).bytes();
    'version:
    while b.find(|c| c.as_ref().is_ok_and(|&c| c == b'S')).is_some() {
        for c0 in "UNDIALS_VERSION_MAJOR".bytes() {
            match b.next() {
                Some(Ok(c)) => {
                    if c != c0 {
                        continue 'version
                    }
                }
                Some(Err(_)) | None => return None
            }
        }
        // Match " : u32 = 6"
        if b.find(|c| c.as_ref().is_ok_and(|&c| c == b'=')).is_some() {
            let is_not_digit = |c: &u8| !c.is_ascii_digit();
            let b = b.skip_while(|c| c.as_ref().is_ok_and(is_not_digit));
            let v: Vec<_> =
                b.map_while(|c| c.ok().filter(|c| c.is_ascii_digit()))
                .collect();
            match String::from_utf8(v) {
                Ok(v) => return v.parse().ok(),
                Err(_) => return None
            }
        }
        return None
    }
    None
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:source_root={}", env::var("CARGO_MANIFEST_DIR").expect("Cargo source root"));
    println!("cargo:rerun-if-changed=wrapper.h");
    // Cargo supplies DEP_* from the selected links dependency, not the caller's
    // environment. Its build metadata invalidates us; do not emit env watches.
    let klu = if cfg!(feature = "klu") {
        Library {
            inc: env::var("DEP_SUITESPARSE_SUITESPARSE_INCLUDE").ok(),
            lib: env::var("DEP_SUITESPARSE_SUITESPARSE_LIB").ok(),
            generated_include: None,
        }
    } else {
        Library { inc: None, lib: None, generated_include: None }
    };
    if let Some(dir) = &klu.inc {
        println!("cargo:rerun-if-changed={}", dir);
    }

    // First, we build the SUNDIALS library, with requested modules with CMake
    let mut sundials = Library { inc: None, lib: None, generated_include: None };
    let mut library_type = "dylib";
    if cfg!(any(feature = "build_libraries", target_family = "wasm")) {
        (sundials, library_type) = build_vendor_sundials(&klu);
    } else {
        println!("cargo:source_kind=external");
        println!("cargo:rerun-if-env-changed=SUNDIALS_INCLUDE_DIR");
        println!("cargo:rerun-if-env-changed=SUNDIALS_LIBRARY_DIR");
        sundials.inc = env::var("SUNDIALS_INCLUDE_DIR").ok();
        sundials.lib = env::var("SUNDIALS_LIBRARY_DIR").ok();
        for dir in [sundials.inc.as_ref(), sundials.lib.as_ref()].into_iter().flatten() {
            println!("cargo:rerun-if-changed={}", dir);
        }
    }

    if sundials.lib.is_none() && sundials.inc.is_none() {
        #[cfg(target_family = "windows")] {
            let vcpkg = vcpkg::Config::new()
                .emit_includes(true)
                .find_package("sundials");
            if vcpkg.is_err() {
                (sundials, library_type) = build_vendor_sundials(&klu);
            }
        }
    }

    // Second, we use bindgen to generate the Rust types

    let bindings_rs = PathBuf::from(env::var("OUT_DIR").unwrap())
        .join("bindings.rs");
    let mut build_vendor = true;
    let mut sundials_version_major = 0;
    if let Ok(bindings) = generate_bindings(
        &[sundials.inc, klu.inc.clone()],
        sundials.generated_include,
    ) {
        write_bindings_if_changed(&bindings, &bindings_rs)
            .expect("Couldn't write file bindings.rs!");
        if let Some(v) = get_sundials_version_major(&bindings_rs) {
            if v >= 6 {
                build_vendor = false;
                sundials_version_major = v;
            } else {
                println!("cargo:warning=System sundials version = \
                          {} < 6, will use the vendor version", v);
            }
        }
    }
    if build_vendor {
        (sundials, library_type) = build_vendor_sundials(&klu);
        if let Ok(bindings) = generate_bindings(
            &[sundials.inc, klu.inc.clone()],
            sundials.generated_include,
        ) {
            write_bindings_if_changed(&bindings, &bindings_rs)
                .expect("Couldn't write file bindings.rs!");
            sundials_version_major = get_sundials_version_major(&bindings_rs)
                .expect("Cannot determine vendor sundials version!");
        } else {
            panic!("Unable to generate bindings of the vendor sundials!");
        }
    }
    println!("cargo::rustc-check-cfg=cfg(sundials_version_major, \
        values(\"6\", \"7\"))");
    println!("cargo:rustc-cfg=sundials_version_major=\"{}\"",
        sundials_version_major);

    // Third, we let Cargo know about the library files

    if let Some(dir) = sundials.lib {
        println!("cargo:rustc-link-search=native={}", dir)
    }

    let mut lib_names = vec![];
    if sundials_version_major >= 7 {
        lib_names.push("core");
    }
    lib_names.append(&mut vec![
        "nvecserial",
        "sunlinsolband",
        "sunlinsoldense",
        "sunlinsolpcg",
        "sunlinsolspbcgs",
        "sunlinsolspfgmr",
        "sunlinsolspgmr",
        "sunlinsolsptfqmr",
        "sunmatrixband",
        "sunmatrixdense",
        "sunmatrixsparse",
        "sunnonlinsolfixedpoint",
        "sunnonlinsolnewton",
    ]);
    if cfg!(feature = "klu") {
        lib_names.push("sunlinsolklu");
    }
    macro_rules! link { ($($s:tt),*) => {
        $(if cfg!(feature = $s) { lib_names.push($s) })*
    }}
    link! ("arkode", "cvode", "cvodes", "ida", "idas", "kinsol",
        "nvecopenmp", "nvecpthreads");

    for lib_name in &lib_names {
        println!(
            "cargo:rustc-link-lib={}=sundials_{}",
            library_type, lib_name
        );
    }
    // And that's all.
}
