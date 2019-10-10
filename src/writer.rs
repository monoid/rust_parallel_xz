use super::decl::*;
use std::io::Write;


pub fn writer_loop(
    output: &mut dyn Write,
    inp_que: TaskReceiver,
    out_que: FreeDataQueueSender
) -> Result<(), ApplicationError> {
    loop {
        let task = inp_que.recv().unwrap();
        match task {
            CompressChunk::Data(task) => {
                let write_result = task.wait().or(Err(ApplicationError::MutexError)
                      ).and_then(|(buf, result)|
                        output.write(result.as_slice()).or_else(
                           |e| Err(ApplicationError::IOError(e))
                        ).and_then(|_|
                            output.flush().or_else(|e| Err(ApplicationError::IOError(e)))
                        ).and_then(|_|
                            out_que.send(InputBuf{ data: buf, result: result }).or(Err(ApplicationError::MpscSendError))
                        )
                    );
                match write_result {
                    Err(e) => break Err(e),
                    _ => {}
                }
            },
            CompressChunk::Eof => break Ok(()),
            CompressChunk::Error(e) => break Err(e),
        }
    }
}
    
