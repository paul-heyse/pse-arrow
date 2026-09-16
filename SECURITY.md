# Security Policy

## Supported versions

`pse-arrow` is in **phase 0 — foundations** and has no released version. Until `v0.1.0`,
only the `main` branch is supported: fixes land on `main` and there are no backports.

| Version | Supported |
|---|---|
| `main` | yes |
| `0.0.x` pre-release tags | no |

## Reporting a vulnerability

**Do not open a public issue, pull request, or discussion for a security problem.**

Report privately, either way:

- **Preferred:** GitHub private vulnerability reporting —
  <https://github.com/paul-heyse/pse-arrow/security/advisories/new>. This creates a
  private advisory thread and lets us credit you and issue a CVE.
- **Email:** paul@heyse.io. If you want an encrypted channel, say so in a first message
  with no details and we will arrange one.

Please include: the affected commit or version, the platform and solver setup, what an
attacker gains, and the smallest input or sequence that reproduces it. A failing test, a
model file, or a `probe_host.json` is worth more than a paragraph.

### What to expect

| Step | Target |
|---|---|
| Acknowledgement of your report | **within 7 days** |
| Initial assessment (severity, affected surface, whether it is in scope) | within 14 days |
| Fix or mitigation on `main`, or a written plan with a date | within 90 days |
| Advisory published and reporter credited (unless you prefer otherwise) | with the fix |

We practise coordinated disclosure and will agree a date with you. If you do not hear
back within 7 days, send a reminder to paul@heyse.io — a missed report is a bug in this
process.

Safe harbour: we will not pursue or support legal action against anyone who reports in
good faith, acts only against their own data and infrastructure, avoids privacy
violations and service degradation, and gives us reasonable time to respond.

## Scope

This project compiles and solves engineering models. It is **not** a sandbox, and the
threat model is "a model or data file from someone you do not fully trust", not "hostile
code you chose to run". In scope:

- **FFI and native code.** `crates/pse-ipopt-sys` and `crates/pse-backend-native` bind
  Ipopt through C. Memory unsafety, missing `catch_unwind` across an FFI callback
  boundary, or an unsound `unsafe` block anywhere is in scope — `unsafe_code` is denied
  outside a four-crate allowlist, so any escape is a finding. So are unsound blocks in
  `pse-kernels-ext` and `pse-py`.
- **Untrusted documents and data.** Parsing a model, manifest, relation store, Arrow IPC
  or Parquet file, or a golden store from an untrusted source must not cause memory
  unsafety, an uncontrolled allocation, an infinite loop, or code execution. Panics on
  malformed input are bugs, but report them as ordinary issues unless they are reachable
  from a library API that documents a `Result`.
- **The Python boundary.** Anything in `pse._native` that can be made to violate a Rust
  invariant from pure Python — buffer lifetimes, extension-type round trips, zero-copy
  claims that are not actually safe.
- **The supply chain of what we ship.** A compromised or malicious pinned dependency, a
  wheel that does not match its attestation, a release workflow that can be made to
  publish something it should not, or a CI workflow that leaks a token or can be
  triggered by an untrusted fork (script injection, `pull_request_target` misuse,
  unpinned actions).
- **The solver image.** `ghcr.io/paul-heyse/pse-solvers` — a vulnerability in how it is
  built or how CI consumes it.

Out of scope:

- Numerical disagreement with IDAES, a solver failing to converge, or a model producing a
  wrong answer. Those are correctness bugs — open a `parity-gap` or `bug-report` issue.
- Vulnerabilities in Ipopt, MUMPS, ASL, IDAES, Arrow, or DataFusion themselves. Report
  those upstream; do tell us if we need to move a pin, and `cargo audit` running in
  `rust / deny` is how we normally learn.
- Denial of service from a model you wrote yourself, resource exhaustion under
  deliberately unbounded input, or anything requiring an attacker who already has
  execution on your machine or write access to the repository.
- Missing hardening that is not exploitable on its own (for example, a missing compiler
  flag). Open an ordinary issue; we will take it seriously, just not privately.

## How we handle dependencies

Every dependency is version-pinned (`=` for Rust, `==` for Python), the lockfiles are
committed, and every gate runs `--locked`. `cargo deny check` (advisories, licences, bans,
sources) and `cargo audit` run on every pull request as `rust / deny`, and `just policy`
runs them strictly on demand.

**Be precise about what that means today.** The project is pre-release: every crate is
`publish = false` and nothing has been published to crates.io or PyPI. While that is true,
`rust / deny` **reports and does not block** — it is `continue-on-error` and is not a
required check — and no dependency or licence is refused on policy grounds (ADR-0066,
[dependency policy](docs/dev/dependency-policy.md)). A known advisory in the graph is
therefore visible on the pull request but does not stop a merge. Register row R-31 carries
the dated obligation to run the strict audit before the first published release.

The advisory `ignore` list still requires an id, a reason, an owner, and a review date.
Dependabot opens grouped updates, and GitHub secret scanning with push protection is
enabled on the repository.
