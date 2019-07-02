use std::thread;
use std::sync::mpsc::{
    channel,
    sync_channel,
    Sender,
    Receiver
};

use crate::error::Result;

pub struct Worker<I: Send, O: Send> {
    handle: thread::JoinHandle<()>,
    input: Sender<Request<I, O>>
}

pub enum Request<I: Send, O: Send> {
    Exec(I, Sender<O>),
    Kill
}

impl<I: Send + 'static, O: Send + 'static> Worker<I, O> {
    pub fn new<S, W, C>(setup_fn: S, work_fn: W) -> Worker<I, O>
        where
            S: 'static + Send + Fn() -> C,
            W: 'static + Send + Fn(&mut C, I) -> O
    {
        let (tx, rx) = channel::<Request<I, O>>();

        let handle = thread::spawn(move || {
            let mut ctx = setup_fn();

            loop {
                // block until we have work to do
                let req = rx.recv().unwrap();
                
                match req {
                    Request::Exec(input, output) => {
                        let res = work_fn(&mut ctx, input);
                        output.send(res).unwrap();
                    }
                    Request::Kill => break
                };
            }
        });

        Worker { handle, input: tx }
    }

    pub fn exec(&self, input: I) -> impl FnOnce() -> O {
        let (tx, rx) = channel::<O>();
        self.input.send(Request::Exec(input, tx)).unwrap();
        move || -> O { rx.recv().unwrap() }
    }
}

impl<I: Send, O: Send> Drop for Worker<I, O> {
    fn drop(&mut self) {
        self.input.send(Request::Kill).unwrap();
    }
}

// pub struct Pool<E: Exec> {
//     workers: Vec<Worker<E>>
// }

// impl<E: Exec + Default + Send + 'static> Pool<E> {
//     pub fn new(worker_count: usize) -> Pool<E> {
//         // TODO: return result based on worker setup

//         let handles = Vec::with_capacity(worker_count);
//         let workers = Vec::with_capacity(worker_count);

//         for i in 0..worker_count {
//             let exec: E = Default::default();
//             let (handle, worker) = Worker::new(exec);
//             handles.push(handle);
//             workers.push(worker);
//         }

//         // TODO: wait for worker setup
//         // TODO: keep handles

//         Pool { workers }
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn worker_test() {
        let worker = Worker::new(
            || 0 as usize,
            |sum, n| {
                for i in 1..n {
                    *sum += i; 
                }
                *sum
            }
        );
        assert_eq!(worker.exec(100)(), 4950);
        assert_eq!(worker.exec(100)(), 9900);
    }

    // #[test]
    // fn pool_test() {
        // let worker 

        //         // (stop forking when we've recursed all the way)
        //         if nums.is_empty() {
        //             return 0;
        //         }

        //         if workers.is_empty() {
        //             return nums.iter().sum();
        //         }

        //         // split numbers into left and right halves
        //         let (left, right) = nums.split_at(nums.len() / 2);

        //         // start working on right half
        //         // (or if out of workers, compute it in this thread).
        //         // returns a join function.
        //         let right_sum_join = workers[0].exec((workers[1..].to_vec(), right));
        //         // sum left half
        //         let left_sum = left.iter().sum();
        //         // combine the sums, waiting for right half if necessary
        //         left_sum + right_sum_join()
        //     }
        // }

        // let nums: Vec<u32> = (0..100).collect();

        // println!("result: {:?}", res_join());
    // }
}

// werk::pool(8, || {
//     let db = rocksdb::open();

//     let work = |n| {
//         n * 2
//     };

//     Ok(work)
// });
