# Rust Integration Template Design

**Status:** Approved for implementation
**Designed:** 2026-09-02

## Purpose

Turn this repository into a runnable reference and an agent playbook for adding a
Rust extension to an existing Python project. The guidance must apply to Python
libraries, FastAPI applications, Django applications, and standalone Python
programs without forcing them to change package managers.

The current `add` extension remains the small native reference. The private
`verified-core` crate now holds the Verus-checked arithmetic examples, while the
PyO3 adapter remains outside the proved boundary. The template does not optimize
Python code or claim that Rust improves performance. Its job is to establish a
safe native-module boundary that agents can extend later.

## Chosen Approach

Use one runnable reference package with a root `AGENTS.md`, a stable Makefile
command surface, and integration recipes for common Python project shapes.

This avoids duplicate FastAPI and Django example applications. It also avoids an
automatic migration script that could rewrite an unfamiliar project incorrectly.
Agents must inspect the target repository, select a topology, and adapt the
reference deliberately.

## Required Changes

### Root `AGENTS.md`

The root playbook is the main operating guide for coding agents. It must contain:

1. The purpose and limits of the template.
2. A discovery checklist for the target project's package manager, build backend,
   Python version range, source layout, test commands, CI, deployment method, and
   public API.
3. A topology decision that distinguishes a mixed distributable package from an
   internal native package used by an application.
4. Setup instructions for Cargo, PyO3, Maturin, the private native module, Python
   re-exports, type stubs, lockfiles, tests, builds, and CI.
5. Integration recipes for Python libraries, standalone programs, FastAPI, and
   Django.
6. A package-manager command matrix for uv, pip or virtualenv, Poetry, and PDM.
7. Rules for testing at the Rust, Python binding, framework, and packaged-wheel
   boundaries.
8. A change checklist for agents that extend the native API.
9. Failure-handling rules and a definition of done.

The instructions must tell agents to preserve the target repository's package
manager, Python layout, framework conventions, and existing command names. Agents
may add Maturin as a build or development tool, but they must not migrate the
whole project to uv, Poetry, PDM, or pip.

### Integration Topologies

The playbook must explain two supported topologies.

#### Mixed distributable package

Use a Maturin mixed project when the Python project publishes one package that
should contain the Rust extension. Keep normal Python modules in the existing
source tree. Install the extension as a private child module such as
`package_name._native`, then re-export the supported API through Python.

Before changing the build backend, the agent must check how the project currently
builds source distributions and wheels. If replacing that backend would break
plugins or release automation, use the internal native package topology instead.

#### Internal native package

Use a separate native package directory when a FastAPI, Django, or standalone
application should keep its current build backend. Build the native wheel with
Maturin and install it into the application's existing environment. The Python
application depends on that local or published wheel through its current package
manager.

The Rust crate must not import FastAPI or Django concepts. Framework code calls a
small Python service or adapter, and that adapter calls the native module.

### Framework Recipes

The library recipe must cover package layout, private extension naming, public
re-exports, type metadata, wheel contents, and import tests.

The FastAPI recipe must place native calls behind a Python service function. Route
tests must prove that the application reaches the native implementation. A long
running native call must not be placed directly on the asynchronous event loop.

The Django recipe must place native calls in a service module rather than models,
settings, migrations, or view initialization. Tests must cover the service and at
least one Django call path that uses it. Imports must not perform database access
or other startup side effects.

The standalone-program recipe must keep native imports behind the program's
normal Python module boundary. CLI or script entry points must not import a Rust
crate name directly.

This reference repository must not add FastAPI or Django example applications.
The framework recipes instead require adopter repositories to add integration
tests through their normal route, view, command, or service call paths.

### Makefile Interface

Add a root `Makefile` that gives humans and agents stable commands for this
reference repository:

- `help`: list targets and their purpose.
- `setup`: synchronize the existing Python environment and Rust dependencies.
- `develop`: build and install the native extension in the active development
  environment.
- `test-python`: run Python binding tests.
- `test-rust`: run Rust unit tests.
- `test`: run both test suites.
- `coverage-python`: run Python coverage for the binding layer.
- `coverage-rust`: run Rust source coverage.
- `lint-python`: run Python format and lint checks.
- `lint-rust`: run Rust format and Clippy checks.
- `lint`: run both lint groups.
- `typecheck`: check Python typing.
- `build`: build the wheel and source distribution.
- `package-check`: reject generated artifacts in the Cargo package and source
  distribution, then smoke-test the wheel outside the checkout.
