// PROBE 4 — num-dual 0.15: does `implicit_derivative` give §9.8/§18.2 the derivative of an
//           implicitly-defined property (the cubic-EOS compressibility root)?
// PROBE 5 — faer 0.24: does `matrix_free` answer §15.5 / §26 F17's "iterative for large" SVD,
//           and does it give the SMALLEST singular values, which is what §15.5 actually asks for?
use num_dual::*;

fn residual<D: DualNum<Primitive = f64> + Copy>(z: D, a: D, b: f64) -> D {
    // NOTE: in num-dual 0.14 this bound is written `D: DualNum<f64>`; in 0.15 the scalar
    // moved to an associated type, so the SOURCE of every kernel body differs between pins.
    z * z * z - z * z + (a - D::from(b) - D::from(b * b)) * z - a * D::from(b)
}

fn probe4() {
    println!("== PROBE 4: num-dual — derivative of an implicitly defined root");
    let b = 0.02_f64;
    let a0 = 0.30_f64;

    // Newton-solve the vapour root in plain f64.
    let mut z = 1.0_f64;
    for _ in 0..100 {
        let f = residual(z, a0, b);
        let df = 3.0 * z * z - 2.0 * z + (a0 - b - b * b);
        if df.abs() < 1e-15 { break; }
        z -= f / df;
    }
    println!("   solved Z(A0)           = {z:.12}   residual = {:.3e}", residual(z, a0, b));

    // dZ/dA by the implicit function theorem, written out by hand — what the platform
    // would otherwise implement per implicit kernel.
    let df_dz = 3.0 * z * z - 2.0 * z + (a0 - b - b * b);
    let df_da = z - b;
    let hand = -df_da / df_dz;
    println!("   dZ/dA by hand (IFT)    = {hand:.12}");

    // The same, from the library.
    // `args` must be the ALREADY-SEEDED dual; the closure receives it pre-lifted.
    let a_seeded = Dual64::from(a0).derivative();
    let d: Dual64 = implicit_derivative(
        |zz: Dual<Dual64>, aa: &Dual<Dual64>| residual(zz, *aa, b),
        z,
        &a_seeded,
    );
    println!("   implicit_derivative    = {:.12}  (value {:.12})", d.eps, d.re);
    println!("   agrees with hand IFT   : {}", (d.eps - hand).abs() < 1e-9);
    println!("   value matches the root : {}", (d.re - z).abs() < 1e-9);

    // The ordinary forward-mode primitives §18.2 step 5 relies on.
    let (v, g): (f64, f64) = first_derivative(|x: Dual64| x * x * x - x * x, z);
    let (_v2, _g2, h): (f64, f64, f64) = second_derivative(|x: Dual2_64| x * x * x - x * x, z);
    println!("   first_derivative       : f={v:.6} f'={g:.6}");
    println!("   second_derivative f''  : {h:.6}   <- §18.2 step 5's kernel second derivatives");
}

#[derive(Debug)]
struct DenseOp { a: faer::Mat<f64> }

impl faer::matrix_free::LinOp<f64> for DenseOp {
    fn apply_scratch(&self, _rhs_ncols: usize, _par: faer::Par) -> faer::dyn_stack::StackReq {
        faer::dyn_stack::StackReq::EMPTY
    }
    fn nrows(&self) -> usize { self.a.nrows() }
    fn ncols(&self) -> usize { self.a.ncols() }
    fn apply(&self, out: faer::mat::MatMut<'_, f64>, rhs: faer::mat::MatRef<'_, f64>, par: faer::Par, _s: &mut faer::dyn_stack::MemStack) {
        faer::linalg::matmul::matmul(out, faer::Accum::Replace, self.a.as_ref(), rhs, 1.0, par);
    }
    fn conj_apply(&self, out: faer::mat::MatMut<'_, f64>, rhs: faer::mat::MatRef<'_, f64>, par: faer::Par, s: &mut faer::dyn_stack::MemStack) {
        self.apply(out, rhs, par, s)
    }
}
impl faer::matrix_free::BiLinOp<f64> for DenseOp {
    fn transpose_apply_scratch(&self, _rhs_ncols: usize, _par: faer::Par) -> faer::dyn_stack::StackReq {
        faer::dyn_stack::StackReq::EMPTY
    }
    fn transpose_apply(&self, out: faer::mat::MatMut<'_, f64>, rhs: faer::mat::MatRef<'_, f64>, par: faer::Par, _s: &mut faer::dyn_stack::MemStack) {
        faer::linalg::matmul::matmul(out, faer::Accum::Replace, self.a.as_ref().transpose(), rhs, 1.0, par);
    }
    fn adjoint_apply(&self, out: faer::mat::MatMut<'_, f64>, rhs: faer::mat::MatRef<'_, f64>, par: faer::Par, s: &mut faer::dyn_stack::MemStack) {
        self.transpose_apply(out, rhs, par, s)
    }
}

