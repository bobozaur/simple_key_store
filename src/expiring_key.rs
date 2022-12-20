use std::time::Instant;

/// Struct used for tracking the expiration of keys in [`std::collections::BinaryHeap`].
#[derive(Debug, Eq)]
pub struct ExpiringKey<K>
where
    K: Eq,
{
    pub key: K,
    pub expiry_time: Instant,
}

impl<K> ExpiringKey<K>
where
    K: Eq,
{
    pub fn new(key: K, expiry_time: Instant) -> Self {
        Self { key, expiry_time }
    }
}

/// Required to implement [`Ord`].
impl<K> PartialEq for ExpiringKey<K>
where
    K: Eq,
{
    /// It's advised that the result is consistent with [`PartialOrd`] and [`Ord`]
    fn eq(&self, other: &Self) -> bool {
        self.expiry_time == other.expiry_time
    }
}

/// Helpful for comparisons with [`Instant`].
impl<K> PartialEq<Instant> for ExpiringKey<K>
where
    K: Eq,
{
    fn eq(&self, other: &Instant) -> bool {
        &self.expiry_time == other
    }
}

/// Helpful for comparisons with [`Instant`].
impl<K> PartialEq<ExpiringKey<K>> for Instant
where
    K: Eq,
{
    fn eq(&self, other: &ExpiringKey<K>) -> bool {
        self == &other.expiry_time
    }
}

/// We fallback to the [`Ord`] impl.
impl<K> PartialOrd for ExpiringKey<K>
where
    K: Eq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Helpful for comparisons with [`Instant`].
impl<K> PartialOrd<Instant> for ExpiringKey<K>
where
    K: Eq,
{
    fn partial_cmp(&self, other: &Instant) -> Option<std::cmp::Ordering> {
        Some(self.expiry_time.cmp(other))
    }
}

/// Helpful for comparisons with [`Instant`].
impl<K> PartialOrd<ExpiringKey<K>> for Instant
where
    K: Eq,
{
    fn partial_cmp(&self, other: &ExpiringKey<K>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other.expiry_time))
    }
}

/// Only order [`ExpiringKey`] instances by their expiry time.
/// We reverse the order as the [`std::collections::BinaryHeap`] is a Max-Heap.
/// This will ensure that the most recent to expire key will be the heap's root.
impl<K> Ord for ExpiringKey<K>
where
    K: Eq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.expiry_time.cmp(&other.expiry_time).reverse()
    }
}
