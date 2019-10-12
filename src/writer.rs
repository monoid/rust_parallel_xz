use super::decl::*;
use std::io::Write;


pub fn writer_loop<W: Write>(
    output: &mut W,
    inp_que: WriterDataReceiver,
    out_que: SpareDataQueueSender
) -> Result<(), ApplicationError> {
    loop {
        let task = inp_que.recv().unwrap();
        match task {
            WriterData::Data(task) => {
                let (buf, result) = task.wait().or(Err(ApplicationError::MutexError))?;
                output.write(result.as_slice()).or_else(
                    |e| Err(ApplicationError::IOError(e))
                )?;
                output.flush().or_else(|e| Err(ApplicationError::IOError(e)))?;
                out_que.send(SpareData{ data: buf, result: result }).or(Err(ApplicationError::MpscSendError))?;
            },
            WriterData::Eof => break Ok(()),
            WriterData::Error(e) => break Err(e),
        }
    }
}
    
