"""Public Python interface for the Rust-backed verified core."""

from ._native import add, binary_search, checked_sum, lower_bound

__all__ = ["add", "checked_sum", "lower_bound", "binary_search"]
