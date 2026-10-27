//! Probe: behaviour of `UnivariatePolynomial<Q>::refine_root_interval` (Symbolica 3.0.1)
//! on endpoint roots and zero-symmetric intervals. One case per process run.
use std::sync::Arc;
use symbolica::{
    atom::Atom,
    domains::rational::{Q, Rational},
    poly::{PolyVariable, univariate::UnivariatePolynomial},
    symbol,
};

fn poly(coefficients: &[i64]) -> UnivariatePolynomial<Q> {
    let x = symbol!("x");
    let _ = Atom::var(x).as_view();
    UnivariatePolynomial::from_coefficients(
        &Q,
        coefficients.iter().map(|&c| Rational::from(c)).collect(),
        Arc::new(PolyVariable::Symbol(x)),
    )
}

fn r(n: i64, d: i64) -> Rational {
    Rational::from((n, d))
}

fn main() {
    let case = std::env::args().nth(1).expect("case name");
    // Same tolerance as the consumer's ladder: 2^-54 (relative stopping rule in 3.0.1).
    let tol = r(1, 1_i64 << 54);
    // roots 1,2,3: x^3 - 6x^2 + 11x - 6 (coefficients low -> high)
    let p123 = poly(&[-6, 11, -6, 1]);
    // roots -1,0,1: x^3 - x
    let pm0p = poly(&[0, -1, 0, 1]);
    // roots -1, 1/4, 1: (x+1)(x-1)(4x-1) = 4x^3 - x^2 - 4x + 1
    let pq = poly(&[1, -4, -1, 4]);
    let (label, p, interval) = match case.as_str() {
        "a1" => ("right-endpoint root 2, interval (3/2, 2], p>0 inside", &p123, (r(3, 2), r(2, 1))),
        "a2" => ("right-endpoint root 3, interval (5/2, 3], p<0 inside", &p123, (r(5, 2), r(3, 1))),
        "b1" => ("symmetric (-1/2, 1/2), root at 0", &pm0p, (r(-1, 2), r(1, 2))),
        "b2" => ("symmetric (-1/2, 1/2), root at 1/4", &pq, (r(-1, 2), r(1, 2))),
        "b3" => ("asymmetric (-1/2, 1/4), root at 0", &pm0p, (r(-1, 2), r(1, 4))),
        "c1" => ("left-endpoint root 1, interval [1, 3/2), p>0 inside", &p123, (r(1, 1), r(3, 2))),
        "c2" => ("left-endpoint root 2, interval [2, 5/2), p<0 inside", &p123, (r(2, 1), r(5, 2))),
        "d1" => ("isolator's own interval (0, 2] for root 1; endpoint 2 is another root", &p123, (r(0, 1), r(2, 1))),
        "d2" => ("isolator's own interval (0, 1] for root 1/4; endpoint 1 is another root", &pq, (r(0, 1), r(1, 1))),
        "n0" => ("control: interior root 2, interval (3/2, 5/2)", &p123, (r(3, 2), r(5, 2))),
        "iso" => {
            for (name, q) in [("p123", &p123), ("pm0p", &pm0p), ("pq", &pq)] {
                println!("isolate_real_root_intervals {name}: {:?}", q.isolate_real_root_intervals());
            }
            return;
        }
        other => panic!("unknown case {other}"),
    };
    println!("case {case}: {label}; input ({}, {})", interval.0, interval.1);
    let started = std::time::Instant::now();
    let result = std::panic::catch_unwind(|| p.refine_root_interval(interval.clone(), &tol));
    match result {
        Ok((lo, hi)) => {
            let contains = |x: &Rational| *x >= lo && *x <= hi;
            println!(
                "returned [{} , {}] ~ [{:e}, {:e}] after {:?}; width {:e}",
                lo,
                hi,
                lo.to_f64(),
                hi.to_f64(),
                started.elapsed(),
                (&hi - &lo).to_f64()
            );
            println!(
                "p(lo) = {}, p(hi) = {}; contains candidate roots: 0:{} 1/4:{} 1:{} 2:{} 3:{}",
                p.evaluate(&lo),
                p.evaluate(&hi),
                contains(&r(0, 1)),
                contains(&r(1, 4)),
                contains(&r(1, 1)),
                contains(&r(2, 1)),
                contains(&r(3, 1))
            );
        }
        Err(e) => {
            let msg = e
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default();
            println!("PANIC after {:?}: {msg}", started.elapsed());
            std::process::exit(101);
        }
    }
}
