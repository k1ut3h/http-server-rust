use std::{thread::{self, JoinHandle}, sync::{mpsc, Arc, Mutex}};

pub struct ThreadPool{
    workers:Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker{
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Drop for ThreadPool{
    fn drop(&mut self){
        drop(self.sender.take());
        for worker in &mut self.workers{
            println!("Worker {} is shutting down.", worker.id);
            if let Some(thread) = worker.thread.take(){
                thread.join().unwrap();
            }
        }
    }
}

impl ThreadPool{
    pub fn new(size:usize)->ThreadPool{
        assert!(size>0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size{
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool{workers, sender: Some(sender)}
    }
    pub fn execute<F>(&self, f:F)
    where 
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Worker{
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>)->Worker{
        let thread =  thread::spawn(move || loop{
            let message = receiver.lock().unwrap().recv();
            match message{
                Ok(job)=>{
                    println!("Worker {id} got a job. Executing.");
                    job();
                },
                Err(_)=>{
                    println!("Worker {id} disconnected. Shutting down.");
                    break;
                }
            } 
        });
        Worker{id, thread:Some(thread)}
    }
}