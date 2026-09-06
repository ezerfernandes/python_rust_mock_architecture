# Python and Rust add extension

**Status:** Approved for implementation
**Drafted:** 2026-09-01
**Reviewed:** 2026-09-02
**Approved:** 2026-09-02
**Stable task:** `TEAM-four-pack-20260901184053-6cea`

## Purpose

This repository is a small, reusable example of a Python package with one Rust
extension function. It must be easy to set up locally, straightforward to
inspect, and honest about the boundary between Python packaging and Rust code.

The package exposes one operation:

```python
from python_rust_mock_architecture import add

assert add(2, 3) == 5
```

The arithmetic is implemented in Rust through PyO3. Python owns the package
namespace and documentation. Maturin builds and installs the extension.

## Approved decisions

The human approved these choices for implementation on 2026-09-02.

1. The distribution name is `python-rust-mock-architecture` and its import
   package is `python_rust_mock_architecture`.
2. `add` accepts Python `int` arguments in the signed 64-bit range and returns
   a Python `int`. The Python minimum is 3.11.
3. Integer overflow is an error. The extension must not wrap the result or
   panic. An input or result outside the signed 64-bit range raises
   `OverflowError`.

Any change to these choices requires a contract update and renewed approval.

## Public API contract

### `python_rust_mock_architecture.add`

Signature:

```python
def add(a: int, b: int) -> int: ...
```

Behavior:

- Return `a + b` when both inputs and the result are in the inclusive range
  `[-2**63, 2**63 - 1]`.
- Accept positional arguments and the named arguments `a` and `b`.
- Accept `bool` as Python does for integer arithmetic, with `False` equal to
  zero and `True` equal to one.
- Raise `TypeError` when an argument is missing, an extra argument is given, or
  a non-integral value such as a `float` or `str` is passed. Support for
  third-party integer-like objects is not part of the public contract.
- Raise `OverflowError` when an input is outside the signed 64-bit range or the
  sum is outside that range.
- Have no I/O, global mutable state, random behavior, or side effects.
- Be deterministic and safe to call repeatedly.

The package-level `__all__` contains only `"add"`. The private native module is
named `python_rust_mock_architecture._native` and registers only `add` as its
callable. The private module exists to keep the compiled extension separate
from the public Python package namespace.

The package must include a `py.typed` marker and a stub for the native function
so type checkers see the same signature as the runtime API.

## Repository shape

The implementation should follow this shape. Small equivalent changes are
allowed when they preserve the import path and build behavior.

```text
.
├── Cargo.toml
├── Cargo.lock
├── pyproject.toml
├── uv.lock
├── python/
│   └── python_rust_mock_architecture/
│       ├── __init__.py
│       ├── _native.pyi
│       └── py.typed
├── src/
│   └── lib.rs
├── tests/
│   └── python/
│       └── test_add.py
├── .github/
│   └── workflows/
│       └── ci.yml
└── README.md
```

The Python source directory is explicitly `python/` in Maturin configuration.
The compiled module is explicitly named
`python_rust_mock_architecture._native`. The Rust `#[pymodule]` name must match
the final `_native` component.

There is one Python project, so the repository uses one root `pyproject.toml`
and one checked-in `uv.lock`. It does not add an artificial uv workspace for
the Rust crate. If this example later gains multiple Python projects, the
repository can add a uv workspace and keep the shared root lockfile. This
matches uv's distinction between a single project and a workspace of multiple
Python projects.

`Cargo.lock` is also checked in. Generated virtual environments, Rust targets,
compiled extension files, wheels, source distributions, and caches remain
ignored by Git.

## Python project and build configuration

`pyproject.toml` must:

- Declare PEP 621 metadata for the distribution name, version, description,
  README, and `requires-python = ">=3.11"`.
- Use Maturin as the PEP 517 build backend with a bounded major-version range
  (`maturin >= 1, < 2` is the contract; the lockfile selects the exact version).
- Set `tool.maturin.python-source = "python"`.
- Set `tool.maturin.module-name =
  "python_rust_mock_architecture._native"`.
- Declare no runtime Python dependencies.
- Put Maturin, Pytest, Ruff, and Mypy in a development dependency group managed
  by uv.
- Configure Ruff and Mypy in the same file so local checks and CI use the same
  rules.

The lockfile must be generated and updated by uv. It must not be hand-edited.
Normal setup and CI use `uv sync --locked` or an equivalent locked invocation
that fails when metadata and the lockfile disagree.

## Rust extension

The crate is a `cdylib` built with PyO3 and configured for Python extension
module output. It uses the stable Python ABI for Python 3.11 and newer. The
PyO3 dependency enables the `abi3-py311` feature. The exact PyO3 release is
pinned in `Cargo.lock`.

The Rust module contains:

- A private `#[pymodule]` initializer named `_native`.
- One `#[pyfunction]`, `add`, with the behavior above.
- A checked integer addition path that maps conversion and overflow failures to
  Python exceptions.
- Rust unit tests for normal values, zero, negative values, the signed 64-bit
  boundaries, and overflow.

The native module must not export subtraction, multiplication, a CLI, classes,
or unrelated demonstration functions. Internal Rust helpers are allowed when
they are not Python exports.

## Python tests

Python tests run against the built or development-installed extension, not a
Python reimplementation. They must cover:

