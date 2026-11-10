//! Borrowed controls on the shared direct translator and its actual workers.
use super::*;
use crate::coefficient::{SerializedFloat, SerializedRational};
use crate::state::{Workspace, WorkspaceObserver};
use numerica::domains::backend::integer::IntegerAllocationContext;

fn add<E>(a: usize, b: usize) -> Result<usize, OperationError<E>> {
    a.checked_add(b).ok_or(OperationError::Overflow)
}
fn mul<E>(a: usize, b: usize) -> Result<usize, OperationError<E>> {
    a.checked_mul(b).ok_or(OperationError::Overflow)
}
fn limbs<E>(bits: usize) -> Result<usize, OperationError<E>> {
    // Native fresh GMP destinations round to limbs; arithmetic destinations can
    // retain the carry limb. Old/new destinations coexist in the working bound.
    mul(add(add(bits, 63)? / 64, 2)?, size_of::<u64>())
}

/// Common-denominator complex-rational height, including unreduced intermediates.
#[derive(Clone, Copy)]
struct Height {
    a: usize,
    d: usize,
}
fn product_bits<E>(a: usize, b: usize) -> Result<usize, OperationError<E>> {
    // A nonnegative integer with at most one bit is exactly zero or one.
    // In particular an at-most-one-bit positive denominator is exactly one.
    if a == 0 || b == 0 {
        Ok(0)
    } else if a == 1 {
        Ok(b)
    } else if b == 1 {
        Ok(a)
    } else {
        add(a, b)
    }
}
impl Height {
    const ONE: Self = Self { a: 1, d: 1 };
    fn sum<E>(self, other: Self) -> Result<Self, OperationError<E>> {
        Ok(Self {
            a: add(
                product_bits(self.a, other.d)?.max(product_bits(other.a, self.d)?),
                1,
            )?,
            d: product_bits(self.d, other.d)?,
        })
    }
    fn product<E>(self, other: Self) -> Result<Self, OperationError<E>> {
        Ok(Self {
            a: add(product_bits(self.a, other.a)?, 1)?,
            d: product_bits(self.d, other.d)?,
        })
    }
    fn power<E>(mut self, exponent: i64) -> Result<Self, OperationError<E>> {
        if exponent < 0 {
            self = Self {
                a: product_bits(self.a, self.d)?,
                d: add(mul(2, self.a)?, 1)?,
            };
        }
        let n = usize::try_from(exponent.unsigned_abs()).map_err(|_| OperationError::Overflow)?;
        Ok(Self {
            a: mul(n, add(self.a, 1)?)?,
            d: if self.d == 1 { 1 } else { mul(n, self.d)? },
        })
    }
    fn bits<E>(self) -> Result<usize, OperationError<E>> {
        add(self.a, self.d)
    }
}

#[derive(Clone, Copy)]
struct Bound {
    height: Height,
    peak: usize,
    nodes: usize,
    wide_float: bool,
}

struct State<'a, C: OperationControl> {
    control: &'a mut C,
    refusal: Option<OperationError<C::Error>>,
    height: usize,
}

struct Session<'a, C: OperationControl> {
    state: Mutex<State<'a, C>>,
    input: usize,
    nodes: usize,
    source_nodes: usize,
    source_bytes: usize,
    max_atom_nodes: usize,
    max_atom_bytes: usize,
    source_depth: usize,
    branch_depth: usize,
    scheme_variables: usize,
    horner: bool,
    parameters: usize,
    outputs: usize,
    names: usize,
    externals: usize,
    workers: usize,
    permutations: usize,
}

