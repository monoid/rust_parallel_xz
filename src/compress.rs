use super::decl::*;

use std::sync::Arc;
use std::io::{Write, Error};
use xz2::write::XzEncoder;


/* Execute CompressTask. */
pub fn compress_data(task: CompressTask, comp_result: Arc<CompressResult>, level: u32) -> Result<(), Error> {
    let data = task.data;
    let mut result = task.result;
    result.clear();
    let mut enc = XzEncoder::new(result, level);
    let (bytes, _) = data.as_slice().split_at(task.length);
    enc.write(bytes).unwrap();
    enc.finish().map(|result| {
        comp_result.notify(data, result);
        ()
    })
}
