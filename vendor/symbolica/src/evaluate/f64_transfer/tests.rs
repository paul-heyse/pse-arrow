use super::*;

#[test]
fn closed_known_head_profile_refuses_foreign_tags_and_restores_without_source_callbacks() {
    use crate::symbol;
    let symbol = symbol!("fixed_f64_transfer_magnitude_v1");
    let head = F64KnownHead {
        tag: 71, version: 1, symbol, arity: 2,
        export_name: "fixed_f64_transfer_magnitude_v1",
        evaluate: |args| args[0].hypot(args[1]),
    };
    let heads = [head];
    let mut external = ExternalFunctionContainer::new(symbol, vec![], vec![]);
    external.imp = Some(Box::new(head.evaluate));
    let source = ExpressionEvaluator {
        stack: vec![f64::NAN; 5], param_count: 2, reserved_indices: 2,
        instructions: vec![(Instr::ExternalFun(2, 0, vec![0, 1]), ComplexPhase::Any)],
        result_indices: vec![2, 2], external_fns: vec![external],
        settings: OptimizationSettings::default(),
    };
    assert!(source.f64_transfer().is_err());
    let borrowed = source.f64_transfer_with_heads(&heads).unwrap();
    let facts = borrowed.extents();
    assert_eq!(facts.external_functions, 1);
    assert!(facts.external_bytes >= size_of::<ExternalFunctionContainer<f64>>() + 16);
    let mut bytes = vec![0; borrowed.encoded_len()];
    borrowed.write_into(&mut bytes).unwrap();
    let view = F64ProgramView::inspect_with_heads(&bytes, &heads).unwrap();
    assert_eq!(view.extents(), facts);
    assert!(F64ProgramView::inspect(&bytes).is_err());
    assert!(matches!(view.restore_with(|_| Err("original head backing denial")),
        Err(F64RestoreError::Admission("original head backing denial"))));
    drop(source);
    let mut restored = view.restore_with(|_| Ok::<_, std::convert::Infallible>(())).unwrap();
    let mut output = [0.; 2];
    restored.evaluate(&[3., 4.], &mut output);
    assert_eq!(output, [5., 5.]);
    restored.evaluate(&[5., 12.], &mut output);
    assert_eq!(output, [13., 13.]);
    let mut incompatible = heads;
    incompatible[0].version = 2;
    assert!(F64ProgramView::inspect_with_heads(&bytes, &incompatible).is_err());
    let mut corrupt = bytes.clone();
    let instruction = HEADER + 8 * facts.results;
    corrupt[instruction + 10..instruction + 18].copy_from_slice(&99_u64.to_le_bytes());
    assert!(F64ProgramView::inspect_with_heads(&corrupt, &heads).is_err());
    let mut arity = bytes.clone();
    arity[instruction + 18..instruction + 26].copy_from_slice(&1_u64.to_le_bytes());
    assert!(F64ProgramView::inspect_with_heads(&arity, &heads).is_err());
    let mut undefined = bytes;
    undefined[instruction + 26..instruction + 34].copy_from_slice(&4_u64.to_le_bytes());
    assert!(matches!(
        F64ProgramView::inspect_with_heads(&undefined, &heads).unwrap()
            .restore_with(|_| Ok::<_, std::convert::Infallible>(())),
        Err(F64RestoreError::Program(F64TransferError::Undefined(4)))
    ));
}
use crate::{
    atom::{Atom, AtomCore},
    parse, symbol,
};
use std::{cell::Cell, convert::Infallible};

fn fixture() -> ExpressionEvaluator<f64> {
    ExpressionEvaluator {
        stack: vec![
            12.,
            13.,
            -0.,
            f64::from_bits(0x7ff8_0000_0000_1234),
            14.,
            15.,
            16.,
            17.,
        ],
        param_count: 2,
        reserved_indices: 4,
        instructions: vec![
            (Instr::Add(4, vec![0, 2]), ComplexPhase::Any),
            (Instr::Mul(5, vec![4, 1]), ComplexPhase::PartialReal(1)),
            (Instr::Pow(6, 5, -1), ComplexPhase::Real),
        ],
        result_indices: vec![6, 0, 2, 3, 6],
        external_fns: vec![],
        settings: OptimizationSettings::default(),
    }
}
fn encode(source: &ExpressionEvaluator<f64>) -> Vec<u8> {
    let expected = source.count_operations();
    let source = source.f64_transfer().unwrap();
    assert_eq!(source.operation_count().unwrap(), expected);
    let mut bytes = vec![0; source.encoded_len()];
    source.write_into(&mut bytes).unwrap();
    assert_eq!(
        F64ProgramView::inspect(&bytes)
            .unwrap()
            .operation_count()
            .unwrap(),
        expected
    );
    bytes
}

