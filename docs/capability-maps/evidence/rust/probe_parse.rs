// PROBE 7 — the authoring boundary: do the parsers give §23.2 a source span, or a panic?
//   serde-saphyr is the recommended serde_yaml replacement (open question Q1);
//   toml::Spanned is the capability the map calls "free and the blueprint does not claim it".
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Pkg { id: String, version: u32 }

#[derive(Debug, Deserialize)]
struct SpannedPkg { id: toml::Spanned<String>, version: toml::Spanned<u32> }

fn probe_toml() {
    println!("== PROBE 7a: toml::Spanned — byte offsets back to the source");
    let src = "id = \"pkg.demo\"\nversion = 3\n";
    match toml::from_str::<SpannedPkg>(src) {
        Ok(p) => {
            println!("   id      span = {:?}  value = {:?}", p.id.span(), p.id.get_ref());
            println!("   version span = {:?}  value = {:?}", p.version.span(), p.version.get_ref());
            let s = p.version.span();
            println!("   source slice at the version span : {:?}", &src[s.start..s.end]);
        }
        Err(e) => println!("   FAILED: {e}"),
    }
    println!("   -- and what does a type error look like?");
    let bad = "id = \"pkg.demo\"\nversion = \"three\"\n";
    match toml::from_str::<Pkg>(bad) {
        Ok(_) => println!("   unexpectedly accepted"),
        Err(e) => {
            println!("   error span   : {:?}", e.span());
            println!("   error message: {}", e.message());
            println!("   full display :\n{}", e.to_string().lines().map(|l| format!("      {l}")).collect::<Vec<_>>().join("\n"));
        }
    }
}

fn probe_saphyr() {
    println!("== PROBE 7b: serde-saphyr — the recommended serde_yaml replacement");
    let good = "id: pkg.demo\nversion: 3\n";
    match serde_saphyr::from_str::<Pkg>(good) {
        Ok(p) => println!("   parsed ok : {p:?}"),
        Err(e) => println!("   FAILED on valid input: {e}"),
    }
    let bad = "id: pkg.demo\nversion: three\n";
    match serde_saphyr::from_str::<Pkg>(bad) {
        Ok(p) => println!("   type error ACCEPTED (bad): {p:?}"),
        Err(e) => {
            let s = e.to_string();
            println!("   type error rejected. message:");
            for l in s.lines().take(8) { println!("      {l}"); }
            let has_pos = s.contains("line") || s.contains("column") || s.contains(':');
            println!("   message carries a position : {has_pos}");
        }
    }
    // The failure mode that matters at an authoring boundary: does malformed input panic?
    println!("   -- malformed / hostile input --");
    for (name, doc) in [
        ("unterminated flow seq", "id: [a, b\n"),
        ("tab indentation",       "id: x\n\tversion: 1\n"),
        ("duplicate key",         "id: a\nid: b\nversion: 1\n"),
        ("deep alias nest",       &format!("id: x\nversion: 1\nx: {}\n", "[".repeat(200))),
    ] {
        let r = std::panic::catch_unwind(|| serde_saphyr::from_str::<Pkg>(doc));
        match r {
            Ok(Ok(_))  => println!("   {name:<22} -> accepted"),
            Ok(Err(_)) => println!("   {name:<22} -> typed Err (no panic)"),
            Err(_)     => println!("   {name:<22} -> PANICKED"),
        }
    }
}

fn main() {
    probe_toml(); println!();
    probe_saphyr();
}
