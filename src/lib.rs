//! A simple, thread-safe, synchronous `key:value` storage.
//!
//! ```
//! # use simple_key_store::KeyStore;
//! #
//! let key_store = KeyStore::new();
//! key_store.insert("a".to_owned(), 1, None);
//!
//! assert_eq!(key_store.get("a"), Some(1));
//! assert_eq!(key_store.remove("a"), Some(1));
//! assert_eq!(key_store.get("a"), None);
//! ```
//!
//! The [`KeyStore`] allows retrieving, deleting and inserting values, optionally allowing settings values to auto-expire when inserted.
//! The expiration is taken care of by a task running in a background thread. Only the last expiration time set for a key is considered.
//! The [`KeyStore`] will ensure the background task is terminated when the last reference to it is dropped.
//!
//! ```
//! # use std::{time::Duration, thread};
//! #
//! # use simple_key_store::KeyStore;
//! #
//! let key_store = KeyStore::new();
//!
//! key_store.insert("b".to_owned(), 2, Some(Duration::from_millis(5)));
//! assert_eq!(key_store.get("b"), Some(2));
//!
//! // Wait for the key to expire
//! thread::sleep(Duration::from_millis(6));
//! assert_eq!(key_store.get("b"), None);
//!
//! // Reinsert the value
//! key_store.insert("b".to_owned(), 2, Some(Duration::from_millis(5)));
//! assert_eq!(key_store.get("b"), Some(2));
//!
//! // Overwrite the value and expiration key
//! key_store.insert("b".to_owned(), 3, Some(Duration::from_millis(10)));
//!
//! // Wait for the first timer to elapse
//! thread::sleep(Duration::from_millis(6));
//!
//! // The key is still here, as the last inserted timer was greater
//! assert_eq!(key_store.get("b"), Some(3));
//!
//! // Wait again, for the last timer to elapse
//! thread::sleep(Duration::from_millis(10));
//!
//! // The key isn't present anymore, as it expired.
//! // assert_eq!(key_store.get("b"), None);
//! ```

mod drop_guard;
mod expiring_key;
mod keystore;
mod notifier;
mod storage;
mod value;

pub use keystore::KeyStore;
