//! Render `rustdoc_types` model fragments back into readable Rust source text.
//!
//! Fidelity target: what a human would write in a trait definition or an `impl`
//! header. Paths are emitted as rustdoc records them (already the resolved
//! public-ish path), not re-qualified, so the text matches upstream source.

use rustdoc_types::*;

pub fn abi(a: &Abi) -> Option<String> {
    let s = match a {
        Abi::Rust => return None,
        Abi::C { unwind } => n("C", *unwind),
        Abi::Cdecl { unwind } => n("cdecl", *unwind),
        Abi::Stdcall { unwind } => n("stdcall", *unwind),
        Abi::Fastcall { unwind } => n("fastcall", *unwind),
        Abi::Aapcs { unwind } => n("aapcs", *unwind),
        Abi::Win64 { unwind } => n("win64", *unwind),
        Abi::SysV64 { unwind } => n("sysv64", *unwind),
        Abi::System { unwind } => n("system", *unwind),
        Abi::Other(o) => o.clone(),
    };
    Some(format!("extern \"{s}\" "))
}
fn n(base: &str, unwind: bool) -> String {
    if unwind { format!("{base}-unwind") } else { base.to_string() }
}

pub fn ty(t: &Type) -> String {
    match t {
        Type::ResolvedPath(p) => path(p),
        Type::DynTrait(d) => {
            let mut parts: Vec<String> = d.traits.iter().map(poly_trait).collect();
            if let Some(lt) = &d.lifetime {
                parts.push(lt.clone());
            }
            format!("dyn {}", parts.join(" + "))
        }
        Type::Generic(g) => g.clone(),
        Type::Primitive(p) => p.clone(),
        Type::FunctionPointer(f) => {
            let hdr = fn_header(&f.header);
            let args: Vec<String> = f.sig.inputs.iter().map(|(_, t)| ty(t)).collect();
            let out = f.sig.output.as_ref().map(|o| format!(" -> {}", ty(o))).unwrap_or_default();
            format!("{hdr}fn({}){out}", args.join(", "))
        }
        Type::Tuple(ts) => {
            if ts.is_empty() {
                "()".into()
            } else if ts.len() == 1 {
                format!("({},)", ty(&ts[0]))
            } else {
                format!("({})", ts.iter().map(ty).collect::<Vec<_>>().join(", "))
            }
        }
        Type::Slice(s) => format!("[{}]", ty(s)),
        Type::Array { type_, len } => format!("[{}; {}]", ty(type_), len),
        Type::Pat { type_, .. } => ty(type_),
        Type::ImplTrait(b) => format!("impl {}", bounds(b, " + ")),
        Type::Infer => "_".into(),
        Type::RawPointer { is_mutable, type_ } => {
            format!("*{} {}", if *is_mutable { "mut" } else { "const" }, ty(type_))
        }
        Type::BorrowedRef { lifetime, is_mutable, type_ } => {
            let lt = lifetime.as_ref().map(|l| format!("{l} ")).unwrap_or_default();
            let m = if *is_mutable { "mut " } else { "" };
            format!("&{lt}{m}{}", ty(type_))
        }
        Type::QualifiedPath { name, args, self_type, trait_ } => {
            let a = args.as_deref().map(generic_args).unwrap_or_default();
            match trait_ {
                // rustdoc emits a `trait_` with an empty path for an inherent
                // associated item; `<F as >::Item` is not valid Rust, `F::Item` is.
                Some(tr) if !tr.path.is_empty() => {
                    format!("<{} as {}>::{}{}", ty(self_type), path(tr), name, a)
                }
                _ => format!("{}::{}{}", ty(self_type), name, a),
            }
        }
    }
}

pub fn path(p: &Path) -> String {
    format!("{}{}", p.path, p.args.as_deref().map(generic_args).unwrap_or_default())
}

fn poly_trait(pt: &PolyTrait) -> String {
    let hrtb = hrtb(&pt.generic_params);
    format!("{hrtb}{}", path(&pt.trait_))
}

fn hrtb(params: &[GenericParamDef]) -> String {
    let lts: Vec<String> = params
        .iter()
        .filter(|p| matches!(p.kind, GenericParamDefKind::Lifetime { .. }))
        .map(|p| p.name.clone())
        .collect();
    if lts.is_empty() { String::new() } else { format!("for<{}> ", lts.join(", ")) }
}

pub fn generic_args(a: &GenericArgs) -> String {
    match a {
        GenericArgs::AngleBracketed { args, constraints } => {
            let mut out: Vec<String> = args.iter().map(generic_arg).collect();
            out.extend(constraints.iter().map(constraint));
            if out.is_empty() { String::new() } else { format!("<{}>", out.join(", ")) }
        }
        GenericArgs::Parenthesized { inputs, output } => {
            let i = inputs.iter().map(ty).collect::<Vec<_>>().join(", ");
            match output {
                Some(o) => format!("({i}) -> {}", ty(o)),
                None => format!("({i})"),
            }
        }
        GenericArgs::ReturnTypeNotation => "(..)".into(),
    }
}

