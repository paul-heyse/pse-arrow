//! Bit-exact record of egobox-doe designs for fixed seeds.
use egobox_doe::{FullFactorial, Lhs, LhsKind, Random, SamplingMethod};
use ndarray::{Array2, arr2};
use rand_xoshiro::Xoshiro256Plus;
use rand_xoshiro::rand_core::SeedableRng;

fn fnv1a(a: &Array2<f64>) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for v in a.iter() {
        for b in v.to_bits().to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
    }
    h
}

fn report(label: &str, a: &Array2<f64>, dump: bool) {
    println!("{label} shape={:?} fnv1a64={:016x}", a.shape(), fnv1a(a));
    if dump {
        for row in a.rows() {
            let bits: Vec<String> = row.iter().map(|v| format!("{:016x}", v.to_bits())).collect();
            println!("  {}", bits.join(" "));
        }
    }
}

fn main() {
    let small = arr2(&[[0.0, 1.0], [-5.0, 5.0], [100.0, 200.0]]);
    let wide = arr2(&[
        [0.0, 1.0], [-5.0, 5.0], [100.0, 200.0],
        [1e-3, 2e-3], [-1.0, 0.0], [10.0, 11.0],
    ]);
    for seed in [42_u64, 7] {
        for (name, kind) in [
            ("classic", LhsKind::Classic),
            ("centered", LhsKind::Centered),
            ("maximin", LhsKind::Maximin),
            ("centered_maximin", LhsKind::CenteredMaximin),
            ("optimized", LhsKind::Optimized),
        ] {
            let d = Lhs::new(&small)
                .kind(kind)
                .with_rng(Xoshiro256Plus::seed_from_u64(seed))
                .sample(10);
            report(&format!("seed={seed} lhs-{name} nx=3 ns=10"), &d, seed == 42);
            let d = Lhs::new(&wide)
                .kind(kind)
                .with_rng(Xoshiro256Plus::seed_from_u64(seed))
                .sample(40);
            report(&format!("seed={seed} lhs-{name} nx=6 ns=40"), &d, false);
        }
        let r = Random::new(&small).with_rng(Xoshiro256Plus::seed_from_u64(seed)).sample(10);
        report(&format!("seed={seed} random nx=3 ns=10"), &r, seed == 42);
    }
    let f = FullFactorial::new(&small).sample(10);
    report("fullfactorial nx=3 ns=10", &f, true);
    // Same seed, two sample() calls on one instance: second continues the stream.
    let lhs = Lhs::new(&small).kind(LhsKind::Classic).with_rng(Xoshiro256Plus::seed_from_u64(42));
    let a = lhs.sample(10);
    let b = lhs.sample(10);
    println!("stateful-continuation equal={}", a == b);
    // Entropy-seeded constructor without with_rng is not reproducible.
    let e1 = Lhs::new(&small).kind(LhsKind::Classic).sample(10);
    let e2 = Lhs::new(&small).kind(LhsKind::Classic).sample(10);
    println!("entropy-default equal={}", e1 == e2);
}
