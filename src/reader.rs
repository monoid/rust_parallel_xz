use super::compress::compress_data;
use super::decl::*;

use std::io::prelude::*;
use std::sync::mpsc;
use std::sync::Arc;

use threadpool::ThreadPool;

pub fn init_spare_queue(
    buffer_size: usize,
    queue_size: usize,
) -> (SpareDataQueueSender, SpareDataQueueReceiver) {
    let (send, recv) = mpsc::sync_channel(queue_size);
    for _ in 0..queue_size {
        let mut data: Vec<u8> = Vec::with_capacity(buffer_size);
        data.resize(buffer_size, 0);
        send.send(SpareData {
            data: data,
            result: Vec::new(),
        })
        .unwrap();
    }
    return (send, recv);
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
    // This flag prevents from sending final empty chunk if file is not empty
    let mut has_any_data = false;

    loop {
        let mut buf = inp_que
            .recv()
            .map_err(|e| ApplicationError::MpscRecvError(e))?;
        let length = input
            .read(&mut buf.data)
            .map_err(|e| ApplicationError::IOError(e))?;
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
                .or(Err(ApplicationError::MpscSendError))?;
        }
    }
}
