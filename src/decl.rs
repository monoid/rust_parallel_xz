use std::io;
use std::mem;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::mpsc::{Receiver, SyncSender, RecvError};


#[derive(Debug)]
pub enum ApplicationError {
    IOError(io::Error),
    MutexError,
    MpscSendError,
    MpscRecvError(RecvError),
    ErrorElsewhere // KLUDGE to avoid copying io::Error (wrap it with Arc?)
}


pub type InputData = Vec<u8>;
pub type OutputData = Vec<u8>;

pub struct SpareData {
    pub data: InputData,
    pub result: OutputData,
}

pub struct CompressFuture {
    condvar: Condvar,
    mutex: Mutex<Option<Result<(InputData, OutputData), ApplicationError>>>
}

pub enum WriterData {
    Eof,
    Error(ApplicationError),
    Data(Arc<CompressFuture>)
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
        let mut complete = self.mutex.lock().or(
            Err(ApplicationError::MutexError)
        )?;
        while (*complete).is_none() {
            complete = self.condvar.wait(complete).unwrap();
        }

        let mut holder = None;
        mem::swap(&mut holder, &mut *complete);
        holder.unwrap()
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
            data: data,
            length: length,
            result: result,
        }
    }
}


pub type SpareDataQueueReceiver = Receiver<SpareData>;
pub type SpareDataQueueSender = SyncSender<SpareData>;

pub type WriterDataReceiver = Receiver<WriterData>;
pub type WriterDataSender = SyncSender<WriterData>;
