# simple_key_store

A simple, in-memory, synchronous, key store library crate that practically behaves as a HashMap with optionally expirable keys.

# How it works:
- the library exposes a thread-safe `KeyStore` which holds data in a HashMap under the hood and tracks expirations through a Min-Heap. 
- a background thread is started when the key store is created which removes keys as they expire.
- when dropped, the key store will trigger the stop of the background thread and wait for it to exit.

# Considerations:
## Async
An async version of this library could be fairly easily implemented, with a few modifications. Main aspects that come to mind are:
- relying on an async background task
- using async locks (`tokio` comes to mind) -- this could be avoided though to have the key store available in sync code too.
- using something like `tokio::sync::Notify` instead of `std::sync::Condvar`.

The main benefit would probably come from the async background task though, as the context switch is not as costly and it would result in more accurate expirations of keys.

## BST
A Binary Search Tree or the built in `std::collections::BTreeMap` could be used for tracking expirations, but we're only ever really interested in the expiration that will happen the soonest, not needing a fully ordered list.
Therefore, a Min-Heap should technically result in less node re-ordering on insertions/removals.
The keystore keeps track of the last expiration set for a key, so even if, say, a `key:value` pair was added to expire in 5 seconds and then added again but with an expiration time of 10 seconds, we don't even bother removing the 5 second expiration from the heap - we simply ignore it as we compare it with the expected expiration.
