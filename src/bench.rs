use std::collections::HashSet;
use std::hash::Hash;

/// 方法 1：HashSet（通用泛型版本）
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

/// 方法 2：排序 + 相邻比较（需元素可排序；此版本不修改原 slice，内部拷贝）
pub fn has_duplicates_sort_copy<T>(data: &[T]) -> bool
where
    T: Ord + Clone,
{
    let mut v = data.to_vec();
    v.sort_unstable();
    v.windows(2).any(|w| w[0] == w[1])
}

/// 方法 2 的 in-place 版本（重用可变缓冲区，供基准时避免重复分配）
pub fn has_duplicates_sort_in_place<T>(v: &mut [T]) -> bool
where
    T: Ord,
{
    v.sort_unstable();
    v.windows(2).any(|w| w[0] == w[1])
}
