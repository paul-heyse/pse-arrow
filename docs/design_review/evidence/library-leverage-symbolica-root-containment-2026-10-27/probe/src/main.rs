//! Probe: cross-factor interval containment in Symbolica 3.0.1
//! `isolate_real_root_intervals`, and its effect on a verbatim replica of the
//! consumer's cubic ladder (`real_points`) for p = (a x - b)^2 (g x - d).
//!
//! Modes:
//!   case A B G D      verbose trace of one polynomial
//!   poly C0 C1 C2 C3  ladder replica + diagnostics on arbitrary integer coefficients
//!   sweep LO HI       A, G in 1..=6; B, D in LO..=HI; distinct roots; one CSV row
//!                     per distinct (double root, simple root) pair, then a summary.
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use symbolica::{
    domains::rational::{Q, Rational},
    poly::{PolyVariable, univariate::UnivariatePolynomial},
    symbol,
};

static FALLBACK_TAKEN: AtomicBool = AtomicBool::new(false);
static LAST_PANIC: Mutex<Option<String>> = Mutex::new(None);

// ---------------------------------------------------------------------------
// Verbatim replica of `real_points` (pse-math implicit_cubic.rs at 9cb8e4f4a).
// Only differences: MathError -> String, and FALLBACK_TAKEN records which branch
// produced the result. Library calls, their order, the 2^-54 precision and the
// screens are unchanged.
// ---------------------------------------------------------------------------
fn real_points(
    polynomial: &UnivariatePolynomial<Q>,
    cancel: &Arc<AtomicBool>,
) -> Result<Vec<f64>, String> {
    if cancel.load(Ordering::Acquire) {
        return Err("cancelled".into());
    }
    let intervals = polynomial.isolate_real_root_intervals();
    if cancel.load(Ordering::Acquire) {
        return Err("cancelled".into());
    }
    let precision = Rational::from((1_i64, 1_i64 << 54));
    let mut points = Vec::with_capacity(intervals.len());
    let mut fallback = false;
    for (a, b, _) in intervals {
        if cancel.load(Ordering::Acquire) {
            return Err("cancelled".into());
        }
        if a == b {
            points.push(a.to_f64());
        } else if polynomial.evaluate(&a).is_zero() || polynomial.evaluate(&b).is_zero() {
            fallback = true;
            break;
        } else {
            let (lower, upper) = polynomial.refine_root_interval((a, b), &precision);
            if cancel.load(Ordering::Acquire) {
                return Err("cancelled".into());
            }
            if lower.to_f64() != upper.to_f64() {
                fallback = true;
                break;
            }
            points.push(((lower + upper) / Rational::from(2)).to_f64());
        }
    }
    if fallback || points.iter().any(|point| !point.is_finite()) {
        FALLBACK_TAKEN.store(true, Ordering::Relaxed);
        points.clear();
        let roots = polynomial.isolate_real_roots();
        if cancel.load(Ordering::Acquire) {
            return Err("cancelled".into());
        }
        for (root, _) in roots {
            points.push(root.refined(&precision).enclosure().center().re.to_f64());
            if cancel.load(Ordering::Acquire) {
                return Err("cancelled".into());
            }
        }
    }
    Ok(points)
}

/// The consumer's coefficient path: binary64 coefficients made exact, trailing
/// zeros popped, then `from_coefficients(&Q, ..)`.
fn consumer_polynomial(coefficients_f64: [f64; 4]) -> UnivariatePolynomial<Q> {
    let mut coefficients = coefficients_f64
        .into_iter()
        .map(|v| Rational::try_from(v).expect("finite coefficient"))
        .collect::<Vec<_>>();
    while coefficients.last().is_some_and(Rational::is_zero) {
        coefficients.pop();
    }
    assert!(coefficients.len() > 1);
    UnivariatePolynomial::from_coefficients(
        &Q,
        coefficients,
        Arc::new(PolyVariable::Symbol(symbol!("x"))),
    )
}


