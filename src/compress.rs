use super::decl::*;

use std::sync::Arc;
use std::io::{Write};
use xz2::write::XzEncoder;


pub fn compress_data(task: CompressTask, comp_result: Arc<CompressFuture>, level: u32) {
    let data = task.data;
    let mut result = task.result;
    result.clear();
    
    let mut enc = XzEncoder::new(result, level);
    let (bytes, _) = data.as_slice().split_at(task.length);

    comp_result.notify(
        enc.write(bytes)
            .and_then(|_| enc.finish())
            .and_then(|result| Ok((data, result)))
            .map_err(|e| ApplicationError::IOError(e))
    );
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty() {
    }

    #[test]
    fn test_data() {
    }
}
