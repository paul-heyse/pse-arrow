// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Private bounded I09/I11 inquiry; no production realization or cache policy.
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use crate::{jets::EvaluationLimits, library, native_compiled};
use std::{hint::black_box, time::Instant};
use symbolica::{
    atom::{AtomCore, Indeterminate},
    evaluate::{CompiledRealEvaluator, JITCompilationSettings},
};

#[test]
fn polynomial_coefficients_compare_interpreted_packed_scalar_reload_and_scalar_jit() {
    crate::initialize().unwrap();
    // Config::default can read a user TOML; this local inquiry does not inherit
    // undeclared compiler choices. No production admission follows from this test.
    assert!(std::env::var_os("SYMJIT_TOML").is_none());
    assert!(!std::path::Path::new("symjit.toml").exists());
    let common = Instant::now();
    let x = library::formal(0).unwrap();
    let y = library::formal(1).unwrap();
    let parameters = [x.clone(), y.clone()];
    let values = vec![&x * &x * &y + &x * &y + &y * &y, &x * &x + &x * &y * &y];
    let mut coefficients = Vec::new();
    for value in &values {
        for a in &parameters {
            let first = value.derivative(Indeterminate::try_from(a.clone()).unwrap());
            coefficients.push(first.clone());
            for b in &parameters {
                coefficients.push(first.derivative(Indeterminate::try_from(b.clone()).unwrap()));
            }
        }
    }
    let symbolic_us = common.elapsed().as_micros();
    let control = pse_diagnostics::Cancellation::default();
    let scope = pse_kernels::ExecutionScope::new(control.clone(), None);
    let start = Instant::now();
    let mut cache = library::ScalarCache::default();
    let mut floating = [&values, &coefficients]
        .into_iter()
        .map(|expressions| {
            library::bounded_scalar_evaluator(
                expressions,
                &parameters,
                library::Optimization::default(),
                &control.flag(),
                EvaluationLimits::default(),
                &mut cache,
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let floating_us = start.elapsed().as_micros();
    let start = Instant::now();
    let mut batch = native_compiled::Batch::default();
    let mut packed = floating
        .iter()
        .map(|product| {
            batch
                .push(
                    &product.evaluator,
                    &product.facts.storage,
                    &product.export,
                    32 << 20,
                    &scope,
                )
                .unwrap()
        })
        .collect::<Vec<_>>();
    batch.finish(&scope).unwrap();
    for evaluator in &mut packed {
        evaluator.instantiate().unwrap();
    }
    let packed_us = start.elapsed().as_micros();
    let settings = JITCompilationSettings::new()
        .direct_translation(true)
        .optimization_level(2)
        .with_option("fastmath", "false")
        .with_option("use_simd", "false")
        .with_option("enable_simd512", "false")
        .with_option("use_threads", "false")
        .with_option("parallel_mul", "false");
    let start = Instant::now();
    let mut jit = floating
        .iter()
        .map(|product| product.evaluator.jit_compile(settings.clone()).unwrap())
        .collect::<Vec<_>>();
    let jit_us = start.elapsed().as_micros();
    // Same-process compatibility probe: exact trusted bytes and function/layout
    // map qualify reload. This does not certify a receiving runtime after restart.
    let (image, first_name) = packed[0].probe_module().unwrap();
    let (same_image, second_name) = packed[1].probe_module().unwrap();
    assert_eq!(image, same_image);
    let directory = tempfile::tempdir().unwrap();
    let pending = directory.path().join("pending");
    let committed = directory.path().join("committed");
    std::fs::create_dir(&pending).unwrap();
    let bytes = std::fs::read(image).unwrap();
    let source_manifest = format!("{first_name}:2:2;{second_name}:2:12");
    let start = Instant::now();
    std::fs::write(pending.join("module.so"), &bytes).unwrap();
    assert!(
        !committed.exists(),
        "partial writer is not a published candidate"
    );
    std::fs::write(pending.join("layout"), &source_manifest).unwrap();
    std::fs::rename(&pending, &committed).unwrap();
    let publication_us = start.elapsed().as_micros();
    assert_ne!(
        source_manifest,
        format!("{first_name}:2:3;{second_name}:2:12"),
        "different output layouts refuse before loading"
    );
    let corrupt = directory.path().join("corrupt.so");
    std::fs::write(&corrupt, b"not a scalar module").unwrap();
    assert_ne!(
        std::fs::read(&corrupt).unwrap(),
        bytes,
        "corruption refuses the exact-byte receiving check"
    );
    assert!(CompiledRealEvaluator::load(&corrupt, first_name).is_err());
    let start = Instant::now();
    assert_eq!(
        std::fs::read_to_string(committed.join("layout")).unwrap(),
        source_manifest
    );
    assert_eq!(std::fs::read(committed.join("module.so")).unwrap(), bytes);
    let mut loaded = [
        CompiledRealEvaluator::load(committed.join("module.so"), first_name).unwrap(),
        CompiledRealEvaluator::load(committed.join("module.so"), second_name).unwrap(),
    ];
    for (index, evaluator) in loaded.iter().enumerate() {
        assert_eq!(evaluator.get_input_len(), 2);
        assert_eq!(evaluator.get_output_len(), [2, 12][index]);
    }
    let reload_us = start.elapsed().as_micros();
    std::fs::remove_dir_all(&committed).unwrap();
    // Unlinking an evicted image does not unload the live library-owned code.
    let repeats = 16;
    for rows in [1, 64, 512] {
        let start = Instant::now();
        let source = (0..rows)
            .map(|row| [0.25 + row as f64 / 2048., 0.75 - row as f64 / 4096.])
            .collect::<Vec<_>>();
        let arguments = source
            .iter()
            .flat_map(|row| row.iter().copied())
            .collect::<Vec<_>>();
        let packing_us = start.elapsed().as_micros();
        let start = Instant::now();
        let mut outputs = [2, 12].map(|columns| vec![0.; rows * columns]);
        let output_us = start.elapsed().as_micros();
        let start = Instant::now();
        for _ in 0..repeats {
            for (product, output) in floating.iter_mut().zip(&mut outputs) {
                let width = product.evaluator.get_output_len();
                for (row, target) in arguments
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .zip(output.chunks_exact_mut(width))
                {
                    product.evaluator.evaluate(black_box(row), target);
                }
            }
        }
        let interpreted_us = start.elapsed().as_micros();
        let expected = outputs.clone();
        let start = Instant::now();
        for _ in 0..repeats {
            for ((product, output), width) in packed.iter_mut().zip(&mut outputs).zip([2, 12]) {
                for (row, target) in arguments
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .zip(output.chunks_exact_mut(width))
                {
                    product.evaluate(black_box(row), target).unwrap();
                }
            }
        }
        let packed_warm_us = start.elapsed().as_micros();
        for (actual, expected) in outputs.iter().flatten().zip(expected.iter().flatten()) {
            assert!((actual - expected).abs() < 1e-11);
        }
        let start = Instant::now();
        for _ in 0..repeats {
            for (product, output) in jit.iter_mut().zip(&mut outputs) {
                product.batch_evaluate(black_box(&arguments), output, rows);
            }
        }
        let jit_warm_us = start.elapsed().as_micros();
        for (actual, expected) in outputs.iter().flatten().zip(expected.iter().flatten()) {
            assert!((actual - expected).abs() < 1e-11);
        }
        for ((product, output), width) in loaded.iter_mut().zip(&mut outputs).zip([2, 12]) {
            for (row, target) in arguments
                .as_chunks::<2>()
                .0
                .iter()
                .zip(output.chunks_exact_mut(width))
            {
                product.evaluate(row, target);
            }
        }
        for (actual, expected) in outputs.iter().flatten().zip(expected.iter().flatten()) {
            assert!((actual - expected).abs() < 1e-11);
        }
        let row = source[0];
        assert!(
            (outputs[0][0] - (row[0] * row[0] * row[1] + row[0] * row[1] + row[1] * row[1])).abs()
                < 1e-12
        );
        assert!((outputs[1][0] - (2. * row[0] * row[1] + row[1])).abs() < 1e-12);
        println!(
            "I09/I11 rows={rows} repeats={repeats} symbolic_us={symbolic_us} floating_us={floating_us} packed_prepare_load_us={packed_us} jit_prepare_us={jit_us} publish_us={publication_us} reload_validate_load_us={reload_us} packing_us={packing_us} output_allocation_us={output_us} interpreted_warm_us={interpreted_us} packed_warm_us={packed_warm_us} jit_warm_us={jit_warm_us} packed_complete_us={} jit_complete_us={} checksum={}",
            symbolic_us + floating_us + packed_us + packing_us + output_us + packed_warm_us,
            symbolic_us + floating_us + jit_us + packing_us + output_us + jit_warm_us,
            black_box(outputs.iter().flatten().sum::<f64>())
        );
    }
    #[cfg(target_arch = "x86_64")]
    println!(
        "I11 scalar JIT backend=SymJIT2.26.0 Native host_avx={} host_avx2={} host_fma={} use_simd=false use_threads=false fastmath=false direct=true opt_level=2 parallel_mul=false; packed C++ baseline=x86-64 generic,ffp-contract=off; no production ISA equivalence granted",
        std::is_x86_feature_detected!("avx"),
        std::is_x86_feature_detected!("avx2"),
        std::is_x86_feature_detected!("fma")
    );
}
