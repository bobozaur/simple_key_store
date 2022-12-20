use std::{
    sync::{Condvar, Mutex, RwLockWriteGuard},
    time::Duration,
};

/// A [`Condvar`] wrapper that essentially allows
/// using any lock guard by relying internally on a [`Mutex`].
///
/// This allows us to use a [`std::sync::RwLock`] to improve concurrent reads.
#[derive(Debug)]
pub struct Notifier {
    condvar: Condvar,
    mutex: Mutex<()>,
}

impl Notifier {
    pub fn new() -> Self {
        Self {
            condvar: Condvar::new(),
            mutex: Mutex::new(()),
        }
    }

    /// Waits indefinitely until notified.
    #[allow(unused_must_use)]
    pub fn wait<T>(&self, _guard: RwLockWriteGuard<'_, T>) {
        // Lock the internal Mutex.
        let guard = self.mutex.lock().unwrap();
        // We acquired the internal lock so we'll drop the external one.
        drop(_guard);
        // Wait
        self.condvar.wait(guard).unwrap();
    }

    /// Similar to [`Notifier::wait`], but relies on [`Condvar::wait_timeout`].
    #[allow(unused_must_use)]
    pub fn wait_timeout<T>(&self, _guard: RwLockWriteGuard<'_, T>, dur: Duration) {
        let guard = self.mutex.lock().unwrap();
        drop(_guard);
        self.condvar.wait_timeout(guard, dur).unwrap();
    }

    /// Notifies one thread that's blocked on the internal [`Condvar`].
    pub fn notify_one(&self) {
        self.condvar.notify_one()
    }
}
