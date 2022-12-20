use std::{
    sync::mpsc::{self, Sender},
    thread,
    time::{Duration, Instant},
};

use simple_key_store::KeyStore;

const KEYS: [&str; 5] = ["abc", "def", "ghi", "jkl", "mno"];
const NUM_THREADS: usize = 20;
const NUM_REPEATS: usize = 1000;

fn thread_task(keystore: KeyStore<String, String>, extension: usize, sender: Sender<Duration>) {
    let mut expire = false;
    let mut total_time = Duration::from_secs(0);

    for (i, key) in KEYS.repeat(NUM_REPEATS).into_iter().enumerate() {
        let k = format!("{key}_{i}_{extension}");

        let expiry_time = if expire {
            expire = false;
            Some(Duration::from_nanos(3))
        } else {
            expire = true;
            None
        };

        keystore.insert(k.clone(), key.to_owned(), expiry_time);

        let now = Instant::now();
        assert!(keystore.get(&k).map(|v| v == key).unwrap_or(true));
        total_time += now.elapsed();

        keystore.remove(&k);

        let now = Instant::now();
        assert!(keystore.get(&k).map(|v| v == key).unwrap_or(true));
        total_time += now.elapsed();
    }

    sender.send(total_time).unwrap();
}

#[test]
fn multithread() {
    let keystore = KeyStore::new();
    let mut handles = Vec::new();
    let (sender, receiver) = mpsc::channel();
    let mut total_time = Duration::from_secs(0);

    for i in 0..NUM_THREADS {
        let ks = keystore.clone();
        let sender_clone = sender.clone();
        let handle = thread::spawn(move || thread_task(ks, i, sender_clone));
        handles.push(handle);
    }

    drop(sender);

    while let Ok(time) = receiver.recv() {
        total_time += time;
    }

    let avg_time = total_time.as_nanos() / (NUM_REPEATS * NUM_REPEATS) as u128;
    println!("Average retrieval time in nanoseconds: {}", avg_time);

    for handle in handles {
        let _ = handle.join();
    }
}