#[test]
fn checked_operation_population_overflow() {
    let mut count = OperationCount::new(usize::MAX, 0, 0, 0);
    // Keep the borrowed instruction alive while testing the shared counter.
    let raw = Instr::Add(4, vec![0, 1]);
    let instruction = NativeInstruction::from_instr(&raw, ComplexPhase::Any, &[], &[]).unwrap();
    assert_eq!(
        instruction.count_into(&mut count),
        Err(F64TransferError::Overflow("operation count"))
    );
    count = OperationCount::new(0, usize::MAX, 0, 0);
    let raw = Instr::Pow(4, 0, -2);
    assert_eq!(
        NativeInstruction::from_instr(&raw, ComplexPhase::Any, &[], &[])
            .unwrap()
            .count_into(&mut count),
        Err(F64TransferError::Overflow("operation count"))
    );
    count = OperationCount::new(0, 0, usize::MAX, 0);
    assert_eq!(
        NativeInstruction::from_instr(&raw, ComplexPhase::Any, &[], &[])
            .unwrap()
            .count_into(&mut count),
        Err(F64TransferError::Overflow("operation count"))
    );
    count = OperationCount::new(0, 0, 0, usize::MAX);
    let raw = Instr::Powf(4, 0, 1);
    assert_eq!(
        NativeInstruction::from_instr(&raw, ComplexPhase::Any, &[], &[])
            .unwrap()
            .count_into(&mut count),
        Err(F64TransferError::Overflow("operation count"))
    );
}
fn restore(bytes: &[u8]) -> ExpressionEvaluator<f64> {
    F64ProgramView::inspect(bytes)
        .unwrap()
        .restore_with(|_| Ok::<_, Infallible>(()))
        .unwrap()
}
fn bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|value| value.to_bits()).collect()
}
fn set_index(bytes: &mut [u8], offset: usize, index: u64) {
    bytes[offset..offset + 8].copy_from_slice(&index.to_le_bytes());
}
fn allocations() -> usize {
    ALLOCATION_ATTEMPTS.with(Cell::get)
}
fn reset_allocations() {
    ALLOCATION_ATTEMPTS.with(|count| count.set(0));
    FAIL_RESERVATION_AT.with(|count| count.set(None));
}

#[test]
fn exact_bits_aliases_order_and_cold_state_roundtrip() {
    let mut source = fixture();
    let bytes = encode(&source);
    let native = source.f64_transfer().unwrap().extents();
    assert_eq!(F64ProgramView::inspect(&bytes).unwrap().extents(), native);
    let mut restored = restore(&bytes);
    assert_eq!(source.instructions, restored.instructions);
    assert_eq!(source.result_indices, restored.result_indices);
    assert_eq!(bits(source.get_constants()), bits(restored.get_constants()));
    for (index, value) in restored.stack.iter().enumerate() {
        if !(2..4).contains(&index) {
            assert!(value.is_nan());
        }
    }
    for params in [[2., 3.], [-2., 1.5], [4., 2.]] {
        let mut before = vec![0.; 5];
        let mut after = vec![0.; 5];
        source.try_evaluate(&params, &mut before).unwrap();
        restored.try_evaluate(&params, &mut after).unwrap();
        assert_eq!(bits(&before), bits(&after));
    }
    assert_eq!(
        encode(&source),
        bytes,
        "warmed parameter/intermediate slots are never encoded"
    );
    source.settings.verbose = true;
    source.settings.n_cores = 7;
    assert_eq!(
        encode(&source),
        bytes,
        "preparation settings are not encoded"
    );
    let cold = restore(&bytes);
    assert!(cold.settings.abort_check.is_none());
    assert!(cold.settings.hot_start.is_none());
    assert!(cold.external_fns.is_empty());
}

#[test]
fn nary_operand_order_is_not_reassociated() {
    let source = ExpressionEvaluator {
        stack: vec![0.; 4],
        param_count: 3,
        reserved_indices: 3,
        instructions: vec![(Instr::Add(3, vec![0, 1, 2]), ComplexPhase::Any)],
        result_indices: vec![3],
        external_fns: vec![],
        settings: OptimizationSettings::default(),
    };
    let mut restored = restore(&encode(&source));
    assert_eq!(
        restored.evaluate_single(&[1e16, -1e16, 1.]).to_bits(),
        1f64.to_bits()
    );
    assert_eq!(restored.instructions, source.instructions);
}

