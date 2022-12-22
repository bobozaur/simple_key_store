use std::{
    borrow::Borrow,
    collections::{BinaryHeap, HashMap},
    hash::Hash,
    sync::{
        atomic::{AtomicBool, Ordering},
        RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
    time::{Duration, Instant},
};

use crate::{expiring_key::ExpiringKey, value::Value};

/// A shared state struct.
/// This will essentially be shared between the actual key store
/// and the cleanup task that removes expired keys.
#[derive(Debug)]
pub struct Shared<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub storage: RwLock<Storage<K, V>>,
    pub run_cleanup: AtomicBool,
}

impl<K, V> Shared<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            storage: RwLock::new(Storage::new()),
            run_cleanup: AtomicBool::new(true),
        }
    }

    /// Convenience method to gain a read lock on the [`Storage`].
    #[inline]
    pub fn read(&self) -> RwLockReadGuard<Storage<K, V>> {
        self.storage.read().unwrap()
    }

    /// Convenience method to gain an exclusive write lock on the [`Storage`].
    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<Storage<K, V>> {
        self.storage.write().unwrap()
    }

    /// Method used to signal the background task to stop.
    #[inline]
    pub fn stop_cleanup(&self) {
        self.run_cleanup.store(false, Ordering::Release);
    }
}

/// The storage of the `key:value` pairs.
/// Keys and their values get stored in a [`HashMap`]
/// and the expiring keys are additionally stored in a Min-[`BinaryHeap`].
#[derive(Debug)]
pub struct Storage<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub kv_pairs: HashMap<K, Value<V>>,
    pub expirations: BinaryHeap<ExpiringKey<K>>,
}

impl<K, V> Storage<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            kv_pairs: HashMap::new(),
            expirations: BinaryHeap::new(),
        }
    }

    /// Returns an owned value from storage.
    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.kv_pairs.get(key).map(Value::clone_inner)
    }

    /// Inserts a value into storage and sets its expiration, if necessary.
    pub fn insert(
        &mut self,
        key: K,
        value: V,
        expire_after: Option<Duration>,
    ) -> (Option<V>, bool) {
        let (when, notify) = if let Some(dur) = expire_after {
            // An expiration time was set on the value.
            // Calculate the expiry time.
            let when = Instant::now() + dur;

            // If the expiration of the newly inserted key
            // would happen sooner than that of the min value from the heap
            // then keep in mind to notify the background task so
            // it picks up the newer and lower expiration time.
            let notify = self
                .expirations
                .peek()
                .map(|exp_key| &when < exp_key)
                .unwrap_or(true);

            // Add the new expiring key to the heap.
            let exp_key = ExpiringKey::new(key.clone(), when);
            self.expirations.push(exp_key);

            // Return the expiration time and the notify flag.
            (Some(when), notify)
        } else {
            // No expiration set, hence no need to notify the task either.
            (None, false)
        };

        // Insert the key: value pair into the HashMap.
        let prev = self
            .kv_pairs
            .insert(key, Value::new(value, when))
            .map(Value::into_inner);

        // Return previous value and notify flag.
        (prev, notify)
    }

    /// Removes a value from storage, returning it.
    #[inline]
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.kv_pairs.remove(key).map(Value::into_inner)
    }

    /// Method used by the background task to
    /// clean up already expired keys and return
    /// the expiry time of the next key, if any is set to expire.
    pub fn cleanup_expired_keys(&mut self) -> Option<Instant> {
        // Store current time
        let now = Instant::now();

        // Look at the min expiration time from the heap.
        while let Some(exp_key) = self.expirations.peek() {
            // If the minimum is bigger than now,
            // then return the expiry time so the background task waits for that amount.
            if exp_key > &now {
                return Some(exp_key.expiry_time);
            }

            // Otherwise, the key is expired, so we'll remove it.
            //
            // However, we never remove values from the heap apart from
            // when their expiration time comes.
            //
            // It is possible that the expiration time of a key gets
            // overwritten with a greater value than it initially was.
            //
            // To overcome that, we store the correct expiration time of the key
            // in the hashmap as well, and compare it with the minimum from the heap.
            //
            // If they are the same, then the expiration time is valid and they key should be removed.
            // Otherwise, the expiration is ignored and simply popped from the heap.
            let exp_is_valid = self
                .kv_pairs
                .get(&exp_key.key)
                .map(|v| v == exp_key)
                .unwrap_or_default();

            // Operations have to be separate to satisfy the borrow checker.
            // We need mutable access here while we used immutable access above.
            if exp_is_valid {
                self.kv_pairs.remove(&exp_key.key);
            }

            self.expirations.pop();
        }

        None
    }
}
