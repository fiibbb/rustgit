use util::*;
use object::*;

use std::collections::HashMap;

const HSIZE: usize = 8; // header last bit
const FSIZE: usize = 1032; // fanout table last bit, 1024 + HSIZE

#[derive(Debug)]
pub struct Pack {
    index: Vec<u8>,
    pack: Vec<u8>,
}

impl Pack {
    pub fn new(index: Vec<u8>, pack: Vec<u8>) -> Pack {
        Pack{index, pack}
    }
    pub fn get_fanout_i(&self, i: u8) -> u32 {
        be_u32(&self.index[HSIZE+(i as usize)*4..HSIZE+((i as usize)+1)*4])
    }

    pub fn get_obj_count(&self) -> u32 {
        be_u32(&self.index[FSIZE-HSIZE..FSIZE])
    }

    fn get_obj_offset(&self, hash: Hash) -> Result<u32, String> {
        Err(String::from("NYI"))
    }

    pub fn get_obj(&self, hash: Hash) -> Result<Box<Object>, String> {
        Err(String::from("NYI"))
    }
}

enum ObjType {
    OBJ_COMMIT,
    OBJ_TREE,
    OBJ_BLOB,
    OBJ_TAG,
    OBJ_OFS_DELTA,
    OBJ_REF_DELTA,
}

fn get_pack_obj_type(b: u8) -> ObjType {
    match (b >> 4) & 0xf {
        1 => ObjType::OBJ_COMMIT,
        2 => ObjType::OBJ_TREE,
        3 => ObjType::OBJ_BLOB,
        4 => ObjType::OBJ_TAG,
        6 => ObjType::OBJ_OFS_DELTA,
        7 => ObjType::OBJ_REF_DELTA,
        _ => panic!("WTF")
    }
}

fn get_pack_obj_size(v: &[u8]) -> u32 {
    let mut offset = 0;
    let mut result = 0 as u32;
    result |= (v[0] & 0xf) as u32;
    let mut i = 0;
    while v[i] & 0x80 != 0 {
        i += 1;
        result |= (v[i] & 0x7f) as u32;
    }
    result
}

pub fn parse_pack(index: &[u8], pack: &[u8]) -> Result<Vec<Box<Object>>, String> {
    // validation
    if be_u32(&index[4..8]) != 2 {
        return Err(format!("unexpected version number {}", be_u32(&index[4..8])));
    }
    if be_u32(&pack[4..8]) != 2 {
        return Err(format!("unexpected version number {}", be_u32(&index[4..8])));
    }
    if pack.len() > 1024 * 1024 * 1024 * 2 {
        return Err(String::from("large pack file not supported"));
    }
    let index_obj_count = be_u32(&index[1028..1032]) as usize;
    let pack_obj_count = be_u32(&pack[8..12]) as usize;
    if index_obj_count != pack_obj_count {
        return Err(format!("unmatched object count {} != {}", index_obj_count, pack_obj_count));
    }
    // parse index
    let mut hash2offset: HashMap<Hash, u32> = HashMap::new();
    let offset_table_start = 1032 + index_obj_count * 20 + index_obj_count * 4;
    for i in 0..index_obj_count {
        let (start, end) = (1032+i*20, 1032+(i+1)*20);
        let res = Hash::from(&index[start..end]).map(|hash| {
            let offset = be_u32(&index[i*4+offset_table_start..(i+1)*4+offset_table_start]);
            hash2offset.insert(hash, offset);
        });
        if let Err(e) = res {
            return Err(e);
        }
    }
    // parse pack

    println!("obj_count: {}", index_obj_count);
    Err(String::from("NYI"))
}