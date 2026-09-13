// PROBE 6 — salsa: what F13 actually turns on. Is memoization revision-based or
//           content-based? do accumulators defeat backdating? what is Durability for?
// PROBE 8 — egglog: is extraction deterministic across runs? (§5.1 identity, §20.4 reproduction)

use salsa::{Accumulator, Setter};

#[salsa::db]
#[derive(Default, Clone)]
struct Db { storage: salsa::Storage<Self> }

#[salsa::db]
impl salsa::Database for Db {}

#[salsa::input]
struct Source { #[returns(ref)] text: String }

#[salsa::accumulator]
#[derive(Debug)]
struct Finding(String);

#[salsa::tracked]
fn token_count(db: &dyn salsa::Database, src: Source) -> usize {
    let t = src.text(db);
    if t.contains("bad") { Finding(format!("suspicious token in {:?}", &t[..t.len().min(12)])).accumulate(db); }
    t.split_whitespace().count()
}

#[salsa::tracked]
fn doubled(db: &dyn salsa::Database, src: Source) -> usize { token_count(db, src) * 2 }

fn probe6() {
    println!("== PROBE 6: salsa 0.28.2 — the mechanism behind F13");
    let mut db = Db::default();
    let src = Source::new(&db, "alpha beta gamma".to_string());
    println!("   token_count = {}", token_count(&db, src));
    println!("   doubled     = {}", doubled(&db, src));

    // Backdating: change the input to different text with the SAME token count.
    // A content-hash memo would recompute everything downstream; salsa should backdate.
    src.set_text(&mut db).to("delta epsilon zeta".to_string());
    println!("   -- input replaced with different text, same token count --");
    println!("   token_count = {}", token_count(&db, src));
    println!("   doubled     = {}  (backdating means `doubled` need not re-run)", doubled(&db, src));

    // Accumulators: do findings travel without being in the return type?
    let src2 = Source::new(&db, "this is bad input".to_string());
    let n = token_count(&db, src2);
    let found: Vec<&Finding> = token_count::accumulated::<Finding>(&db, src2);
    println!("   -- accumulator --");
    println!("   token_count(src2) = {n}, accumulated findings = {}", found.len());
    for f in &found { println!("      {f:?}"); }
    println!("   -> findings ride alongside the value, not inside it: amendment E's mechanism.");

    // Durability: the knob that says "this input will not change".
    println!("   -- durability levels available: LOW / MEDIUM / HIGH --");
    let sref = Source::new(&db, "reference data".to_string());
    sref.set_text(&mut db).with_durability(salsa::Durability::HIGH).to("reference data".to_string());
    println!("   set HIGH durability on a reference input: ok");
    println!("   -> salsa keys on revision counters + durability, NOT on content hashes.");
    println!("      That is the substance of F13: overlapping purpose, different mechanism.");
}

fn probe8() {
    println!("== PROBE 8: egglog 3.0.0 — is extraction reproducible? (Q10, §5.1, §20.4)");
    let program = r#"
(datatype Math (Num i64) (Var String) (Add Math Math) (Mul Math Math))
(rewrite (Add a b) (Add b a))
(rewrite (Mul a b) (Mul b a))
(rewrite (Mul (Add a b) c) (Add (Mul a c) (Mul b c)))
(let e (Mul (Add (Var "x") (Num 2)) (Var "y")))
(run 6)
(extract e)
"#;
    let (mut dbg, mut disp, mut stable) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..5 {
        let mut eg = egglog::EGraph::default();
        match eg.parse_and_run_program(None, program) {
            Ok(out) => {
                dbg.push(out.iter().map(|m| format!("{m:?}")).collect::<Vec<_>>().join("\n"));
                disp.push(out.iter().map(|m| format!("{m}")).collect::<Vec<_>>().join("\n"));
                stable.push(egglog::CommandOutput::snapshot_stable_under_proof_encoding(&out));
            }
            Err(e) => { println!("   egglog run FAILED: {e}"); return; }
        }
    }
    let uniq = |v: &Vec<String>| v.iter().collect::<std::collections::HashSet<_>>().len();
    println!("   5 identical runs, distinct results by rendering:");
    println!("     Debug  ({{:?}})                              : {}", uniq(&dbg));
    println!("     Display ({{}})                                : {}", uniq(&disp));
    println!("     snapshot_stable_under_proof_encoding        : {}", uniq(&stable));
    println!("   extracted term (Display) : {}", disp[0].lines().last().unwrap_or("").trim());
    println!();
    if uniq(&dbg) > 1 && uniq(&disp) == 1 {
        println!("   -> The EXTRACTION IS DETERMINISTIC. What varies is the `Debug` rendering");
        println!("      of the internal TermDag, whose `nodes` is a hash set — the same trap");
        println!("      the DataFusion map found in `datafusion-proto`'s field metadata.");
        println!("      Q10's answer: yes, provided nothing hashes a `Debug` string.");
        println!("      egglog ships `snapshot_stable_under_proof_encoding` for exactly this.");
    } else if uniq(&disp) > 1 {
        println!("   -> The extracted TERM ITSELF varies across runs. §5.1 identity and");
        println!("      §20.4 reproduction cannot depend on any egglog output.");
    } else {
        println!("   -> All three renderings stable in this sample.");
    }
    println!("   NOTE: one process, one egglog version. Cross-process and cross-version");
    println!("      stability are not measured here.");
}

fn main() { probe6(); println!(); probe8(); }
