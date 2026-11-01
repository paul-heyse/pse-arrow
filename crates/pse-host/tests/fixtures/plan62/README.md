# Compiler feasibility inputs

These isolated investigation inputs are not a production compiler producer or an automatically
discovered Cargo test. Plan62d owns the decisions and evidence limits. They use the checkout's
pinned compiler and its explicitly installed matching rustc-dev component. No extra Cargo
workspace, toolchain override or root manifest change is needed.

From the checkout root, reproduce in a new private scratch directory:

```bash
plan62_sysroot=$(rustc --print sysroot)
plan62_scratch=$(mktemp -d)
pse run -- rustc --edition=2024 crates/pse-host/tests/fixtures/plan62/compiler-driver.rs -o "$plan62_scratch/driver" -C prefer-dynamic -C "link-arg=-Wl,-rpath,$plan62_sysroot/lib"
pse run -- "$plan62_scratch/driver" crates/pse-host/tests/fixtures/plan62/compiler-input.rs --edition=2024 --crate-type=lib --sysroot "$plan62_sysroot" --emit=metadata -o "$plan62_scratch/input.rmeta"
pse run -- rustc crates/pse-host/tests/fixtures/plan62/compiler-input.rs --edition=2024 --crate-type=lib --emit=mir -o "$plan62_scratch/input.mir" '-Zdump-mir=partial_move & ElaborateDrops | cannot_propagate & AbortUnwindingCalls | may_propagate & AbortUnwindingCalls' "-Zdump-mir-dir=$plan62_scratch/mir" -Zmir-include-spans=yes -Cpanic=unwind
pse run -- rustc crates/pse-host/tests/fixtures/plan62/send-diagnostic.rs --edition=2024 --crate-type=lib --emit=metadata -o "$plan62_scratch/send.rmeta" --error-format=json
```

The last command must fail with the non-Send Rc diagnostic; the driver and ordinary artifact
commands must succeed. Inspect original stdout/stderr through each returned invocation. Driver
output must show pair.0 shared capture, pair.1 moved capture, two suspension variants and early
MIR becoming stolen after optimization. It intentionally prints debug observations and stops
after selected queries: this is not full crate compilation or a stable schema. Runtime cleanup,
arbitrary nested bodies and cancellation are not established. Keep outputs in the existing
invocation owner; remove only this authenticated scratch directory after its operations settle.
