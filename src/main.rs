use rand::{self, Rng};
use std::thread;
use std::time::{Duration, Instant};

mod threadpool;

fn main() {
    let start = Instant::now();

    let pool = threadpool::ThreadPool::new(4);
    println!("Setup after: {:?}", start.elapsed());

    let answers: Vec<_> = (0..20)
        .map(|i| {
            pool.execute(move || {
                let delay = rand::thread_rng().gen_range(10..=2000);
                thread::sleep(Duration::from_millis(delay));
                i * i
            })
        })
        .collect();
    println!("Sent after: {:?}", start.elapsed());

    for answer in answers {
        println!("Answer: {}", answer.recv().unwrap());
    }

    pool.shutdown();
    println!("Time elapsed: {:?}", start.elapsed());
}
