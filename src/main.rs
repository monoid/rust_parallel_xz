mod decl;
mod reader;
mod writer;
mod compress;

use std::io;
use threadpool::ThreadPool;
use std::thread;
use clap::{Arg, App};

const DEFAULT_COMPRESS_LEVEL: u32 = 3;
const DEFAULT_BUFFER_SIZE : usize = 1 << 20;


fn main() {
    let matches = App::new("Parallel XZ compressor")
        .version("0.1.0")
        .author("Ivan Boldyrev <lispnik@gmail.com>")
        .arg(Arg::with_name("threads_num")
             .short("t")
             .long("threads-num")
             .takes_value(true)
             .help("Compression threads number (default: number of CPUs + 1)"))
        .arg(Arg::with_name("buffer_size")
             .short("b")
             .long("buffer-size")
             .takes_value(true)
             .help("Buffer size for each thread in megabytes (default: 1)"))
        .arg(Arg::with_name("compress_level")
             .short("c")
             .long("compress-level")
             .takes_value(true)
             .help("XZ compression level (default: 3)"))
        .get_matches();

    let compress_level = match matches.value_of("compress_level") {
        None => DEFAULT_COMPRESS_LEVEL,
        Some(s) => s.parse::<u32>().expect("Malformed compress-level")
    };
    let nthread = match matches.value_of("threads_num") {
        None => num_cpus::get() + 1,
        Some(s) => s.parse::<usize>().expect("Malformed threads-num")
    };
    let buffer_size = match matches.value_of("buffer_size") {
        None => DEFAULT_BUFFER_SIZE,
        Some(s) => 1024 * 1024 * s.parse::<usize>().expect("Malformed buffer-size")
    };

    
    let (free_recvr, free_sendr) = reader::init_spare_queue(
        buffer_size,
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
    let (_task_recvr, reader_result) = reader::reader_loop(
        &mut io::stdin(),
        free_sendr,
        task_recvr,
        pool,
        compress_level
    );
    reader_result.expect("OOPS");

    writer.join().expect("OOPS").expect("OOPS");
}