fn generic_arg(a: &GenericArg) -> String {
    match a {
        GenericArg::Lifetime(l) => l.clone(),
        GenericArg::Type(t) => ty(t),
        GenericArg::Const(c) => c.expr.clone(),
        GenericArg::Infer => "_".into(),
    }
}

fn constraint(c: &AssocItemConstraint) -> String {
    let a = c.args.as_deref().map(generic_args).unwrap_or_default();
    match &c.binding {
        AssocItemConstraintKind::Equality(t) => format!("{}{a} = {}", c.name, term(t)),
        AssocItemConstraintKind::Constraint(b) => format!("{}{a}: {}", c.name, bounds(b, " + ")),
    }
}

fn term(t: &Term) -> String {
    match t {
        Term::Type(x) => ty(x),
        Term::Constant(c) => c.expr.clone(),
    }
}

pub fn bound(b: &GenericBound) -> String {
    match b {
        GenericBound::TraitBound { trait_, generic_params, modifier } => {
            let m = match modifier {
                TraitBoundModifier::None => "",
                TraitBoundModifier::Maybe => "?",
                TraitBoundModifier::MaybeConst => "~const ",
            };
            format!("{}{m}{}", hrtb(generic_params), path(trait_))
        }
        GenericBound::Outlives(l) => l.clone(),
        GenericBound::Use(args) => {
            let a: Vec<String> = args
                .iter()
                .map(|p| match p {
                    PreciseCapturingArg::Lifetime(l) => l.clone(),
                    PreciseCapturingArg::Param(s) => s.clone(),
                })
                .collect();
            format!("use<{}>", a.join(", "))
        }
    }
}

pub fn bounds(b: &[GenericBound], sep: &str) -> String {
    b.iter().map(bound).collect::<Vec<_>>().join(sep)
}

/// `<T: Bound, 'a, const N: usize>` — empty string when there are no params.
pub fn generic_params(g: &Generics) -> String {
    let parts: Vec<String> = g
        .params
        .iter()
        .filter(|p| !synthetic(p))
        .map(|p| match &p.kind {
            GenericParamDefKind::Lifetime { outlives } => {
                if outlives.is_empty() {
                    p.name.clone()
                } else {
                    format!("{}: {}", p.name, outlives.join(" + "))
                }
            }
            GenericParamDefKind::Type { bounds: b, default, .. } => {
                let mut s = p.name.clone();
                if !b.is_empty() {
                    s.push_str(&format!(": {}", bounds(b, " + ")));
                }
                if let Some(d) = default {
                    s.push_str(&format!(" = {}", ty(d)));
                }
                s
            }
            GenericParamDefKind::Const { type_, default } => {
                let mut s = format!("const {}: {}", p.name, ty(type_));
                if let Some(d) = default {
                    s.push_str(&format!(" = {d}"));
                }
                s
            }
        })
        .collect();
    if parts.is_empty() { String::new() } else { format!("<{}>", parts.join(", ")) }
}

fn synthetic(p: &GenericParamDef) -> bool {
    matches!(p.kind, GenericParamDefKind::Type { is_synthetic: true, .. })
}

/// ` where A: B, 'a: 'b` — empty string when there are no predicates.
pub fn where_clause(g: &Generics) -> String {
    let parts: Vec<String> = g
        .where_predicates
        .iter()
        .map(|w| match w {
            WherePredicate::BoundPredicate { type_, bounds: b, generic_params: gp } => {
                format!("{}{}: {}", hrtb(gp), ty(type_), bounds(b, " + "))
            }
            WherePredicate::LifetimePredicate { lifetime, outlives } => {
                if outlives.is_empty() {
                    lifetime.clone()
                } else {
                    format!("{}: {}", lifetime, outlives.join(" + "))
                }
            }
            WherePredicate::EqPredicate { lhs, rhs } => format!("{} == {}", ty(lhs), term(rhs)),
        })
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() { String::new() } else { format!(" where {}", parts.join(", ")) }
}

pub fn fn_header(h: &FunctionHeader) -> String {
    let mut s = String::new();
    if h.is_const { s.push_str("const "); }
    if h.is_async { s.push_str("async "); }
    if h.is_unsafe { s.push_str("unsafe "); }
    if let Some(a) = abi(&h.abi) { s.push_str(&a); }
    s
}

/// Full `fn` signature as written in source, without a body.
pub fn function(name: &str, f: &Function) -> String {
    let args: Vec<String> = f
        .sig
        .inputs
        .iter()
        .map(|(n, t)| {
            // rustdoc encodes the receiver as a parameter literally named "self".
            if n == "self" {
                match t {
                    Type::BorrowedRef { lifetime, is_mutable, type_ }
                        if matches!(**type_, Type::Generic(ref g) if g == "Self") =>
                    {
                        let lt = lifetime.as_ref().map(|l| format!("{l} ")).unwrap_or_default();
                        let m = if *is_mutable { "mut " } else { "" };
                        format!("&{lt}{m}self")
                    }
                    Type::Generic(g) if g == "Self" => "self".to_string(),
                    other => format!("self: {}", ty(other)),
                }
            } else {
                format!("{n}: {}", ty(t))
            }
        })
        .collect();
    let mut args = args;
    if f.sig.is_c_variadic {
        args.push("...".into());
    }
    let out = f.sig.output.as_ref().map(|o| format!(" -> {}", ty(o))).unwrap_or_default();
    format!(
        "{}fn {}{}({}){}{}",
        fn_header(&f.header),
        name,
        generic_params(&f.generics),
        args.join(", "),
        out,
        where_clause(&f.generics)
    )
}

