use super::decl::*;
use std::io::Write;


pub fn writer_loop<W: Write>(
    output: &mut W,
    inp_que: TaskReceiver,
    out_que: FreeDataQueueSender
) -> Result<(), ApplicationError> {
    loop {
        let task = inp_que.recv().unwrap();
        match task {
            CompressChunk::Data(task) => {
                let (buf, result) = task.wait().or(Err(ApplicationError::MutexError))?;
                output.write(result.as_slice()).or_else(
                    |e| Err(ApplicationError::IOError(e))
                )?;
                output.flush().or_else(|e| Err(ApplicationError::IOError(e)))?;
                out_que.send(InputBuf{ data: buf, result: result }).or(Err(ApplicationError::MpscSendError))?;
            },
            CompressChunk::Eof => break Ok(()),
            CompressChunk::Error(e) => break Err(e),
        }
    }
}
    
