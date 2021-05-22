use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use super::message::Message;
use std::time::Duration;

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>, timeout: u64) -> Self {
        let thread: thread::JoinHandle<()> = thread::spawn(move || loop {
            match receiver.lock().unwrap().recv_timeout(Duration::from_millis(timeout)){
                Ok(message) => {
                    match message {
                        Message::NewJob(job) => {
                            println!("Worker {} got a job; executing.", id);
                            job();
                        }
                        Message::Terminate => {
                            println!("Worker {} was told to terminate.", id);
                            break;
                        }
                    }
                },
                Err(_) => {
                    println!("Timeout!");
                    break;
                }
            }
        }); 
        Worker { id, thread: Some(thread) }
    }

    pub fn drop(&mut self) {
        println!("Shutting down worker {}", self.id);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}