impl<C: OperationControl> Session<'_, C> {
    fn fail(&self, error: OperationError<C::Error>) -> ! {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.refusal.is_none() {
            state.refusal = Some(error);
        }
        drop(state);
        panic::resume_unwind(Box::new(Stop));
    }

    fn value<T>(&self, value: Result<T, OperationError<C::Error>>) -> T {
        match value {
            Ok(value) => value,
            Err(error) => self.fail(error),
        }
    }

    fn extent(
        &self,
        height: usize,
        stage: OperationStage,
        work: usize,
    ) -> Result<OperationExtent, OperationError<C::Error>> {
        let n = self.nodes.max(1);
        let integer = limbs(mul(4, add(height, 2)?)?)?;
        let coefficient = add(size_of::<Complex<Rational>>(), mul(4, integer)?)?;
        // Horner's coefficient grouping/power distribution adds at most a
        // Mul/Num or Pow wrapper per edge, hence three canonical nodes per
        // source node. Only one source expression/body is normalized at a time
        // on each worker; shared alias bodies are not expanded by Horner.
        let node_bytes = add(64, mul(4, integer)?)?;
        let all_atoms = add(
            self.source_bytes,
            mul(mul(3, self.source_nodes)?, node_bytes)?,
        )?;
        let local_atom = add(
            self.max_atom_bytes,
            mul(mul(3, self.max_atom_nodes)?, node_bytes)?,
        )?;
        let depth = mul(3, mul(self.source_depth, add(self.scheme_variables, 1)?)?)?;
        let workspace = if self.horner {
            add(all_atoms, mul(mul(4, add(depth, 1)?)?, local_atom)?)?
        } else {
            0
        };
        // Eight complex/rational temporaries cover visible arithmetic and
        // unreduced cross-products. Backend-private GMP/MPFR algorithm scratch
        // remains in the caller's separately admitted opaque allowance.
        let arithmetic = mul(add(depth, 1)?, mul(8, coefficient)?)?;
        let entries = add(self.parameters, mul(9, n)?)?;
        let instructions = mul(12, n)?;
        let output = add(
            mul(entries, size_of::<Complex<Rational>>())?,
            add(
                mul(mul(3, n)?, mul(4, integer)?)?,
                add(
                    mul(
                        instructions,
                        add(size_of::<(Instr, ComplexPhase)>(), 4 * size_of::<usize>())?,
                    )?,
                    add(
                        mul(self.outputs, size_of::<usize>())?,
                        add(
                            self.names,
                            mul(
                                self.externals,
                                size_of::<ExternalFunctionContainer<Complex<Rational>>>(),
                            )?,
                        )?,
                    )?,
                )?,
            )?,
        )?;
        // Native lowerer tables: borrowed subexpressions, constant values,
        // fixed external keys and recursion/parameter bindings. Branch caches
        // clone only the subexpression table. No CPE quadratic predictor is
        // charged here: actual pair/cache/line populations are admitted at the
        // optimizer safe point immediately before each real native pass.
        let values = mul(3, n)?;
        let subexpressions = mul(
            add(self.branch_depth, 2)?,
            table_bytes::<AtomView, Slot, C::Error>(values)?,
        )?;
        let constants = add(
            table_bytes::<Complex<Rational>, usize, C::Error>(values)?,
            mul(values, mul(4, integer)?)?,
        )?;
        let externals = add(
            table_bytes::<(Symbol, Vec<Atom>, Option<Vec<Complex<Rational>>>), usize, C::Error>(
                self.externals,
            )?,
            mul(mul(2, self.externals)?, coefficient)?,
        )?;
        let bindings = add(
            table_bytes::<usize, (), C::Error>(values)?,
            table_bytes::<AtomView, Slot, C::Error>(self.parameters)?,
        )?;
        let instructions_work = add(
            mul(mul(6, n)?, size_of::<Instruction>())?,
            mul(mul(6, n)?, size_of::<Slot>())?,
        )?;
        let records = add(
            mul(
                2,
                add(subexpressions, add(constants, add(externals, bindings)?)?)?,
            )?,
            instructions_work,
        )?;
        let schemes = mul(
            add(self.workers, 3)?,
            mul(
                self.scheme_variables,
                add(size_of::<Indeterminate>(), self.max_atom_bytes)?,
            )?,
        )?;
        let permutation_bytes = add(
            mul(
                self.permutations,
                add(3 * size_of::<Vec<usize>>(), 9 * size_of::<usize>())?,
            )?,
            table_bytes::<usize, usize, C::Error>(9)?,
        )?;
        let temporary = add(
            self.input,
            add(
                mul(add(self.workers, 2)?, add(workspace, arithmetic)?)?,
                add(
                    mul(3, add(output, records)?)?,
                    add(schemes, permutation_bytes)?,
                )?,
            )?,
        )?;
        let extent = OperationExtent {
            stage,
            work,
            input_bytes: self.input,
            temporary_bytes: temporary,
            output_bytes: output,
            output_entries: entries,
        };
        extent.peak_bytes().ok_or(OperationError::Overflow)?;
        Ok(extent)
    }

    fn check(&self, height: usize, stage: OperationStage, work: usize) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.refusal.is_some() {
            drop(state);
            panic::resume_unwind(Box::new(Stop));
        }
        let height = height.max(state.height);
        let extent = self.extent(height, stage, work);
        let result = match extent {
            Ok(extent) => state.control.check(extent).map_err(OperationError::Control),
            Err(error) => Err(error),
        };
        match result {
            Ok(()) => state.height = height,
            Err(error) => {
                state.refusal = Some(error);
                drop(state);
                panic::resume_unwind(Box::new(Stop));
            }
        }
    }

    fn finalize(&self, product: &ExpressionEvaluator<Complex<Rational>>) {
        let resident = self.value(super::checked_copy::extent(product, control::payload));
        self.report(OperationExtent {
            stage: OperationStage::Finalization,
            work: 0,
            input_bytes: self.input,
            temporary_bytes: 0,
            output_bytes: resident.input_bytes,
            output_entries: product.stack.len(),
        });
    }

    fn report(&self, extent: OperationExtent) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.refusal.is_some() {
            drop(state);
            panic::resume_unwind(Box::new(Stop));
        }
        if let Err(error) = state.control.check(extent) {
            state.refusal = Some(OperationError::Control(error));
            drop(state);
            panic::resume_unwind(Box::new(Stop));
        }
    }

    fn float(&self, value: SerializedFloat<'_>) -> (Height, bool) {
        let precision = value
            .0
            .get(..4)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u32::from_le_bytes)
            .unwrap_or(0) as usize;
        if precision == 0 {
            self.fail(OperationError::Unsupported(
                "invalid serialized float precision",
            ));
        }
        // Stage one precedes MPFR parsing/decoding. Parsing text and precision
        // backing is admitted without materializing an integer ratio.
        self.check(
            self.value(add(precision, self.value(mul(value.0.len(), 8)))),
            OperationStage::Construction,
            0,
        );
        let decoded = value.to_float();
        let (numerator, denominator) = decoded.rational_bit_bounds().unwrap_or_else(|| {
            self.fail(OperationError::Unsupported(
                "nonfinite or unqualified float backend",
            ))
        });
        let a = usize::try_from(numerator).unwrap_or_else(|_| self.fail(OperationError::Overflow));
        let d =
            usize::try_from(denominator).unwrap_or_else(|_| self.fail(OperationError::Overflow));
        let wide = a.max(d) > i32::MAX as usize;
        (Height { a, d }, wide)
    }

    fn coefficient(&self, value: CoefficientView<'_>) -> (Height, bool) {
        fn rational<E>(value: SerializedRational<'_>) -> Result<(usize, usize), OperationError<E>> {
            match value {
                SerializedRational::Natural(n, d) => Ok((
                    (64 - n.unsigned_abs().leading_zeros()) as usize,
                    (64 - d.unsigned_abs().leading_zeros()) as usize,
                )),
                SerializedRational::Large(value) => Ok((
                    mul(value.num_digits.len(), 8)?,
                    mul(value.den_digits.len(), 8)?,
                )),
            }
        }
        let ((nr, dr), (ni, di), wide) = match value {
            CoefficientView::Natural(nr, dr, ni, di) => (
                self.value(rational(SerializedRational::Natural(nr, dr))),
                self.value(rational(SerializedRational::Natural(ni, di))),
                false,
            ),
            CoefficientView::Large(r, i) => {
                (self.value(rational(r)), self.value(rational(i)), false)
            }
            CoefficientView::Float(r, i) => {
                let (r, rw) = self.float(r);
                let (i, iw) = self.float(i);
                ((r.a, r.d), (i.a, i.d), rw || iw)
            }
            _ => self.fail(OperationError::Unsupported(
                "checked construction coefficient domain",
            )),
        };
        (
            Height {
                a: self
                    .value(product_bits(nr, di))
                    .max(self.value(product_bits(ni, dr))),
                d: self.value(product_bits(dr.max(1), di.max(1))),
            },
            wide,
        )
    }

    fn bound(&self, atom: AtomView<'_>) -> Bound {
        let mut height = Height::ONE;
        let mut peak = 2;
        let mut nodes = 1;
        let mut wide_float = false;
        match atom {
            AtomView::Num(number) => {
                (height, wide_float) = self.coefficient(number.get_coeff_view());
            }
            AtomView::Var(_) => {}
            AtomView::Add(_) | AtomView::Mul(_) | AtomView::Fun(_) => {
                // Balance the positive absolute-numerator/common-denominator
                // envelope, not the native arithmetic. Its unreduced numerator
                // dominates every sequential partial sum (all omitted positive
                // denominator factors are >= 1). This avoids a carry bit per
                // source operand while retaining a bound on native temporaries.
                let mut sums = [None::<Height>; usize::BITS as usize];
                let children = match atom {
                    AtomView::Add(a) => a.iter(),
                    AtomView::Mul(m) => m.iter(),
                    AtomView::Fun(f) => f.iter(),
                    _ => unreachable!(),
                };
                for child in children {
                    let child = self.bound(child);
                    nodes = self.value(add(nodes, child.nodes));
                    peak = peak.max(child.peak);
                    wide_float |= child.wide_float;
                    height = if matches!(atom, AtomView::Add(_)) {
                        let mut carry = child.height;
                        let mut level = 0;
                        while let Some(previous) = sums[level].take() {
                            carry = self.value(previous.sum(carry));
                            level += 1;
                        }
                        sums[level] = Some(carry);
                        Height::ONE
                    } else if matches!(atom, AtomView::Mul(_)) {
                        self.value(height.product(child.height))
                    } else {
                        Height::ONE
                    };
                }
                if matches!(atom, AtomView::Add(_)) {
                    let mut total = None::<Height>;
                    for part in sums.into_iter().flatten() {
                        total = Some(if let Some(previous) = total {
                            self.value(previous.sum(part))
                        } else {
                            part
                        });
                    }
                    height = total.unwrap_or(Height::ONE);
                }
            }
            AtomView::Pow(power) => {
                let (base, exponent) = power.get_base_exp();
                let base = self.bound(base);
                let exp_bound = self.bound(exponent);
                nodes = self.value(add(1, self.value(add(base.nodes, exp_bound.nodes))));
                peak = base.peak.max(exp_bound.peak);
                wide_float = base.wide_float || exp_bound.wide_float;
                height = if let AtomView::Num(exp) = exponent
                    && let CoefficientView::Natural(n, 1, 0, _) = exp.get_coeff_view()
                {
                    self.value(base.height.power(n))
                } else {
                    base.height
                };
            }
        }
        peak = peak.max(self.value(height.bits()));
        Bound {
            height,
            peak,
            nodes,
            wide_float,
        }
    }
}

