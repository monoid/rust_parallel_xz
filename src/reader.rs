#![forbid(unsafe_code)]

use super::compress::compress_data;
use super::decl::*;

use std::io::prelude::*;
use std::sync::mpsc;
use std::sync::Arc;

use threadpool::ThreadPool;

pub fn init_spares_queue(
    buffer_size: usize,
    queue_size: usize,
) -> (SpareDataQueueSender, SpareDataQueueReceiver) {
    let (send, recv) = mpsc::sync_channel(queue_size);
    for _ in 0..queue_size {
        let data = vec![0; buffer_size];
        send.send(SpareData {
            data,
            result: Vec::new(),
        })
        .unwrap();
    }
    (send, recv)
}

pub fn init_task_queue(cpu_count: usize) -> (WriterDataSender, WriterDataReceiver) {
    mpsc::sync_channel(cpu_count)
}

pub fn reader_loop<R: Read>(
    input: &mut R,
    inp_que: SpareDataQueueReceiver,
    out_que: WriterDataSender,
    pool: ThreadPool,
    compress_level: u32, // TODO compressor factory instead
) -> (SpareDataQueueReceiver, Result<(), ApplicationError>) {
    // We call another function to circumvent too strict borrowchecker.
    let res = reader_loop_nested(input, &inp_que, out_que, pool, compress_level);
    (inp_que, res)
}

fn reader_loop_nested<R: Read>(
    input: &mut R,
    inp_que: &SpareDataQueueReceiver,
    out_que: WriterDataSender,
    pool: ThreadPool,
    compress_level: u32,
) -> Result<(), ApplicationError> {
    // This flag allows us to send an empty chunk if file is empty.
    // Otherwise we will get empty output that is not OK.
    let mut has_any_data = false;

    loop {
        let mut buf = inp_que.recv().map_err(ApplicationError::MpscRecv)?;
        let length = input.read(&mut buf.data).map_err(ApplicationError::IO)?;
        if length == 0 && has_any_data {
            return Ok(());
        } else {
            let result = buf.result;
            let task = CompressTask::new(buf.data, length, result);
            let comp_result = Arc::new(CompressFuture::new());
            let comp_result1 = comp_result.clone();
            pool.execute(move || {
                compress_data(task, comp_result1, compress_level);
            });
            has_any_data = true;
            out_que
                .send(comp_result)
                .or(Err(ApplicationError::MpscSend))?;
        }
    }
}
