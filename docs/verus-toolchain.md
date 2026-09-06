# Pinned Verus toolchain

This repository validates Verus with a fixed release rather than a moving
branch or a `latest` download. The pin is recorded in
[`tools/verus-toolchain.toml`](../tools/verus-toolchain.toml), and the repository
[`rust-toolchain.toml`](../rust-toolchain.toml) selects the Rust version required
by that release.

## Pin

The validated release is `0.2026.08.30.b432e82`, published from commit
`b432e82fed7e05090fd53b5e5fc39020f725aabe`. Its x86_64 Linux archive has this
SHA-256 digest:

```text
067f5f72a457fe66b77c0c10b180f2a919a9c7481a8baa024ffc716aa931a41b
```

The archive supplies `verus`, `cargo-verus`, and Z3. The paired versions used by
the proof spike are:

| Component | Pin |
| --- | --- |
| Verus | `0.2026.08.30.b432e82` |
| Rust toolchain | `1.97.1-x86_64-unknown-linux-gnu` |
| `cargo-verus` | bundled in the Verus release archive |
| Z3 | `4.16.0`, bundled in the Verus release archive |
| `vstd` | exact crates.io version `0.0.0-2026-08-30-0159` |
| `vstd` source revision | `b432e82fed7e05090fd53b5e5fc39020f725aabe` |

The `vstd` version and its transitive dependency versions are locked in
[`tools/verus-spike/Cargo.lock`](../tools/verus-spike/Cargo.lock). The source
revision records the Verus release that supplied the matching standard library.

## Installation on Linux

The commands below install the pinned Ubuntu 24.04 x86_64 release in a versioned
user directory. They verify the downloaded archive before extraction.

```bash
verus_version="0.2026.08.30.b432e82"
verus_archive="verus-${verus_version}-x86-linux.zip"
verus_root="${XDG_DATA_HOME:-$HOME/.local/share}/verus"
verus_download="https://github.com/verus-lang/verus/releases/download/release%2F${verus_version}/${verus_archive}"
verus_sha256="067f5f72a457fe66b77c0c10b180f2a919a9c7481a8baa024ffc716aa931a41b"
download_dir="$(mktemp -d)"
curl -fL --output "$download_dir/$verus_archive" "$verus_download"
printf '%s  %s\n' "$verus_sha256" "$download_dir/$verus_archive" | sha256sum --check
rustup toolchain install 1.97.1-x86_64-unknown-linux-gnu --profile minimal
mkdir -p "$verus_root"
unzip -q "$download_dir/$verus_archive" -d "$download_dir"
mv "$download_dir/verus-x86-linux" "$verus_root/$verus_version"
export PATH="$verus_root/$verus_version:$HOME/.cargo/bin:$PATH"
```

Check the installed tools:

```text
verus --version
cargo verus --help
rustc --version
cargo --version
z3 --version
```

`rust-toolchain.toml` makes normal Cargo commands use Rust 1.97.1 from this
checkout. Keep the Verus release directory and the rustup shims on `PATH`, for
example:

```bash
export PATH="$verus_root/$verus_version:$HOME/.cargo/bin:$PATH"
```

With this pin, `rustc --version` starts with `rustc 1.97.1`, `cargo --version`
starts with `cargo 1.97.1`, and `verus --version` reports
`0.2026.08.30.b432e82`. `cargo-verus` uses the same pinned toolchain when it
invokes Cargo.

## Proof and normal Cargo build

[`tools/verus-spike`](../tools/verus-spike) is a small standalone library. It is
opted into verification, imports the `vstd` prelude, and proves that a checked
`u64` increment returns the mathematical successor when the precondition rules
out overflow. The fixture has no PyO3 or Python dependency.

Run the full proof and then build the same source with normal Cargo:

```text
export PATH="${XDG_DATA_HOME:-$HOME/.local/share}/verus/0.2026.08.30.b432e82:$PATH"
cargo verus verify --manifest-path tools/verus-spike/Cargo.toml --locked
cargo build --manifest-path tools/verus-spike/Cargo.toml --locked
```

The root `Cargo.toml` excludes this fixture from its workspace, so the commands
above use the spike's own lockfile and remain valid after the workspace is
loaded. The successful proof reports zero errors for the spike. Verification
also checks the pinned `vstd` dependency. Normal Cargo erases the Verus
annotations and builds the library as an ordinary Rust crate.

## Verify the repository core

After installing the pin, run the repository targets from its root:

```text
make verus-hygiene
make verus-verify
```

`verus-verify` runs the full workspace proof with
`--fwd-verus-args roots -- -V check-api-safety`, so the API-safety argument is
scoped to the selected workspace roots while dependencies are still verified.
`make verus-focus` is a faster development shortcut that verifies only
`verified-core` without rechecking dependencies. The current private core proof
covers total checked `i64` addition, checked left-to-right summation, sorted
lower-bound search, and first-match binary search. The root PyO3 adapter and
Python package remain outside the proved core.

The complete local gate is:

```text
make check
```

It runs the proof gate before typing, tests, coverage, and package checks. Rust
coverage also requires `cargo-llvm-cov`; install it with:

```text
cargo +stable install cargo-llvm-cov --locked
```

The CI workflow installs that coverage tool, but it does not yet install Verus
or run the repository proof target.

## Platform scope

The validated platform for this repository is Ubuntu 24.04 x86_64, which is the
Linux CI target. The official release also publishes prebuilt archives for
macOS 14 arm64, macOS 15 x86_64, and Windows 2022 x86_64, but this repository
does not test those hosts yet. Other operating systems and architectures may
require a source build and are outside this pin's validation evidence.

The proof spike is an isolated toolchain check. It does not verify PyO3, Python,
CPython, Maturin, Z3 itself, rustc, LLVM, or the Rust adapter that calls the
current verified core. Those remain outside the proved boundary.

## Official references

- [Verus releases](https://github.com/verus-lang/verus/releases)
- [Verus installation guide](https://github.com/verus-lang/verus/blob/main/INSTALL.md)
- [Using Verus via Cargo](https://verus-lang.github.io/verus/guide/cargo_verus.html)
