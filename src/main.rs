mod decl;
mod reader;
mod writer;
mod compress;

use crate::decl::*;
use std::io;
use threadpool::ThreadPool;
use std::thread;


fn main() {
    let compress_level = 3;
    let nthread = DEFAULT_NTHREAD;
    
    let (free_recvr, free_sendr) = reader::init_free_data_queue(
        DEFAULT_BUFFER_SIZE,
        // nthreads + 1 for reader, and 1 for writer, the queue
        // is filled only at its start position and at end.
        nthread + 2
    );
    let (task_recvr, task_sendr) = reader::init_task_queue(nthread + 1);

    let pool = ThreadPool::new(nthread);

    let writer = thread::spawn(move || {
        writer::writer_loop(&mut io::stdout(), task_sendr, free_recvr)
    });

    // Keep reference to channel before writer is complete
    let _task_recvr = reader::reader_thread(
        &mut io::stdin(),
        free_sendr,
        task_recvr,
        pool,
        compress_level
    ).expect("OOPS");

    writer.join().expect("OOPS").expect("OOPS");
}
