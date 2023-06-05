use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use std::thread;

pub struct ThreadPool<T> {
    workers: Vec<Worker>,
    work_sender: Sender<Message<T>>,
}

impl<T: Send + 'static> ThreadPool<T> {
    pub fn new(size: usize) -> Self {
        let (work_sender, work_receiver) = unbounded();

        let workers = (0..size)
            .map(|_| Worker::new(work_receiver.clone()))
            .collect();

        Self {
            workers,
            work_sender,
        }
    }

    pub fn execute<F>(&self, task: F) -> Receiver<T>
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let (sender, receiver) = bounded(1);
        let msg = Message::RunTask {
            task: Box::new(task),
            sender,
        };
        self.work_sender.send(msg).unwrap();
        receiver
    }

    pub fn shutdown(self) {
        for _ in &self.workers {
            self.work_sender.send(Message::Terminate).unwrap();
        }

        for worker in self.workers {
            worker.thread.join().unwrap();
        }
    }
}

struct Worker {
    thread: thread::JoinHandle<()>,
}

impl Worker {
    pub fn new<T: Send + 'static>(receiver: Receiver<Message<T>>) -> Self {
        let thread = thread::spawn(move || loop {
            match receiver.recv().unwrap() {
                Message::RunTask { task, sender } => {
                    let out = task();
                    sender.send(out).unwrap();
                }
                Message::Terminate => {
                    println!("Terminating worker");
                    return;
                }
            }
        });

        Self { thread }
    }
}

enum Message<T> {
    RunTask {
        task: BoxedTask<T>,
        sender: Sender<T>,
    },
    Terminate,
}

type BoxedTask<T> = Box<dyn FnOnce() -> T + Send + 'static>;
