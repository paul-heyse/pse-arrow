# CoolProp sys native-provider integration

This maintained copy retains `coolprop-sys` 8.0.0's upstream process-wide access lock,
native handle API and licenses. Its original published archive is SHA256
`7d388a1023cf8236d1a9f372db5f3ade4dc752b7588eef68848bba2050b2eb08`.
The source VCS revision is `30801b93f8a27dfa7f93537151bc5d39630d3f24`.

The local manifest removes bundled platform-image dependencies. The build consumes
`COOLPROP_DIR` from explicit immutable native preparation, checks its original receipt/image
and records that image for the existing global loader. It neither downloads nor builds native
code. The matched C declaration adds the library-owned second saturation derivative export;
`xtask codegen --only bindgen` regenerates the function table. The backend additionally checks
the declared native source/patch recipe and this maintained supplier's complete delta.

The native C export patch is separate from this Rust supplier delta. It lives inside the
original CoolProp wrapper's handle manager and exception boundary; no second state owner,
EOS, interpolation or production finite-difference derivative is introduced (ADR-0238).
