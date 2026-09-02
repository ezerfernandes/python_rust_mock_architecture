# Python and Rust add extension

This repository is a small example of a Python package with a Rust extension.
It uses PyO3 to implement one checked integer operation and Maturin to build
and install the extension. Python owns the package namespace, public
documentation, and typing metadata. The native module stays private so the
package can control its public API.

## API

```python
from python_rust_mock_architecture import add

assert add(2, 3) == 5
```

`add(a, b)` accepts Python integers in the signed 64-bit range,
`[-2**63, 2**63 - 1]`, and returns their sum. Inputs or results outside that
range raise `OverflowError`. Floats, strings, missing arguments, and extra
arguments raise `TypeError`. Boolean operands follow normal Python integer
behavior, so `True` is one and `False` is zero.

The package exports only `add` through `__all__`. The compiled extension is
loaded as `python_rust_mock_architecture._native`; it is an implementation
detail rather than a second public API.

## Requirements

Install [uv](https://docs.astral.sh/uv/) and a stable Rust toolchain with
`rustc` and `cargo`. The project requires Python 3.11 or newer. uv can install
the selected Python interpreter with `uv python install 3.11`.

## Local setup

From the repository root:

```text
uv python install 3.11
uv sync --locked
uv run --locked maturin develop --locked
```

The last command builds the Rust extension and installs it into the uv
environment. There is no Python fallback implementation, so the extension
must be built before importing the package from a clean checkout.

## Tests and quality checks

Run the Python tests against the development-installed extension:

```text
uv run --locked pytest
```

Run the Rust unit tests:

```text
cargo test --locked
```

The checks used by CI are:

```text
uv run --locked ruff format --check .
uv run --locked ruff check .
uv run --locked mypy python tests/python
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
uv lock --check
```

## Wheel and source distribution

Build both distribution formats twice into the ignored `dist/` directory.
The second build checks that the first build's artifacts are not copied into
the new source distribution:

```bash
mkdir -p dist
uv run --locked maturin build --locked --sdist --out dist
uv run --locked maturin build --locked --sdist --out dist
find dist -maxdepth 1 -type f \( -name '*.whl' -o -name '*.tar.gz' \) -print
sdist="$(find dist -maxdepth 1 -type f -name '*.tar.gz' -print -quit)"
if tar -tzf "$sdist" | grep -Eq '(^|/)(dist|target|.venv|.mypy_cache|.pytest_cache|.ruff_cache|__pycache__)(/|$)|\.(whl|tar\.gz|so|pyd|dylib|dll|pyc|pyo)$'; then
  echo "generated artifacts found in sdist"
  exit 1
fi
```

Cargo's source-package listing can be inspected with:

```text
cargo package --list --allow-dirty --locked
```

To verify the wheel without importing from the checkout, create an isolated
environment and run the smoke test from a temporary directory:

```bash
wheel="$(find dist -maxdepth 1 -type f -name '*.whl' -print -quit)"
smoke_env="$(mktemp -d)"
uv venv --python 3.11 "$smoke_env"
uv pip install --python "$smoke_env/bin/python" "$wheel"
(
  cd "$(mktemp -d)"
  "$smoke_env/bin/python" -c '
from pathlib import Path

import python_rust_mock_architecture as package
import python_rust_mock_architecture._native as native

assert package.__all__ == ["add"]
assert package.add(40, 2) == 42
assert Path(native.__file__ or "").suffix in {".so", ".pyd"}
'
)
```

The import check runs outside the repository and confirms that the installed
package loads a compiled extension.

## Source tree

```text
python/                       Python package namespace and typing metadata
src/lib.rs                    PyO3 module and checked Rust addition
tests/python/test_add.py      Python API and boundary tests
pyproject.toml                Python metadata, Maturin, Ruff, and Mypy config
Cargo.toml                    Rust crate and PyO3 dependency
uv.lock                      Locked Python development dependencies
Cargo.lock                   Locked Rust dependencies
.github/workflows/ci.yml     CI checks and isolated wheel verification
```

The project has one Python package and one uv lockfile. It does not use an
artificial uv workspace. A workspace would make sense if the repository later
gains another independent Python project.

## Non-goals

This example has no CLI, extra arithmetic functions, Python fallback, web
service, database, async runtime, benchmark suite, publishing workflow, or
cross-platform release-wheel matrix.