- The documented package-level import.
- Positive, negative, and zero operands.
- Keyword arguments `a` and `b`.
- Boolean inputs, which follow Python's zero and one behavior.
- The signed 64-bit minimum and maximum values when the result is representable.
- Result overflow in both directions.
- `float` and `str` input failures, plus missing or extra arguments.
- The package's documented public export, including `__all__ == ["add"]`.

The tests must not rely on the repository root accidentally shadowing an
installed package. The README and CI commands must build/install the extension
before running Python tests.

## Required developer checks

The README must document these commands, or exact equivalent commands that use
the same tools and checks:

```text
uv sync --locked
uv run --locked maturin develop --locked
uv run --locked pytest
uv run --locked ruff format --check .
uv run --locked ruff check .
uv run --locked mypy python tests/python
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --locked
uv lock --check
uv run --locked maturin build --locked --sdist --out dist
```

The commands must work from a clean checkout after installing uv and a stable
Rust toolchain. The setup must not require a globally installed Python package
manager or a manually activated virtual environment.

## Packaging verification

The project must provide a reproducible build check:

1. Run `uv run --locked maturin build --locked --sdist --out dist`. The two
   `--locked` flags check `uv.lock` and `Cargo.lock`, respectively.
2. Confirm that the expected wheel and source distribution are produced under
   the ignored distribution directory.
3. Install the wheel into a fresh isolated environment without adding the
   checkout to `PYTHONPATH`.
4. Run a small import and arithmetic smoke test from outside the repository.
5. Confirm that the installed package exposes `add` and that the compiled
   extension loads successfully.

The check must not pass by importing an in-tree Python fallback. There is no
fallback implementation.

## Continuous integration

`.github/workflows/ci.yml` must run on pull requests and pushes to the default
branch. It must:

- Install uv and a stable Rust toolchain.
- Use the checked-in `uv.lock` and `Cargo.lock` in locked mode.
- Run Python formatting, linting, type checking, and tests.
- Run Rust formatting, Clippy with warnings denied, and Rust tests.
- Build the wheel and source distribution.
- Run the isolated wheel smoke test.
- Exercise at least the minimum supported Python version and one newer
  supported Python version. The workflow may use a Python matrix for this.

CI does not publish a package and does not need release automation. A failure
in any check fails the workflow.

## README requirements

The README must be a usable guide for someone who has not seen the repository.
It must explain:

- What the example demonstrates and why the Rust module is private.
- The package import and `add` behavior, including integer range and overflow.
- Required tools: uv and a stable Rust toolchain.
- Clean local setup and the command that installs the extension in the uv
  environment.
- How to run Python and Rust tests.
- How to run format, lint, type, lock, and build checks.
- How to perform the isolated wheel smoke test.
- The relevant source tree and the role of `pyproject.toml`, `Cargo.toml`,
  `uv.lock`, and `Cargo.lock`.
- The non-goals and the absence of a Python fallback.

Commands in the README must be copyable and must match CI.

## Non-goals

This change does not include:

- A command-line interface.
- Additional arithmetic functions or a larger Rust API.
- A Python fallback implementation.
- A web service, database, async runtime, benchmark suite, or performance claim.
- Publishing to PyPI or another package index.
- Cross-compilation or a release-wheel matrix for every operating system.
- A second Python workspace member before there is a real second Python project.
- Unrelated repository cleanup.

## Alternatives considered

### Pure Rust Maturin layout

The default pure-Rust layout is compact, but it makes the compiled module the
package boundary and is less useful for demonstrating a mixed package with
Python-owned documentation and typing metadata. It also does not use the
dedicated Python source directory requested for this example.

### Dedicated `python/` source layout with a root Rust crate

This is the recommended approach. It keeps `python/` as an explicit package
source root, puts Rust in the normal root Cargo layout, and lets the package
re-export one carefully named function. It also avoids the common import-path
pitfall documented by Maturin.

### Separate `python/` and `rust/` projects with a uv workspace

This is useful once a repository has multiple independently managed Python
packages. For one extension package it adds a second manifest and more path
configuration without adding a user-visible benefit. The first version stays
as one uv project with a shared lockfile.

## Acceptance criteria

The change is complete when all of the following are true:

1. A clean checkout can be synced with uv in locked mode.
2. The extension builds through Maturin and imports as
   `python_rust_mock_architecture._native`.
3. `from python_rust_mock_architecture import add` works after the documented
   setup command.
4. `add` meets the argument, result, error, and side-effect contract.
5. Only `add` is documented and re-exported as the package's public callable.
6. Rust and Python tests cover normal operation and failure boundaries.
7. Formatting, linting, type checking, lock checks, Rust checks, and packaging
   verification are documented and pass.
8. CI runs the same checks and fails on a stale lockfile or any test/tool error.
9. The README is sufficient to reproduce setup, testing, and wheel verification.
10. Implementation begins from this approved revision of the specification.

## References

- [uv project structure and lockfiles](https://docs.astral.sh/uv/concepts/projects/layout/)
- [uv locking and syncing](https://docs.astral.sh/uv/concepts/projects/sync/)
- [uv workspaces](https://docs.astral.sh/uv/concepts/projects/workspaces/)
- [Maturin project layout](https://www.maturin.rs/project_layout.html)
- [Maturin configuration](https://www.maturin.rs/config.html)
- [Maturin distribution builds](https://www.maturin.rs/distribution.html)
- [PyO3 Python modules](https://pyo3.rs/main/module)
- [PyO3 building and distribution](https://pyo3.rs/main/building-and-distribution)
- [GitHub's Rust workflow guidance](https://docs.github.com/en/actions/tutorials/build-and-test-code/rust)
