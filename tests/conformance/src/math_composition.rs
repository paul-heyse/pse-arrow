// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use faer::sparse::{Pair, SparseColMat, SymbolicSparseColMat};
use symbolica::atom::AtomCore;

#[test]
fn opaque_provider_partials_bind_into_symbolica_chain_derivatives() {
    use symbolica::atom::{Atom, FunctionBuilder, NamespacedSymbol, SymbolBuilder};
    let a = pse_math::library::formal(0).unwrap();
    let b = pse_math::library::formal(1).unwrap();
    let name = NamespacedSymbol::try_from("pse_math_probe::property").unwrap();
    let function = SymbolBuilder::new(name).build().unwrap();
    let property = FunctionBuilder::new(function)
        .add_arg(&a)
        .add_arg(&b)
        .finish();
    let first = property.derivative(symbolica::atom::Indeterminate::try_from(a.clone()).unwrap());
    let second = first.derivative(symbolica::atom::Indeterminate::try_from(a.clone()).unwrap());
    let outer = property.pow(Atom::num(2)) + &a + &b;
    let expressions = [
        outer.clone(),
        outer.derivative(symbolica::atom::Indeterminate::try_from(a.clone()).unwrap()),
        outer
            .derivative(symbolica::atom::Indeterminate::try_from(a.clone()).unwrap())
            .derivative(symbolica::atom::Indeterminate::try_from(a.clone()).unwrap()),
    ];
    // Library-produced function/partial atoms are bound as local evaluator parameters.
    // No global callbacks, finite differences, or project chain-rule implementation.
    let parameters = [a, b, property, first, second];
    let mut evaluator = Atom::evaluator_multiple(&expressions, &parameters)
        .cores(1)
        .build()
        .unwrap()
        .map_coeff(&|c| c.re.to_f64());
    let mut output = [0.0; 3];
    // At (2,3), the provider is a*b, with partials (b,0) w.r.t. a.
    evaluator
        .try_evaluate(&[2.0, 3.0, 6.0, 3.0, 0.0], &mut output)
        .unwrap();
    assert_eq!(output, [41.0, 37.0, 18.0]);
}

#[test]
fn duplicate_order_refill_and_alias_hessian() {
    // f(a,b)=a*b. Both local mixed partials contribute when a and b alias x.
    // Symbolica supplies derivatives; faer owns duplicate coordinate accumulation.
    let a = pse_math::library::formal(0).unwrap();
    let b = pse_math::library::formal(1).unwrap();
    let f = &a * &b;
    let mixed = f
        .derivative(symbolica::atom::Indeterminate::try_from(a.clone()).unwrap())
        .derivative(symbolica::atom::Indeterminate::try_from(b.clone()).unwrap());
    assert_eq!(mixed, symbolica::atom::Atom::num(1));
    let (pattern, order) = SymbolicSparseColMat::<usize>::try_new_from_indices(
        1,
        1,
        &[Pair::new(0, 0), Pair::new(0, 0)],
    )
    .unwrap();
    let alias =
        SparseColMat::<usize, f64>::new_from_argsort(pattern.clone(), &order, &[1.0, 1.0]).unwrap();
    assert_eq!(alias.to_dense()[(0, 0)], 2.0);
    let refill =
        SparseColMat::<usize, f64>::new_from_argsort(pattern, &order, &[3.0, 5.0]).unwrap();
    assert_eq!(refill.to_dense()[(0, 0)], 8.0);
    let (separate, order) = SymbolicSparseColMat::<usize>::try_new_from_indices(
        2,
        2,
        &[Pair::new(0, 1), Pair::new(1, 0)],
    )
    .unwrap();
    let control = SparseColMat::<usize, f64>::new_from_argsort(separate, &order, &[1.0, 1.0])
        .unwrap()
        .to_dense();
    assert_eq!(control[(0, 0)], 0.0);
    assert_eq!(control[(1, 0)], 1.0);
    assert_eq!(control[(0, 1)], 1.0);
    assert!(SymbolicSparseColMat::<usize>::try_new_from_indices(1, 1, &[Pair::new(1, 0)]).is_err());
}

#[test]
fn fallible_num_dual_second_derivatives() {
    use num_dual::DualNum;
    let point = nalgebra::SVector::<f64, 2>::new(2.0, 3.0);
    let result: Result<_, &'static str> = num_dual::hessian(
        |x| {
            if x[0].re <= 0.0 {
                return Err("positive first argument required");
            }
            Ok(x[0].powi(2) * x[1] + x[1].ln())
        },
        &point,
    );
    let (value, gradient, hessian) = result.unwrap();
    assert!((value - (12.0 + 3.0_f64.ln())).abs() < 1e-13);
    assert!((gradient[0] - 12.0).abs() < 1e-13);
    assert!((hessian[(0, 0)] - 6.0).abs() < 1e-13);
    assert!((hessian[(0, 1)] - 4.0).abs() < 1e-13);
    let invalid = nalgebra::SVector::<f64, 2>::new(-1.0, 3.0);
    let result: Result<_, &'static str> = num_dual::hessian(
        |x| {
            if x[0].re <= 0.0 {
                Err("domain")
            } else {
                Ok(x[0].powi(2) * x[1])
            }
        },
        &invalid,
    );
    assert!(result.is_err());
}

#[test]
fn feos_coherent_state_multi_output() {
    use feos::pcsaft::{PcSaft, PcSaftParameters};
    use feos_core::{Contributions, State, parameter::IdentifierOption};
    use quantity::{KELVIN, METER, MOL, PASCAL};
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/pcsaft-propane.json");
    let parameters =
        PcSaftParameters::from_json(vec!["propane"], path, None, IdentifierOption::Name).unwrap();
    let eos = PcSaft::new(parameters);
    let volume = 1.5e-3 * METER.powi::<3>();
    let state = State::new_nvt(&&eos, 300.0 * KELVIN, volume, MOL).unwrap();
    let pressure = (state.pressure(Contributions::Total) / PASCAL).into_value();
    let recovered = (state.volume().unwrap() / volume).into_value();
    assert!(pressure > 0.0 && pressure.is_finite());
    assert!((recovered - 1.0).abs() < 1e-12);
}
