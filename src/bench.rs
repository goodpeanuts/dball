use std::collections::HashSet;
use std::hash::Hash;

/// Method 1: `HashSet` (generic version)
pub fn has_duplicates_hashset<T>(data: &[T]) -> bool
where
    T: Eq + Hash,
{
    let mut seen = HashSet::with_capacity(data.len());
    for item in data {
        if !seen.insert(item) {
            return true;
        }
    }
    false
}

/// Method 2: Sort + adjacent comparison (requires sortable elements; this version doesn't modify original slice, uses internal copy)
pub fn has_duplicates_sort_copy<T>(data: &[T]) -> bool
where
    T: Ord + Clone,
{
    let mut v = data.to_vec();
    v.sort_unstable();
    v.windows(2).any(|w| w[0] == w[1])
}

/// In-place version of method 2 (reuses mutable buffer to avoid repeated allocation during benchmarking)
pub fn has_duplicates_sort_in_place<T>(v: &mut [T]) -> bool
where
    T: Ord,
{
    v.sort_unstable();
    v.windows(2).any(|w| w[0] == w[1])
}
