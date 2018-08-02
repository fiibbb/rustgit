extern crate flate2;

use self::flate2::read::ZlibDecoder;

use object;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::Path;


fn is_valid_obj_sub_dir(dir: &Path) -> bool {
    return true
}

pub struct Repo {
    raw_objs: HashMap<String,Vec<u8>>
}

impl Repo {
    pub fn new() -> Repo {
        Repo {
            raw_objs: HashMap::new()
        }
    }

    pub fn load(&mut self, dir: &Path) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let sub_dir = entry?.path();
                let sub_dir = sub_dir.as_path();
                if sub_dir.is_dir() && is_valid_obj_sub_dir(&sub_dir) {
                    for entry in fs::read_dir(sub_dir)? {
                        let file_path = entry?.path();
                        let file_path = file_path.as_path();
                        if file_path.is_file() {
                            let file_bytes = fs::read(file_path)?;
                            let file_name = match file_path.iter().last() {
                                Some(f_oss) => {
                                    match f_oss.to_str() {
                                        Some(f) => f,
                                        None => panic!("wtf")
                                    }
                                },
                                None => panic!("wtf")
                            };
                            let mut decoder = ZlibDecoder::new(&file_bytes[..]);
                            let mut decoded: Vec<u8> = Vec::new();
                            decoder.read_to_end(&mut decoded).unwrap();
                            let obj = object::parse_object(&decoded).unwrap();
                            let sha = object::Hash::from(&decoded).hex_string();
                            // println!("{:?} {:?}", sha, obj);
                            self.raw_objs.insert(file_name.to_string(), decoded);
                        }
                    }
                }
            }
        }
        return Ok(())
    }
}