use rand::{self, Rng};
use std::thread::{sleep, spawn};
use std::time::Duration;

fn main() {
    let handle = spawn(|| {
        let delay = rand::thread_rng().gen_range(10..=2000);
        sleep(Duration::from_millis(delay));
        println!("Hello from a thread!");
        5
    });
    println!("Awaiting thread");
    let res = handle.join().unwrap();
    println!("Thread returned {}", res);
}
