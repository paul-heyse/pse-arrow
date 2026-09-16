<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Synthetic currency fixture, version 1.0.0

`USD_2000` and `USD_2001` use invented relative indices 500 and 1000 to exercise the
ratio 2. They are not historical prices, inflation data, exchange rates, or a cost
engineering recommendation. This package is selected only by the explicit Rust
fixture projection and tests. Production packages must supply their own actual
dated currency declarations and evidence.
