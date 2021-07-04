use crate::domain::entities::message::Message;
use crate::domain::entities::worker::Worker;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(quantity: usize) -> Self {
        assert!(quantity > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(quantity);
        for id in 0..quantity {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }

    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        match self.sender.send(Message::NewJob(job)) {
            Ok(_) => {} //
            Err(_) => {
                //si no hay ningun thread para agarrar el job, mandamos el
                //job a una cola??
                println!("Oops! Failed sending message")
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            match self.sender.send(Message::Terminate) {
                Ok(_) => {} //
                Err(_) => {
                    println!("Oops! Failed sending terminate message");
                }
            }
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            worker.shutdown();
        }
    }
}
