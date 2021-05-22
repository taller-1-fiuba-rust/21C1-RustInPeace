use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use crate::entities::worker::Worker;
use crate::entities::message::Message;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>
}

impl ThreadPool {
    pub fn new(quantity: usize, timeout: u64) -> Self {
        assert!(quantity > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(quantity);
        for id in 0..quantity {
            workers.push(Worker::new(id, Arc::clone(&receiver), timeout));
        }
        ThreadPool { workers, sender }
    }

    pub fn spawn<F>(&self, f: F)
    where 
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            worker.drop();
        }
    }
}