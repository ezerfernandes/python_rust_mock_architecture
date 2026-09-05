SHELL := /usr/bin/env bash
.SHELLFLAGS := -eu -o pipefail -c
.DEFAULT_GOAL := help
.NOTPARALLEL:

UV ?= uv
CARGO ?= cargo
PYTHON_VERSION ?=
SMOKE_PYTHON ?= 3.11
PYTHON_COVERAGE_MIN ?= 100
RUST_COVERAGE_MIN ?= 80
UV_PYTHON_ARGUMENT := $(if $(strip $(PYTHON_VERSION)),--python $(PYTHON_VERSION),)
VERUS ?= verus
CARGO_VERUS ?= $(CARGO) verus
CARGO_VERUS_BIN ?= cargo-verus
VERUS_VERSION ?= 0.2026.08.30.b432e82
VERUS_ARCHIVE_SHA256 ?= 067f5f72a457fe66b77c0c10b180f2a919a9c7481a8baa024ffc716aa931a41b
VERUS_RELEASE_URL ?= https://github.com/verus-lang/verus/releases/download/release%2F$(VERUS_VERSION)/verus-$(VERUS_VERSION)-x86-linux.zip
VERUS_INSTALL_DOCS ?= docs/verus-toolchain.md
VERUS_PROOF_DIR ?= crates/verified-core
VERUS_HYGIENE_PATTERN ?= (^|[^[:alnum:]_])(assume|admit|axiom)[[:space:]]*[({:]|external_body|assume_specification

.PHONY: help require-uv require-cargo require-verus setup-python setup-rust \
	setup develop test-python test-rust test coverage-python coverage-rust \
	lint-python lint-rust lint typecheck verus-hygiene verus-focus verus-verify \
	build package-check check

help: ## List the supported development targets.
	@awk 'BEGIN {FS = ":.*## "; printf "Usage: make <target>\n\n"} /^[a-zA-Z0-9_-]+:.*## / {printf "  %-18s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

require-uv:
	@command -v "$(UV)" >/dev/null || { echo "uv is required: https://docs.astral.sh/uv/getting-started/installation/" >&2; exit 127; }

require-cargo:
	@command -v "$(CARGO)" >/dev/null || { echo "Cargo is required: https://rustup.rs/" >&2; exit 127; }

require-verus: require-cargo
	@command -v "$(VERUS)" >/dev/null || { \
		echo "Verus $(VERUS_VERSION) is required." >&2; \
		echo "Install command: curl -fL --output verus.zip '$(VERUS_RELEASE_URL)' && printf '%s  %s\\n' '$(VERUS_ARCHIVE_SHA256)' verus.zip | sha256sum --check && unzip -q verus.zip" >&2; \
		echo "See $(VERUS_INSTALL_DOCS) for the complete checksum-verified installation." >&2; \
		exit 127; \
	}
	@command -v "$(CARGO_VERUS_BIN)" >/dev/null || { \
		echo "cargo-verus for Verus $(VERUS_VERSION) is required." >&2; \
		echo "Install the bundled cargo-verus from the pinned release described in $(VERUS_INSTALL_DOCS)." >&2; \
		exit 127; \
	}

setup-python: require-uv
	$(UV) sync --locked $(UV_PYTHON_ARGUMENT)

setup-rust: require-cargo
	$(CARGO) fetch --locked

setup: setup-python setup-rust ## Synchronize locked Python and Rust dependencies.

develop: setup ## Build and install the native extension for local development.
	$(UV) run --locked maturin develop --locked

test-python: develop ## Run the Python binding tests.
	$(UV) run --locked pytest

test-rust: setup-rust ## Run the Rust unit tests.
	$(CARGO) test --workspace --locked

test: test-python test-rust ## Run Python and Rust tests.

coverage-python: develop ## Run Python coverage and require 100 percent.
	$(UV) run --locked pytest --cov=python_rust_mock_architecture --cov-report=term-missing --cov-fail-under=$(PYTHON_COVERAGE_MIN) tests/python

coverage-rust: setup-rust ## Run Rust coverage and require 80 percent.
	@command -v cargo-llvm-cov >/dev/null || { echo "cargo-llvm-cov is required: cargo +stable install cargo-llvm-cov --locked" >&2; exit 127; }
	$(CARGO) llvm-cov --workspace --locked --fail-under-lines $(RUST_COVERAGE_MIN)

lint-python: setup-python ## Check Python formatting and lint rules.
	$(UV) run --locked ruff format --check .
	$(UV) run --locked ruff check .
	$(UV) lock --check

lint-rust: setup-rust ## Check Rust formatting and Clippy warnings.
	$(CARGO) fmt --check
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

lint: lint-python lint-rust ## Run Python and Rust lint checks.

typecheck: develop ## Run strict Python type checking against the native stub.
	$(UV) run --locked mypy python tests/python

verus-hygiene: ## Reject unsafe proof escapes in project-authored core sources.
	@command -v rg >/dev/null || { echo "ripgrep is required for proof hygiene: install ripgrep" >&2; exit 127; }
	@scan_status=0; \
	rg -n --no-heading --glob '*.rs' --glob '!target/**' "$(VERUS_HYGIENE_PATTERN)" "$(VERUS_PROOF_DIR)" || scan_status=$$?; \
	if (( scan_status == 0 )); then \
		echo "project-authored proof escape detected under $(VERUS_PROOF_DIR)" >&2; \
		exit 1; \
	elif (( scan_status != 1 )); then \
		echo "proof hygiene scan failed under $(VERUS_PROOF_DIR)" >&2; \
		exit "$$scan_status"; \
	fi

verus-focus: require-verus verus-hygiene ## Verify the core without rechecking dependencies.
	$(CARGO_VERUS) focus --package verified-core --locked

verus-verify: require-verus verus-hygiene ## Verify every workspace member with the pinned Verus toolchain.
	$(CARGO_VERUS) verify --workspace --locked -- -V check-api-safety

build: setup ## Build a wheel and source distribution in dist.
	mkdir -p dist
	$(UV) run --locked maturin build --locked --sdist --out dist

package-check: setup ## Verify clean packages and smoke-test the installed wheel.
	@artifact_dir="$$(mktemp -d)"; \
	smoke_env="$$(mktemp -d)"; \
	smoke_work="$$(mktemp -d)"; \
	cleanup() { rm -rf -- "$$artifact_dir" "$$smoke_env" "$$smoke_work"; }; \
	trap cleanup EXIT; \
	read -r project_name project_version < <($(UV) run --locked python -c 'import tomllib; from pathlib import Path; project = tomllib.loads(Path("pyproject.toml").read_text())["project"]; print(project["name"], project["version"])'); \
	wheel_stem="$${project_name//-/_}-$${project_version}"; \
	$(UV) run --locked maturin build --locked --sdist --out "$$artifact_dir"; \
	$(UV) run --locked maturin build --locked --sdist --out "$$artifact_dir"; \
	mapfile -t wheels < <(find "$$artifact_dir" -maxdepth 1 -type f -name "$${wheel_stem}-*.whl" -print); \
	mapfile -t sdists < <(find "$$artifact_dir" -maxdepth 1 -type f -name "$${wheel_stem}.tar.gz" -print); \
	artifact_count="$$(find "$$artifact_dir" -maxdepth 1 -type f | wc -l)"; \
	if (( $${#wheels[@]} != 1 || $${#sdists[@]} != 1 || artifact_count != 2 )); then \
		echo "expected exactly one $${wheel_stem}-*.whl and one $${wheel_stem}.tar.gz" >&2; \
		find "$$artifact_dir" -maxdepth 1 -type f -print >&2; \
		exit 1; \
	fi; \
	package_list="$$($(CARGO) package --list --allow-dirty --locked)"; \
	if grep -Eq '(^|/)(dist|target|\.venv|\.mypy_cache|\.pytest_cache|\.ruff_cache|__pycache__)(/|$$)|\.(whl|tar\.gz|so|pyd|dylib|dll|pyc|pyo)$$' <<< "$$package_list"; then \
		echo "generated artifacts found in Cargo package" >&2; \
		grep -En '(^|/)(dist|target|\.venv|\.mypy_cache|\.pytest_cache|\.ruff_cache|__pycache__)(/|$$)|\.(whl|tar\.gz|so|pyd|dylib|dll|pyc|pyo)$$' <<< "$$package_list" >&2; \
		exit 1; \
	fi; \
	if tar -tzf "$${sdists[0]}" | grep -Eq '(^|/)(dist|target|\.venv|\.mypy_cache|\.pytest_cache|\.ruff_cache|__pycache__)(/|$$)|\.(whl|tar\.gz|so|pyd|dylib|dll|pyc|pyo)$$'; then \
		echo "generated artifacts found in source distribution" >&2; \
		exit 1; \
	fi; \
	$(UV) venv --python "$(SMOKE_PYTHON)" "$$smoke_env"; \
	$(UV) pip install --python "$$smoke_env/bin/python" "$${wheels[0]}"; \
	cd "$$smoke_work"; \
	"$$smoke_env/bin/python" -c 'from pathlib import Path; import python_rust_mock_architecture as package; import python_rust_mock_architecture._native as native; assert package.__all__ == ["add"]; assert package.add(40, 2) == 42; assert Path(native.__file__ or "").suffix in {".so", ".pyd"}'

check: lint verus-verify typecheck test coverage-python coverage-rust package-check ## Run the complete local verification sequence.
