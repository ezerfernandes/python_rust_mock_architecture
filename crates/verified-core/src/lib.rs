use vstd::prelude::*;

verus! {
    /// Failure returned when an addition is outside the `i64` range.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ArithmeticError {
        Overflow,
    }

    /// Add two `i64` values without wrapping or panicking.
    ///
    /// This is a total API: every pair of representable inputs returns either
    /// the exact mathematical sum or `ArithmeticError::Overflow`.
    pub fn add(left: i64, right: i64) -> (result: Result<i64, ArithmeticError>)
        ensures
            result is Ok ==> (result->Ok_0 as int) == (left as int) + (right as int),
            result is Ok ==> (i64::MIN as int) <= (left as int) + (right as int)
                <= (i64::MAX as int),
            result is Err ==> result->Err_0 is Overflow,
            result is Err ==> (left as int) + (right as int) > (i64::MAX as int)
                || (left as int) + (right as int) < (i64::MIN as int),
    {
        // Check the only bound that can be crossed for each sign of `right`
        // before performing the addition. The guarded `+` operations are
        // therefore checked arithmetic without relying on wrapping behavior.
        if right > 0 {
            if left > i64::MAX - right {
                Err(ArithmeticError::Overflow)
            } else {
                Ok(left + right)
            }
        } else if right < 0 {
            if left < i64::MIN - right {
                Err(ArithmeticError::Overflow)
            } else {
                Ok(left + right)
            }
        } else {
            Ok(left)
        }
    }

    /// Minimal topology proof; algorithm contracts are added in later tasks.
    pub fn topology_identity(value: u64) -> (result: u64)
        ensures result == value,
    {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::{add, topology_identity, ArithmeticError};

    #[test]
    fn add_returns_exact_successful_sums() {
        assert_eq!(add(2, 3), Ok(5));
        assert_eq!(add(-2, 3), Ok(1));
        assert_eq!(add(0, 0), Ok(0));
        assert_eq!(add(i64::MAX, 0), Ok(i64::MAX));
        assert_eq!(add(i64::MIN, 0), Ok(i64::MIN));
        assert_eq!(add(i64::MAX - 1, 1), Ok(i64::MAX));
        assert_eq!(add(i64::MIN + 1, -1), Ok(i64::MIN));
    }

    #[test]
    fn add_reports_both_overflow_directions() {
        assert_eq!(add(i64::MAX, 1), Err(ArithmeticError::Overflow));
        assert_eq!(add(i64::MIN, -1), Err(ArithmeticError::Overflow));
    }

    #[test]
    fn identity_preserves_boundary_values() {
        assert_eq!(topology_identity(0), 0);
        assert_eq!(topology_identity(u64::MAX), u64::MAX);
    }
}
