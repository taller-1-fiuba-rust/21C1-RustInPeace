//! Servicio que implementa el threadpool para atender clientes del servidor.

use crate::domain::entities::message::Message;
use crate::domain::entities::worker::Worker;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

/// El Threadpool se compone por un vector de Workers y un sender.
/// Los workers se encargan de atender solicitudes de clientes del servidor, en relación de un worker por cliente.
/// El sender permite enviar mensajes a cada worker para indicarles si deben atender o no un nuevo cliente.
#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

/// En este contexto, un threadpool consiste en un conjunto de hilos (workers) que estarán pendientes de la llegada de un nuevo cliente del servidor.
/// El proposito del threadpool es lograr la concurrencia de tareas para tener un servidor más eficiente que pueda atender múltiples clientes concurrentemente.
impl ThreadPool {
    /// Crea una instancia de Threadpool.
    ///
    /// Crea un nuevo Threadpool con `quantity` workers.
    /// Crea los workers y los guarda en un vector.
    /// Además, crea un canal para poder enviar mensajes a cada worker.
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

    /// Delega una tarea a un worker.
    ///
    /// Recibe una conexión y la envía a los workers. Alguno de ellos se adjudicará la tarea de resolver las solicitudes de dicha conexión.
    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        match self.sender.send(Message::NewJob(job)) {
            Ok(_) => {} //
            Err(_) => {
                println!("Oops! Failed sending message")
            }
        }
    }
}
