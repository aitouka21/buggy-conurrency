use std::{sync::Arc, thread};

use rand::seq::SliceRandom;
use std::sync::mpsc;

fn main() {
    let vec = Arc::new(init());

    let (txs, rxs): (Vec<_>, Vec<_>) = (0..10)
        .map(|_| mpsc::sync_channel::<i32>(5)) // TODO: explain why it will be deadlocked when buffer size < 5
        .unzip();

    rxs.into_iter()
        .enumerate()
        .map(|(i, rx)| {
            let vec = Arc::clone(&vec);
            let txs = txs.clone();

            thread::spawn(move || unsafe {
                let vec_ptr = Arc::into_raw(vec);
                let vec = &mut *(vec_ptr as *mut Vec<i32>);
                for j in 0..10_000_000 {
                    let pos = i * 10_000_000 + j as usize;
                    let current = vec[pos];
                    if current != i as i32 {
                        txs[current as usize].send(current).unwrap();
                        vec[pos] = rx.recv().unwrap();
                    }
                }
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|h| h.join().unwrap());

    (0..100_000_000).for_each(|i| assert_eq!(vec[i as usize], i / 10_000_000));
}

fn init() -> Vec<i32> {
    let mut vec = Vec::new();
    for i in 0..10 {
        for _ in 0..10_000_000 {
            vec.push(i);
        }
    }
    let mut rng = rand::thread_rng();
    vec.shuffle(&mut rng);
    vec
}
