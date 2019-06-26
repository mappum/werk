use std::thread;
use std::sync::mpsc::{
    channel,
    sync_channel,
    Sender,
    Receiver
};

use crate::error::Result;

pub trait Exec {
    type Input: Send + 'static;
    type Output: Send + 'static;

    fn setup(&mut self) {} // TODO: result
    fn exec(&mut self, input: Self::Input) -> Self::Output;
}

pub struct Worker<E: Exec> {
    handle: thread::JoinHandle<()>,
    input: Sender<Request<E::Input>>,
    output: Receiver<E::Output>
}

pub enum Request<T: Send> {
    Exec(T),
    Kill
}

impl<E: Exec + Send + 'static> Worker<E> {
    pub fn new(mut exec: E) -> Worker<E> {
        let (input_tx, input_rx) = channel::<Request<E::Input>>();
        let (output_tx, output_rx) = sync_channel::<E::Output>(0);

        let handle = thread::spawn(move || {
            let input = input_rx;
            let output = output_tx;

            // TODO: when this returns Result, handle sending back
            exec.setup();

            loop {
                // block until we have work to do
                let req = input.recv().unwrap();
                
                let res = match req {
                    Request::Exec(input) => exec.exec(input),
                    Request::Kill => break
                };

                output.send(res).unwrap();
            }
        });

        Worker {
            handle,
            input: input_tx,
            output: output_rx
        }
    }

    pub fn exec(&self, input: E::Input) -> Result<E::Output> {
        if let Err(error) = self.input.send(Request::Exec(input)) {
            bail!("Could not send work to worker");
        }
        Ok(self.output.recv()?)
    }
}

impl<E: Exec> Drop for Worker<E> {
    fn drop(&mut self) {
        self.input.send(Request::Kill).unwrap();
    }
}

pub struct Pool<E: Exec> {
    workers: Vec<Worker<E>>
}

impl<E: Exec> Pool<E> {
    pub fn new(worker_count: usize) -> Pool<E> {
        let workers = Vec::with_capacity(worker_count);

        for i in 0..worker_count {
            // let worker = Worker::new()
            // workers.push(worker);
        }

        Pool { workers }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn worker_test() {
        struct TestExec(u64);
        impl Exec for TestExec{
            type Input = u64;
            type Output = u64;

            fn exec(&mut self, n: u64) -> u64 {
                for i in 1..=n {
                    self.0 += i;
                }
                self.0
            }
        }

        let worker = Worker::new(TestExec(1));
        println!("result: {:?}", worker.exec(100));
    }
}
