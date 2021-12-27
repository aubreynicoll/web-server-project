use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    pool: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Terminating all workers...");
        for _ in &self.pool {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.pool {
            println!("Dropping worker: id {}", worker.id);
            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }
        }
    }
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let mut pool: Vec<Worker> = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel::<Message>();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            pool.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { pool, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let handle = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} executing job", id);
                    job();
                }
                Message::Terminate => {
                    println!("Terminating worker {}", id);
                    break;
                }
            }
        });

        Worker {
            id,
            handle: Some(handle),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}
