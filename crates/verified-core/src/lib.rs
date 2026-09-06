use vstd::prelude::*;

verus! {
    /// Failure returned when an addition is outside the `i64` range.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ArithmeticError {
        Overflow,
    }

    /// Mathematical sum of the first `count` values in a sequence.
    pub open spec fn prefix_sum(values: Seq<i64>, count: int) -> int
        recommends
            0 <= count <= values.len(),
        decreases count,
    {
        if count > 0 {
            prefix_sum(values, count - 1) + values[count - 1]
        } else {
            0
        }
    }

    fn value_at(values: &[i64], index: usize) -> (value: i64)
        requires
            index < values@.len(),
        ensures
            value == values@[index as int],
    {
        values[index]
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

    /// Sum every value from left to right, reporting the first prefix overflow.
    pub fn checked_sum(values: &[i64]) -> (result: Result<i64, ArithmeticError>)
        ensures
            result is Ok
                ==> (result->Ok_0 as int) == prefix_sum(values@, values@.len() as int),
            result is Ok ==> forall|j: int|
                0 <= j <= values@.len()
                    ==> (i64::MIN as int) <= #[trigger] prefix_sum(values@, j)
                        <= (i64::MAX as int),
            result is Err ==> exists|j: int|
                0 < j <= values@.len()
                    && ((prefix_sum(values@, j) > (i64::MAX as int))
                        || (prefix_sum(values@, j) < (i64::MIN as int)))
                    && forall|k: int|
                        0 <= k < j
                            ==> (i64::MIN as int) <= #[trigger] prefix_sum(values@, k)
                                <= (i64::MAX as int),
    {
        let mut index: usize = 0;
        let mut total: i64 = 0;

        while index < values.len()
            invariant
                0 <= index <= values@.len(),
                (total as int) == prefix_sum(values@, index as int),
                forall|j: int|
                    0 <= j <= index as int
                        ==> (i64::MIN as int) <= #[trigger] prefix_sum(values@, j)
                            <= (i64::MAX as int),
            decreases values@.len() - index as int,
        {
            let value = value_at(values, index);
            match add(total, value) {
                Ok(next) => {
                    total = next;
                    index += 1;
                }
                Err(error) => {
                    proof {
                        assert((total as int) + (value as int) > (i64::MAX as int)
                            || (total as int) + (value as int) < (i64::MIN as int));
                        assert(0 < index as int + 1 <= values@.len());
                        assert(prefix_sum(values@, index as int + 1)
                            == prefix_sum(values@, index as int) + values@[index as int]) by {
                            reveal(prefix_sum);
                        }
                        assert(exists|j: int|
                            j == index as int + 1
                                && 0 < j <= values@.len()
                                && ((prefix_sum(values@, j) > (i64::MAX as int))
                                    || (prefix_sum(values@, j) < (i64::MIN as int)))
                                && forall|k: int|
                                    0 <= k < j
                                        ==> (i64::MIN as int)
                                            <= #[trigger] prefix_sum(values@, k)
                                            <= (i64::MAX as int));
                    }
                    return Err(error);
                }
            }
        }

        Ok(total)
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
    use super::{add, checked_sum, topology_identity, ArithmeticError};

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
    fn checked_sum_handles_empty_mixed_and_boundary_inputs() {
        assert_eq!(checked_sum(&[]), Ok(0));
        assert_eq!(checked_sum(&[1, -2, 3, 4]), Ok(6));
        assert_eq!(checked_sum(&[i64::MAX]), Ok(i64::MAX));
        assert_eq!(checked_sum(&[i64::MIN]), Ok(i64::MIN));
        assert_eq!(checked_sum(&[i64::MAX - 1, 1]), Ok(i64::MAX));
        assert_eq!(checked_sum(&[i64::MIN + 1, -1]), Ok(i64::MIN));
    }

    #[test]
    fn checked_sum_reports_positive_and_negative_prefix_overflow() {
        assert_eq!(checked_sum(&[i64::MAX, 1]), Err(ArithmeticError::Overflow));
        assert_eq!(checked_sum(&[i64::MIN, -1]), Err(ArithmeticError::Overflow));
    }

    #[test]
    fn checked_sum_keeps_prefix_overflow_after_later_cancellation() {
        assert_eq!(
            checked_sum(&[i64::MAX, 1, -1]),
            Err(ArithmeticError::Overflow)
        );
        assert_eq!(
            checked_sum(&[i64::MIN, -1, 1]),
            Err(ArithmeticError::Overflow)
        );
    }

    #[test]
    fn identity_preserves_boundary_values() {
        assert_eq!(topology_identity(0), 0);
        assert_eq!(topology_identity(u64::MAX), u64::MAX);
    }
}
