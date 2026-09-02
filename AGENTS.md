# Agent playbook for adding Rust to Python

Use this repository as a working reference when a Python project needs a Rust
extension. The goal is reliable integration. Do not optimize code, add benchmarks,
or claim a speed improvement unless a separate, approved task requires it.

## Rules that do not change

- Preserve the target project's package manager, build process, source layout,
  framework conventions, test runner, and command names.
- Add the smallest native API that satisfies the approved Python contract.
- Keep the compiled module private, usually `package_name._native`. Re-export the
  supported API from Python.
- Keep FastAPI, Django, CLI, and business-domain concepts out of the Rust crate.
- Do not change the public API, minimum Python version, error types, build backend,
  or deployment model without approval.
- Do not introduce a Python fallback unless the target project requires one and
  tests both implementations.
- Do not use Rust panics for expected failures. Map failures to documented Python
  exceptions.

## This reference repository

This repository is a Maturin mixed project managed by uv. It exposes one private
PyO3 module, `python_rust_mock_architecture._native`, and one public Python
function, `add`.

Use the Makefile instead of assembling local commands from memory:

```text
make help
make develop
make test
make check
```

`make coverage-python` requires the locked `pytest-cov` dependency and enforces
100% coverage of the small Python wrapper. `make coverage-rust` requires
`cargo-llvm-cov` and enforces 80% line coverage. Install the Rust tool with:

```text
cargo +stable install cargo-llvm-cov --locked
```

CI uses `taiki-e/install-action@cargo-llvm-cov` instead of compiling that tool on
every run.

## 1. Inspect the target project

Before editing, record these facts in the task notes or handoff:

- Package manager and lockfile: uv, pip or virtualenv, Poetry, PDM, or another
  existing tool.
- Build backend from `[build-system]`: Maturin, setuptools, Hatchling,
  Poetry Core, PDM Backend, or another backend.
- Supported Python versions and platforms.
- Import-package name, distribution name, and source layout such as `src/`,
  `python/`, or a flat package.
- Existing setup, test, lint, type-check, build, and CI commands.
- Whether the deliverable is a published library, FastAPI service, Django service,
  standalone program, or a combination.
- Current public behavior, exception types, startup behavior, and deployment
  process.

Useful read-only checks include:

```text
rg --files
git status --short --branch
python --version
rustc --version
cargo --version
```

Also inspect `pyproject.toml`, lockfiles, CI workflows, container files, existing
agent instructions, and the current tests. Do not infer the build backend from the
package manager. They are separate choices.

## 2. Choose an integration topology

### Mixed distributable package

Use this when one published Python distribution should contain the extension and
the project can safely use Maturin as its build backend.

Typical layout:

```text
pyproject.toml
Cargo.toml
Cargo.lock
src/lib.rs
src_python/package_name/__init__.py
src_python/package_name/_native.pyi
src_python/package_name/py.typed
tests/
```

Map the native library into the existing Python package:

```toml
[build-system]
requires = ["maturin>=1,<2"]
build-backend = "maturin"

[tool.maturin]
python-source = "src_python"
module-name = "package_name._native"
features = ["extension-module"]
```

Adapt `python-source` to the target layout. Never rename the existing package just
to match this example. Before replacing another backend, check editable installs,
package data, entry points, plugins, source distributions, release jobs, and
downstream build automation. Stop for approval if any behavior would change.

### Internal native package

Use this for FastAPI, Django, and standalone applications that must keep their
current build backend. It is also the safe choice when a library relies on backend
features that Maturin cannot replace without migration work.

Typical layout:

```text
app-or-project-files/
native/
  pyproject.toml
  Cargo.toml
  Cargo.lock
  src/lib.rs
  python/project_native/__init__.py
  python/project_native/_native.pyi
  python/project_native/py.typed
```

Build `native/` as its own wheel and add that local or published distribution
through the application's current package manager. The application imports a
Python adapter. The adapter imports `project_native`; application code does not
import the Rust crate name or raw extension directly.

## 3. Preserve the package manager

Use the row that matches the target repository. Keep its lockfile checked in when
the project already tracks one.

| Tool | Add development tools | Run Maturin | Run tests |
| --- | --- | --- | --- |
| uv | `uv add --dev maturin pytest-cov` | `uv run maturin develop` | `uv run pytest` |
| pip or virtualenv | add tools to the existing development extra or requirements file, then install it | `.venv/bin/maturin develop` | `.venv/bin/python -m pytest` |
| Poetry | `poetry add --group dev maturin pytest-cov` | `poetry run maturin develop` | `poetry run pytest` |
| PDM | `pdm add -dG dev maturin pytest-cov` | `pdm run maturin develop` | `pdm run pytest` |

Do not add a second environment manager. Do not create `uv.lock` in a Poetry or
PDM project. Do not add Poetry or PDM metadata to a uv or pip project.

For an internal native package, run Maturin with its manifest or from its
directory. Add the resulting package with the target manager's supported local
path dependency syntax. Keep that dependency reproducible in CI and deployment
builds.

## 4. Add the native boundary

1. Add a Cargo `cdylib` crate and a checked-in `Cargo.lock` for reproducible
   application and example builds.
2. Add PyO3 and enable its `extension-module` feature for extension builds.
3. Use an `abi3` feature only when the target Python range and required CPython
   APIs support it. Match the feature to the minimum supported Python version.
4. Name the PyO3 module to match the final component of Maturin's `module-name`.
5. Put public documentation and compatibility behavior in Python. Keep the raw
   extension private.
