#![forbid(unsafe_code)]

use super::decl::*;

use std::io::Write;
use std::sync::Arc;
use xz2::write::XzEncoder;

pub fn compress_data(task: CompressTask, comp_result: Arc<CompressFuture>, level: u32) {
    let data = task.data;
    let mut result = task.result;
    result.clear();

    let mut enc = XzEncoder::new(result, level);
    let bytes = &data[..task.length];

    comp_result.notify(
        enc.write(bytes)
            .and_then(|_| enc.finish())
            .map(|result| (data, result))
            .map_err(ApplicationError::IOError),
    );
}

#[cfg(test)]
mod test {
    use super::super::decl::*;
    use super::compress_data;
    use std::sync::Arc;

    #[test]
    fn test_empty() {
        let comp_task = CompressTask {
            data: Vec::new(),
            length: 0,
            result: Vec::new(),
        };
        let comp_result = Arc::new(CompressFuture::new());

        compress_data(comp_task, comp_result.clone(), 3);

        let res = comp_result.wait();
        assert!(res.is_ok());
        let (_buf, data) = res.unwrap();
        assert_eq!(data.len(), 32); // Size of compressed empty chunk
    }

    #[test]
    fn test_data() {
        let comp_task = CompressTask {
            data: vec![0, 1, 2, 3, 4],
            length: 5,
            result: Vec::new(),
        };
        let comp_result = Arc::new(CompressFuture::new());

        compress_data(comp_task, comp_result.clone(), 3);

        let res = comp_result.wait();
        assert!(res.is_ok());
        let (_buf, data) = res.unwrap();
        assert_eq!(data.len(), 64);
    }
}
