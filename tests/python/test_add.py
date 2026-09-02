from pathlib import Path

import pytest
import python_rust_mock_architecture as package
from python_rust_mock_architecture import _native, add

I64_MIN = -(2**63)
I64_MAX = 2**63 - 1


def test_package_exports_native_add() -> None:
    assert package.__all__ == ["add"]
    assert package.add is add
    assert Path(_native.__file__ or "").suffix in {".so", ".pyd"}
    assert {
        name
        for name in dir(_native)
        if callable(getattr(_native, name)) and not name.startswith("__")
    } == {"add"}


@pytest.mark.parametrize(
    ("a", "b", "expected"),
    [
        (2, 3, 5),
        (-8, 3, -5),
        (0, 0, 0),
        (True, False, 1),
        (False, True, 1),
        (I64_MAX, -1, I64_MAX - 1),
        (I64_MIN, 1, I64_MIN + 1),
    ],
)
def test_add_returns_expected_value(a: int, b: int, expected: int) -> None:
    assert add(a, b) == expected


def test_add_accepts_keyword_arguments() -> None:
    assert add(a=12, b=-7) == 5


@pytest.mark.parametrize(
    ("a", "b"),
    [(I64_MAX, 1), (I64_MIN, -1)],
)
def test_add_rejects_result_overflow(a: int, b: int) -> None:
    with pytest.raises(OverflowError):
        add(a, b)


@pytest.mark.parametrize("value", [I64_MAX + 1, I64_MIN - 1])
def test_add_rejects_out_of_range_inputs(value: int) -> None:
    with pytest.raises(OverflowError):
        add(value, 0)


@pytest.mark.parametrize("value", [1.5, "1"])
def test_add_rejects_non_integral_inputs(value: object) -> None:
    with pytest.raises(TypeError):
        add(value, 1)  # type: ignore[arg-type]


def test_add_rejects_missing_and_extra_arguments() -> None:
    with pytest.raises(TypeError):
        add(1)  # type: ignore[call-arg]
    with pytest.raises(TypeError):
        add(1, 2, 3)  # type: ignore[call-arg]
