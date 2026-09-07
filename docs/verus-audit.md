# Final proof, package, and trust-boundary audit

Audit date: 2026-09-07

Status: local evidence is complete. The CI workflow is configured and reviewed,
but the audit commit has not been pushed, so no GitHub Actions run exists for
this exact revision.

## Recorded environment

The audit used the pinned Linux release and the repository's locked dependency
metadata:

| Component | Recorded version or source |
| --- | --- |
| Verus | `0.2026.08.30.b432e82` (`linux_x86_64`) |
| `cargo-verus` | bundled with the pinned Verus release; `cargo verus --help` passed |
| Rust and Cargo | `1.97.1-x86_64-unknown-linux-gnu` |
| Z3 | `4.16.0`, bundled with the pinned release |
| `vstd` | `0.0.0-2026-08-30-0159`, revision `b432e82fed7e05090fd53b5e5fc39020f725aabe` |
| Python | `3.14.6` for the local run; CI exercises 3.11 and 3.13 |
| uv | `0.11.26` |
| Maturin | `1.15.0` |
| PyO3 | `0.27.2` from `Cargo.lock` |
| `cargo-llvm-cov` | `0.9.1` |

The pinned Verus archive and checksum are recorded in
[`tools/verus-toolchain.toml`](../tools/verus-toolchain.toml). The validated
host is Ubuntu 24.04 x86_64. macOS, Windows, other operating systems, and other
architectures are not exercised by this repository's CI.

## Verification evidence

The clean-target command below ran the full dependency and project proof with
the API-safety check forwarded only to workspace roots:

```text
verus_bin=/home/ezer/.local/share/verus/0.2026.08.30.b432e82
export PATH="$verus_bin:$PATH"
proof_target="$(mktemp -d)"
CARGO_TARGET_DIR="$proof_target" make verus-verify
```

It reported `2045 verified, 0 errors` for dependencies and `15 verified, 0
errors` for `verified-core`. The complete local `make check` then passed:

- Ruff format/check, lock validation, Cargo fmt, and Clippy with warnings denied;
- 36 Python tests and 16 Rust tests;
- Mypy with no issues;
- 100% Python line coverage and 96.92% Rust line coverage overall (100% for
  `verified-core`), above the configured 80% Rust threshold;
- two source-distribution/wheel builds, complete archive inspection, and an
  isolated Python 3.11 wheel install and smoke import outside the checkout.

`make verus-hygiene` passed. The project-authored proof scan found no
`assume`, admitted axiom, `external_body`, `assume_specification`,
`external_fn_specification`, or `verifier::external`. The core does contain
explicit `#[trigger]` annotations in quantified summation, sortedness, and
search invariants; these were reviewed as solver-instantiation guidance, not
trusted facts or external specifications.

The public core functions reachable from Python (`add`, `checked_sum`,
`lower_bound`, and `binary_search`) have no `requires` clauses. Internal helpers
with preconditions are called only after executable checks and verified loop
facts establish those preconditions. The PyO3 adapter remains outside the
proved core and is covered by Rust binding tests, Python contract tests, and the
isolated wheel smoke test.

## Trusted computing base

The proof claim is limited to the opted-in pure Rust `verified-core` crate and
the specifications proved by the pinned Verus run. The following remain
outside that claim: Verus, Z3, rustc, LLVM, `vstd`'s supplied specifications,
PyO3, CPython, Maturin, Python conversion and exception handling, and the root
PyO3 adapter. No project-authored trust escape was found, and no runtime or
packaging behavior is inferred from the proof alone.

## CI status

`.github/workflows/ci.yml` uses Ubuntu 24.04 x86_64, installs the checksum-
verified Verus release, runs the shared `make verus-verify` target, and makes
the package job depend on Python, Rust, and Verus jobs. The latest recorded
remote CI run (`34103650460`, `d4ce383`) passed before the pinned Verus job was
added. The post-`4865710` workflow and this audit are local and require a push
before their GitHub Actions result can be recorded.
