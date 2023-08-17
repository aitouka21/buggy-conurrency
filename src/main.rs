use std::{collections::HashMap, sync::Arc, thread};

use rand::seq::SliceRandom;
use std::sync::mpsc;

fn main() {
    let v = Arc::new(init());

    let mut txs = Vec::new();
    let mut rxs = HashMap::new();

    (0..10).for_each(|i| {
        // will not be deadlocked since async channel has an infinite buffer
        // let (tx, rx) = mpsc::channel();

        //deadlocked when buffer size < 5
        let (tx, rx) = mpsc::sync_channel::<i32>(4);
        txs.push(tx);
        rxs.insert(i, rx);
    });

    (0..10)
        .map(|i| {
            let v = Arc::clone(&v);
            let txs = txs.clone();
            let rx = rxs.remove(&i).unwrap();

            thread::spawn(move || unsafe {
                let v_ptr = Arc::into_raw(v);
                let v = &mut *(v_ptr as *mut Vec<i32>);

                for j in 0..10_000_000 {
                    let pos = i * 10_000_000 + j as usize;
                    let current = v[pos];
                    if current != i as i32 {
                        txs[current as usize].send(current).unwrap();
                        v[pos] = rx.recv().unwrap();
                    }
                }
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|h| h.join().unwrap());

    (0..100_000_000).for_each(|i| assert_eq!(v[i as usize], i / 10_000_000));
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
