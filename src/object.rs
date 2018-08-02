extern crate sha1;

use std::str::from_utf8;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Hash {
    hash: [u8;20]
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
pub trait Object {
    fn hash(&self) -> Hash;
}

#[derive(Debug)]
pub struct Blob {
    raw: Vec<u8>,
    size: usize,
    data: Vec<u8>,
}

#[derive(Debug)]
pub struct TreeEntry {
    mode: String,
    name: String,
    hash: Hash,
}

#[derive(Debug)]
pub struct Tree {
    raw: Vec<u8>,
    size: usize,
    children: Vec<TreeEntry>,
}

#[derive(Debug)]
pub struct Commit {
    raw: Vec<u8>,
    size: usize,
    tree: Hash,
    parents: Vec<Hash>,
}

impl Hash {
    pub fn from(data: &Vec<u8>) -> Hash {
        Hash {
            hash: sha1::Sha1::from(data).digest().bytes()
        }
    }
    pub fn hex_string(&self) -> String {
        let s: Vec<String> = self.hash.iter().map(|b| format!("{:02x}", b)).collect();
        s.join("")
    }
}

impl Object for Blob {
    fn hash(&self) -> Hash {
        Hash::from(&self.raw)
    }
}

impl Object for Tree {
    fn hash(&self) -> Hash {
        Hash::from(&self.raw)
    }
}

impl Object for Commit {
    fn hash(&self) -> Hash {
        Hash::from(&self.raw)
    }
}

fn decode_hex_str(s: &str) -> [u8;20] {
    // TODO: fix this
    // This should be turnning a 40 byte string into a 20 byte array.
    [0;20]
}

fn parse_blob(raw: &Vec<u8>, header: &Header, body: &Vec<u8>) -> Result<Blob, String> {
    Ok(Blob{
        raw: raw.clone(),
        size: header.size,
        data: body.clone(),
    })
}

fn parse_tree(raw: &Vec<u8>, header: &Header, body: &Vec<u8>) -> Result<Tree, String> {
    let next_raw_tree_entry = |tail: Vec<u8>| -> Option<(Vec<u8>, Vec<u8>)> {
        tail.iter().position(|&b| b == 0u8).map(|l| (tail[..l+40].to_vec(), tail[l+40..].to_vec()))
    };
    let mut raw_tree_entries: Vec<Vec<u8>> = Vec::new();
    let mut tail: Vec<u8> = body.clone();
    loop {
        match next_raw_tree_entry(tail) {
            Some((h, t)) => {
                raw_tree_entries.push(h);
                tail = t;
            },
            None => break,
        }
    };
    let tree_entries: Vec<TreeEntry> = raw_tree_entries.iter().map(|re| {
        re.iter().position(|&b| {
            b == b' '
        }).and_then(|l1| {
            re.iter().position(|&b| {
                b == 0u8
            }).and_then(|l2| {
                from_utf8(&re[..l1]).ok().and_then(|mode| {
                    from_utf8(&re[l1+1..l2]).ok().and_then(|name| {
                        from_utf8(&re[l2+1..l2+41]).ok().map(|hash_str| {
                            let hash = decode_hex_str(hash_str);
                            TreeEntry{
                                mode: mode.to_string(),
                                name: name.to_string(),
                                hash: Hash{hash},
                            }
                        })
                    })
                })
            })
        })
    }).filter_map(|o| o).collect();
    if tree_entries.len() != raw_tree_entries.len() {
        return Err(String::from("failed to parse at least one tree entry"));
    } else {
        return Ok(Tree{
            raw: raw.clone(),
            size: header.size,
            children: tree_entries,
        });
    }
}

fn parse_commit(raw: &Vec<u8>, header: &Header, body: &Vec<u8>) -> Result<Commit, String> {
    Err(String::from("NYI"))
}

fn parse_header(raw_header: &Vec<u8>) -> Result<Header, String> {
    let parse = |x, t| {
        from_utf8(&raw_header[x..]).map_err(|e| {
            e.to_string()
        }).and_then(|size_string| {
            size_string.parse::<usize>().map_err(|e| e.to_string())
        }).map(|size_usize| {
            Header{
                typp: t,
                size: size_usize,
            }
        })
    };
    if raw_header[0..4] == *b"tree" {
        parse(5, Type::Tree)
    } else if raw_header[0..4] == *b"blob" {
        parse(5, Type::Blob)
    } else if raw_header[0..6] == *b"commit" {
        parse(7, Type::Commit)
    } else {
        Err(String::from("unrecognized header"))
    }
}

pub fn parse_object(raw: &Vec<u8>) -> Result<Box<Object>, String> {
    raw.iter().position(|&b| b == 0u8).map(|l| {
        let raw_header: Vec<u8> = raw.iter().take(l).map(|b| *b).collect();
        let raw_body: Vec<u8> = raw.iter().skip(l+1).map(|b| *b).collect();
        (raw_header, raw_body)
    }).and_then(|(rh, rb)| {
        parse_header(&rh).map(|h| (h, rb)).ok()
    }).and_then(|(h, rb)| {
        let obj_opt: Option<Box<Object>> = match h.typp {
            Type::Blob => parse_blob(raw, &h, &rb).map(|b| Box::new(b) as Box<Object>).ok(),
            Type::Tree => parse_tree(raw, &h, &rb).map(|t| Box::new(t) as Box<Object>).ok(),
            Type::Commit => parse_commit(raw, &h, &rb).map(|c| Box::new(c) as Box<Object>).ok(),
        };
        obj_opt
    }).ok_or(String::from("failed to parse object"))
}