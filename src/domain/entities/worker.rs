//! Representa un hilo de ejecución, encargado de atender solicitudes de un cliente.

use super::message::Message;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

/// Un Worker se compone por un id único y un hilo de ejecución en donde estará pendiente de recibir mensajes del threadpool.
/// Dichos mensajes pueden ser un nuevo cliente (una nueva conexión) o dejar de atender.
#[derive(Debug)]
pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Crea una nueva instancia Worker.
    ///
    /// Crea un hilo de ejecución donde queda pendiente de recibir mensajes del Threadpool.
    /// Cada mensaje de tipo `NewJob` es un nuevo cliente que le es asignado para atender.
    /// Espera mensajes hasta recibir el mensaje `Terminate`, este mensaje le indica que debe dejar de atender clientes.
    /// # Ejemplo
    /// ```
    /// use proyecto_taller_1::domain::entities::worker;
    /// use std::sync::mpsc;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let (sender, receiver) = mpsc::channel();
    /// let recv = Arc::new(Mutex::new(receiver));
    /// let worker = worker::Worker::new(1, recv);
    /// ```
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

    /// Cierra el hilo de ejecución del worker.
    pub fn shutdown(&mut self) {
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
