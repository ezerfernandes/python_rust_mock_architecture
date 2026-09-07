# Verus-verified Rust core for Python

Date: 2026-09-04
Status: Approved design

Implementation status as of 2026-09-07: the private core contains and locally
verifies total checked `i64` addition, checked left-to-right summation, sorted
lower-bound search, and first-match binary search. The public Python adapter
exposes all four functions and remains outside the proved boundary. CI now
installs the pinned Verus release, runs the full proof target, and gates package
checks on that job. The final proof, package, and trust-boundary audit is
recorded in [`docs/verus-audit.md`](../../verus-audit.md); GitHub Actions run
`34137761607` passed for the pinned proof and package gates.

## Goal

Extend this reference repository so it teaches how to put a Verus-verified Rust
core behind a Python API. The first proof covers the existing checked `add`
operation. Three new public Python functions provide progressively richer
examples:

```python
checked_sum(values: Sequence[int]) -> int
lower_bound(values: Sequence[int], target: int) -> int
binary_search(values: Sequence[int], target: int) -> int | None
```

This work is about functional correctness, not performance. It must preserve uv,
Maturin, Python 3.11 or newer, the existing distribution and import names, the
private native-module convention, and the current build backend.

## Architecture and trust boundary

Convert the Rust portion into a small Cargo workspace without moving the root
Maturin package. The root crate remains the private PyO3 extension. A new private
workspace crate contains Rust code that Verus can verify and normal Cargo can
compile. The PyO3 crate calls that core and maps its results to documented Python
values and exceptions.

The core owns algorithms and their executable Verus specifications. It must not
depend on PyO3 or Python concepts. The adapter owns Python extraction, return-value
conversion, and exception construction. This isolates unsupported external code
from the proof and keeps the native Python API private.

Verification claims apply only to the opted-in core crate and only under its
recorded assumptions. Python, PyO3, the adapter, Verus, Z3, rustc, LLVM, external
specifications, and any code marked as external remain outside the proved core.
Runtime and package tests cover the Python-facing boundary. Documentation must
state this distinction whenever it calls code "verified."

No public core function called from unverified code may rely on an unchecked
`requires` clause. It must accept every input its safe Rust types can represent
or perform an executable check and return a fallible result. This prevents Python
or PyO3 from silently violating a Verus precondition.

## Function contracts

### `add`

Keep the existing Python signature and behavior. The verified core returns the
mathematical sum when it fits in `i64` and reports overflow in either direction
otherwise. PyO3 continues to map result overflow and out-of-range Python integers
to `OverflowError`, and invalid Python argument types to `TypeError`.

### `checked_sum`

Accept a Python sequence whose elements convert to `i64`. Fold from left to right
with checked `i64` addition. Return the exact accumulated sum if every prefix sum
fits in `i64`. Raise `OverflowError` if an element is outside the `i64` range or
if any prefix sum overflows, even when later values could bring the mathematical
total back into range. Invalid containers or element types raise `TypeError`.

The proof relates the executable accumulator to a mathematical sequence-prefix
sum. It proves the successful result and the absence of unchecked arithmetic.

### `lower_bound`

Accept a nondecreasing sequence of `i64` values and an `i64` target. Return the
first index whose value is greater than or equal to the target, or `len(values)`
if no such value exists. Duplicate values therefore return the first eligible
index.

The safe core API checks sortedness at runtime. Unsorted input maps to Python
`ValueError`. Out-of-range values map to `OverflowError`; wrong Python types
map to `TypeError`. The proof establishes the returned index bounds and the two
partitions on either side of the index.

### `binary_search`

Use the same input and error contract as `lower_bound`. Return the first matching
index or `None` when the target is absent. Implement it in terms of the verified
lower-bound result so the proof establishes first-match behavior rather than an
unspecified matching index.

## Verus agent guide

Extend `AGENTS.md` with an original, task-focused account of the official Verus
tutorial. It must cover:

- the `vstd` prelude, `verus!` macro, Cargo opt-in, and normal Cargo builds;
- executable, specification, and proof modes plus ghost erasure;
- `requires`, named returns, `ensures`, `recommends`, and modular contracts;
- `int`, `nat`, fixed-width integers, casts, and overflow obligations;
- proof functions, proof blocks, static `assert`, and specialized prover modes;
- recursive specifications, `decreases`, fuel, loops, and invariants;
- sequence views with `@`, mutation with `old` and `final`, and extensional
  equality;
- `forall`, `exists`, `choose`, implication, triggers, and trigger debugging;
- external code, safe APIs for unverified callers, assumptions, and the trusted
  computing base;
- proof-failure diagnosis, verification performance, and completion checks.

The guide will link to the official tutorial and use short, original examples. It
will not reproduce the tutorial wholesale or imply that Verus supports every Rust
feature.

## Commands, tests, and CI

Preserve all current Make targets. Add a focused local verification target and
include it in `check`. Missing Verus tooling must produce a direct installation
message. Pin a compatible Verus release and `vstd` dependency together. Keep the
normal Cargo test, lint, coverage, and Maturin build paths working.

CI installs the pinned Verus toolchain and runs the same Make target used locally.
The full proof command must run before merge. A focus or incremental command may
exist for development but cannot replace the full proof gate.

Required evidence includes:

- Verus reports zero verification errors for the whole core crate.
- Project-authored proof code contains no `assume`, admitted axiom, or
  undocumented external specification.
- Rust tests cover success, boundaries, overflow, empty input, duplicates,
  unsorted input, and absent targets.
- Python tests cover exports, types, exact exceptions, boundaries, duplicates,
  empty sequences, unsorted input, and native loading.
- Stubs, `__all__`, README documentation, coverage, linting, typing, packaging,
  and isolated-wheel smoke tests reflect all four functions.
- The final audit records every trusted component and every unverified boundary.

## Beads work graph

Create one feature epic with small dependency-linked tasks:

1. Pin and validate a compatible Verus, Rust, Cargo, and `vstd` toolchain.
2. Add the Cargo workspace and private verified-core crate.
3. Specify and verify checked `add`.
4. Specify and verify `checked_sum`.
5. Verify sortedness checking and `lower_bound`.
6. Specify and verify first-match `binary_search`.
7. Add PyO3 adapters and exact Python exception mapping.
8. Update public Python exports, stubs, and contract tests.
9. Add Makefile verification commands and proof hygiene checks.
10. Add the pinned Verus CI gate.
11. Update README and package smoke checks.
12. Run the final proof, test, package, and trust-boundary audit.

Each task will contain scope, implementation notes, acceptance criteria, tests,
and dependencies. Algorithm tasks can proceed independently after the verified
core exists. Binding work depends on all core contracts. CI and documentation
depend on a stable local command. The final audit depends on every implementation
and integration task.

## Non-goals

- No benchmarks or speed claims.
- No package-manager, build-backend, Python-version, distribution-name, or
  import-package migration.
- No Python fallback.
- No attempt to verify PyO3, CPython, Verus, Z3, rustc, or LLVM.
- No concurrency, unsafe-code verification, or general algorithm library.
- No unchecked preconditions exposed to Python.

## Completion

The goal is complete only when the proof gate, ordinary Rust checks, Python
contract tests, coverage, typing, linting, package checks, and isolated-wheel
smoke test all pass from documented commands. Verification claims must match the
recorded trust boundary.
