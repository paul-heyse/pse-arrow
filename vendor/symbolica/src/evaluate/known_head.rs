//! Finite Taylor composition of closed local-partial descriptions.
use super::*;

impl KnownHead {
    pub(super) fn find<E>(heads: &[Self], symbol: Symbol) -> Result<&Self, OperationError<E>> {
        let mut matches = heads.iter().filter(|head| head.symbol == symbol);
        let head = matches.next().ok_or(OperationError::Unsupported("unknown numerical head"))?;
        if matches.next().is_some() || !(1..=2).contains(&head.arity) {
            return Err(OperationError::Unsupported("ambiguous head or unsupported arity"));
        }
        Ok(head)
    }

    pub(super) fn demand<E>(&self, depth: usize, heads: &[Self]) -> Result<(), OperationError<E>> {
        let first = if depth > 0 { self.arity } else { 0 };
        let second = if depth > 1 { self.arity * (self.arity + 1) / 2 } else { 0 };
        for symbol in self.first[..first].iter().chain(&self.second[..second]) {
            let symbol = symbol.ok_or(OperationError::Unsupported("unavailable local partial"))?;
            if Self::find::<E>(heads, symbol)?.arity != self.arity {
                return Err(OperationError::Unsupported("local partial arity"));
            }
        }
        Ok(())
    }

    // Upper bound on scalar instructions in the finite generated evaluator. Its
    // parameter and constant backing is counted separately by the operation owner.
    pub(super) fn instruction_bound<E>(&self, width: usize, table: usize) -> Result<usize, OperationError<E>> {
        let pairs = self.arity * (self.arity + 1) / 2;
        let calls = 1 + self.arity + pairs;
        let linear = width.checked_mul(2 * self.arity).ok_or(OperationError::Overflow)?;
        let quadratic = table.checked_mul(4 * pairs).ok_or(OperationError::Overflow)?;
        calls.checked_add(linear).and_then(|n| n.checked_add(quadratic))
            .and_then(|n| n.checked_add(width)).ok_or(OperationError::Overflow)
    }
}

pub(super) struct KnownDualizer<'a, D: DualNumberStructure> {
    pub(super) dualizer: &'a Dualizer<D>,
    pub(super) generated: HashMap<Symbol, ExpressionEvaluator<Complex<Rational>>>,
}

impl<D: DualNumberStructure> Vectorize<Complex<Rational>> for KnownDualizer<'_, D> {
    fn duplicate_constants(&self) -> bool { self.dualizer.duplicate_constants() }
    fn get_dimension(&self) -> usize { self.dualizer.get_dimension() }
    fn allow_explicit_vector_function(&self) -> bool { false }
    fn map_instruction(&self, instruction: &VectorInstruction,
        instructions: &mut InstructionList<Complex<Rational>>) -> Vec<VectorInstruction> {
        self.dualizer.map_instruction(instruction, instructions)
    }
    fn vectorize_function(&self, symbol: Symbol, tags: &[Atom], argument_count: usize)
        -> Result<Option<ExpressionEvaluator<Complex<Rational>>>, String> {
        if !tags.is_empty() { return Err("Tagged known head is unsupported".into()); }
        let evaluator = self.generated.get(&symbol).ok_or("Unknown qualified numerical head")?;
        if evaluator.param_count != argument_count * self.get_dimension() {
            return Err("Qualified head arity mismatch".into());
        }
        Ok(Some(evaluator.clone()))
    }
}

