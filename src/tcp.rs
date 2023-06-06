use crossbeam_channel::{unbounded, Receiver};
use rand::{self, Rng};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use crate::threadpool::ThreadPool;

const PORT: u16 = 12321;

/// Making this simple for packets... Each message is 8 bytes long, a u64.
/// There are no types here. In the future, we would need some encode/decode wrapper,
/// but I want to focus on the Select / polling part of it now.
pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new() -> Self {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", PORT)).unwrap();
        Self { listener }
    }

    pub fn listen(self) {
        // only get one stream, then shut down
        let stream = self.listener.incoming().next();
        self.handle_client(stream.unwrap().unwrap());
    }

    /// This is where we need to feed work into the thread-pool
    fn handle_client(&self, mut stream: TcpStream) {
        println!("    (Server) Handling client");

        let (response_tx, response_rx) = unbounded::<Receiver<u64>>();

        // one thread to read network, sending to thread pool
        let pool = ThreadPool::new(4);
        let mut read_stream = stream.try_clone().unwrap();
        thread::spawn(move || {
            loop {
                // read and parse data
                let buf = &mut [0; 8];
                let bytes_read = read_stream.read(buf).unwrap();
                if bytes_read == 0 {
                    println!("    (Server) Connection closed");
                    break;
                }
                let val = u64::from_be_bytes(*buf);
                println!("    (Server) Received: {}", val);

                // send work to threadpool
                let res = pool.execute(move || {
                    let delay = rand::thread_rng().gen_range(10..=2000);
                    thread::sleep(Duration::from_millis(delay));
                    println!("    (Server) Computed {}", val * val);
                    val * val
                });

                // send response channel to response thread
                response_tx.send(res).unwrap();
            }

            // once we finish all this, we shutdown the thread pool (end of queue, so after all work is done)
            pool.shutdown();
        });

        // other thread reading thread pool responses, sending to network
        for result in response_rx {
            let out = result.recv().unwrap();
            let msg = out.to_be_bytes();
            stream.write(&msg).unwrap();
        }
        println!("    (Server) Finished sending responses");
    }
}

/// This should send a bunch of work, and then wait for the results
pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn connect() -> Self {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", PORT)).unwrap();
        Self { stream }
    }

    // Send all info then wait for response
    pub fn work(&mut self) {
        println!("(Client) Sending work");

        // do writing in another thread, one packet every 200 ms (total 4s send)
        let mut client_stream = self.stream.try_clone().unwrap();
        let writer = thread::spawn(move || {
            (0..20u64).for_each(|i| {
                let msg = i.to_be_bytes();
                client_stream.write(&msg).unwrap();
                println!("(Client) sent {}", i);
                thread::sleep(Duration::from_millis(200));
            });
        });

        println!("(Client) Waiting for response");
        let mut results = 0;
        loop {
            let buf = &mut [0; 8];
            let bytes_read = self.stream.read(buf).unwrap();
            if bytes_read == 0 {
                println!("(Client) Connection closed");
                break;
            }
            let val = u64::from_be_bytes(*buf);
            println!("(Client) Received: {}", val);
            results += 1;
            if results == 20 {
                println!("(Client) Received all results");
                break;
            }
        }

        writer.join().unwrap();
        self.stream.shutdown(std::net::Shutdown::Both).unwrap();
    }
}
