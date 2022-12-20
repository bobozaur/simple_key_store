use std::time::Instant;

use crate::expiring_key::ExpiringKey;

/// A wrapper over a keystore value, used to store in the [`std::collections::HashMap`]
/// the expiration time of the value along with the data itself.
#[derive(Debug, PartialEq, Eq)]
pub struct Value<V>
where
    V: Clone,
{
    data: V,
    expires_at: Option<Instant>,
}

impl<V> Value<V>
where
    V: Clone,
{
    pub fn new(data: V, expiry_time: Option<Instant>) -> Self {
        Self {
            data,
            expires_at: expiry_time,
        }
    }
    #[inline]
    pub fn into_inner(self) -> V {
        self.data
    }

    #[inline]
    pub fn clone_inner(&self) -> V {
        self.data.clone()
    }

    #[inline]
    pub fn expires_at(&self) -> Option<&Instant> {
        self.expires_at.as_ref()
    }
}

/// Helpful for comparisons with [`Instant`].
impl<V> PartialEq<Instant> for Value<V>
where
    V: Clone,
{
    fn eq(&self, other: &Instant) -> bool {
        self.expires_at
            .as_ref()
            .map(|exp| exp == other)
            .unwrap_or_default()
    }
}

/// Helpful for comparisons with [`Instant`].
impl<V> PartialEq<Value<V>> for Instant
where
    V: Clone,
{
    fn eq(&self, other: &Value<V>) -> bool {
        other
            .expires_at
            .as_ref()
            .map(|exp| exp == self)
            .unwrap_or_default()
    }
}

/// Helpful for comparisons with [`Instant`].
impl<K, V> PartialEq<ExpiringKey<K>> for Value<V>
where
    K: Eq,
    V: Clone,
{
    fn eq(&self, other: &ExpiringKey<K>) -> bool {
        self.expires_at
            .as_ref()
            .map(|exp| exp == &other.expiry_time)
            .unwrap_or_default()
    }
}

/// Helpful for comparisons with [`Instant`].
impl<K, V> PartialEq<Value<V>> for ExpiringKey<K>
where
    K: Eq,
    V: Clone,
{
    fn eq(&self, other: &Value<V>) -> bool {
        other
            .expires_at
            .as_ref()
            .map(|exp| exp == self)
            .unwrap_or_default()
    }
}
