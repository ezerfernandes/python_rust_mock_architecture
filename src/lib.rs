use pyo3::exceptions::{PyOverflowError, PyValueError};
use pyo3::prelude::*;
use verified_core::{
    add as verified_add, binary_search as verified_binary_search,
    checked_sum as verified_checked_sum, lower_bound as verified_lower_bound, ArithmeticError,
    SearchError, SearchResult,
};

/// Add two signed 64-bit integers without wrapping on overflow.
#[pyfunction]
fn add(a: i64, b: i64) -> PyResult<i64> {
    match verified_add(a, b) {
        Ok(result) => Ok(result),
        Err(ArithmeticError::Overflow) => {
            Err(PyOverflowError::new_err("integer addition overflow"))
        }
    }
}

/// Sum signed 64-bit integers without allowing prefix overflow.
#[pyfunction]
fn checked_sum(values: Vec<i64>) -> PyResult<i64> {
    match verified_checked_sum(&values) {
        Ok(result) => Ok(result),
        Err(ArithmeticError::Overflow) => {
            Err(PyOverflowError::new_err("integer addition overflow"))
        }
    }
}

/// Return the first value at least as large as `target`.
#[pyfunction]
fn lower_bound(values: Vec<i64>, target: i64) -> PyResult<usize> {
    match verified_lower_bound(&values, target) {
        Ok(result) => Ok(result),
        Err(SearchError::Unsorted) => Err(PyValueError::new_err("values must be sorted")),
    }
}

/// Return the first matching index, or `None` when `target` is absent.
#[pyfunction]
fn binary_search(values: Vec<i64>, target: i64) -> PyResult<Option<usize>> {
    match verified_binary_search(&values, target) {
        Ok(SearchResult::Found(index)) => Ok(Some(index)),
        Ok(SearchResult::Absent) => Ok(None),
        Err(SearchError::Unsorted) => Err(PyValueError::new_err("values must be sorted")),
    }
}

/// Private native implementation for the public Python package.
#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(checked_sum, m)?)?;
    m.add_function(wrap_pyfunction!(lower_bound, m)?)?;
    m.add_function(wrap_pyfunction!(binary_search, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{add, binary_search, checked_sum, lower_bound};

    #[test]
    fn adds_positive_values() {
        assert_eq!(add(2, 3).unwrap(), 5);
    }

    #[test]
    fn handles_zero_and_negative_values() {
        assert_eq!(add(0, 0).unwrap(), 0);
        assert_eq!(add(-8, 3).unwrap(), -5);
    }

    #[test]
    fn handles_signed_64_bit_boundaries() {
        assert_eq!(add(i64::MAX, 0).unwrap(), i64::MAX);
        assert_eq!(add(i64::MIN, 0).unwrap(), i64::MIN);
        assert_eq!(add(i64::MAX, -1).unwrap(), i64::MAX - 1);
        assert_eq!(add(i64::MIN, 1).unwrap(), i64::MIN + 1);
    }

    #[test]
    fn rejects_result_overflow_in_both_directions() {
        assert!(add(i64::MAX, 1).is_err());
        assert!(add(i64::MIN, -1).is_err());
    }

    #[test]
    fn delegates_checked_sum_and_searches_to_verified_core() {
        assert_eq!(checked_sum(vec![1, -2, 3]).unwrap(), 2);
        assert_eq!(lower_bound(vec![1, 2, 2, 4], 2).unwrap(), 1);
        assert_eq!(binary_search(vec![1, 2, 2, 4], 2).unwrap(), Some(1));
        assert_eq!(binary_search(vec![1, 2, 2, 4], 3).unwrap(), None);
    }

    #[test]
    fn maps_search_domain_errors() {
        assert!(lower_bound(vec![2, 1], 1).is_err());
        assert!(binary_search(vec![2, 1], 1).is_err());
        assert!(checked_sum(vec![i64::MAX, 1]).is_err());
    }
}
