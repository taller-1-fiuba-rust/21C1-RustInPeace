use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use super::message::Message;

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        let thread: thread::JoinHandle<()> = thread::spawn(move || loop {
            //let message = receiver.lock().unwrap().recv();
            match receiver.lock() {
                Ok(inner) => {
                    match inner.recv() {
                        Ok(message) => {
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
                        }
                        Err(_) => {
                            println!("no more messages can ever be 
                            received on this channel by Worker {}",id);
                            break
                        }
                    }
                }
                Err(_) => {
                    println!("Another user of this mutex panicked while holding the mutex,
                    then Worker {} couldn't handle MutexGuard",id);
                    break
                }
            }
        }); 
        Worker { id, thread: Some(thread) }
    }

    pub fn drop(&mut self) {
        println!("Shutting down worker {}", self.id);
        if let Some(thread) = self.thread.take() {
            match thread.join(){
                Ok(_)=> {
                    println!("Thread succesfuly joined")
                }
                Err(_)=>{
                    println!("Couldn't join on the thread associated to Worker {}",self.id)
                }
            }
        }
    }
}