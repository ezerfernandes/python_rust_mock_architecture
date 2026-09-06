use pyo3::exceptions::PyOverflowError;
use pyo3::prelude::*;
use verified_core::{add as verified_add, ArithmeticError};

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

/// Private native implementation for the public Python package.
#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::add;

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
}
