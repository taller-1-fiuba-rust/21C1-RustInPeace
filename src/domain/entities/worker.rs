use super::message::Message;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let thread: thread::JoinHandle<()> = thread::spawn(move || loop {
            let message = receiver
                .lock()
                .unwrap_or_else(|_e| panic!("Lock error, worker {}", id))
                .recv()
                .unwrap_or(Message::Terminate);
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job: executing.", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }

    pub fn drop_(&mut self) {
        println!("Shutting down worker {}", self.id);
        if let Some(thread) = self.thread.take() {
            match thread.join() {
                Ok(_) => {
                    println!("Thread succesfuly joined")
                }
                Err(_) => {
                    println!(
                        "Couldn't join on the thread associated to Worker {}",
                        self.id
                    )
                }
            }
        }
    }
}
