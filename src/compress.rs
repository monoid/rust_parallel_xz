use super::decl::*;

use std::sync::Arc;
use std::io::{Write};
use xz2::write::XzEncoder;


/* Execute CompressTask. */
pub fn compress_data(task: CompressTask, comp_result: Arc<CompressResult>, level: u32) {
    let data = task.data;
    let mut result = task.result;
    result.clear();
    let mut enc = XzEncoder::new(result, level);
    let (bytes, _) = data.as_slice().split_at(task.length);
    let enc_result = enc.write(bytes).and_then(
        |_| enc.finish()
    );
    match enc_result {
        Ok(result) => comp_result.notify(data, result),
        Err(err) => comp_result.notify_error(ApplicationError::IOError(err))
    }
}
