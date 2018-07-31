extern crate sha1;

use std::str::from_utf8;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Hash {
    hash: [u8;20]
}

impl Hash {
    pub fn new(data: &Vec<u8>) -> Hash {
        Hash {
            hash: sha1::Sha1::from(data).digest().bytes()
        }
    }

    pub fn hex_string(&self) -> String {
        let s: Vec<String> = self.hash.iter().map(|b| format!("{:02x}", b)).collect();
        s.connect("")
    }
}

#[derive(Debug)]
pub enum Type {
    Blob,
    Tree,
    Commit,
}

#[derive(Debug)]
pub struct Header {
    typp: Type,
    size: usize,
}

pub fn get_header(data: &Vec<u8>) -> Result<Header, &str> {
    let raw_header: Vec<u8> = data.iter().take_while(|b| **b != 0u8).map(|b| *b).collect();
    if raw_header[0..4] == *b"tree" {
        let size = from_utf8(&raw_header[5..]).unwrap().parse::<usize>().unwrap();
        return Ok(Header {
            typp: Type::Tree,
            size: size,
        });
    } else if raw_header[0..4] == *b"blob" {
        let size = from_utf8(&raw_header[5..]).unwrap().parse::<usize>().unwrap();
        return Ok(Header {
            typp: Type::Blob,
            size: size,
        });
    } else if raw_header[0..6] == *b"commit" {
        let size = from_utf8(&raw_header[7..]).unwrap().parse::<usize>().unwrap();
        return Ok(Header {
            typp: Type::Commit,
            size: size,
        });
    } else {
        return Err("unrecognized header");
    }
}

pub trait Object {
    fn hash(&self) -> Hash;
}

pub struct Blob {
    raw: Vec<u8>,
    size: usize,
}

impl Blob {
    fn new(data: Vec<u8>) -> Blob {
        Blob {
            raw: data,
            size: 0,
        }
    }
}

impl Object for Blob {
    fn hash(&self) -> Hash {
        Hash::new(&self.raw)
    }
}

pub enum TreeEntryType {
    Blob,
    Tree,
}

pub struct TreeEntry {
    mode: String,
    typp: TreeEntryType,
    hash: Hash,
    name: String,
}

pub struct Tree {
    raw: Vec<u8>,
    size: usize,
    entries: Vec<TreeEntry>,
}

impl Tree {
    fn new(data: Vec<u8>) -> Tree {
        Tree {
            raw: data,
            size: 0,
            entries: Vec::new(),
        }
    }
}

impl Object for Tree {
    fn hash(&self) -> Hash {
        Hash::new(&self.raw)
    }
}

pub struct Commit {
    raw: Vec<u8>,
    size: usize,
}

impl Commit {
    fn new(data: Vec<u8>) -> Commit {
        Commit {
            raw: data,
            size: 0,
        }
    }
}

impl Object for Commit {
    fn hash(&self) -> Hash {
        Hash::new(&self.raw)
    }
}