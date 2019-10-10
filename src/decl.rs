// pub const DEFAULT_BUFFER_SIZE : usize = 1 << 16;
pub const DEFAULT_BUFFER_SIZE : usize = 1 << 20;
pub const DEFAULT_NTHREAD : usize = 4;

use std::io;
use std::mem;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::mpsc::{Receiver, SyncSender, SendError, RecvError};


pub type InputData = Vec<u8>;
pub type OutputData = Vec<u8>;

pub struct InputBuf {
    pub data: InputData,
    pub result: OutputData,
}

pub struct CompressResult {
    condvar: Condvar,
    mutex: Mutex<Option<(InputData, OutputData)>>
}

pub enum CompressChunk {
    Eof,
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

    pub fn wait(&self) -> (InputData, OutputData) {
        let mut complete = self.mutex.lock().unwrap();
        while *complete == None {
            complete = self.condvar.wait(complete).unwrap();
        }

        let mut holder = None;
        mem::swap(&mut holder, &mut *complete);
        holder.unwrap()
    }

    pub fn notify(&self, input: InputData, result: OutputData) {
        let mut complete = self.mutex.lock().unwrap();
        *complete = Some((input, result));
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


#[derive(Debug)]
pub enum ReaderThreadError {
    IOError(io::Error),
    MpscSendError(SendError<CompressChunk>),
    MpscRecvError(RecvError)
}
