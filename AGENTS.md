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
100% coverage of the small Python wrapper. `make coverage-rust` runs
`cargo llvm-cov --workspace` and enforces 80% line coverage. It requires
`cargo-llvm-cov`; install the Rust tool with:

```text
cargo +stable install cargo-llvm-cov --locked
```

CI uses `taiki-e/install-action@cargo-llvm-cov` instead of compiling that tool on
every run. A dedicated CI job installs the pinned Verus archive after checking
its SHA-256 digest and runs `make verus-verify`; the package job depends on that
proof job. Local `make check` runs the same proof target.

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
| `test-rust` | fetch Rust dependencies, then execute all workspace Rust unit tests |
| `test` | run both test suites |
| `coverage-python` | run binding tests with `pytest-cov` and fail below 100% |
| `coverage-rust` | run `cargo llvm-cov --workspace` and fail below 80% line coverage |
| `lint-python` | run Ruff format-check and lint |
| `lint-rust` | run Cargo format-check and workspace Clippy with warnings denied |
| `lint` | run both lint groups |
| `typecheck` | install the extension, then run Mypy |
| `build` | build one wheel and one source distribution into `dist/` |
| `package-check` | build twice in a fresh directory, inspect exact artifacts, and smoke-test the wheel |
| `check` | run lint, full Verus verification, typing, tests, coverage, and package verification in order |

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

## 9. Verify the Rust core with Verus

Verus is a static verifier for Rust. It checks executable Rust against
user-written specifications with an SMT solver. Specifications and proofs are
ghost code: Verus erases them before normal compilation, so verification does not
add runtime checks.