/// Build scalar native instructions directly. This never invokes a symbol's
/// arbitrary derivative callback and never starts a nested symbolic builder.
pub(super) fn generate_known<D: DualNumberStructure, C: OperationControl>(head: &KnownHead,
    dualizer: &Dualizer<D>, control: &mut C, extent: OperationExtent)
    -> Result<ExpressionEvaluator<Complex<Rational>>, OperationError<C::Error>> {
    let check_extent = OperationExtent { work: 0, stage: OperationStage::KnownHeadGeneration, ..extent };
    control.check(check_extent).map_err(OperationError::Control)?;
    let shape = dualizer.dual.get_shape();
    let width = shape.len();
    let params = head.arity * width;
    let half = params;
    let reserved = params + 1;
    let mut instructions = Vec::new();
    let mut external_fns = Vec::new();
    let base_args = (0..head.arity).map(|argument| argument * width).collect::<Vec<_>>();
    fn call(symbol: Symbol, args: &[usize], reserved: usize,
        instructions: &mut Vec<(Instr, ComplexPhase)>,
        external_fns: &mut Vec<ExternalFunctionContainer<Complex<Rational>>>) -> usize {
        let index = external_fns.iter().position(|external| external.symbol == symbol)
            .unwrap_or_else(|| {
                external_fns.push(ExternalFunctionContainer::new(symbol, vec![], vec![]));
                external_fns.len() - 1
            });
        let output = reserved + instructions.len();
        instructions.push((Instr::ExternalFun(output, index, args.to_vec()), ComplexPhase::Any));
        output
    }
    let value = call(head.symbol, &base_args, reserved, &mut instructions, &mut external_fns);
    let depth = dualizer.dual.get_max_depth();
    let first = if depth > 0 { head.first[..head.arity].iter().map(|symbol| {
        call(symbol.unwrap(), &base_args, reserved, &mut instructions, &mut external_fns)
    }).collect::<Vec<_>>() } else { Vec::new() };
    let pairs = head.arity * (head.arity + 1) / 2;
    let second = if depth > 1 { head.second[..pairs].iter().map(|symbol| {
        call(symbol.unwrap(), &base_args, reserved, &mut instructions, &mut external_fns)
    }).collect::<Vec<_>>() } else { Vec::new() };
    let mut results = vec![value];
    for (component, powers) in shape.iter().enumerate().skip(1) {
        control.check(check_extent).map_err(OperationError::Control)?;
        let mut terms = Vec::new();
        for argument in 0..head.arity {
            let output = reserved + instructions.len();
            instructions.push((Instr::Mul(output, vec![first[argument], argument * width + component]), ComplexPhase::Any));
            terms.push(output);
        }
        if powers.iter().sum::<usize>() == 2 {
            let mut pair = 0;
            for left in 0..head.arity {
                for right in left..head.arity {
                    for &(a, b, destination) in dualizer.dual.get_multiplication_table() {
                        if a == 0 || b == 0 || destination != component { continue; }
                        let product = reserved + instructions.len();
                        instructions.push((Instr::Mul(product, vec![left * width + a, right * width + b]), ComplexPhase::Any));
                        let output = reserved + instructions.len();
                        let mut args = vec![product, second[pair]];
                        if left == right { args.push(half); }
                        // Keep every scalar instruction binary, including the
                        // diagonal 1/2 scaling, for the shared inliner/optimizer.
                        instructions.push((Instr::Mul(output, args[..2].to_vec()), ComplexPhase::Any));
                        let output = if args.len() == 3 {
                            let scaled = reserved + instructions.len();
                            instructions.push((Instr::Mul(scaled, vec![output, half]), ComplexPhase::Any));
                            scaled
                        } else { output };
                        terms.push(output);
                    }
                    pair += 1;
                }
            }
        }
        let output = if terms.len() == 1 { terms[0] } else {
            let output = reserved + instructions.len();
            instructions.push((Instr::Add(output, terms), ComplexPhase::Any));
            output
        };
        results.push(output);
    }
    let mut stack = vec![Complex::default(); reserved + instructions.len()];
    stack[half] = Complex::from(Rational::from((1, 2)));
    control.check(check_extent).map_err(OperationError::Control)?;
    Ok(ExpressionEvaluator { stack, param_count: params, reserved_indices: reserved,
        instructions, result_indices: results, external_fns, settings: OptimizationSettings::default() })
}
