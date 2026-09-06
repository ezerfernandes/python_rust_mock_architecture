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

    /// Failure returned when a search input is not nondecreasing.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SearchError {
        Unsorted,
    }

    /// Search result without relying on unchecked `Option` specifications.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SearchResult {
        Found(usize),
        Absent,
    }

    /// True when every adjacent pair in `values` is nondecreasing.
    pub open spec fn nondecreasing(values: Seq<i64>) -> bool {
        forall|index: int|
            0 <= index < values.len() - 1 ==> #[trigger] values[index] <= values[index + 1]
    }

    proof fn nondecreasing_pair(values: Seq<i64>, left: int, right: int)
        requires
            nondecreasing(values),
            0 <= left <= right < values.len(),
        ensures
            values[left] <= values[right],
        decreases right - left,
    {
        if left < right {
            nondecreasing_pair(values, left, right - 1);
            assert(values[right - 1] <= values[right]);
        }
    }

    fn scan_nondecreasing(values: &[i64]) -> (result: bool)
        ensures
            result ==> nondecreasing(values@),
    {
        if values.is_empty() {
            return true;
        }
        if values.len() == 1 {
            return true;
        }
        proof {
            assert(2 <= values@.len());
        }
        let mut index: usize = 0;
        while index < values.len().saturating_sub(1)
            invariant
                0 <= index <= values@.len(),
                forall|j: int|
                    0 <= j < index as int ==> #[trigger] values@[j] <= values@[j + 1],
            decreases values@.len() - index as int,
        {
            let previous = value_at(values, index);
            let current = value_at(values, index + 1);
            if previous > current {
                return false;
            }
            proof {
                assert forall|j: int|
                    0 <= j < index as int + 1
                        implies #[trigger] values@[j] <= values@[j + 1]
                by {
                    if j < index as int {
                    } else {
                        assert(j == index as int);
                        assert(values@[j] == previous);
                        assert(values@[j + 1] == current);
                    }
                };
            }
            index += 1;
        }
        true
    }

    /// Return the first index whose value is at least `target`.
    pub fn lower_bound(values: &[i64], target: i64) -> (result: Result<usize, SearchError>)
        ensures
            result is Err ==> result->Err_0 is Unsorted,
            result is Ok ==> nondecreasing(values@),
            result is Ok ==> 0 <= result->Ok_0 as int <= values@.len(),
            result is Ok ==> forall|index: int|
                0 <= index < result->Ok_0 as int ==> values@[index] < target,
            result is Ok ==> forall|index: int|
                result->Ok_0 as int <= index < values@.len() ==> target <= values@[index],
    {
        if !scan_nondecreasing(values) {
            return Err(SearchError::Unsorted);
        }

        let mut low: usize = 0;
        let mut high: usize = values.len();
        while low < high
            invariant
                0 <= low <= high <= values@.len(),
                nondecreasing(values@),
                forall|index: int|
                    0 <= index < low as int ==> values@[index] < target,
                forall|index: int|
                    high as int <= index < values@.len() ==> target <= values@[index],
            decreases high as int - low as int,
        {
            let midpoint = low + (high - low) / 2;
            let middle = value_at(values, midpoint);
            proof {
                assert(low <= midpoint < high);
            }
            if middle < target {
                proof {
                    assert forall|index: int|
                        0 <= index < midpoint as int + 1
                            implies #[trigger] values@[index] < target
                    by {
                        if index < low as int {
                        } else {
                            assert(index <= midpoint as int);
                            nondecreasing_pair(values@, index, midpoint as int);
                            assert(values@[midpoint as int] == middle);
                        }
                    };
                }
                low = midpoint + 1;
            } else {
                proof {
                    assert forall|index: int|
                        midpoint as int <= index < values@.len()
                            implies target <= #[trigger] values@[index]
                    by {
                        if index < high as int {
                            if index == midpoint as int {
                                assert(values@[index] == middle);
                            } else {
                                nondecreasing_pair(values@, midpoint as int, index);
                                assert(values@[midpoint as int] == middle);
                            }
                        }
                    };
                }
                high = midpoint;
            }
        }

        Ok(low)
    }

    /// Return the first matching index, or an absent result when `target` is missing.
    pub fn binary_search(
        values: &[i64],
        target: i64,
    ) -> (result: Result<SearchResult, SearchError>)
        ensures
            result is Err ==> result->Err_0 is Unsorted,
            result is Ok ==> nondecreasing(values@),
            result is Ok ==> result->Ok_0 is Found
                ==> exists|found: usize|
                    result->Ok_0 == SearchResult::Found(found)
                        && 0 <= (found as int) < values@.len()
                        && #[trigger] values@[found as int] == target
                        && forall|index: int|
                            0 <= index < found as int ==> values@[index] < target,
            result is Ok ==> result->Ok_0 is Absent ==> forall|index: int|
                0 <= index < values@.len() ==> values@[index] != target,
    {
        match lower_bound(values, target) {
            Err(error) => Err(error),
            Ok(index) => {
                if index < values.len() {
                    let value = value_at(values, index);
                    if value == target {
                        Ok(SearchResult::Found(index))
                    } else {
                        proof {
                            assert(value > target);
                            assert forall|candidate: int|
                                0 <= candidate < values@.len()
                                    implies #[trigger] values@[candidate] != target
                            by {
                                if candidate < index as int {
                                    assert(values@[candidate] < target);
                                } else {
                                    nondecreasing_pair(values@, index as int, candidate);
                                    assert(values@[index as int] == value);
                                    assert(values@[candidate] > target);
                                }
                            };
                        }
                        Ok(SearchResult::Absent)
                    }
                } else {
                    proof {
                        assert forall|candidate: int|
                            0 <= candidate < values@.len()
                                implies #[trigger] values@[candidate] != target
                        by {
                            assert(values@[candidate] < target);
                        };
                    }
                    Ok(SearchResult::Absent)
                }
            }
        }
    }

    /// Identity helper retained for workspace topology tests.
    pub fn topology_identity(value: u64) -> (result: u64)
        ensures result == value,
    {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::{
        add, binary_search, checked_sum, lower_bound, topology_identity, ArithmeticError,
        SearchError, SearchResult,
    };

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
    fn lower_bound_returns_first_eligible_index() {
        assert_eq!(lower_bound(&[], 7), Ok(0));
        assert_eq!(lower_bound(&[1], 0), Ok(0));
        assert_eq!(lower_bound(&[1], 1), Ok(0));
        assert_eq!(lower_bound(&[1], 2), Ok(1));
        assert_eq!(lower_bound(&[1, 2, 2, 4], 2), Ok(1));
        assert_eq!(lower_bound(&[i64::MIN, 0, i64::MAX], i64::MAX), Ok(2));
        assert_eq!(lower_bound(&[i64::MIN, 0, i64::MAX], 1), Ok(2));
    }

    #[test]
    fn lower_bound_rejects_unsorted_input() {
        assert_eq!(lower_bound(&[1, 3, 2], 2), Err(SearchError::Unsorted));
    }

    #[test]
    fn binary_search_returns_first_match_and_none_for_absent_values() {
        assert_eq!(binary_search(&[], 7), Ok(SearchResult::Absent));
        assert_eq!(binary_search(&[1, 2, 2, 4], 2), Ok(SearchResult::Found(1)));
        assert_eq!(binary_search(&[1, 2, 2, 4], 3), Ok(SearchResult::Absent));
        assert_eq!(
            binary_search(&[i64::MIN, 0, i64::MAX], i64::MIN),
            Ok(SearchResult::Found(0))
        );
        assert_eq!(
            binary_search(&[i64::MIN, 0, i64::MAX], i64::MAX),
            Ok(SearchResult::Found(2))
        );
    }

    #[test]
    fn binary_search_rejects_unsorted_input() {
        assert_eq!(binary_search(&[1, 3, 2], 2), Err(SearchError::Unsorted));
    }

    #[test]
    fn identity_preserves_boundary_values() {
        assert_eq!(topology_identity(0), 0);
        assert_eq!(topology_identity(u64::MAX), u64::MAX);
    }
}
