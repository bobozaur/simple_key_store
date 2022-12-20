use std::thread;

use simple_key_store::KeyStore;

const KEYS: [&str; 5] = ["abc", "def", "ghi", "jkl", "mno"];
const NUM_THREADS: usize = 20;
const NUM_REPEATS: usize = 1000;

fn thread_task(keystore: KeyStore<String, String>, extension: usize) {
    for (i, key) in KEYS.repeat(NUM_REPEATS).into_iter().enumerate() {
        let k = format!("{key}_{i}_{extension}");

        keystore.insert(k.clone(), key.to_owned(), None);
        assert_eq!(keystore.get(&k), Some(key.to_owned()));

        keystore.remove(&k);
        assert_eq!(keystore.get(&k), None);
    }
}

#[test]
fn multithread() {
    let keystore = KeyStore::new();
    let mut handles = Vec::new();

    for i in 0..NUM_THREADS {
        let ks = keystore.clone();
        let handle = thread::spawn(move || thread_task(ks, i));
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join();
    }
}
