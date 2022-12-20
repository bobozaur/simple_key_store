use std::{borrow::Borrow, hash::Hash, sync::Arc, time::Duration};

use crate::drop_guard::DropGuard;

/// A shareable key store that wraps the inner [`DropGuard`] in an [`Arc`]
/// to make it shareable across threads.
///
/// The keys must be thread safe, cloneable and respect [`std::collections::HashMap`] trait requirements.
/// The values must be thread safe and cloneable.
///
/// The thread safe aspect of the key store along with it having a background task for
/// cleaning up expired `key:value` pairs enforces the requirements above.
///
/// It might be worth wrapping memory expensive keys or values in an [`Arc`].
#[derive(Debug, Clone)]
pub struct KeyStore<K, V>(Arc<DropGuard<K, V>>)
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Sync + Send + 'static;

impl<K, V> KeyStore<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,

    V: Clone + Sync + Send + 'static,
{
    pub fn new() -> Self {
        Self(Arc::new(DropGuard::new()))
    }

    /// Returns a. owned value from storage.
    pub fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.0.read().get(key)
    }

    /// Inserts a value into storage which expires after a [`Duration`], if given.
    /// Returns the previous value associated with the key, if any.
    pub fn insert(&self, key: K, value: V, expire_after: Option<Duration>) -> Option<V> {
        let (prev, notify) = self.0.write().insert(key, value, expire_after);

        // Notify background task if needed.
        // This will happen if a value with an expiration
        // time lower than anything in the heap was inserted.
        if notify {
            self.0.notify();
        }

        // Return previous value.
        prev
    }

    /// Removes a value from storage.
    pub fn remove<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.0.write().remove(key)
    }
}

impl<K, V> Default for KeyStore<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,

    V: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
