use std::{
    hash::Hash,
    ops::Deref,
    sync::{atomic::Ordering, Arc},
    thread::{self, JoinHandle},
    time::Instant,
};

use crate::storage::Shared;
/// Struct that handles graceful exit of the background task.
/// Acts as a smart pointer over the [`Shared`] state.
#[derive(Debug)]
pub struct DropGuard<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    shared: Arc<Shared<K, V>>,
    cleanup_task: Option<JoinHandle<()>>,
}

impl<K, V> DropGuard<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        let shared = Arc::new(Shared::new());
        let shared_clone = shared.clone();
        let cleanup_task = thread::spawn(move || Self::cleanup_task(shared_clone));

        Self {
            shared,
            cleanup_task: Some(cleanup_task),
        }
    }

    pub fn notify(&self) {
        let _ = self.cleanup_task.as_ref().map(|t| t.thread().unpark());
    }

    /// The background task that will clean up expired keys from storage.
    fn cleanup_task(shared: Arc<Shared<K, V>>) {
        // As long as the task is supposed to run
        while shared.run_cleanup.load(Ordering::Acquire) {
            // Get exclusive access to storage
            let opt_exp = shared.write().cleanup_expired_keys();

            // Cleanup expired keys and retrieve
            // an optional time when the next key will expire.
            //
            // Afterwards wait for either the next key's expiration
            // or to be notified of a key expiring even sooner.
            //
            // Notifying the task simply loops again.
            match opt_exp {
                Some(expiry_time) => thread::park_timeout(Instant::now() - expiry_time),
                None => thread::park(),
            }
        }
    }
}

/// While abusing [`Deref`] is frowned upon, I'd argue that this is a legitimate case.
impl<K, V> Deref for DropGuard<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    type Target = Shared<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.shared
    }
}

/// Custom [`Drop`] implementation that notifies
/// the background cleanup task to stop and waits for that to happen.
impl<K, V> Drop for DropGuard<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn drop(&mut self) {
        // Toggle flag
        self.shared.stop_cleanup();
        // Notify task so that it proceeds
        self.notify();
        // Join the handle to wait for task's completion
        self.cleanup_task.take().map(|h| h.join());
    }
}
