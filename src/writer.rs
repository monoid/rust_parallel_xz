use super::decl::*;
use std::io::Write;


pub fn writer_loop(
    output: &mut dyn Write,
    inp_que: TaskReceiver,
    out_que: FreeDataQueueSender) {
    loop {
        let task = inp_que.recv().unwrap();
        match task {
            CompressChunk::Eof => break,
            CompressChunk::Data(task) => {
                let (buf, result) = task.wait();
                output.write(result.as_slice()).unwrap();
                output.flush().unwrap();
                out_que.send(InputBuf{ data: buf, result: result }).unwrap();
            }
        }
    }
}
    
