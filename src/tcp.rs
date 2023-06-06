use rand::{self, Rng};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

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

        loop {
            let buf = &mut [0; 8];
            let bytes_read = stream.read(buf).unwrap();
            if bytes_read == 0 {
                println!("    (Server) Connection closed");
                break;
            }
            let val = u64::from_be_bytes(*buf);
            // TODO: send work to threadpool
            println!("    (Server) Received: {}", val);
            let delay = rand::thread_rng().gen_range(10..=2000);
            thread::sleep(Duration::from_millis(delay));
            let out = val * val;
            let msg = out.to_be_bytes();
            stream.write(&msg).unwrap();
        }
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
        (0..20u64).for_each(|i| {
            let msg = i.to_be_bytes();
            self.stream.write(&msg).unwrap();
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

        self.stream.shutdown(std::net::Shutdown::Both).unwrap();
    }
}
