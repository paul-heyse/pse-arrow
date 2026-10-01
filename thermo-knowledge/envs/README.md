<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Side environments

Most sources are read in the tree's core Python 3.14 environment (`thermo-knowledge/.venv`).
Three groups of libraries cannot live there because their constraints conflict with it, so each
has its own locked environment. They serve two purposes only:

- **readers** for formats that genuinely need the library, and
- **oracle harnesses** that evaluate the library's own models at given points, for comparison.

**The rule.** A side environment exchanges Arrow/Parquet or JSON files with the core pipeline.
It never imports `thermo_knowledge`, and the core never imports it. Run a side environment's
code with `tk-env-run`; its output is a file the core reads.

| Environment | Tool | Python | Hosts | Why separate |
|---|---|---|---|---|
| `thermotools` | uv | 3.11 | thermopack, pykingas, surfpack, matplotlib, pyarrow | needs `numpy<2`; pykingas publishes wheels only up to cp311 |
| `geochem` | micromamba (conda-forge) | 3.12 | reaktoro 2.13.0, thermofun, chemicalfun, gems3k (C++ library only), pyarrow; phreeqcrm from pip | native C++ stacks pinned to each other (reaktoro pins `thermofun 0.6.1.*`) |
| `rmg` | micromamba (conda-forge, rmg) | 3.11 | RMG-Py (`rmg` 4.0.0, with its `rmgdatabase` dependency), pyarrow | Python `<3.12`, `numpy<2`, `cython<3.1`, a large native chemistry stack |

Each directory `envs/<name>/` holds:

| File | Meaning |
|---|---|
| `pyproject.toml`, `.python-version`, `uv.lock` | the uv project (`thermotools`) |
| `environment.yml` | the human-written micromamba spec (`geochem`, `rmg`) |
| `environment.lock.txt` | the explicit lock solved from the spec: one URL with checksum per package, for `linux-64` |
| `pip.in`, `pip.lock.txt` | pip-only packages of a micromamba environment and their hashed lock (`geochem`) |
| `smoke.py` | imports each library, makes one real call per library and prints one JSON object |

The environments are not members of any uv workspace. Their prefixes are created under
`thermo-knowledge/.store/envs/<name>` (gitignored), never under a global environments directory;
micromamba's package cache is `thermo-knowledge/.store/mamba-root/pkgs`. The recipes call
`envs/tk-env.sh`, which ignores the caller's environment (`VIRTUAL_ENV`, the repository
`UV_PROJECT_ENVIRONMENT=.venv`, the root `.venv` on `PATH`, `CONDA_*`/`MAMBA_*`, user
micromamba configuration and the user site directory), so a recipe behaves the same from any
shell.

## Commands

From the repository root `just -f thermo-knowledge/justfile <recipe>`; inside `thermo-knowledge/`
`just <recipe>`.

| Recipe | Does |
|---|---|
| `tk-env-sync <name>` | create or refresh the environment from its lock (a micromamba prefix is rebuilt only when a lock changed) |
| `tk-env-lock <name>` | re-solve and rewrite the lock: `uv lock` for `thermotools`; for a micromamba environment a fresh solve of `environment.yml` into the prefix, then the explicit export (and the hashed pip lock) |
| `tk-env-run <name> <command>...` | run a command inside the environment, in `thermo-knowledge/` |

Creation and solving run in a memory-capped scope through `scripts/memory-cap.sh` of the
repository when it exists. The installs are large (the `rmg` prefix is about 3 GB); create them
one at a time.

```bash
just -f thermo-knowledge/justfile tk-env-sync thermotools
just -f thermo-knowledge/justfile tk-env-run thermotools python envs/thermotools/smoke.py
```

`tk-env-lock` on a micromamba environment takes the newest packages the spec allows, so commit the
new lock only when the change is intended. Channels are fixed by the spec with strict priority,
whatever the user's `~/.condarc` says. `thermotools` keeps its pins on `tk-env-lock`; upgrade one
with `uv lock --project envs/thermotools --upgrade-package <name>`.

## Notes on the libraries

- `surfpack` 1.0.0 declares no dependencies yet imports `matplotlib` when the package is
  imported, so `thermotools` lists `matplotlib`.
- `gems3k` is the GEMS3K C++ library (`libGEMS3K.so`); it has no Python binding. `chemicalfun`
  and `thermofun` are its Python-facing relatives.
- The installed `thermofun` Python metadata says 0.6.0 while the conda package is 0.6.1; the
  lock names the conda package.
- `rmgdatabase` is a hard dependency of the conda `rmg` package. The loader in `smoke.py` reads
  the RMG database from the raw store instead, through RMG's own `ThermoDatabase`.
- pykingas treats a single-component model as a binary: pass two mole fractions.
