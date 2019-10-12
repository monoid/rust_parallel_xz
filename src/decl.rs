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

pub struct InputBuf {
    pub data: InputData,
    pub result: OutputData,
}

pub struct CompressResult {
    condvar: Condvar,
    mutex: Mutex<Option<Result<(InputData, OutputData), ApplicationError>>>
}

pub enum CompressChunk {
    Eof,
    Error(ApplicationError),
    Data(Arc<CompressResult>)
}

pub struct CompressTask {
    pub data: InputData,
    pub length: usize,
    pub result: OutputData,
}

impl CompressResult {
    pub fn new() -> CompressResult {
        let mutex = Mutex::new(None);
        let condvar = Condvar::new();

        CompressResult {
            mutex: mutex,
            condvar: condvar,
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

    pub fn notify(&self, input: InputData, result: OutputData) {
        let mut complete = self.mutex.lock().unwrap();
        *complete = Some(Ok((input, result)));
        self.condvar.notify_one();
    }

    pub fn notify_error(&self, err: ApplicationError) {
        let mut complete = self.mutex.lock().unwrap();
        *complete = Some(Err(err));
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


pub type FreeDataQueueReceiver = Receiver<InputBuf>;
pub type FreeDataQueueSender = SyncSender<InputBuf>;

pub type TaskReceiver = Receiver<CompressChunk>;
pub type TaskSender = SyncSender<CompressChunk>;
