#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]

mod compress;
mod decl;
mod reader;
mod writer;

use clap::Parser;
use std::io;
use std::thread;
use threadpool::ThreadPool;

const DEFAULT_COMPRESS_LEVEL: u32 = 3;
const DEFAULT_BUFFER_SIZE_MB: usize = 1;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Number of compression threads [default: number of CPUs + 1]
    #[arg(short, long)]
    threads_num: Option<usize>,
    /// Buffer size for each thread in megabytes
    #[arg(short, long, default_value_t = DEFAULT_BUFFER_SIZE_MB)]
    buffer_size: usize,
    // TODO a parser that accepts only valid values
    /// XZ compression level
    #[arg(short, long, default_value_t = DEFAULT_COMPRESS_LEVEL)]
    compress_level: u32,
}

fn main() {
    let args = Args::parse();

    let compress_level = args.compress_level;
    let nthread = args.threads_num.unwrap_or_else(|| num_cpus::get() + 1);
    let buffer_size = 1024 * 1024 * args.buffer_size;

    let (free_recvr, free_sendr) = reader::init_spares_queue(
        buffer_size,
        // nthreads + 1 for reader, and 1 for writer, the queue
        // is filled only at its start position and at end.
        nthread + 2,
    );
    let (task_recvr, task_sendr) = reader::init_task_queue(nthread + 1);

    let pool = ThreadPool::new(nthread);

    let writer =
        thread::spawn(move || writer::writer_loop(&mut io::stdout(), task_sendr, free_recvr));

    // Keep reference to channel before writer is complete
    let (_task_recvr, reader_result) = reader::reader_loop(
        &mut io::stdin(),
        free_sendr,
        task_recvr,
        pool,
        compress_level,
    );
    reader_result.expect("OOPS");

    writer.join().expect("OOPS").expect("OOPS");
}
