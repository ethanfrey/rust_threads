use rand::{self, Rng};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let start = Instant::now();
    let (sender, receiver) = mpsc::channel::<String>();

    let _handles: Vec<_> = (0..10)
        .map(|i| {
            let delay = rand::thread_rng().gen_range(10..=2000);
            let builder = thread::Builder::new().name(format!("Thread-{}", i));
            let my_sender = sender.clone();
            builder
                .spawn(move || {
                    println!("Thread {} sleeping for {} ms", i, delay);
                    thread::sleep(Duration::from_millis(delay));
                    let name = thread::current().name().unwrap().to_string();
                    my_sender.send(name).unwrap();
                })
                .unwrap()
        })
        .collect();

    println!("Awaiting threads");
    for _ in 0..10 {
        let name = receiver.recv().unwrap();
        println!("Received: {}", name);
    }

    println!("Time elapsed: {:?}", start.elapsed());
}