/// Diagnostic only (not part of the replica): what `refine_root_interval` returns for each
/// non-degenerate interval with non-root endpoints, and whether both ends round to one binary64.
fn diagnose(p: &UnivariatePolynomial<Q>) {
    let precision = Rational::from((1_i64, 1_i64 << 54));
    for (lo, hi, mult) in p.isolate_real_root_intervals() {
        if lo == hi || p.evaluate(&lo).is_zero() || p.evaluate(&hi).is_zero() {
            println!("  diag ({lo}, {hi}) m{mult}: degenerate or endpoint root (not refined)");
            continue;
        }
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            p.refine_root_interval((lo.clone(), hi.clone()), &precision)
        }));
        match r {
            Ok((a, b)) => println!(
                "  diag ({lo}, {hi}) m{mult}: refined to [{:?}, {:?}] (f64), agree={}",
                a.to_f64(),
                b.to_f64(),
                a.to_f64() == b.to_f64()
            ),
            Err(_) => println!(
                "  diag ({lo}, {hi}) m{mult}: panic {}",
                LAST_PANIC.lock().unwrap().clone().unwrap_or_default()
            ),
        }
    }
}

fn q(n: i64) -> Rational {
    Rational::from(n)
}

struct Case {
    a: i64,
    b: i64,
    g: i64,
    d: i64,
}

impl Case {
    /// Low-to-high integer coefficients of (a x - b)^2 (g x - d).
    fn coefficients(&self) -> [i64; 4] {
        let (a, b, g, d) = (self.a, self.b, self.g, self.d);
        [-b * b * d, 2 * a * b * d + b * b * g, -a * a * d - 2 * a * b * g, a * a * g]
    }
    fn double_root(&self) -> Rational {
        Rational::from((self.b, self.a))
    }
    fn simple_root(&self) -> Rational {
        Rational::from((self.d, self.g))
    }
    /// Square-free part u = (a x - b)(g x - d), evaluated exactly.
    fn u(&self, x: &Rational) -> Rational {
        (q(self.a) * x - &q(self.b)) * &(q(self.g) * x - &q(self.d))
    }
}

#[derive(Default)]
struct Row {
    intervals: String,
    n_intervals: usize,
    straddle0: bool,
    containment: bool,
    no_sign_change: bool,
    endpoint_root: bool,
    ladder: String,
    fallback: bool,
    points: String,
    recovered_all: bool,
    lost: usize,
    duplicated: usize,
    spurious: usize,
    iso_route_ok: bool,
}

fn near(x: f64, r: f64) -> bool {
    (x - r).abs() <= 1e-14 * r.abs().max(1.0)
}

/// Returns (all recovered, lost roots, roots returned more than once, spurious points).
fn classify(points: &[f64], roots: &[f64]) -> (bool, usize, usize, usize) {
    let counts = roots
        .iter()
        .map(|&r| points.iter().filter(|&&p| near(p, r)).count())
        .collect::<Vec<_>>();
    let spurious = points
        .iter()
        .filter(|&&p| !roots.iter().any(|&r| near(p, r)))
        .count();
    let lost = counts.iter().filter(|&&c| c == 0).count();
    let dup = counts.iter().filter(|&&c| c > 1).count();
    (lost == 0, lost, dup, spurious)
}