/// `impl<T> Trait<T> for Type` / `impl Type` header text.
pub fn impl_header(i: &Impl) -> String {
    let un = if i.is_unsafe { "unsafe " } else { "" };
    let neg = if i.is_negative { "!" } else { "" };
    match &i.trait_ {
        Some(t) => format!(
            "{un}impl{} {neg}{} for {}{}",
            generic_params(&i.generics),
            path(t),
            ty(&i.for_),
            where_clause(&i.generics)
        ),
        None => format!(
            "{un}impl{} {}{}",
            generic_params(&i.generics),
            ty(&i.for_),
            where_clause(&i.generics)
        ),
    }
}

/// `pub trait Name<T>: Super where ...` header text.
pub fn trait_header(name: &str, t: &Trait) -> String {
    let un = if t.is_unsafe { "unsafe " } else { "" };
    let au = if t.is_auto { "auto " } else { "" };
    let sup = if t.bounds.is_empty() {
        String::new()
    } else {
        format!(": {}", bounds(&t.bounds, " + "))
    };
    format!(
        "{un}{au}trait {}{}{}{}",
        name,
        generic_params(&t.generics),
        sup,
        where_clause(&t.generics)
    )
}

// ---------------------------------------------------------------------------
// `#[async_trait]` re-sugaring
//
// The macro rewrites `async fn f(&self) -> T` into
// `fn f<'life0, 'async_trait>(&'life0 self) -> Pin<Box<dyn Future<Output = T> + Send + 'async_trait>>`
// plus a pile of `'life0: 'async_trait` predicates. rustdoc faithfully records
// the rewritten form, but that is a macro artifact, not the contract an
// implementor writes. Restore the source form.
// ---------------------------------------------------------------------------

fn is_synthetic_lifetime(lt: &str) -> bool {
    lt == "'async_trait" || (lt.starts_with("'life") && lt[5..].chars().all(|c| c.is_ascii_digit()))
}

/// Extract `X` from `...Future<Output = X> + ...`, honouring nesting.
fn future_output(s: &str) -> Option<String> {
    let start = s.find("Future<Output = ")? + "Future<Output = ".len();
    let bytes = s.as_bytes();
    let (mut depth, mut i) = (0i32, start);
    while i < bytes.len() {
        match bytes[i] {
            b'<' | b'(' | b'[' => depth += 1,
            b'>' | b')' | b']' => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            b'+' if depth == 0 => break,
            _ => {}
        }
        i += 1;
    }
    Some(s[start..i].trim().to_string())
}

pub fn desugar_async_trait(sig: &str) -> String {
    let looks_async = sig.contains("'async_trait") && sig.contains("Future<Output = ");
    if !looks_async {
        return sig.to_string();
    }
    let Some(arrow) = sig.find(" -> ") else { return sig.to_string() };
    let (head, tail) = sig.split_at(arrow);
    let ret_and_where = &tail[4..];
    let (ret, _) = match ret_and_where.find(" where ") {
        Some(i) => (&ret_and_where[..i], &ret_and_where[i..]),
        None => (ret_and_where, ""),
    };
    let Some(out) = future_output(ret) else { return sig.to_string() };

    // Drop synthetic lifetimes from the generic parameter list.
    let mut head = head.to_string();
    if let (Some(a), Some(b)) = (head.find('<'), head.find('(')) {
        if a < b {
            let close = head[..b].rfind('>').unwrap_or(a);
            let kept: Vec<&str> = head[a + 1..close]
                .split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty() && !is_synthetic_lifetime(p))
                .collect();
            let repl = if kept.is_empty() { String::new() } else { format!("<{}>", kept.join(", ")) };
            head = format!("{}{}{}", &head[..a], repl, &head[close + 1..]);
        }
    }
    // Drop synthetic lifetimes from parameter types: `&'life0 self` -> `&self`.
    for lt in ["'async_trait "] {
        head = head.replace(lt, "");
    }
    let mut cleaned = String::with_capacity(head.len());
    let mut rest = head.as_str();
    while let Some(p) = rest.find("'life") {
        let (a, b) = rest.split_at(p);
        cleaned.push_str(a);
        let digits = b[5..].chars().take_while(|c| c.is_ascii_digit()).count();
        rest = &b[5 + digits..];
        rest = rest.strip_prefix(' ').unwrap_or(rest);
    }
    cleaned.push_str(rest);

    let out = if out == "()" { String::new() } else { format!(" -> {out}") };
    format!("async {}{}", cleaned.trim_start_matches("async "), out)
}