impl<C: OperationControl + Send> WorkspaceObserver for Session<'_, C>
where
    C::Error: Send,
{
    fn enter(&self, atom: AtomView<'_>) -> usize {
        let bound = self.bound(atom);
        if bound.wide_float && (atom.needs_normalization() || matches!(atom, AtomView::Fun(_))) {
            self.fail(OperationError::Unsupported(
                "native float arithmetic precision/exponent range",
            ));
        }
        // sqrt/log/exp can raise MPFR precision. Their visible precision bound
        // is admitted from the actual current operands before normalization;
        // nested results are scanned again before the next native operation.
        self.check(
            self.value(add(self.value(mul(2, bound.peak)), 64)),
            OperationStage::Construction,
            1,
        );
        0
    }
    fn leave(&self, _: usize) {}
    fn power(&self, base: CoefficientView<'_>, exponent: CoefficientView<'_>) {
        let (height, wide) = self.coefficient(base);
        if wide {
            self.fail(OperationError::Unsupported(
                "native float arithmetic precision/exponent range",
            ));
        }
        let height = match exponent {
            CoefficientView::Natural(n, 1, 0, _) => self.value(height.power(n)),
            _ if matches!(base, CoefficientView::Float(..))
                || matches!(exponent, CoefficientView::Float(..)) =>
            {
                let (other, wide) = self.coefficient(exponent);
                if wide {
                    self.fail(OperationError::Unsupported(
                        "native float arithmetic precision/exponent range",
                    ));
                }
                Height {
                    a: self.value(add(self.value(height.bits()), self.value(other.bits()))),
                    d: 64,
                }
            }
            _ => self.fail(OperationError::Unsupported(
                "new exact numerical fractional-power normalization",
            )),
        };
        self.check(self.value(height.bits()), OperationStage::Construction, 1);
    }
}

impl<C: OperationControl + Send> BuildObserver for Session<'_, C>
where
    C::Error: Send,
{
    fn horner(&self, atom: AtomView<'_>) {
        let bound = self.bound(atom);
        self.check(bound.peak, OperationStage::Construction, 1);
    }
    fn linearization(&self, atom: AtomView<'_>) {
        let bound = self.bound(atom);
        self.check(bound.peak, OperationStage::Construction, 1);
    }
    fn float_rational(&self, value: &Float) {
        let (numerator, denominator) = value.rational_bit_bounds().unwrap_or_else(|| {
            self.fail(OperationError::Unsupported(
                "nonfinite or unqualified float backend",
            ))
        });
        let bits = numerator
            .checked_add(denominator)
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or_else(|| self.fail(OperationError::Overflow));
        // Stage two precedes the actual exponent-sized dyadic allocation.
        self.check(bits, OperationStage::Construction, 1);
    }
    fn stage(&self, stage: OperationStage) {
        self.check(1, stage, 0);
    }
    fn optimization(&self, programme: &ExpressionEvaluator<Complex<Rational>>, pairs: bool) {
        let mut extent = self.value(optimizer_extent(programme, pairs));
        extent.input_bytes = self.input;
        self.report(extent);
    }
}

fn alias<'a>(map: &'a FunctionMap, atom: AtomView<'_>) -> Option<&'a Expr> {
    if let AtomView::Var(v) = atom {
        return map.tagged_fn_map.get(&(v.get_symbol(), Vec::new()));
    }
    if let AtomView::Fun(f) = atom
        && map.tag.get(&f.get_symbol()).copied() != Some(f.get_nargs())
    {
        return None;
    }
    map.tagged_fn_map.iter().find_map(|((symbol, tags), body)| {
        let matches = match atom {
            AtomView::Var(v) => v.get_symbol() == *symbol && tags.is_empty(),
            AtomView::Fun(f) => {
                f.get_symbol() == *symbol
                    && f.get_nargs() == tags.len()
                    && f.iter().zip(tags).all(|(a, b)| a == b.as_view())
            }
            _ => false,
        };
        matches.then_some(body)
    })
}

fn qualify<E>(
    atom: AtomView<'_>,
    params: &[Atom],
    map: &FunctionMap,
    heads: &[KnownHead],
    depth: usize,
    opaque: bool,
) -> Result<(usize, usize), OperationError<E>> {
    if depth
        > map
            .tagged_fn_map
            .len()
            .checked_add(1)
            .ok_or(OperationError::Overflow)?
    {
        return Err(OperationError::Unsupported("cyclic alias construction"));
    }
    let symbol = match atom {
        AtomView::Var(v) => Some(v.get_symbol()),
        AtomView::Fun(f) => Some(f.get_symbol()),
        _ => None,
    };
    let is_parameter = params.iter().any(|p| p.as_view() == atom);
    let mut names = 0;
    if let Some(symbol) = symbol {
        if symbol.get_normalization_function().is_some() || symbol.is_linear() {
            return Err(OperationError::Unsupported(
                "custom or linear-head normalization",
            ));
        }
        if let Some(info) = symbol.get_evaluation_info()
            && info.has_evaluator::<Complex<Float>>()
        {
            return Err(OperationError::Unsupported(
                "custom floating-point normalization callback",
            ));
        }
        names = symbol
            .get_aliases()
            .iter()
            .map(|a| a.len())
            .max()
            .unwrap_or(0)
            .max(symbol.get_name().len());
        if !opaque && !is_parameter {
            if alias(map, atom).is_some() {
                let mut nodes = 1;
                if let AtomView::Fun(f) = atom {
                    for child in f {
                        let (n, child_names) = qualify(child, params, map, heads, depth, true)?;
                        nodes = add(nodes, n)?;
                        names = add(names, child_names)?;
                    }
                }
                return Ok((nodes, names));
            }
            let builtin = symbol.is_fixed_builtin();
            if !builtin {
                let head = KnownHead::find::<E>(heads, symbol)?;
                let arity = if let AtomView::Fun(f) = atom {
                    f.get_nargs()
                } else {
                    0
                };
                if arity != head.arity {
                    return Err(OperationError::Unsupported("qualified head arity"));
                }
            }
        }
    }
    let mut nodes = 1;
    match atom {
        AtomView::Add(a) => {
            for child in a {
                let (n, names_) = qualify(child, params, map, heads, depth, opaque)?;
                nodes = add(nodes, n)?;
                names = add(names, names_)?;
            }
        }
        AtomView::Mul(m) => {
            for child in m {
                let (n, names_) = qualify(child, params, map, heads, depth, opaque)?;
                nodes = add(nodes, n)?;
                names = add(names, names_)?;
            }
        }
        AtomView::Fun(f) => {
            for child in f {
                let (n, names_) =
                    qualify(child, params, map, heads, depth, opaque || is_parameter)?;
                nodes = add(nodes, n)?;
                names = add(names, names_)?;
            }
        }
        AtomView::Pow(p) => {
            for child in [p.get_base(), p.get_exp()] {
                let (n, names_) = qualify(child, params, map, heads, depth, opaque)?;
                nodes = add(nodes, n)?;
                names = add(names, names_)?;
            }
        }
        AtomView::Num(n) => {
            if !matches!(
                n.get_coeff_view(),
                CoefficientView::Natural(..)
                    | CoefficientView::Large(..)
                    | CoefficientView::Float(..)
            ) {
                return Err(OperationError::Unsupported(
                    "checked construction coefficient domain",
                ));
            }
        }
        AtomView::Var(_) => {}
    }
    Ok((nodes, names))
}

