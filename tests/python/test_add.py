from pathlib import Path

import pytest
import python_rust_mock_architecture as package
from python_rust_mock_architecture import (
    _native,
    add,
    binary_search,
    checked_sum,
    lower_bound,
)

I64_MIN = -(2**63)
I64_MAX = 2**63 - 1


def test_package_exports_native_functions() -> None:
    assert package.__all__ == ["add", "checked_sum", "lower_bound", "binary_search"]
    assert package.add is add
    assert Path(_native.__file__ or "").suffix in {".so", ".pyd"}
    assert {
        name
        for name in dir(_native)
        if callable(getattr(_native, name)) and not name.startswith("__")
    } == {"add", "checked_sum", "lower_bound", "binary_search"}


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


@pytest.mark.parametrize(
    ("values", "expected"),
    [([], 0), ([1, -2, 3, 4], 6), ((2, 3, -1), 4)],
)
def test_checked_sum_returns_expected_value(
    values: list[int] | tuple[int, ...], expected: int
) -> None:
    assert checked_sum(values) == expected


@pytest.mark.parametrize("values", [[I64_MAX, 1], [I64_MIN, -1], [I64_MAX, 1, -1]])
def test_checked_sum_rejects_prefix_overflow(values: list[int]) -> None:
    with pytest.raises(OverflowError):
        checked_sum(values)


def test_checked_sum_rejects_invalid_values() -> None:
    with pytest.raises(OverflowError):
        checked_sum([I64_MAX + 1])
    with pytest.raises(TypeError):
        checked_sum([1.5])  # type: ignore[list-item]


def test_new_functions_accept_keyword_arguments() -> None:
    assert checked_sum(values=[1, 2, 3]) == 6
    assert lower_bound(values=[1, 2, 3], target=2) == 1
    assert binary_search(values=[1, 2, 3], target=2) == 1


def test_new_functions_reject_missing_and_extra_arguments() -> None:
    with pytest.raises(TypeError):
        checked_sum()  # type: ignore[call-arg]
    with pytest.raises(TypeError):
        checked_sum([1], [2])  # type: ignore[call-arg]
    with pytest.raises(TypeError):
        lower_bound([1])  # type: ignore[call-arg]
    with pytest.raises(TypeError):
        lower_bound([1], 1, 2)  # type: ignore[call-arg]
    with pytest.raises(TypeError):
        binary_search([1])  # type: ignore[call-arg]
    with pytest.raises(TypeError):
        binary_search([1], 1, 2)  # type: ignore[call-arg]


def test_new_functions_reject_invalid_containers() -> None:
    with pytest.raises(TypeError):
        checked_sum(1)  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        lower_bound(1, 1)  # type: ignore[arg-type]
    with pytest.raises(TypeError):
        binary_search(1, 1)  # type: ignore[arg-type]


@pytest.mark.parametrize(
    ("values", "target", "expected"),
    [
        ([], 3, 0),
        ([1, 2, 2, 4], 2, 1),
        ([1, 2, 2, 4], 3, 3),
        ([1, 2, 2, 4], 5, 4),
        ([I64_MIN, 0, I64_MAX], I64_MAX, 2),
    ],
)
def test_lower_bound_returns_first_eligible_index(
    values: list[int], target: int, expected: int
) -> None:
    assert lower_bound(values, target) == expected


def test_lower_bound_rejects_unsorted_and_out_of_range_input() -> None:
    with pytest.raises(ValueError):
        lower_bound([2, 1], 1)
    with pytest.raises(OverflowError):
        lower_bound([1], I64_MAX + 1)
    with pytest.raises(TypeError):
        lower_bound([1.5], 1)  # type: ignore[list-item]


@pytest.mark.parametrize(
    ("values", "target", "expected"),
    [([], 3, None), ([1, 2, 2, 4], 2, 1), ([1, 2, 2, 4], 3, None)],
)
def test_binary_search_returns_first_match_or_none(
    values: list[int], target: int, expected: int | None
) -> None:
    assert binary_search(values, target) == expected


def test_binary_search_rejects_unsorted_and_invalid_input() -> None:
    with pytest.raises(ValueError):
        binary_search([2, 1], 1)
    with pytest.raises(OverflowError):
        binary_search([1], I64_MIN - 1)
    with pytest.raises(TypeError):
        binary_search(["1"], 1)  # type: ignore[list-item]