fn probe5() {
    use faer::dyn_stack::{MemBuffer, MemStack, StackReq};
    use faer::matrix_free::{BiLinOp, LinOp};
    println!("== PROBE 5: faer matrix_free — the iterative route for §15.5 / §26 F17");

    let n = 24usize; let m = 20usize;
    let diag: Vec<f64> = (0..m).map(|j| 10f64.powi(3 - j as i32)).collect(); // 1e3 .. 1e-16
    let a = faer::Mat::from_fn(n, m, |i, j| {
        if i == j { diag[j] } else { 0.01 * ((i + 2 * j) as f64).sin() }
    });
    let op = DenseOp { a: a.clone() };

    // 1. Does the BiLinOp integration actually work? Apply the operator both ways and
    //    check against a direct product. This is the part the platform would implement
    //    for a sparse Jacobian it never materialises.
    let x = faer::Mat::from_fn(m, 1, |i, _| 1.0 + (i as f64) * 0.25);
    let mut y = faer::Mat::<f64>::zeros(n, 1);
    let mut buf = MemBuffer::new(StackReq::EMPTY);
    op.apply(y.as_mut(), x.as_ref(), faer::Par::Seq, MemStack::new(&mut buf));
    let y_direct = &a * &x;
    let err: f64 = (0..n).map(|i| (y[(i, 0)] - y_direct[(i, 0)]).abs()).fold(0.0, f64::max);
    println!("   LinOp::apply matches a direct product      : max|diff| = {err:.3e}");

    let mut z = faer::Mat::<f64>::zeros(m, 1);
    let yv = faer::Mat::from_fn(n, 1, |i, _| 1.0 / (1.0 + i as f64));
    op.transpose_apply(z.as_mut(), yv.as_ref(), faer::Par::Seq, MemStack::new(&mut buf));
    let z_direct = a.as_ref().transpose() * &yv;
    let terr: f64 = (0..m).map(|i| (z[(i, 0)] - z_direct[(i, 0)]).abs()).fold(0.0, f64::max);
    println!("   BiLinOp::transpose_apply matches A^T * y   : max|diff| = {terr:.3e}");
    println!("   -> the matrix-free operator contract is 5 + 3 methods and it works;");
    println!("      a sparse Jacobian never has to be materialised to use it.");

    // 2. Which END of the spectrum does partial_svd target? This is what decides Q7.
    println!("   faer's own documentation for `partial_svd` says it computes");
    println!("   \"the singular values ... with the LARGEST magnitude\".");
    println!("   §15.5 asks for the SMALLEST k singular values (rank deficiency,");
    println!("   condition number) — the opposite end of the spectrum.");
    println!("   test matrix diagonal spans {:.0e} down to {:.0e}", diag[0], diag[m - 1]);
    println!("   -> partial_svd alone does NOT answer §26 F17. Reaching the smallest end");
    println!("      with a Krylov method needs shift-invert, i.e. applying A^-1, which");
    println!("      requires a sparse factorisation — faer's sparse LU. That combination");
    println!("      is the route, and it is two components, not one.");
    println!("   NOTE: partial_svd was not executed here. Its dimension preconditions");
    println!("      (`max_dim < min(m, n)` against a `v0` of length ncols) were not");
    println!("      satisfiable in this harness; recorded as [UNVERIFIED] rather than");
    println!("      guessed at. The directional finding rests on the crate's own docs.");
}

fn main() { probe4(); println!(); probe5(); }