Use the [official Verus tutorial and
reference](https://verus-lang.github.io/verus/guide/overview.html) as the source of
truth. Verus is under active development and does not support every Rust feature
or library. Check the
[supported-features table](https://verus-lang.github.io/verus/guide/features.html)
before choosing a design.

### Verification scope and trust boundary

Verify a pure Rust core, not the PyO3 adapter. Keep Python extraction, Python
exceptions, module registration, and other interpreter operations in the root
PyO3 crate. Put algorithms and their Verus specifications in a private Rust crate
that has no Python or PyO3 concepts.

Say exactly what was verified. A successful proof for the core does not verify
Python, CPython, PyO3, Maturin, Verus, Z3, rustc, LLVM, or an
`#[verifier::external_body]`. Those items are trusted or covered by runtime
tests. Record every assumption and external specification in the handoff.

Code called by Python is called by unverified code. Do not expose a safe Rust
function with a Verus precondition that Python can violate. Prefer a precondition-
free public core function that validates input and returns `Result` or `Option`.
An internal helper may use `requires` after the public function proves the
precondition. The guide recommends checking such APIs with
`-V check-safe-api`.

### Install and pin the toolchain

Use an official Verus release that contains both `verus` and `cargo-verus`.
This repository pins Verus `0.2026.08.30.b432e82`, Rust
`1.97.1-x86_64-unknown-linux-gnu`, Z3 `4.16.0`, and `vstd`
`0.0.0-2026-08-30-0159`. The exact archive checksum and installation commands
are in [`docs/verus-toolchain.md`](docs/verus-toolchain.md). Pin the Verus
release, its required Rust toolchain, and the compatible `vstd` revision
together. Do not use an unpinned branch in CI. Follow the official
[installation instructions](https://github.com/verus-lang/verus/blob/main/INSTALL.md)
and the [Cargo integration
guide](https://verus-lang.github.io/verus/guide/cargo_verus.html).

Confirm the installed commands before changing code:

```text
verus --version
cargo verus --help
rustc --version
cargo --version
```

The Verus release directory and the rustup Cargo shims must both be on `PATH`.
The installation document shows the complete export. On the pinned Linux
toolchain, `rustc --version` starts with `rustc 1.97.1` and `verus --version`
reports `0.2026.08.30.b432e82`.

For a crate opted into Cargo verification, add `vstd`, import its prelude, and
mark the package for verification:

```toml
[package.metadata.verus]
verify = true

[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(verus_only)'] }
```

Use the dependency declaration generated or required by the pinned Verus release.
Do not guess a `vstd` version. Normal `cargo build` must continue to compile
annotated code.

Use the repository Make targets rather than copying a partial Cargo command:

```text
make verus-hygiene
make verus-verify
make verus-focus
```

`verus-verify` runs
`cargo verus verify --workspace --locked --fwd-verus-args roots -- -V check-api-safety`
after the proof-hygiene scan. It checks the opted-in core and
its dependencies while forwarding the API-safety argument only to the selected
workspace roots.
`verus-focus` runs the same proof for `verified-core` without rechecking
dependencies and is only a development shortcut. Run the full target before
handoff. A normal extension build still uses Maturin and Cargo.

The current private core proves total checked `i64` addition, checked
left-to-right summation, sorted lower-bound search, and first-match binary
search. The root PyO3 adapter delegates all four public functions to that core
and maps its fallible results to Python exceptions and values. Python, PyO3, and
the adapter remain outside the proved boundary.

The small `tools/verus-spike` crate is excluded from the root workspace and has
its own lockfile. To validate the pinned release independently, run:

```text
cargo verus verify --manifest-path tools/verus-spike/Cargo.toml --locked
cargo build --manifest-path tools/verus-spike/Cargo.toml --locked
```

The spike proof is a toolchain check. It does not prove the PyO3 adapter or
Python package.

### Start a Verus source file

A typical verified module imports the prelude and puts verified items inside the
`verus!` macro:

```rust
use vstd::prelude::*;

verus! {
    spec fn double(x: int) -> int {
        x + x
    }

    fn checked_double(x: i64) -> (result: Option<i64>)
        ensures
            result is Some ==> result->0 as int == double(x as int),
            result is None ==> double(x as int) > i64::MAX
                || double(x as int) < i64::MIN,
    {
        x.checked_add(x)
    }
}
```

Verus verifies items inside `verus!` and normally ignores items outside it.
Attribute-based syntax also exists, but prefer one style consistently within a
crate. Short tutorial excerpts often omit the prelude and enclosing macro; real
files need both.

### Know the three code modes

| Mode | Purpose | Compiled |
| --- | --- | --- |
| `exec` | Ordinary executable Rust | yes |
| `spec` | Pure mathematical definitions used in contracts | no |
| `proof` | Lemmas and steps that establish specifications | no |

`exec` is the default, so `fn` usually means `exec fn`. A `spec fn` can
call specification code only. A `proof fn` can call specification and proof
code. Executable functions can contain `proof { ... }` blocks, but executable
values must never depend on erased ghost values.

Variables follow similar modes. Executable variables are compiled. Ghost
variables are erased and may contain mathematical values. Tracked variables are
also erased but obey linearity and lifetime rules. Do not introduce tracked state
until the proof needs ownership reasoning that ordinary borrowing cannot express.

### Write modular contracts

Use `requires` for facts a verified caller must establish and `ensures` for
facts a function guarantees:

```rust
fn increment(x: u64) -> (result: u64)
    requires
        x < u64::MAX,
    ensures
        result == x + 1,
{
    x + 1
}
```

Name a return value with `-> (name: Type)` so an `ensures` clause can refer to
it. Each function is verified from its own preconditions, body, and callees'
postconditions. Callers do not get to inspect an executable callee's body.

`recommends` describes the intended domain of a specification function without
creating the same logical obligation as `requires`. Use it to detect likely
specification mistakes, not to hide a needed executable check.

`assert(predicate)` asks Verus to prove a static fact. It is not Rust's runtime
`assert!(predicate)`. `assume(predicate)` accepts a fact without proof and can
make false theorems pass. Temporary assumptions can help diagnose a proof, but no
completed project-authored proof may contain `assume`.

### Use mathematical integers in specifications

Verus adds `int` for all mathematical integers and `nat` for nonnegative
mathematical integers. They exist only in ghost code. Prefer `int` for general
arithmetic specifications, `nat` for lengths or decreasing natural measures,
and Rust fixed-width types for executable values.

Cast executable integers explicitly in specifications when inference is unclear:

```rust
assert((value as int) <= i64::MAX);
```

Arithmetic in ghost code is mathematical and commonly widens to `int`.
Executable fixed-width arithmetic must be proved not to overflow or must use a
checked operation. This difference is useful: describe the ideal mathematical
result in the specification, then prove that the executable branch either
produces it or reports overflow.

Remember these edge cases:

- `int` and `nat` cannot be stored in executable variables.
- A narrowing `as` cast needs a proved range. Out-of-range ghost casts have an
  arbitrary target value unless truncation is requested and specified.
- Ghost division and remainder use Euclidean semantics, which differ from Rust
  truncation for some negative operands.
- Division by zero is unspecified in ghost code and fails in executable code.

### Express logic clearly

Verus supports chained comparisons and implication:

```rust
0 <= index < length
sorted(values) ==> lower_bound(values, target) <= values.len()
```

`a ==> b` means implication, and `a <==> b` means boolean equivalence.
`&&&` and `|||` are low-precedence conjunction and disjunction forms used as
one operator per line. Use them when a long predicate reads better as a list.

Use `===` when structural equality is required for values whose executable
`==` implementation is not the intended mathematical relation. Sequence, set,
and map equality may need extensional equality, `=~=` or `=~~=`, plus the
appropriate library lemma.

### Relate containers to mathematical views

The view operator `@` exposes the ghost model of many executable containers. A
`Vec<T>` view is a mathematical `Seq<T>`:

```rust
let values: Vec<i64> = Vec::new();
assert(values@.len() == 0);
```

Write algorithm specifications against `Seq`, `Set`, `Map`, and other
`vstd` mathematical types. Keep actual storage executable. For mutable
references, `old(value)` refers to the value at function entry and
`final(value)` refers to the value after the call.

When indexing a sequence, prove `0 <= index < sequence.len()`. A slice or vector
algorithm often needs these facts in both its function contract and its loop
invariants.

### Verify loops and recursion

Every loop needs invariants strong enough to prove three things:

1. The invariant holds before the first iteration.
2. One iteration preserves it.
3. The invariant plus the negated loop condition proves the postcondition.

Loops are verified in isolation by default. They do not automatically inherit
every fact from the enclosing function. Repeat any required precondition, bound,
sortedness fact, sequence relationship, and accumulator fact in the invariant.

A search loop usually records:

- bounds such as `low <= high <= values.len()`;
- which part of the original sequence remains under consideration;
- facts established for indices below `low` and at or above `high`;
- the sortedness fact needed to preserve those partitions;
- a decreasing measure such as `high - low`.

Recursive specification, proof, and executable functions need a `decreases`
clause that proves termination. Multiple measures are lexicographic. Recursive
specification functions have limited unfolding fuel. Use lemmas, `reveal`, or
`reveal_with_fuel` deliberately when automatic unfolding is insufficient. Do
not raise fuel globally to mask a poorly factored proof.

### Use proof functions and solver modes

A `proof fn` is a lemma. Put its premises in `requires` and its conclusion in
`ensures`. A proof body can be empty when the default solver establishes the
result. Otherwise, add local assertions or call smaller lemmas.

```rust
proof fn ordered_step(a: int, b: int, c: int)
    requires
        a <= b,
        b <= c,
    ensures
        a <= c,
{
}
```

Use `assert(fact) by { ... }` to isolate a local proof. Specialized solver modes
are available for proof domains the default solver handles poorly:

```rust
assert(bits ^ bits == 0u64) by (bit_vector);
assert(x * x >= 0) by (nonlinear_arith);
```

Use the narrowest applicable solver and make its required facts explicit. Split a
large proof into lemmas before increasing solver resource limits.

### Quantifiers, witnesses, and triggers

Use quantifiers for properties over every index or for the existence of a value:

```rust
forall|i: int| 0 <= i < values.len() ==> values[i] <= target
exists|i: int| 0 <= i < values.len() && values[i] == target
```

`choose|x: T| predicate(x)` extracts a ghost witness only after proving that one
exists. Use `assert forall|x: T| ... by { ... }` when the solver needs a proof
for an arbitrary value.

SMT quantifiers are instantiated through triggers. Let Verus choose triggers
first. If it reports low confidence, misses a needed instantiation, creates a
matching loop, or exhausts resources, inspect them with `--triggers` and the
quantifier profiler. Add `#[trigger]` or `#![trigger ...]` only when the chosen
term contains all quantified variables and matches expressions that will appear
at use sites. Treat manual triggers as proof code that needs review.

### Mark unverified code honestly

`#[verifier::external_body]` makes Verus trust a function's specification
without checking its body. `assume_specification` attaches a trusted
specification to existing external code. A wrong specification can prove a false
result, so both constructs expand the trusted computing base.

Before adding either construct:

1. Confirm that `vstd` does not already provide the specification.
2. Keep the wrapper as small as possible.
3. Document why it cannot be verified.
4. Add runtime tests for the assumed behavior.
5. List the assumption in the final verification report.

Do not put PyO3 calls inside verified algorithms and then label the whole function
verified through `external_body`. The better boundary is a verified pure core
called by a small unverified adapter.

### Develop and debug proofs

When a proof fails:

1. Run Verus with `--expand-errors` and read recommendation failures.
2. Add local `assert` statements to find the first missing fact.
3. Check integer ranges, casts, sequence-index bounds, and loop invariants.
4. Check recursive-function fuel and whether a definition needs `reveal`.
5. Check quantifier triggers, nonlinear arithmetic, bit-vector operations, and
   extensional equality with their specific tools.
6. Extract a small lemma with only the premises it needs.

Use `--time`, `--time-expanded`, or `--output-json` to find slow proof
queries. Treat proof time as a maintainability property. Prefer smaller lemmas,
opaque complex definitions, selective `reveal`, and isolated `assert ... by`
blocks to a larger global resource limit. A proof that is flaky after unrelated
changes is not finished.

### Contract for each verified Python function

Before implementation, record:

- the public Python signature and accepted container or scalar types;
- the mathematical specification of each successful return value;
- behavior for empty input, duplicates, boundaries, and integer overflow;
- exact Python exceptions for conversion, domain, and Rust failures;
- executable checks that protect internal Verus preconditions;
- the verified core function and the unverified PyO3 adapter;
- every external specification or trusted assumption;
- the Verus, Rust, Z3, and `vstd` versions used for the proof.

For sorted-input algorithms, do not assume Python passes a sorted sequence. Check
the order in executable core code, prove that the check establishes the internal
sortedness predicate, and map unsorted input to the documented Python exception.

For checked arithmetic, specify whether overflow refers to the final mathematical
result or to any intermediate prefix. Tests and proofs must use the same rule.

### Verify every layer

When verified core code changes:

1. Run the full Verus proof target and require zero errors.
2. Scan project-authored proof code for `assume`, axioms, and new external
   specifications.
3. Run Rust unit tests against the compiled executable implementation.
4. Run Python contract tests through the public wrapper.
5. Update the private native stub, Python re-exports, README, and package smoke
   test.
6. Run normal Rust formatting, Clippy, tests, and coverage. A proof does not
   replace those checks.
7. Run the isolated-wheel test outside the checkout.

Negative runtime tests establish error mapping. Where practical, keep a small
compile-fail or proof-failure fixture that demonstrates that weakening a required
invariant or falsifying a postcondition makes Verus reject the code. Do not make
the main suite depend on editing production source during the test.

### Official guide map

Use the focused chapter that matches the current proof:

- [Getting started on the command
  line](https://verus-lang.github.io/verus/guide/getting_started_cmd_line.html)
- [The `verus!`
  macro](https://verus-lang.github.io/verus/guide/verus_macro_intro.html)
- [Preconditions, postconditions, assertions, and ghost
  code](https://verus-lang.github.io/verus/guide/requires_ensures.html)
- [Integer types and
  arithmetic](https://verus-lang.github.io/verus/guide/integers.html)
- [Specification, proof, and executable
  modes](https://verus-lang.github.io/verus/guide/modes.html)
- [Loops and
  invariants](https://verus-lang.github.io/verus/guide/while.html)
- [Quantifiers](https://verus-lang.github.io/verus/guide/quants.html)
- [Proof troubleshooting
  checklist](https://verus-lang.github.io/verus/guide/checklist.html)
- [Calling verified code from unverified
  code](https://verus-lang.github.io/verus/guide/calling-verified-from-unverified.html)
- [Assumptions and trusted
  components](https://verus-lang.github.io/verus/guide/tcb.html)
- [Cargo integration](https://verus-lang.github.io/verus/guide/cargo_verus.html)
- [Syntax by example](https://verus-lang.github.io/verus/guide/syntax.html)

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
- Each opted-in Verus crate verifies with zero errors through the documented
  full-proof command.
- Project-authored proof code contains no `assume`, admitted axiom, or
  undocumented external specification.
- Public core entry points called by Python have no unchecked Verus preconditions.
- The handoff separates proved properties from runtime-tested behavior and lists
  the complete trusted computing base.

<!-- bv-agent-instructions-v3 -->

---

## Beads Workflow Integration

This project uses [beads_rust](https://github.com/Dicklesworthstone/beads_rust) (`br`) for issue tracking and [beads_viewer](https://github.com/Dicklesworthstone/beads_viewer) (`bv`) for graph-aware triage. Issues are stored in `.beads/` and tracked in git. Current `br` workspaces normally export `.beads/issues.jsonl`; older `bd`/legacy workspaces may use `.beads/beads.jsonl`. `bv` auto-discovers the supported JSONL files, so agents should use `br`/`bv` commands instead of hard-coding a single filename.

### Using bv as an AI sidecar

bv is a graph-aware triage engine for Beads projects. Instead of parsing .beads/issues.jsonl / .beads/beads.jsonl directly or hallucinating graph traversal, use robot flags for deterministic, dependency-aware outputs with precomputed metrics (PageRank, betweenness, critical path, cycles, HITS, eigenvector, k-core).

**Scope boundary:** bv handles *what to work on* (triage, priority, planning). `br` handles creating, modifying, and closing beads.

**CRITICAL: Use ONLY --robot-* flags. Bare bv launches an interactive TUI that blocks your session.**

#### The Workflow: Start With Triage

**`bv --robot-triage` is your single entry point.** It returns everything you need in one call:
- `quick_ref`: at-a-glance counts + top 3 picks
- `recommendations`: ranked actionable items with scores, reasons, unblock info
- `quick_wins`: low-effort high-impact items
- `blockers_to_clear`: items that unblock the most downstream work
- `project_health`: status/type/priority distributions, graph metrics
- `commands`: copy-paste shell commands for next steps

```bash
bv --robot-triage        # THE MEGA-COMMAND: start here
bv --robot-next          # Minimal: just the single top pick + claim command

# Token-optimized output (TOON) for lower LLM context usage:
bv --robot-triage --format toon
```

Before claiming, verify current state with `br show <id> --json` or `br ready --json`. `recommendations` can include graph-important blocked or assigned work; only `quick_ref.top_picks` and non-empty `claim_command` fields represent claimable work.

#### Other bv Commands

| Command | Returns |
|---------|---------|
| `--robot-plan` | Parallel execution tracks with unblocks lists |
| `--robot-priority` | Priority misalignment detection with confidence |
| `--robot-insights` | Full metrics: PageRank, betweenness, HITS, eigenvector, critical path, cycles, k-core |
| `--robot-alerts` | Stale issues, blocking cascades, priority mismatches |
| `--robot-suggest` | Hygiene: duplicates, missing deps, label suggestions, cycle breaks |
| `--robot-diff --diff-since <ref>` | Changes since ref: new/closed/modified issues |
| `--robot-graph [--graph-format=json\|dot\|mermaid]` | Dependency graph export |

#### Scoping & Filtering

```bash
bv --robot-plan --label backend              # Scope to label's subgraph
bv --robot-insights --as-of HEAD~30          # Historical point-in-time
bv --recipe actionable --robot-plan          # Pre-filter: ready to work (no blockers)
bv --recipe high-impact --robot-triage       # Pre-filter: top PageRank scores
```

### br Commands for Issue Management

```bash
br ready --json                       # Show issues ready to work (no blockers)
br list --status=open --json          # All open issues
br show <id> --json                   # Full issue details with dependencies
br create --title="..." --type=task --priority=2 --json
br update <id> --status=in_progress --json
br close <id> --reason="Completed" --json
br close <id1> <id2> --reason="Completed" --json
br sync --flush-only                  # Export DB to JSONL after Beads mutations
```

### Workflow Pattern

1. **Triage**: Run `bv --robot-triage` to find the highest-impact actionable work
2. **Claim**: Use `br update <id> --status=in_progress --json`
3. **Work**: Implement the task
4. **Complete**: Use `br close <id> --reason="Completed" --json`
5. **Sync**: Run `br sync --flush-only` after Beads mutations so the JSONL export is current

### Key Concepts

- **Dependencies**: Issues can block other issues. `br ready --json` shows only unblocked work.
- **Priority**: P0=critical, P1=high, P2=medium, P3=low, P4=backlog (use numbers 0-4, not words)
- **Types**: task, bug, feature, epic, chore, docs, question
- **Blocking**: `br dep add <issue> <depends-on>` to add dependencies

### Git Policy

`br` never commits or pushes. Follow this repository's own git instructions before staging, committing, or pushing. If the repository says "commit only when asked," that rule overrides any generic workflow advice.

<!-- end-bv-agent-instructions -->
