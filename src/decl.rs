#![forbid(unsafe_code)]

use std::fmt;
use std::io;
use std::sync::mpsc::{Receiver, RecvError, SyncSender};
use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
pub enum ApplicationError {
    IOError(io::Error),
    MutexError,
    MpscSendError,
    MpscRecvError(RecvError),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ApplicationError: {}",
            match self {
                ApplicationError::IOError(ref e) => e.to_string(),
                ApplicationError::MutexError => "Internal error: mutex".to_string(),
                ApplicationError::MpscSendError => "Internal error: mpsc".to_string(),
                ApplicationError::MpscRecvError(ref e) => e.to_string(),
            }
        )
    }
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
        let mut complete = self.mutex.lock().or(Err(ApplicationError::MutexError))?;
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