#[test]
fn fixed_builtins_integer_power_and_powf_use_supplier_execution() {
    let mut source = ExpressionEvaluator {
        stack: vec![0.; 10],
        param_count: 1,
        reserved_indices: 1,
        instructions: vec![],
        result_indices: (1..10).collect(),
        external_fns: vec![],
        settings: OptimizationSettings::default(),
    };
    for (index, symbol) in [
        Symbol::EXP,
        Symbol::LOG,
        Symbol::SIN,
        Symbol::COS,
        Symbol::SQRT,
        Symbol::ABS,
        Symbol::CONJ,
    ]
    .into_iter()
    .enumerate()
    {
        source
            .instructions
            .push((Instr::BuiltinFun(index + 1, symbol, 0), ComplexPhase::Real));
    }
    source
        .instructions
        .push((Instr::Pow(8, 0, -3), ComplexPhase::Real));
    source
        .instructions
        .push((Instr::Powf(9, 0, 0), ComplexPhase::Any));
    let mut restored = restore(&encode(&source));
    let mut a = [0.; 9];
    let mut b = [0.; 9];
    source.try_evaluate(&[1.], &mut a).unwrap();
    restored.try_evaluate(&[1.], &mut b).unwrap();
    assert_eq!(bits(&a), bits(&b));
    assert_eq!(source.instructions, restored.instructions);
}

#[test]
fn integer_power_source_domain_endpoints_and_refusals_precede_admission() {
    for exponent in [-(i32::MAX as i64), i32::MAX as i64] {
        let mut source = ExpressionEvaluator {
            stack: vec![0.; 2],
            param_count: 1,
            reserved_indices: 1,
            instructions: vec![(Instr::Pow(1, 0, exponent), ComplexPhase::Any)],
            result_indices: vec![1],
            external_fns: vec![],
            settings: OptimizationSettings::default(),
        };
        let bytes = encode(&source);
        let mut restored = restore(&bytes);
        assert_eq!(source.instructions, restored.instructions);
        for base in [-1., 1.] {
            assert_eq!(
                source.evaluate_single(&[base]).to_bits(),
                restored.evaluate_single(&[base]).to_bits()
            );
        }
    }
    let bytes = encode(&fixture());
    for exponent in [
        i64::MIN,
        i64::MAX,
        -(i32::MAX as i64) - 1,
        i32::MAX as i64 + 1,
    ] {
        let mut source = fixture();
        source.instructions[2].0 = Instr::Pow(6, 5, exponent);
        assert!(matches!(
            source.f64_transfer(),
            Err(F64TransferError::Unsupported(_))
        ));
        let mut forged = bytes.clone();
        let end = forged.len();
        forged[end - 8..].copy_from_slice(&exponent.to_le_bytes());
        reset_allocations();
        let called = Cell::new(false);
        let inspected = F64ProgramView::inspect(&forged).map(|view| {
            view.restore_with(|_| {
                called.set(true);
                Ok::<_, Infallible>(())
            })
        });
        assert!(matches!(inspected, Err(F64TransferError::Unsupported(_))));
        assert!(!called.get());
        assert_eq!(allocations(), 0);
    }
    // Floating powers have their original separate f64 semantics, not an integer
    // cast or a substitute implementation for refused integer instructions.
    let mut source = ExpressionEvaluator {
        stack: vec![0.; 3],
        param_count: 2,
        reserved_indices: 2,
        instructions: vec![(Instr::Powf(2, 0, 1), ComplexPhase::Any)],
        result_indices: vec![2],
        external_fns: vec![],
        settings: OptimizationSettings::default(),
    };
    let mut restored = restore(&encode(&source));
    for power in [i64::MAX as f64, 0.5, -0.5] {
        assert_eq!(
            source.evaluate_single(&[1., power]).to_bits(),
            restored.evaluate_single(&[1., power]).to_bits()
        );
    }
}

#[test]
fn actual_requested_growth_requires_native_admission_even_for_tiny_encoding() {
    let mut bytes = encode(&fixture());
    set_index(&mut bytes, 28, 1_000_000);
    let view = F64ProgramView::inspect(&bytes).unwrap();
    assert!(bytes.len() < 512);
    assert_eq!(view.extents().stack, 1_000_000);
    assert_eq!(view.extents().stack_bytes, 8_000_000);
    reset_allocations();
    let result = view.restore_with(|actual| {
        assert_eq!(actual.stack_bytes, 8_000_000);
        assert_eq!(allocations(), 0);
        Err("original growth denial")
    });
    assert!(matches!(
        result,
        Err(F64RestoreError::Admission("original growth denial"))
    ));
    assert_eq!(allocations(), 0);
}

