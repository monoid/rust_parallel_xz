#![forbid(unsafe_code)]

use super::decl::*;
use std::io::Write;

pub fn writer_loop<W: Write>(
    output: &mut W,
    inp_que: WriterDataReceiver,
    out_que: SpareDataQueueSender,
) -> Result<(), ApplicationError> {
    for task in inp_que.iter() {
        let (buf, result) = task.wait().or(Err(ApplicationError::MutexError))?;
        output
            .write(&result[..])
            .map_err(|e| ApplicationError::IOError(e))?;
        output.flush().map_err(|e| ApplicationError::IOError(e))?;
        out_que
            .send(SpareData {
                data: buf,
                result: result,
            })
            .or(Err(ApplicationError::MpscSendError))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::decl::*;
    use super::writer_loop;
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_empty() {
        let (writer_thread, _spare_reader) = {
            let (spare_writer, spare_reader) = mpsc::sync_channel(4);
            let (_task_writer, task_reader) = mpsc::sync_channel(4);

            let writer_thread = thread::spawn(move || {
                let mut output = Vec::<u8>::new();
                writer_loop(&mut output, task_reader, spare_writer).unwrap();
                output
            });
            (writer_thread, spare_reader)
        };
        let data = writer_thread.join().unwrap();
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn test_normal() {
        let (writer_thread, _spare_reader) = {
            let (spare_writer, spare_reader) = mpsc::sync_channel(4);
            let (task_writer, task_reader) = mpsc::sync_channel(4);

            let writer_thread: std::thread::JoinHandle<
                std::result::Result<std::vec::Vec<u8>, ApplicationError>,
            > = thread::spawn(move || {
                let mut output = Vec::<u8>::new();
                writer_loop(&mut output, task_reader, spare_writer)?;
                Ok(output)
            });

            let task = Arc::new(CompressFuture::new());
            task.notify(Ok((Vec::new(), vec![0, 1, 2, 3, 4, 5])));
            task_writer.send(task.clone()).unwrap();
            (writer_thread, spare_reader)
        };
        let data = writer_thread.join().unwrap().unwrap();
        assert_eq!(data.len(), 6);
    }

    #[test]
    fn test_write_fail() {}

    #[test]
    fn test_flush_fail() {}
}
