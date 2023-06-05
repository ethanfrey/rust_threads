use rand::{self, Rng};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let start = Instant::now();
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let delay = rand::thread_rng().gen_range(10..=2000);
            let builder = thread::Builder::new().name(format!("Thread-{}", i));
            builder
                .spawn(move || {
                    println!("Thread {} sleeping for {} ms", i, delay);
                    thread::sleep(Duration::from_millis(delay));
                    thread::current().name().unwrap().to_string()
                })
                .unwrap()
        })
        .collect();

    println!("Awaiting threads");
    for h in handles {
        println!("{} finished", h.join().unwrap());
    }

    println!("Time elapsed: {:?}", start.elapsed());
}