#[test]
fn scalar_phases_are_preserved_and_partial_real_is_nary_bounded() {
    for phase in [
        ComplexPhase::Real,
        ComplexPhase::Imag,
        ComplexPhase::Any,
        ComplexPhase::PartialReal(0),
        ComplexPhase::PartialReal(2),
    ] {
        let mut source = fixture();
        source.instructions[0].1 = phase;
        let restored = restore(&encode(&source));
        assert_eq!(source.instructions, restored.instructions);
    }
    let mut source = fixture();
    source.instructions[0].1 = ComplexPhase::PartialReal(3);
    assert!(source.f64_transfer().is_err());
    source.instructions[0].1 = ComplexPhase::Any;
    source.instructions[2].1 = ComplexPhase::PartialReal(1);
    assert!(source.f64_transfer().is_err());
}

#[test]
fn structurally_malformed_bytes_never_invoke_admission() {
    let original = encode(&fixture());
    let start = F64ProgramView::inspect(&original)
        .unwrap()
        .instruction_start;
    let mut cases = vec![];
    for count in 0..original.len() {
        cases.push(original[..count].to_vec());
    }
    let mut bad = original.clone();
    bad.push(0);
    cases.push(bad);
    for (offset, value) in [(0, 0), (8, 1), (10, 1), (start, 12), (start + 1, 9)] {
        let mut bad = original.clone();
        bad[offset] = value;
        cases.push(bad);
    }
    for (offset, value) in [
        (12, u64::MAX),
        (20, u64::MAX),
        (28, u64::MAX),
        (36, u64::MAX),
        (44, u64::MAX),
        (HEADER + 16, 8),
        (start + 2, 0),
        (start + 2, 8),
        (start + 10, 0),
        (start + 10, u64::MAX),
        (start + 18, 8),
        (start + 26, u64::MAX),
    ] {
        let mut bad = original.clone();
        set_index(&mut bad, offset, value);
        cases.push(bad);
    }
    for bad in cases {
        reset_allocations();
        let calls = Cell::new(0);
        let result = F64ProgramView::inspect(&bad).map(|view| {
            view.restore_with(|_| {
                calls.set(calls.get() + 1);
                Ok::<_, Infallible>(())
            })
        });
        assert!(result.is_err());
        assert_eq!(calls.get(), 0);
        assert_eq!(allocations(), 0);
    }
}

#[test]
fn definition_before_read_and_results_are_checked_after_admission() {
    let original = encode(&fixture());
    let start = F64ProgramView::inspect(&original)
        .unwrap()
        .instruction_start;
    for (offset, index) in [(start + 18, 4), (start + 18, 6), (HEADER + 16, 7)] {
        let mut bytes = original.clone();
        set_index(&mut bytes, offset, index);
        let view = F64ProgramView::inspect(&bytes).unwrap();
        reset_allocations();
        let calls = Cell::new(0);
        let result = view.restore_with(|_| {
            assert_eq!(allocations(), 0);
            calls.set(calls.get() + 1);
            Ok::<_, Infallible>(())
        });
        assert!(matches!(
            result,
            Err(F64RestoreError::Program(F64TransferError::Undefined(_)))
        ));
        assert_eq!(calls.get(), 1);
        assert_eq!(
            allocations(),
            1,
            "only admitted definition scratch, no evaluator allocation"
        );
    }
}

#[derive(Debug)]
struct Denied {
    payload: Box<str>,
    cause: Box<std::io::Error>,
}
impl fmt::Display for Denied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.payload.fmt(f)
    }
}
impl Error for Denied {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.cause)
    }
}