// Count the native alias DAG within the actual branch-cache scopes. Merely
// summing recursively-expanded alias bodies would charge shared arithmetic many
// times, whereas each native local scope caches an alias after its first use.
fn lower_nodes<C: OperationControl>(
    atom: AtomView<'_>,
    params: &[Atom],
    map: &FunctionMap,
    seen: &mut [u8],
    branch_depth: usize,
    max_branch_depth: &mut usize,
    input: usize,
    control: &mut C,
) -> Result<usize, OperationError<C::Error>> {
    control
        .check(OperationExtent {
            stage: OperationStage::Construction,
            work: 1,
            input_bytes: input,
            temporary_bytes: mul(seen.len(), add(branch_depth, 2)?)?,
            ..OperationExtent::default()
        })
        .map_err(OperationError::Control)?;
    if params.iter().any(|p| p.as_view() == atom) {
        return Ok(1);
    }
    if let Some(body) = alias(map, atom) {
        let status = seen
            .get_mut(body.id)
            .ok_or(OperationError::Unsupported("invalid alias identity"))?;
        if *status == 1 {
            return Err(OperationError::Unsupported("cyclic alias construction"));
        }
        if *status == 2 {
            return Ok(1);
        }
        *status = 1;
        let nodes = lower_nodes(
            body.body.as_view(),
            params,
            map,
            seen,
            branch_depth,
            max_branch_depth,
            input,
            control,
        )?;
        seen[body.id] = 2;
        return add(nodes, 1);
    }
    if let AtomView::Fun(f) = atom
        && f.get_symbol() == Symbol::IF
        && f.get_nargs() == 3
    {
        let mut args = f.iter();
        let condition = args
            .next()
            .ok_or(OperationError::Unsupported("conditional arity"))?;
        let then_branch = args
            .next()
            .ok_or(OperationError::Unsupported("conditional arity"))?;
        let else_branch = args
            .next()
            .ok_or(OperationError::Unsupported("conditional arity"))?;
        let condition = lower_nodes(
            condition,
            params,
            map,
            seen,
            branch_depth,
            max_branch_depth,
            input,
            control,
        )?;
        let next_depth = add(branch_depth, 1)?;
        *max_branch_depth = (*max_branch_depth).max(next_depth);
        control
            .check(OperationExtent {
                stage: OperationStage::Construction,
                work: 0,
                input_bytes: input,
                temporary_bytes: mul(seen.len(), add(next_depth, 2)?)?,
                ..OperationExtent::default()
            })
            .map_err(OperationError::Control)?;
        let mut child = seen.to_vec();
        let then_count = lower_nodes(
            then_branch,
            params,
            map,
            &mut child,
            next_depth,
            max_branch_depth,
            input,
            control,
        )?;
        child.copy_from_slice(seen);
        let else_count = lower_nodes(
            else_branch,
            params,
            map,
            &mut child,
            next_depth,
            max_branch_depth,
            input,
            control,
        )?;
        // Native lowering emits IfElse, Goto, two Labels and Join. A fully
        // numerical condition only removes records, so both branches are a bound.
        return add(5, add(condition, add(then_count, else_count)?)?);
    }
    let mut nodes = 1;
    let children = match atom {
        AtomView::Add(a) => Some(a.iter()),
        AtomView::Mul(m) => Some(m.iter()),
        AtomView::Fun(f) => Some(f.iter()),
        _ => None,
    };
    if let Some(children) = children {
        for child in children {
            nodes = add(
                nodes,
                lower_nodes(
                    child,
                    params,
                    map,
                    seen,
                    branch_depth,
                    max_branch_depth,
                    input,
                    control,
                )?,
            )?;
        }
    } else if let AtomView::Pow(p) = atom {
        for child in [p.get_base(), p.get_exp()] {
            nodes = add(
                nodes,
                lower_nodes(
                    child,
                    params,
                    map,
                    seen,
                    branch_depth,
                    max_branch_depth,
                    input,
                    control,
                )?,
            )?;
        }
    }
    Ok(nodes)
}

fn external_population<E>(
    atom: AtomView<'_>,
    params: &[Atom],
    map: &FunctionMap,
) -> Result<usize, OperationError<E>> {
    if params.iter().any(|p| p.as_view() == atom) {
        return Ok(0);
    }
    // E as an exponential base lowers to the EXP instruction, not an external
    // constant. Direct pi/e values do occupy exact external constant slots.
    if let AtomView::Pow(p) = atom
        && p.get_base() == crate::atom::InlineVar::new(Symbol::E).as_view()
    {
        return external_population(p.get_exp(), params, map);
    }
    let mut count = 0;
    if let AtomView::Fun(f) = atom
        && !f.get_symbol().is_fixed_builtin()
        && alias(map, atom).is_none()
    {
        count = 1;
    } else if let AtomView::Var(v) = atom
        && alias(map, atom).is_none()
        && (!v.get_symbol().is_fixed_builtin() || [Symbol::PI, Symbol::E].contains(&v.get_symbol()))
    {
        count = 1;
    }
    let children = match atom {
        AtomView::Add(a) => Some(a.iter()),
        AtomView::Mul(m) => Some(m.iter()),
        AtomView::Fun(f) => Some(f.iter()),
        _ => None,
    };
    if let Some(children) = children {
        for child in children {
            count = add(count, external_population(child, params, map)?)?;
        }
    } else if let AtomView::Pow(p) = atom {
        for child in [p.get_base(), p.get_exp()] {
            count = add(count, external_population(child, params, map)?)?;
        }
    }
    Ok(count)
}

fn raw_shape<E>(atom: AtomView<'_>) -> Result<(usize, usize), OperationError<E>> {
    let mut nodes = 1;
    let mut depth = 1;
    let children = match atom {
        AtomView::Add(a) => Some(a.iter()),
        AtomView::Mul(m) => Some(m.iter()),
        AtomView::Fun(f) => Some(f.iter()),
        _ => None,
    };
    if let Some(children) = children {
        for child in children {
            let (n, d) = raw_shape(child)?;
            nodes = add(nodes, n)?;
            depth = depth.max(add(d, 1)?);
        }
    } else if let AtomView::Pow(p) = atom {
        for child in [p.get_base(), p.get_exp()] {
            let (n, d) = raw_shape(child)?;
            nodes = add(nodes, n)?;
            depth = depth.max(add(d, 1)?);
        }
    }
    Ok((nodes, depth))
}

fn table_bytes<K, V, E>(entries: usize) -> Result<usize, OperationError<E>> {
    if entries == 0 {
        return Ok(0);
    }
    let buckets = entries
        .checked_mul(8)
        .and_then(|n| n.checked_add(6))
        .map(|n| n / 7)
        .and_then(usize::checked_next_power_of_two)
        .ok_or(OperationError::Overflow)?
        .max(4);
    add(
        mul(buckets, add(size_of::<(K, V)>(), 1)?)?,
        16 + std::mem::align_of::<(K, V)>(),
    )
}

