extern crate flate2;

use std::io::prelude::*;
use std::str::from_utf8;

pub fn utf8(v: &[u8]) -> Result<String, String> {
    from_utf8(v).map_err(|e| e.to_string()).map(|s| s.to_string())
}

pub fn encode(v: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(v).map_err(|e| e.to_string()).and_then(|_| encoder.finish().map_err(|e| e.to_string()))
}

pub fn decode(v: &[u8]) -> Result<Vec<u8>, String> {
    let mut decompressed = Vec::new();
    flate2::read::ZlibDecoder::new(v).read_to_end(&mut decompressed).map_err(|e| e.to_string()).map(|_| decompressed)
}