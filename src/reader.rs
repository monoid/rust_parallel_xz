use super::decl::*;
use super::compress::compress_data;

use std::io::prelude::*;
use std::sync::mpsc;
use std::sync::Arc;

use threadpool::ThreadPool;


pub fn init_free_data_queue(buffer_size: usize, cpu_count: usize) -> (SpareDataQueueSender, SpareDataQueueReceiver) {
    let (send, recv) = mpsc::sync_channel(cpu_count);
    for _ in 0..cpu_count {
        let mut data: Vec<u8> = Vec::with_capacity(buffer_size);
        data.resize(buffer_size, 0);
        send.send(SpareData{ data: data, result: Vec::new() }).unwrap();
    }
    return (send, recv);
}

pub fn init_task_queue(cpu_count: usize) -> (WriterDataSender, WriterDataReceiver) {
    mpsc::sync_channel(cpu_count)
}

pub fn reader_thread(
    input: &mut dyn Read,
    inp_que: SpareDataQueueReceiver,
    out_que: WriterDataSender,
    pool: ThreadPool,
    compress_level: u32  // TODO compressor factory instead
) -> (SpareDataQueueReceiver, Result<(), ApplicationError>) {
    let loop_result = the_loop(
        input, &inp_que, &out_que, &pool, compress_level
    );
    (inp_que, loop_result)
}

fn the_loop(
    input: &mut dyn Read,
    inp_que: &SpareDataQueueReceiver,
    out_que: &WriterDataSender,
    pool: &ThreadPool,
    compress_level: u32
) -> Result<(), ApplicationError> {
    // This flag prevents from sending final empty chunk if file is not empty
    let mut has_any_data = false;

    loop {
        match inp_que.recv().map_err(
            |e| ApplicationError::MpscRecvError(e)) {
            Ok(mut buf) => {
                let length = input.read(&mut buf.data).map_err(|e| ApplicationError::IOError(e))?;
                if length == 0 && has_any_data {
                    out_que.send(WriterData::Eof).unwrap();
                    return Ok(())
                } else {
                    let result = buf.result;
                    let task = CompressTask::new(buf.data, length, result);
                    let comp_result = Arc::new(
                        CompressFuture::new()
                    );
                    let comp_result1 = comp_result.clone();
                    pool.execute(move || {
                        compress_data(task, comp_result1, compress_level);
                    });
                    has_any_data = true;
                    out_que.send(WriterData::Data(comp_result))
                        .or(Err(ApplicationError::MpscSendError))?;
                }
            },
            Err(e) => {
                // TODO: deside who handles e: out_que receiver or by caller of this func
                out_que.send(WriterData::Error(
                    ApplicationError::ErrorElsewhere)).unwrap();
                return Err(e);
                
            }
        }
    }
}