fn optimizer_extent<E>(
    e: &ExpressionEvaluator<Complex<Rational>>,
    pairs: bool,
) -> Result<OperationExtent, OperationError<E>> {
    let resident = super::checked_copy::extent(e, control::payload)?;
    let instructions = e.instructions.len();
    let mut operands = 0;
    let mut occurrences = 0;
    for (instruction, _) in &e.instructions {
        match instruction {
            Instr::Add(_, args) | Instr::Mul(_, args) => {
                operands = add(operands, args.len())?;
                occurrences = add(
                    occurrences,
                    mul(args.len(), args.len().saturating_sub(1))? / 2,
                )?;
            }
            Instr::ExternalFun(_, _, args) => operands = add(operands, args.len())?,
            _ => {}
        }
    }
    let keys = if pairs {
        if occurrences <= e.settings.max_common_pair_cache_entries {
            occurrences
        } else {
            add(e.settings.max_common_pair_cache_entries, 1)?
        }
    } else {
        0
    };
    let lines = occurrences.min(mul(keys, instructions)?);
    let introduced = if pairs { keys.min(operands / 2) } else { 0 };
    let next_instructions = add(instructions, introduced)?;
    let next_stack = add(e.stack.len(), introduced)?;
    let instruction_capacity = mul(2, next_instructions)?
        .max(4)
        .max(e.instructions.capacity());
    let stack_capacity = mul(2, next_stack)?.max(4).max(e.stack.capacity());
    let output = add(
        resident.input_bytes,
        add(
            mul(
                instruction_capacity - e.instructions.capacity(),
                size_of::<(Instr, ComplexPhase)>(),
            )?,
            add(
                mul(
                    stack_capacity - e.stack.capacity(),
                    size_of::<Complex<Rational>>(),
                )?,
                mul(mul(2, introduced)?, size_of::<usize>())?,
            )?,
        )?,
    )?;
    // CSE keys borrow operands; their largest discriminated key is bounded by
    // four words, with two-word position/branch values. Include old/new hash
    // table growth and the exact cloned-operand population.
    let cse = add(
        mul(
            2,
            table_bytes::<[usize; 4], (usize, usize), E>(instructions)?,
        )?,
        add(
            mul(instructions, size_of::<(Instr, ComplexPhase)>())?,
            mul(operands, size_of::<usize>())?,
        )?,
    )?;
    // Live stack optimization/label/DAG/rename maps and heap. These counts use
    // actual private stack/instruction populations, including capacity overlap.
    let stack_work = add(
        mul(add(next_stack, next_instructions)?, 8 * size_of::<usize>())?,
        mul(2, table_bytes::<usize, usize, E>(next_instructions)?)?,
    )?;
    let pair_work = if pairs {
        let hashes = add(
            table_bytes::<u64, u32, E>(keys)?,
            add(
                table_bytes::<u64, u32, E>(operands)?,
                add(
                    table_bytes::<u32, Vec<(usize, u32)>, E>(instructions)?,
                    table_bytes::<usize, usize, E>(introduced)?,
                )?,
            )?,
        )?;
        let pair_vectors = add(
            mul(mul(2, keys)?.max(4), size_of::<(u64, Vec<u32>)>())?,
            add(
                mul(add(mul(3, lines)?, mul(4, keys)?)?, size_of::<u32>())?,
                add(
                    mul(
                        mul(2, instructions)?.max(4),
                        size_of::<(u32, u32, u32, u32)>(),
                    )?,
                    mul(
                        mul(2, introduced)?.max(4),
                        size_of::<(usize, usize, Instr)>(),
                    )?,
                )?,
            )?,
        )?;
        let replacements = add(
            mul(mul(2, operands)?.max(4), size_of::<(usize, u32)>())?,
            instructions,
        )?;
        add(mul(2, hashes)?, add(pair_vectors, replacements)?)?
    } else {
        0
    };
    let temporary = add(resident.input_bytes, add(cse, add(stack_work, pair_work)?)?)?;
    let extent = OperationExtent {
        stage: OperationStage::Optimization,
        work: 0,
        input_bytes: 0,
        temporary_bytes: temporary,
        output_bytes: output,
        output_entries: next_stack,
    };
    extent.peak_bytes().ok_or(OperationError::Overflow)?;
    Ok(extent)
}

