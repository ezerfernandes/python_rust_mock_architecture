# Rust integration template for Python projects

This repository is a runnable example and an agent playbook for adding a Rust
extension to an existing Python project. It covers Python libraries, standalone
programs, FastAPI services, and Django services without forcing a package-manager
migration.

The example uses PyO3 to expose checked integer operations and Maturin to
build and install the extension. Python owns the package namespace, public
documentation, and typing metadata. The native module stays private so the
package controls its public API. The Rust workspace also contains a private
Verus-verified core. Its proof covers checked `i64` addition, left-to-right
checked summation, sorted lower-bound search, and first-match binary search. The
Python adapter remains outside the proved boundary and performs conversion and
exception mapping.

Agents should start with [AGENTS.md](AGENTS.md). It contains the topology decision,
tooling-preservation rules, framework recipes, testing layers, package-manager
command matrix, and extension checklist.

## API

```python
from python_rust_mock_architecture import (
    add,
    binary_search,
    checked_sum,
    lower_bound,
)

assert add(2, 3) == 5
assert checked_sum([1, -2, 3]) == 2
assert lower_bound([1, 2, 2, 4], 2) == 1
assert binary_search([1, 2, 2, 4], 2) == 1
```

`add(a, b)` accepts Python integers in the signed 64-bit range,
`[-2**63, 2**63 - 1]`, and returns their sum. Inputs or results outside that
range raise `OverflowError`. Floats, strings, missing arguments, and extra
arguments raise `TypeError`. Boolean operands follow normal Python integer
behavior, so `True` is one and `False` is zero.

The package exports the four supported functions through `__all__`. The compiled extension is
loaded as `python_rust_mock_architecture._native`; it is an implementation
detail rather than a second public API. `checked_sum` raises `OverflowError` on
any overflowing prefix. The search functions raise `ValueError` for unsorted
inputs and return the first eligible or matching index; `binary_search` returns
`None` when the target is absent.

## Requirements

Install [uv](https://docs.astral.sh/uv/) and a Rust toolchain with `rustc` and
`cargo`. The project requires Python 3.11 or newer. uv can install the selected
Python interpreter with `uv python install 3.11`.

The pinned Verus workflow uses Verus `0.2026.08.30.b432e82`, Rust `1.97.1`, and
`vstd` `0.0.0-2026-08-30-0159`. Follow
[`docs/verus-toolchain.md`](docs/verus-toolchain.md) before running proof
targets.

## Quick start

From the repository root:

```text
uv python install 3.11
make setup PYTHON_VERSION=3.11
make develop PYTHON_VERSION=3.11
make test
```

`make develop` builds the Rust extension and installs it into the uv environment.
There is no Python fallback implementation, so the extension must be built before
importing the package from a clean checkout.

List every supported command:

```text
make help
```

## Tests and quality checks

Run both test suites:

```text
make test
```

Run formatting, linting, typing, tests, coverage, and package verification:

```text
make check
```

`make check` also runs the full workspace Verus proof and the proof-hygiene
scan. It stops before Rust coverage if `cargo-llvm-cov` is missing.

Python coverage uses the locked `pytest-cov` dependency and requires 100% for the
small Python wrapper. Rust coverage requires `cargo-llvm-cov` and an 80% line
threshold:

```text
cargo +stable install cargo-llvm-cov --locked
make coverage-python
make coverage-rust
```

The pinned Verus release, standalone proof/build spike, and repository proof
gate are documented in [`docs/verus-toolchain.md`](docs/verus-toolchain.md). The
spike validates the
toolchain used by the private verified core. The repository proof targets are:

```text
make verus-hygiene
make verus-verify
```

`verus-verify` checks the workspace with root-scoped `-V check-api-safety` while
still verifying dependencies. Use `make verus-focus` for a faster development
proof of `verified-core`; run the full target before handoff. The pinned
standalone spike commands are also in the toolchain document.

## Wheel and source distribution

Build normal developer artifacts in `dist/`:

```text
make build
```

Run the stricter package gate:

```text
make package-check
```

`package-check` builds twice in a fresh temporary directory, derives the expected
artifact names from project metadata, rejects generated files in the source
distribution, and imports the installed wheel from outside the checkout.

## Source tree

```text
AGENTS.md                    Agent playbook for adapting the Rust pattern
Makefile                     Stable local and CI command surface
python/                       Python package namespace and typing metadata
src/lib.rs                    PyO3 module and thin verified-core adapters
crates/verified-core/         Private pure-Rust Verus-verified core workspace member
tools/verus-spike/            Standalone pinned-toolchain proof fixture
tools/verus-toolchain.toml    Verus, Rust, Z3, and vstd pins
rust-toolchain.toml           Rust toolchain selected by Cargo and rustup
tests/python/test_add.py      Python API and boundary tests
pyproject.toml                Python metadata, Maturin, Ruff, and Mypy config
Cargo.toml                    Rust workspace, extension crate, and dependencies
uv.lock                      Locked Python development dependencies
Cargo.lock                   Locked Rust dependencies
.github/workflows/ci.yml     CI checks through the Makefile interface
```

The project has one Python package and one uv lockfile. It does not use an
artificial uv workspace. The Rust portion is a small Cargo workspace: the root
package remains the Maturin extension, while `crates/verified-core` contains
private pure-Rust code opted into Verus verification.

## Non-goals

This template does not optimize code or add benchmarks. It has no Python
fallback, web service, database, async runtime, publishing workflow, or
cross-platform release-wheel matrix.
