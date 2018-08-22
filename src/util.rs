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

pub fn be_u32(v: &[u8]) -> u32 {
    ((v[0] as u32) << 24) | ((v[1] as u32) << 16) | ((v[2] as u32) << 8) | ((v[3] as u32))
}

pub fn var_u32_tail(v: &[u8]) -> (u32, &[u8]) {
    let mut res = (v[0] & 0x7f) as u32;
    let mut i = 0;
    while v[i] & 0x80 != 0 {
        i += 1;
        res |= ((v[i] & 0x7f) as u32) << (7 * i);
    }
    (res, &v[i+1..])
}

pub fn varint(v: &[u8]) -> (Vec<u8>, &[u8]) {
    let mut res = Vec::new();
    let mut i = 0;
    while v[i] & 0x80 != 0 {
        i += 1;
        res.push(v[i] & 0x7f);
    }
    (res, &v[i+1..])
}

pub fn var_u32(v: &[u8]) -> u32 {
    let mut res = 0 as u32;
    let mut i = 0;
    while i < v.len() {
        res |= (v[i] as u32) << (7 * i);
    }
    res
}