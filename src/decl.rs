#![forbid(unsafe_code)]

use std::io;
use std::sync::mpsc::{Receiver, RecvError, SyncSender};
use std::sync::{Arc, Condvar, Mutex};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("IO error {0:?}")]
    IO(#[from] io::Error),
    #[error("Internal error: mutex")]
    Mutex,
    #[error("Internal error: mpsc")]
    MpscSend,
    #[error("Internal error: mpsc {0:?}")]
    MpscRecv(RecvError),
}

pub type InputData = Vec<u8>;
pub type OutputData = Vec<u8>;

pub struct SpareData {
    pub data: InputData,
    pub result: OutputData,
}

pub struct CompressFuture {
    condvar: Condvar,
    mutex: Mutex<Option<Result<(InputData, OutputData), ApplicationError>>>,
}

pub struct CompressTask {
    pub data: InputData,
    pub length: usize,
    pub result: OutputData,
}

impl CompressFuture {
    pub fn new() -> CompressFuture {
        CompressFuture {
            mutex: Mutex::new(None),
            condvar: Condvar::new(),
        }
    }

    pub fn wait(&self) -> Result<(InputData, OutputData), ApplicationError> {
        let mut complete = self.mutex.lock().or(Err(ApplicationError::Mutex))?;
        while (*complete).is_none() {
            complete = self.condvar.wait(complete).unwrap();
        }

        (*complete).take().unwrap()
    }

    pub fn notify(&self, result: Result<(InputData, OutputData), ApplicationError>) {
        let mut complete = self.mutex.lock().unwrap();
        *complete = Some(result);
        self.condvar.notify_one();
    }
}

impl CompressTask {
    pub fn new(data: InputData, length: usize, result: OutputData) -> CompressTask {
        CompressTask {
            data,
            length,
            result,
        }
    }
}

pub type SpareDataQueueReceiver = Receiver<SpareData>;
pub type SpareDataQueueSender = SyncSender<SpareData>;

pub type WriterDataReceiver = Receiver<Arc<CompressFuture>>;
pub type WriterDataSender = SyncSender<Arc<CompressFuture>>;
