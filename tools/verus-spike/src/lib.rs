use vstd::prelude::*;

verus! {
    pub fn checked_add_one(value: u64) -> (result: u64)
        requires value < u64::MAX,
        ensures result == value + 1,
    {
        value + 1
    }
}
