use std::sync::mpsc;

use crate::worker;

error_chain! {
    foreign_links {
        WorkerSend(mpsc::SendError<worker::Request>);
        WorkerRecv(mpsc::RecvError);
    }
}