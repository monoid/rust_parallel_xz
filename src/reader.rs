use super::decl::*;
use super::compress::compress_data;

use std::io::prelude::*;
use std::sync::mpsc;
use std::sync::Arc;

use threadpool::ThreadPool;


pub fn init_free_data_queue(buffer_size: usize, cpu_count: usize) -> (FreeDataQueueSender, FreeDataQueueReceiver) {
    let (send, recv) = mpsc::sync_channel(cpu_count);
    for _ in 0..cpu_count {
        let mut data: Vec<u8> = Vec::with_capacity(buffer_size);
        data.resize(buffer_size, 0);
        send.send(InputBuf{ data: data, result: Vec::new() }).unwrap();
    }
    return (send, recv);
}

pub fn init_task_queue(cpu_count: usize) -> (TaskSender, TaskReceiver) {
    mpsc::sync_channel(cpu_count)
}

pub fn reader_thread(
    input: &mut dyn Read,
    inp_que: FreeDataQueueReceiver,
    out_que: TaskSender,
    pool: ThreadPool,
    compress_level: u32  // TODO compressor factory instead
) -> Result<FreeDataQueueReceiver, ReaderThreadError> {
    // This flag prevents from sending final empty chunk if file is not empty
    let mut has_any_data = false;
    loop {
        let buf = inp_que.recv();
        match buf {
            Err(e) => { return Err(ReaderThreadError::MpscRecvError(e)) },
            Ok(mut buf) => {
                let result = input.read(&mut buf.data).or_else(|e| Err(ReaderThreadError::IOError(e))).and_then(|length| {
                    if length == 0 && has_any_data {
                        Ok(length)
                    } else {
                        let result = buf.result;
                        let task = CompressTask::new(buf.data, length, result);
                        let comp_result = Arc::new(
                            CompressResult::new()
                        );
                        let comp_result1 = comp_result.clone();
                        pool.execute(move || {
                            compress_data(task, comp_result1, compress_level).unwrap()
                        });
                        has_any_data = true;
                        out_que.send(CompressChunk::Data(comp_result))
                            .and(Ok(length))
                            .or_else(|e| Err(ReaderThreadError::MpscSendError(e)))
                    }
                });
                match result {
                    Ok(len) => { if len == 0 {
                        out_que.send(CompressChunk::Eof).unwrap();
                        return Ok(inp_que)
                    } }
                    Err(e) => { return Err(e); }
                };
            }
        }
    }
}
