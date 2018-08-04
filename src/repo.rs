use object;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;


fn is_valid_obj_sub_dir(dir: &Path) -> bool {
    return true
}

pub struct Repo {
    objs: HashMap<object::Hash,Box<object::Object>>
}

impl Repo {
    pub fn new() -> Repo {
        Repo {objs: HashMap::new()}
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
                            println!("parsing object {:?}", file_path);
                            let file_bytes = fs::read(file_path)?;
                            match object::deflate(&file_bytes) {
                                Ok(obj) => {
                                    self.objs.insert(obj.hash(), obj);
                                    ()
                                }
                                Err(e) => {println!("{:?}", e);()},
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn save(&self, dir: &Path) -> Result<(), String> {
        if !dir.is_dir() {
            return Err(format!("not a directory: {:?}", dir));
        }
        self.objs.iter().for_each(|(hash, obj)| {
            let file_path = dir.join(hash.hex());
            let file_content = obj.encode().unwrap();
            fs::write(file_path.as_path(), file_content).unwrap();
        });
        Ok(())
    }
}