- `check`: run the complete local verification sequence.

This repository's target bodies may use uv because uv is already its package
manager. `AGENTS.md` must tell agents to keep the target names but rewrite their
bodies to use the target repository's existing package manager and test tools.
Make targets must be non-interactive and suitable for CI.

The coverage targets may install or require dedicated coverage tools. Their
prerequisites and failure messages must be explicit. The initial thresholds must
cover the current native logic without setting an arbitrary policy for projects
that copy the playbook. The implementation must name the exact Python and Rust
coverage tools, commands, and thresholds in both `AGENTS.md` and `Makefile`.

Target dependencies must run in this order:

1. `setup` synchronizes locked Python dependencies and fetches locked Rust
   dependencies.
2. `develop` depends on `setup` and installs the native extension before Python
   binding tests, Python coverage, or type checks run.
3. `test` runs `test-python` and `test-rust` after their setup requirements.
4. `build` creates the normal developer artifacts in `dist/`.
5. `package-check` performs its own two builds in a fresh temporary output
   directory. It must not trust artifacts left by `build`.
6. `check` runs lint, type checks, tests, coverage, and `package-check` in a
   deterministic sequence. It must not run conflicting native builds in parallel.

### Native API Rules

The existing `_native.add` implementation remains the reference behavior. New
native functions must follow these rules:

- Keep the extension private and expose supported functions through Python.
- Define argument, return, and exception behavior before implementation.
- Add matching Python stubs or generated typing information.
- Return mapped Python exceptions for expected failures. Do not expose Rust
  panics as an error strategy.
- Avoid import-time work and framework dependencies in Rust.
- Preserve a Python fallback only when the target project's contract requires
  one. Do not add an untested fallback automatically.

## Agent Workflow

The playbook must direct agents through this sequence:

1. Record the target project's current tooling and commands.
2. Define the smallest Python-facing native API.
3. Choose the mixed-package or internal-package topology.
4. Add the Rust crate and private extension mapping.
5. Add the Python adapter, re-export, and typing information.
6. Add Rust unit tests and Python binding tests before framework wiring.
7. Add the framework integration and its existing style of tests.
8. Add or adapt Makefile commands without replacing working project commands.
9. Update CI and package verification.
10. Run the complete local check and report any platform coverage not exercised.

Agents must stop and ask before changing the package manager, public API, minimum
Python version, build backend, deployment model, or error semantics when those
choices are not already approved.

## Testing and Verification

The implementation is complete when all of these conditions hold:

- Existing `add` behavior and its Python and Rust tests still pass.
- The Makefile targets call the same locked tools already used by this repository.
- Rust tests exercise normal results, boundary values, and error paths.
- Python tests exercise the public wrapper, native loading, typing surface, and
  exception mapping.
- Framework recipes require an integration test through the framework's normal
  call path.
- Python and Rust coverage targets run successfully and cover the current source.
- Wheel and source-distribution builds pass twice in a nonempty output directory.
- `package-check` uses a fresh temporary output directory and derives the expected
  distribution name and version from project metadata. It requires exactly one
  matching wheel and one matching source distribution after the repeated builds.
  Unrelated or stale archives must fail the check rather than satisfy it.
- The source distribution contains no nested builds, compiled extensions, caches,
  or bytecode.
- A fresh Python environment can install the wheel and call the public API from
  outside the checkout.
- CI uses the Makefile command surface for the checks it shares with local work.
- `README.md` points humans to the playbook and documents the Makefile quick start.

## Error Handling

Make targets must stop on the first failing command and return a nonzero status.
Missing tools must produce a short message that names the required installation.
Package inspection must fail if it cannot locate exactly the expected wheel and
source distribution.

The playbook must tell agents to preserve target-project exception behavior and
to test failures on both sides of the Python and Rust boundary. Silent fallback,
ignored build errors, and exception-type changes are not allowed without an
approved contract change.

## Non-goals

- Performance profiling, benchmarking, or optimization.
- Automatic rewriting of arbitrary repositories.
- Complete example applications for each Python framework.
- Replacing the target project's package manager or framework conventions.
- Publishing packages or deployment images.
- A cross-platform release-wheel matrix beyond the existing CI scope.