6. Add a `.pyi` stub for the extension and `py.typed` for typed packages.
7. Exclude `dist/`, `target/`, environments, caches, bytecode, and generated native
   binaries from Cargo source packages.

Keep `extension-module` out of the default feature set used by `cargo test` and
`cargo llvm-cov`. Extension-style linking can leave CPython symbols unresolved in
Rust test binaries. Let Maturin enable that feature when it builds the extension,
as this repository does through `[tool.maturin].features`.

For each native function, write down the following contract before coding:

- Python signature and accepted argument types.
- Return type and ownership rules.
- Exact Python exceptions for invalid input and Rust failures.
- Threading, cancellation, and side-effect behavior.
- Serialization or conversion limits across the boundary.

## 5. Integrate with the Python project

### Python library or module

- Put the extension under the existing import package.
- Re-export only supported names from Python and keep `__all__` explicit.
- Test the public import and the installed wheel, not only `_native`.
- Confirm that package data, entry points, and typing metadata survive the backend
  change.

### FastAPI

- Import the native package in a Python service module.
- Call that service from dependencies or route handlers.
- Keep request parsing, response models, HTTP errors, and dependency injection in
  Python.
- Do not run a long native call directly on the asynchronous event loop. Use the
  application's established worker or thread-offload pattern when needed.
- Add a route test that reaches the native-backed service through FastAPI's normal
  test client. This reference repository does not add a FastAPI example app.

### Django

- Import the native package in a service module.
- Keep models, settings, migrations, middleware setup, and app initialization free
  of native work.
- Do not perform database or network access at native-module import time.
- Add a service test and a view, command, task, or other normal Django call-path
  test. This reference repository does not add a Django example app.

### Standalone program or script

- Wrap native calls in the program's normal Python module boundary.
- Keep CLI parsing, configuration, logging, and process exit behavior in Python.
- Test the Python entry point and wrapper. Do not make entry points import a Rust
  crate name directly.

## 6. Keep a stable Makefile surface

When the target repository has a Makefile, preserve working targets and add only
missing commands. When it has no Makefile, this repository's target names are the
recommended interface. Rewrite target bodies for the existing package manager.

| Target | Contract in this repository |
| --- | --- |
| `setup` | sync locked Python dependencies, then fetch locked Rust dependencies |
| `develop` | run `setup`, then install the native extension into the development environment |
| `test-python` | run `develop`, then execute Python binding tests |
| `test-rust` | fetch Rust dependencies, then execute Rust unit tests |
| `test` | run both test suites |
| `coverage-python` | run binding tests with `pytest-cov` and fail below 100% |
| `coverage-rust` | run `cargo llvm-cov` and fail below 80% line coverage |
| `lint-python` | run Ruff format-check and lint |
| `lint-rust` | run Cargo format-check and Clippy with warnings denied |
| `lint` | run both lint groups |
| `typecheck` | install the extension, then run Mypy |
| `build` | build one wheel and one source distribution into `dist/` |
| `package-check` | build twice in a fresh directory, inspect exact artifacts, and smoke-test the wheel |
| `check` | run lint, typing, tests, coverage, and package verification in order |

Targets must be non-interactive, stop on the first error, and work in CI. Do not
run native builds concurrently. Missing-tool errors must name an installation
command.

## 7. Test every boundary

| Layer | Required evidence |
| --- | --- |
| Rust unit | normal values, boundary values, and every mapped error branch |
| Python binding | public wrapper, native loading, arguments, returns, exceptions, and typing surface |
| Framework | a normal route, view, command, task, or service call in the adopter repository |
| Package | wheel and source distribution build, clean archive contents, isolated wheel install and import |

When a native function is added or changed:

1. Add or update Rust unit tests.
2. Add or update Python contract tests.
3. Update `_native.pyi` and Python re-exports.
4. Add the adopter's framework call-path test when the function is wired into an
   application.
5. Run coverage and keep the repository's accepted thresholds.
6. Run the packaged-wheel smoke test outside the checkout.

Coverage is a missing-test signal, not permission to write tests that only execute
lines. Assert behavior and error mapping. Adopter repositories keep their existing
coverage tools and thresholds unless a separate change is approved.

## 8. Verify packaging and CI

- Build twice in a fresh temporary output directory.
- Derive the distribution name and version from `pyproject.toml`.
- Require exactly one matching wheel and one matching source distribution after
  repeated builds.
- Reject unrelated archives, nested `dist/` or `target/`, environments, caches,
  bytecode, and compiled extensions in the source distribution.
- Install the wheel into a fresh environment with the minimum supported Python.
- Run the smoke import from outside the repository so the source tree cannot mask
  a broken wheel.
- Run the Make targets in CI instead of maintaining a second command list.
- Report platforms and Python versions that CI did not exercise.

## Stop conditions

Stop and ask for approval when the work would change any of these:

- Package manager or build backend.
- Distribution name, import package, or public API.
- Minimum Python version or stable-ABI policy.
- Existing exception types or fallback behavior.
- Framework startup, deployment, or release process.
- Supported operating systems or architectures.

## Definition of done

- The target project's existing setup and tests still pass.
- The native API contract is documented and typed.
- Rust unit, Python binding, and adopter framework tests pass where applicable.
- Python and Rust coverage commands pass with accepted thresholds.
- Format, lint, type, lockfile, build, archive, and isolated-wheel checks pass.
- CI uses the same command surface as local development.
- Documentation names prerequisites, commands, supported platforms, and known
  limits.
- No optimization, benchmark, package-manager migration, or unapproved fallback
  was added.
