use std::sync::mpsc;

use crate::worker;

error_chain! {
    foreign_links {
        WorkerRecv(mpsc::RecvError);
    }
}