fn run(case: &Case, verbose: bool) -> Row {
    let mut row = Row::default();
    let coefficients = case.coefficients().map(|c| c as f64);
    let p = consumer_polynomial(coefficients);
    let roots = [case.double_root().to_f64(), case.simple_root().to_f64()];
    if verbose {
        println!(
            "p = ({}x - {})^2 ({}x - {}); coefficients low->high {:?}; roots double {} simple {}",
            case.a,
            case.b,
            case.g,
            case.d,
            case.coefficients(),
            case.double_root(),
            case.simple_root()
        );
    }

    // 1. The isolator's intervals, inspected independently of the ladder.
    let intervals = p.isolate_real_root_intervals();
    row.n_intervals = intervals.len();
    let exact_roots = [case.double_root(), case.simple_root()];
    let mut parts = vec![];
    for (lo, hi, mult) in &intervals {
        parts.push(format!("({lo},{hi};m{mult})"));
        if lo == hi {
            continue;
        }
        let zero = Rational::from(0);
        if *lo < zero && *hi > zero {
            row.straddle0 = true;
        }
        let inside = exact_roots.iter().filter(|r| *r > lo && *r < hi).count();
        if inside >= 2 {
            row.containment = true;
        }
        let ua = case.u(lo);
        let ub = case.u(hi);
        if !ua.is_zero() && !ub.is_zero() && ua.is_negative() == ub.is_negative() {
            row.no_sign_change = true;
        }
        if p.evaluate(lo).is_zero() || p.evaluate(hi).is_zero() {
            row.endpoint_root = true;
        }
        if verbose {
            println!(
                "  interval ({lo}, {hi}) mult {mult}: roots strictly inside {inside}; u(lo)={ua} u(hi)={ub}; p(lo)={} p(hi)={}",
                p.evaluate(lo),
                p.evaluate(hi)
            );
        }
    }
    row.intervals = parts.join(" ");

    // 2. The consumer ladder, verbatim.
    FALLBACK_TAKEN.store(false, Ordering::Relaxed);
    *LAST_PANIC.lock().unwrap() = None;
    let cancel = Arc::new(AtomicBool::new(false));
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| real_points(&p, &cancel)));
    row.fallback = FALLBACK_TAKEN.load(Ordering::Relaxed);
    match outcome {
        Ok(Ok(mut points)) => {
            points.sort_by(f64::total_cmp);
            let (all, lost, dup, spurious) = classify(&points, &roots);
            row.ladder = "ok".into();
            row.points = points.iter().map(|p| format!("{p:?}")).collect::<Vec<_>>().join(" ");
            row.recovered_all = all;
            row.lost = lost;
            row.duplicated = dup;
            row.spurious = spurious;
        }
        Ok(Err(e)) => row.ladder = format!("err:{e}"),
        Err(_) => {
            row.ladder = format!(
                "panic:{}",
                LAST_PANIC.lock().unwrap().clone().unwrap_or_default()
            )
        }
    }

    // 3. The ladder's certified fallback route alone (isolate_real_roots + refined).
    let precision = Rational::from((1_i64, 1_i64 << 54));
    let iso = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        p.isolate_real_roots()
            .into_iter()
            .map(|(root, _)| root.refined(&precision).enclosure().center().re.to_f64())
            .collect::<Vec<_>>()
    }));
    row.iso_route_ok = match iso {
        Ok(points) => {
            let (all, _, dup, spurious) = classify(&points, &roots);
            all && dup == 0 && spurious == 0
        }
        Err(_) => false,
    };
    if verbose {
        println!("  ladder: {} fallback={} points=[{}]", row.ladder, row.fallback, row.points);
        println!(
            "  recovered_all={} lost={} duplicated={} spurious={} ; isolate_real_roots route ok={}",
            row.recovered_all, row.lost, row.duplicated, row.spurious, row.iso_route_ok
        );
    }
    row
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_default();
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file().rsplit("/src/").next().unwrap_or(""), l.line()))
            .unwrap_or_default();
        *LAST_PANIC.lock().unwrap() = Some(format!("{message} @ {location}"));
    }));
    println!(
        "profile: {}",
        if cfg!(debug_assertions) { "debug-assertions ON" } else { "debug-assertions OFF" }
    );
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let int = |i: usize| args[i].parse::<i64>().expect("integer argument");
    match args.first().map(String::as_str) {
        Some("case") => {
            let case = Case { a: int(1), b: int(2), g: int(3), d: int(4) };
            run(&case, true);
            diagnose(&consumer_polynomial(case.coefficients().map(|c| c as f64)));
        }
        Some("poly") => {
            // Arbitrary low->high integer coefficients through the same ladder replica.
            let c = [int(1), int(2), int(3), int(4)].map(|c| c as f64);
            let p = consumer_polynomial(c);
            println!("poly {:?}: intervals {:?}", c, p.isolate_real_root_intervals());
            FALLBACK_TAKEN.store(false, Ordering::Relaxed);
            let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                real_points(&p, &Arc::new(AtomicBool::new(false)))
            }));
            println!(
                "  ladder: {:?} fallback={}",
                out.map_err(|_| LAST_PANIC.lock().unwrap().clone()),
                FALLBACK_TAKEN.load(Ordering::Relaxed)
            );
            diagnose(&p);
        }
        Some("sweep") => {
            let (lo, hi) = (int(1), int(2));
            let mut seen = std::collections::BTreeSet::new();
            println!(
                "alpha,beta,gamma,delta,r_double,r_simple,n_intervals,straddle0,containment,no_sign_change,endpoint_root,ladder,fallback,recovered_all,lost,duplicated,spurious,iso_route_ok,points,intervals"
            );
            let mut total = 0;
            let mut tally = std::collections::BTreeMap::<&str, usize>::new();
            for a in 1..=6 {
                for b in lo..=hi {
                    for g in 1..=6 {
                        for d in lo..=hi {
                            let case = Case { a, b, g, d };
                            let (r2, r1) = (case.double_root(), case.simple_root());
                            if r2 == r1 || !seen.insert((r2.to_string(), r1.to_string())) {
                                continue;
                            }
                            total += 1;
                            let row = run(&case, false);
                            println!(
                                "{a},{b},{g},{d},{},{},{},{},{},{},{},\"{}\",{},{},{},{},{},{},\"{}\",\"{}\"",
                                r2, r1, row.n_intervals, row.straddle0, row.containment,
                                row.no_sign_change, row.endpoint_root, row.ladder, row.fallback,
                                row.recovered_all, row.lost, row.duplicated, row.spurious,
                                row.iso_route_ok, row.points, row.intervals
                            );
                            let flags: [(&str, bool); 12] = [
                                ("straddle0", row.straddle0),
                                ("containment (>=2 roots strictly inside)", row.containment),
                                ("no sign change of u across a non-degenerate interval", row.no_sign_change),
                                ("endpoint is a root", row.endpoint_root),
                                ("intervals != 2", row.n_intervals != 2),
                                ("ladder panicked", row.ladder.starts_with("panic")),
                                ("ladder returned Err", row.ladder.starts_with("err")),
                                ("ladder took fallback", row.fallback),
                                ("ladder ok but lost >=1 root", row.ladder == "ok" && row.lost > 0),
                                ("ladder ok but duplicated a root", row.ladder == "ok" && row.duplicated > 0),
                                ("ladder ok but spurious point", row.ladder == "ok" && row.spurious > 0),
                                ("isolate_real_roots route NOT ok", !row.iso_route_ok),
                            ];
                            for (name, on) in flags {
                                if on {
                                    *tally.entry(name).or_default() += 1;
                                }
                            }
                            if row.containment {
                                *tally
                                    .entry(if row.ladder.starts_with("panic") {
                                        "containment AND ladder panicked"
                                    } else if row.ladder == "ok" && !row.recovered_all {
                                        "containment AND ladder lost a root"
                                    } else {
                                        "containment AND ladder recovered all"
                                    })
                                    .or_default() += 1;
                            }
                        }
                    }
                }
            }
            println!("# SUMMARY distinct (double, simple) root pairs: {total}");
            for (name, count) in tally {
                if count > 0 {
                    println!("# {name}: {count}");
                }
            }
        }
        _ => panic!("usage: case A B G D | sweep LO HI | poly C0 C1 C2 C3"),
    }
}