pub(super) fn build<C: OperationControl + Send>(
    expressions: &[AtomView<'_>],
    map: &FunctionMap,
    params: &[Atom],
    settings: OptimizationSettings,
    heads: &[KnownHead],
    control: &mut C,
) -> Result<ExpressionEvaluator<Complex<Rational>>, OperationError<C::Error>>
where
    C::Error: Send,
{
    control
        .check(OperationExtent {
            stage: OperationStage::Construction,
            ..OperationExtent::default()
        })
        .map_err(OperationError::Control)?;
    if !settings.direct_translation {
        return Err(OperationError::Unsupported(
            "legacy evaluation-tree construction",
        ));
    }
    if settings.hot_start.is_some() || settings.abort_check.is_some() || settings.abort_level != 0 {
        return Err(OperationError::Unsupported(
            "retained hot-start or legacy abort callback",
        ));
    }
    if settings.n_cores == 0 {
        return Err(OperationError::Unsupported("zero Horner worker count"));
    }
    if map
        .tagged_fn_map
        .values()
        .any(|body| !body.is_alias || !body.args.is_empty() || !body.options.should_inline())
    {
        return Err(OperationError::Unsupported(
            "non-alias or retained function definitions",
        ));
    }
    let mut source_nodes = 0;
    let mut names = 0;
    let mut source_bytes = 0;
    let mut max_atom_nodes = 0;
    let mut max_atom_bytes = 0;
    let mut source_depth = 0;
    let mut externals = 0;
    for atom in expressions
        .iter()
        .copied()
        .chain(params.iter().map(Atom::as_view))
        .chain(map.tagged_fn_map.values().map(|body| body.body.as_view()))
    {
        let (n, names_) = qualify(atom, params, map, heads, 0, false)?;
        source_nodes = add(source_nodes, n)?;
        names = add(names, names_)?;
        let (atom_nodes, depth) = raw_shape::<C::Error>(atom)?;
        externals = add(
            externals,
            external_population::<C::Error>(atom, params, map)?,
        )?;
        max_atom_nodes = max_atom_nodes.max(atom_nodes);
        max_atom_bytes = max_atom_bytes.max(atom.get_byte_size());
        source_depth = source_depth.max(depth);
        source_bytes = add(source_bytes, atom.get_byte_size())?;
    }
    let mut input = add(source_bytes, mul(params.len(), size_of::<Atom>())?)?;
    input = add(input, mul(expressions.len(), size_of::<AtomView>())?)?;
    for atom in params {
        input = add(input, atom.get_capacity())?;
    }
    for ((_, tags), body) in &map.tagged_fn_map {
        input = add(input, body.body.get_capacity())?;
        input = add(input, mul(tags.capacity(), size_of::<Atom>())?)?;
        for tag in tags {
            input = add(input, tag.get_capacity())?;
        }
    }
    input = add(
        input,
        mul(
            map.tagged_fn_map
                .capacity()
                .checked_next_power_of_two()
                .ok_or(OperationError::Overflow)?,
            2 * (size_of::<((Symbol, Vec<Atom>), Expr)>() + 16),
        )?,
    )?;
    input = add(
        input,
        mul(
            map.tag
                .capacity()
                .checked_next_power_of_two()
                .ok_or(OperationError::Overflow)?,
            2 * (size_of::<(Symbol, usize)>() + 16),
        )?,
    )?;
    control
        .check(OperationExtent {
            stage: OperationStage::Construction,
            work: 0,
            input_bytes: input,
            temporary_bytes: map.tagged_fn_map.len(),
            ..OperationExtent::default()
        })
        .map_err(OperationError::Control)?;
    let mut seen = Vec::new();
    seen.try_reserve_exact(map.tagged_fn_map.len())
        .map_err(|_| OperationError::Allocation)?;
    seen.resize(map.tagged_fn_map.len(), 0u8);
    for body in map.tagged_fn_map.values() {
        let id = seen
            .get_mut(body.id)
            .ok_or(OperationError::Unsupported("invalid alias identity"))?;
        if *id != 0 {
            return Err(OperationError::Unsupported("duplicate alias identity"));
        }
        *id = 1;
    }
    seen.fill(0);
    let mut nodes = 0;
    let mut branch_depth = 0;
    for expression in expressions {
        nodes = add(
            nodes,
            lower_nodes(
                *expression,
                params,
                map,
                &mut seen,
                0,
                &mut branch_depth,
                input,
                control,
            )?,
        )?;
    }
    drop(seen);
    let scheme_variables = if settings.horner_iterations == 0 {
        0
    } else {
        let mut expression_nodes = 0;
        for expression in expressions {
            expression_nodes = add(expression_nodes, raw_shape::<C::Error>(*expression)?.0)?;
        }
        control
            .check(OperationExtent {
                stage: OperationStage::Construction,
                work: expression_nodes,
                input_bytes: input,
                temporary_bytes: mul(
                    2,
                    table_bytes::<AtomView, usize, C::Error>(expression_nodes)?,
                )?,
                ..OperationExtent::default()
            })
            .map_err(OperationError::Control)?;
        AtomView::horner_indeterminates(expressions)
            .len()
            .min(settings.max_horner_scheme_variables)
    };
    let mut permutations = 0;
    let mut factorial = 1usize;
    for count in 1..=9
        .min(source_nodes)
        .min(settings.max_horner_scheme_variables)
    {
        factorial = mul(factorial, count)?;
        if factorial <= settings.horner_iterations {
            permutations = factorial;
        }
    }
    let workers = LicenseManager::max_threads(settings.n_cores)
        .min(settings.horner_iterations.saturating_sub(1));
    let session = Session {
        state: Mutex::new(State {
            control,
            refusal: None,
            height: 1,
        }),
        input,
        nodes,
        source_nodes,
        source_bytes,
        max_atom_nodes,
        max_atom_bytes,
        source_depth,
        branch_depth,
        scheme_variables,
        horner: settings.horner_iterations > 0,
        parameters: params.len(),
        outputs: expressions.len(),
        names,
        externals,
        workers,
        permutations,
    };
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        IntegerAllocationContext::fresh().run(|| {
            session.check(1, OperationStage::Construction, 0);
            for expression in expressions {
                let bound = session.bound(*expression);
                session.check(bound.peak, OperationStage::Construction, 0);
            }
            for body in map.tagged_fn_map.values() {
                let bound = session.bound(body.body.as_view());
                session.check(bound.peak, OperationStage::Construction, 0);
            }
            let product =
                AtomView::to_evaluator_observed(expressions, map, params, settings, Some(&session));
            if let Ok(product) = &product {
                session.finalize(product);
            }
            product
        })
    }));
    let state = session
        .state
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match result {
        Err(payload) if payload.is::<Stop>() => {
            Err(state.refusal.unwrap_or(OperationError::Interrupted))
        }
        Err(payload) => panic::resume_unwind(payload),
        Ok(_) if state.refusal.is_some() => Err(state.refusal.unwrap()),
        Ok(result) => result.map_err(|error| OperationError::InvalidProgram(error.to_string())),
    }
}

/// Private unwind payload for an already-recorded original typed refusal.
pub(crate) struct Stop;

pub(crate) fn preserve_panic(
    current: &mut Option<Box<dyn std::any::Any + Send>>,
    incoming: Box<dyn std::any::Any + Send>,
) {
    if current
        .as_ref()
        .is_none_or(|previous| previous.is::<Stop>() && !incoming.is::<Stop>())
    {
        *current = Some(incoming);
    }
}