#[test]
fn original_admission_error_and_source_survive_without_any_allocation() {
    let bytes = encode(&fixture());
    let view = F64ProgramView::inspect(&bytes).unwrap();
    let expected = view.extents();
    let denied = Denied {
        payload: "original cancellation payload".into(),
        cause: Box::new(std::io::Error::new(
            std::io::ErrorKind::Interrupted,
            "original source",
        )),
    };
    let cause = &*denied.cause as *const std::io::Error;
    reset_allocations();
    let calls = Cell::new(0);
    let error = view
        .restore_with(|actual| {
            assert_eq!(*actual, expected);
            assert_eq!(allocations(), 0);
            calls.set(calls.get() + 1);
            Err(denied)
        })
        .unwrap_err();
    assert_eq!(calls.get(), 1);
    assert_eq!(allocations(), 0);
    let original = error.source().unwrap().downcast_ref::<Denied>().unwrap();
    assert_eq!(original.payload.as_ref(), "original cancellation payload");
    assert_eq!(&*original.cause as *const _, cause);
    assert_eq!(
        original
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::Interrupted
    );
    assert!(matches!(error, F64RestoreError::Admission(_)));
}

#[test]
fn allocation_refusal_is_distinct_at_every_materialization_step() {
    let bytes = encode(&fixture());
    let view = F64ProgramView::inspect(&bytes).unwrap();
    reset_allocations();
    view.restore_with(|_| Ok::<_, Infallible>(())).unwrap();
    let requests = allocations();
    assert!(requests >= 6);
    for failed in 1..=requests {
        reset_allocations();
        FAIL_RESERVATION_AT.with(|value| value.set(Some(failed)));
        let calls = Cell::new(0);
        let error = view
            .restore_with(|_| {
                assert_eq!(allocations(), 0);
                calls.set(calls.get() + 1);
                Ok::<_, Infallible>(())
            })
            .unwrap_err();
        assert_eq!(calls.get(), 1);
        assert!(matches!(error, F64RestoreError::Allocation(_)));
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<TryReserveError>()
                .is_some()
        );
        assert_eq!(allocations(), failed);
    }
    reset_allocations();
}

#[test]
fn producer_refuses_control_flow_externals_and_protected_destinations() {
    for instruction in [
        Instr::IfElse(0, super::super::Label(0)),
        Instr::Goto(super::super::Label(0)),
        Instr::Label(super::super::Label(0)),
        Instr::Join(4, 0, 0, 0),
        Instr::ExternalFun(4, 0, vec![0]),
        Instr::Add(0, vec![0]),
        Instr::Add(4, vec![]),
    ] {
        let mut source = fixture();
        source.instructions = vec![(instruction, ComplexPhase::Any)];
        assert!(source.f64_transfer().is_err());
    }
    let mut source = fixture();
    source
        .external_fns
        .push(super::super::ExternalFunctionContainer {
            export_name: "unused external".into(),
            symbol: Symbol::EXP,
            tags: vec![],
            fixed_args: vec![],
            imp: None,
            constant_index: None,
            stack: vec![],
            body: None,
        });
    assert!(source.f64_transfer().is_err());
    let source = fixture();
    let borrowed = source.f64_transfer().unwrap();
    assert!(matches!(
        borrowed.write_into(&mut []),
        Err(F64TransferError::Destination { .. })
    ));
}

#[test]
fn actual_direct_producer_value_and_first_derivatives_roundtrip() {
    let x = symbol!("checked_transfer_x");
    let y = symbol!("checked_transfer_y");
    let parameters = [Atom::var(x), Atom::var(y)];
    let value = parse!(
        "checked_transfer_x^3+checked_transfer_x*checked_transfer_y+sin(checked_transfer_x)+exp(checked_transfer_y)+log(checked_transfer_x)+sqrt(checked_transfer_y)+(checked_transfer_x+checked_transfer_y)^(checked_transfer_x-checked_transfer_y)"
    );
    let expressions = [value.clone(), value.derivative(x), value.derivative(y)];
    let mut source = Atom::evaluator_multiple(&expressions, &parameters)
        .direct_translation(true)
        .cores(1)
        .horner_iterations(10)
        .cpe_iterations(Some(1))
        .max_common_pair_cache_entries(65536)
        .max_common_pair_distance(256)
        .max_horner_scheme_variables(500)
        .verbose(false)
        .build()
        .unwrap()
        .map_coeff(&|coefficient| coefficient.re.to_f64());
    let bytes = encode(&source);
    let mut restored = restore(&bytes);
    assert_eq!(source.instructions, restored.instructions);
    for parameters in [[1.2, 0.7], [2.3, 1.8], [4.1, 2.2]] {
        let mut before = [0.; 3];
        let mut after = [0.; 3];
        source.try_evaluate(&parameters, &mut before).unwrap();
        restored.try_evaluate(&parameters, &mut after).unwrap();
        assert_eq!(bits(&before), bits(&after));
        assert!(after.iter().all(|value| value.is_finite()));
    }
    assert_eq!(encode(&source), bytes);
}