pub(crate) trait BuildObserver: WorkspaceObserver {
    fn horner(&self, atom: AtomView<'_>);
    fn linearization(&self, atom: AtomView<'_>);
    fn float_rational(&self, value: &Float);
    fn stage(&self, stage: OperationStage);
    fn optimization(&self, _programme: &ExpressionEvaluator<Complex<Rational>>, _pairs: bool) {
        self.stage(OperationStage::Optimization);
    }
}

pub(crate) fn horner(
    atom: AtomView<'_>,
    scheme: &[Indeterminate],
    observer: Option<&dyn BuildObserver>,
) -> Atom {
    if let Some(observer) = observer {
        observer.horner(atom);
        let workspace = Workspace::controlled(observer);
        atom.horner_scheme_in(&workspace, Some(scheme), true, true)
    } else {
        atom.horner_scheme(Some(scheme), true, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domains::backend::integer::IntegerAllocationContext, parse};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn checked_builder_staged_cpe_tracks_actual_operands_and_configured_cache() {
        let parameters = (0..17)
            .map(|i| crate::parse!(format!("checked_cpe_parameter_{i}").as_str()))
            .collect::<Vec<_>>();
        let first = parameters[..16]
            .iter()
            .fold(Atom::num(0), |sum, value| sum + value);
        let second = parameters[..15]
            .iter()
            .fold(Atom::num(0), |sum, value| sum + value)
            + &parameters[16];
        let expressions = [first, second];
        let mut peaks = Vec::new();
        for cache in [0, 65536] {
            let mut peak = 0;
            let mut final_bytes = 0;
            let product = Atom::evaluator_multiple(&expressions, &parameters)
                .direct_translation(true)
                .horner_iterations(1)
                .max_common_pair_cache_entries(cache)
                .cpe_iterations(Some(1))
                .try_build_with(&mut |extent: OperationExtent| {
                    if extent.stage == OperationStage::Optimization {
                        peak = peak.max(extent.temporary_bytes);
                    }
                    if extent.stage == OperationStage::Finalization {
                        final_bytes = extent.output_bytes;
                    }
                    Ok::<_, std::convert::Infallible>(())
                })
                .unwrap();
            assert_eq!(
                final_bytes,
                super::super::checked_copy::extent::<_, std::convert::Infallible>(
                    &product,
                    control::payload
                )
                .unwrap()
                .input_bytes
            );
            let mut numeric = product.map_coeff(&|value| value.re.to_f64());
            let inputs = (1..=17).map(f64::from).collect::<Vec<_>>();
            let mut output = [0.; 2];
            numeric.evaluate(&inputs, &mut output);
            assert_eq!(output, [136., 137.]);
            peaks.push(peak);
        }
        assert!(
            peaks[1] > peaks[0],
            "the finite native cache cap changes actual optimizer admission"
        );
        #[derive(Debug, PartialEq, Eq)]
        struct Refused;
        let refusal = Atom::evaluator_multiple(&expressions, &parameters)
            .direct_translation(true)
            .horner_iterations(1)
            .cpe_iterations(Some(1))
            .try_build_with(&mut |extent: OperationExtent| {
                if extent.stage == OperationStage::Optimization {
                    Err(Refused)
                } else {
                    Ok(())
                }
            });
        assert_eq!(refusal.unwrap_err(), OperationError::Control(Refused));
    }

    #[test]
    fn checked_builder_branch_scopes_keep_alias_values_and_refuse_before_optimizer() {
        let parameters = [
            parse!("checked_branch_condition"),
            parse!("checked_branch_x"),
            parse!("checked_branch_y"),
        ];
        let alias = parse!("checked_branch_shared");
        let expressions = [
            parse!("if(checked_branch_condition,checked_branch_shared+1,checked_branch_shared-1)"),
            alias.clone(),
        ];
        let definitions = [(alias, &parameters[1] * &parameters[2])];
        let mut final_bytes = 0;
        let product = EvaluatorBuilder::try_new_aliases_with(
            &expressions,
            &parameters,
            &definitions,
            &mut |_: OperationExtent| Ok::<_, std::convert::Infallible>(()),
        )
        .unwrap()
        .direct_translation(true)
        .horner_iterations(1)
        .cpe_iterations(Some(1))
        .try_build_with(&mut |extent: OperationExtent| {
            if extent.stage == OperationStage::Finalization {
                final_bytes = extent.output_bytes;
            }
            Ok::<_, std::convert::Infallible>(())
        })
        .unwrap();
        assert_eq!(
            final_bytes,
            super::super::checked_copy::extent::<_, std::convert::Infallible>(
                &product,
                control::payload
            )
            .unwrap()
            .input_bytes
        );
        let mut numeric = product
            .try_map_f64_with(&mut |_: OperationExtent| Ok::<_, std::convert::Infallible>(()))
            .unwrap();
        let mut output = [0.; 2];
        numeric.evaluate(&[1., 2., 3.], &mut output);
        assert_eq!(output, [7., 6.]);
        numeric.evaluate(&[0., 2., 3.], &mut output);
        assert_eq!(output, [5., 6.]);
        let stopped = EvaluatorBuilder::try_new_aliases_with(
            &expressions,
            &parameters,
            &definitions,
            &mut |_: OperationExtent| Ok::<_, std::convert::Infallible>(()),
        )
        .unwrap()
        .direct_translation(true)
        .horner_iterations(1)
        .try_build_with(&mut |extent: OperationExtent| {
            if extent.stage == OperationStage::Optimization {
                Err("original branch refusal")
            } else {
                Ok(())
            }
        });
        assert_eq!(
            stopped.unwrap_err(),
            OperationError::Control("original branch refusal")
        );
    }

    #[test]
    fn checked_builder_counts_pi_e_constants_separately_from_exp_marker() {
        let map = FunctionMap::new();
        let pi = crate::atom::InlineVar::new(Symbol::PI).as_view().to_owned();
        let e = crate::atom::InlineVar::new(Symbol::E).as_view().to_owned();
        let parameters = [parse!("checked_builtin_x")];
        let exponential = parse!("exp(checked_builtin_x)");
        assert_eq!(
            external_population::<std::convert::Infallible>(pi.as_view(), &[], &map).unwrap(),
            1
        );
        assert_eq!(
            external_population::<std::convert::Infallible>(e.as_view(), &[], &map).unwrap(),
            1
        );
        assert_eq!(
            external_population::<std::convert::Infallible>(
                exponential.as_view(),
                &parameters,
                &map
            )
            .unwrap(),
            0
        );
        let expressions = [pi, e];
        let product = Atom::evaluator_multiple(&expressions, &[] as &[Atom])
            .direct_translation(true)
            .try_build_with(&mut |_: OperationExtent| Ok::<_, std::convert::Infallible>(()))
            .unwrap();
        assert_eq!(product.external_fns.len(), 2);
        assert!(
            product
                .external_fns
                .iter()
                .all(|external| external.constant_index.is_some())
        );
        let mut numeric = product
            .try_map_f64_with(&mut |_: OperationExtent| Ok::<_, std::convert::Infallible>(()))
            .unwrap();
        let mut output = [0.; 2];
        numeric.evaluate(&[], &mut output);
        assert_eq!(output, [std::f64::consts::PI, std::f64::consts::E]);
    }

    #[test]
    fn checked_builder_preserves_horner_products_and_reports_retained_backing() {
        let x = parse!("checked_builder_x");
        let expression = parse!("(2*checked_builder_x+2)^128 + checked_builder_x^2");
        let mut final_bytes = 0;
        let product = expression
            .evaluator(&[x.clone()])
            .direct_translation(true)
            .horner_iterations(16)
            .cores(4)
            .cpe_iterations(Some(3))
            .try_build_with(&mut |extent: OperationExtent| {
                if extent.stage == OperationStage::Finalization {
                    final_bytes = extent.output_bytes;
                }
                Ok::<_, std::convert::Infallible>(())
            })
            .unwrap();
        let resident = super::super::checked_copy::extent::<_, std::convert::Infallible>(
            &product,
            control::payload,
        )
        .unwrap();
        assert_eq!(final_bytes, resident.input_bytes);
        assert_eq!(product.settings.horner_iterations, 16);
        assert_eq!(product.settings.n_cores, 4);
        assert_eq!(product.settings.cpe_iterations, Some(3));
        assert!(product.settings.abort_check.is_none());
        let mut product = product.map_coeff(&|c| c.re.to_f64());
        let mut baseline = expression
            .evaluator(&[x])
            .direct_translation(true)
            .horner_iterations(16)
            .cores(4)
            .cpe_iterations(Some(3))
            .build()
            .unwrap()
            .map_coeff(&|c| c.re.to_f64());
        for x in [0., 0.25, 1.] {
            assert_eq!(
                product.evaluate_single(&[x]),
                baseline.evaluate_single(&[x])
            );
        }
    }

    #[test]
    fn checked_builder_preserves_original_typed_refusal_and_sticky_worker_stop() {
        #[derive(Debug, PartialEq, Eq)]
        struct Refused(u64);
        let inputs = [
            parse!("checked_stop_a"),
            parse!("checked_stop_b"),
            parse!("checked_stop_c"),
        ];
        let expression = parse!(
            "(checked_stop_a+checked_stop_b+checked_stop_c)^2+checked_stop_a*checked_stop_b+checked_stop_b*checked_stop_c"
        );
        let mut calls = 0;
        let mut refused = false;
        let result = expression
            .evaluator(&inputs)
            .direct_translation(true)
            .horner_iterations(64)
            .cores(4)
            .try_build_with(&mut |_: OperationExtent| {
                assert!(!refused, "an original refusal is sticky across all workers");
                calls += 1;
                if calls == 40 {
                    refused = true;
                    Err(Refused(67))
                } else {
                    Ok(())
                }
            });
        assert_eq!(result.unwrap_err(), OperationError::Control(Refused(67)));
        assert!(refused);
        assert!(!IntegerAllocationContext::capture().is_fresh());
    }

    #[test]
    fn checked_builder_admits_float_precision_then_exponent_before_dyadic_growth() {
        #[derive(Debug, PartialEq, Eq)]
        struct Refused;
        let expression = Atom::num(Float::deserialize(b"1@100000", 53));
        let mut largest = 0;
        let result = expression
            .evaluator(&[] as &[Atom])
            .direct_translation(true)
            .horner_iterations(0)
            .try_build_with(&mut |extent: OperationExtent| {
                largest = largest.max(extent.peak_bytes().unwrap());
                if largest > 8_000_000 {
                    Err(Refused)
                } else {
                    Ok(())
                }
            });
        assert_eq!(result.unwrap_err(), OperationError::Control(Refused));
        assert!(largest > 8_000_000);
    }

    #[test]
    fn checked_builder_keeps_fixed_tag_aliases_and_function_shaped_inputs() {
        let head = crate::symbol!(
            "checked_builder_known_head",
            eval = EvaluationInfo::new().register(|args: &[f64]| args[0] + 3.)
        );
        let parameter = parse!("checked_builder_opaque(checked_builder_local)");
        let alias = parse!("checked_builder_alias(7)");
        let expression = head.call(&[parameter.clone()][..]) + &alias;
        let definitions = [(alias, &parameter * Atom::num(2))];
        let heads = [KnownHead {
            symbol: head,
            arity: 1,
            first: [None; 2],
            second: [None; 3],
        }];
        let product = expression
            .evaluator(&[parameter])
            .add_aliases(definitions)
            .unwrap()
            .direct_translation(true)
            .horner_iterations(16)
            .cores(4)
            .try_build_known_with(&heads, &mut |_: OperationExtent| {
                Ok::<_, std::convert::Infallible>(())
            })
            .unwrap();
        let mut numeric = product.map_coeff(&|c| c.re.to_f64());
        assert_eq!(numeric.evaluate_single(&[2.]), 9.);
    }

    #[test]
    fn checked_builder_refuses_custom_normalization_before_callback_or_growth() {
        static INVOCATIONS: AtomicUsize = AtomicUsize::new(0);
        let head = crate::symbol!(
            "checked_builder_custom_normalization",
            norm = |_, _| {
                INVOCATIONS.fetch_add(1, Ordering::SeqCst);
            }
        );
        let x = parse!("checked_builder_custom_x");
        let mut expression = Atom::new();
        expression.to_fun(head).add_arg(x.as_view());
        let mut reports = 0;
        let result = expression
            .evaluator(&[x])
            .direct_translation(true)
            .try_build_with(&mut |extent: OperationExtent| {
                reports += 1;
                assert_eq!(extent.output_bytes, 0);
                Ok::<_, std::convert::Infallible>(())
            });
        assert!(matches!(result, Err(OperationError::Unsupported(_))));
        assert_eq!(reports, 1);
        assert_eq!(INVOCATIONS.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn checked_alias_constructor_admits_native_map_before_growth_and_shares_builder() {
        let x = parse!("checked_ctor_x");
        let first = parse!("checked_ctor_alias(3)");
        let second = parse!("checked_ctor_result");
        let aliases = [
            (first.clone(), &x * Atom::num(2)),
            (second.clone(), &first + Atom::num(1)),
        ];
        let expressions = [second];
        #[derive(Debug, PartialEq, Eq)]
        struct Refused;
        let refused = EvaluatorBuilder::try_new_aliases_with(
            &expressions,
            &[x.clone()],
            &aliases,
            &mut |extent: OperationExtent| {
                if extent.output_bytes > 0 {
                    Err(Refused)
                } else {
                    Ok(())
                }
            },
        );
        assert_eq!(refused.unwrap_err(), OperationError::Control(Refused));
        let mut reports = 0;
        let builder = EvaluatorBuilder::try_new_aliases_with(
            &expressions,
            &[x],
            &aliases,
            &mut |extent: OperationExtent| {
                assert!(extent.peak_bytes().is_some());
                if extent.output_bytes > 0 {
                    reports += 1;
                }
                Ok::<_, std::convert::Infallible>(())
            },
        )
        .unwrap();
        assert!(reports >= 2);
        let mut product = builder
            .direct_translation(true)
            .try_build_with(&mut |_: OperationExtent| Ok::<_, std::convert::Infallible>(()))
            .unwrap()
            .map_coeff(&|value| value.re.to_f64());
        assert_eq!(product.evaluate_single(&[4.]), 9.);
    }

    #[derive(Default)]
    struct Probe {
        active: AtomicUsize,
        powers: AtomicUsize,
        threads: Mutex<std::collections::HashSet<std::thread::ThreadId>>,
        stop_power: bool,
    }
    impl WorkspaceObserver for Probe {
        fn enter(&self, _: AtomView<'_>) -> usize {
            assert!(IntegerAllocationContext::capture().is_fresh());
            self.active.fetch_add(1, Ordering::SeqCst);
            1
        }
        fn leave(&self, n: usize) {
            self.active.fetch_sub(n, Ordering::SeqCst);
        }
        fn power(&self, _: CoefficientView<'_>, exponent: CoefficientView<'_>) {
            self.powers.fetch_add(1, Ordering::SeqCst);
            if self.stop_power
                && matches!(exponent, CoefficientView::Natural(n, 1, 0, 1) if n > 128)
            {
                panic::resume_unwind(Box::new(Stop));
            }
        }
    }
    impl BuildObserver for Probe {
        fn horner(&self, _: AtomView<'_>) {
            assert!(IntegerAllocationContext::capture().is_fresh());
            self.threads
                .lock()
                .unwrap()
                .insert(std::thread::current().id());
        }
        fn linearization(&self, _: AtomView<'_>) {}
        fn float_rational(&self, _: &Float) {}
        fn stage(&self, _: OperationStage) {}
    }

    #[test]
    fn controlled_horner_normalizer_stops_before_factored_integer_power() {
        let expression = parse!("(2*controlled_horner_x+2)^1000000");
        let observer = Probe {
            stop_power: true,
            ..Probe::default()
        };
        let stopped = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            IntegerAllocationContext::fresh()
                .run(|| horner(expression.as_view(), &[], Some(&observer)))
        }));
        assert!(stopped.unwrap_err().is::<Stop>());
        assert!(observer.powers.load(Ordering::SeqCst) > 0);
        assert_eq!(
            observer.active.load(Ordering::SeqCst),
            0,
            "normalization leases unwind"
        );
        assert!(!IntegerAllocationContext::capture().is_fresh());
    }

    #[test]
    fn controlled_horner_workers_keep_native_parallel_search_and_values() {
        let parameters = [
            parse!("controlled_parallel_a"),
            parse!("controlled_parallel_b"),
            parse!("controlled_parallel_c"),
            parse!("controlled_parallel_d"),
        ];
        let expression = parse!(
            "(controlled_parallel_a+controlled_parallel_b+controlled_parallel_c+controlled_parallel_d)^2+controlled_parallel_a*controlled_parallel_b+controlled_parallel_b*controlled_parallel_c+controlled_parallel_c*controlled_parallel_d"
        );
        let settings = OptimizationSettings {
            direct_translation: true,
            horner_iterations: 32,
            n_cores: 4,
            ..OptimizationSettings::default()
        };
        let observer = Probe::default();
        let evaluator = IntegerAllocationContext::fresh()
            .run(|| {
                AtomView::to_evaluator_observed(
                    &[expression.as_view()],
                    &FunctionMap::new(),
                    &parameters,
                    settings.clone(),
                    Some(&observer),
                )
            })
            .unwrap();
        let baseline = AtomView::to_evaluator(
            &[expression.as_view()],
            &FunctionMap::new(),
            &parameters,
            settings,
        )
        .unwrap();
        assert_eq!(observer.active.load(Ordering::SeqCst), 0);
        assert_eq!(
            observer.threads.lock().unwrap().len(),
            LicenseManager::max_threads(4).min(31)
        );
        let mut actual = evaluator.map_coeff(&|c| c.re.to_f64());
        let mut expected = baseline.map_coeff(&|c| c.re.to_f64());
        for inputs in [[1., 2., 3., 4.], [0., -2., 0.5, 4.]] {
            assert_eq!(
                actual.evaluate_single(&inputs),
                expected.evaluate_single(&inputs)
            );
        }
    }
}
