extern crate hex;
extern crate sha1;

use util::*;

use std::fmt;
use std::str::from_utf8;


#[derive(PartialEq, Eq, Hash)]
pub struct Hash {
    hash: [u8;20],
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hash {{ {} }}", self.hex())
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

#[derive(Debug)]
pub struct Blob {
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
    children: Vec<TreeEntry>,
}

#[derive(Debug)]
pub struct Record {
    name: String,
    email: String,
    time: u64,
    time_zone: String,
}

#[derive(Debug)]
pub struct Commit {
    tree: Hash,
    parents: Vec<Hash>,
    author: Record,
    committer: Record,
    message: String,
}

impl Hash {
    pub fn sum(data: &[u8]) -> Hash {
        Hash{hash:sha1::Sha1::from(data).digest().bytes()}
    }
    pub fn from(v: &[u8]) -> Result<Hash, String> {
        let mut hash: [u8;20] = [0;20];
        if v.len() == 20 {
            hash.copy_from_slice(&v[..20]);
            Ok(Hash{hash})
        } else {
            Err(format!("unexpected length: {} != 20", v.len()))
        }
    }
    pub fn from_hex(v: &[u8]) -> Result<Hash, String> {
        utf8(v).and_then(|s| hex::decode(s).map_err(|e| e.to_string()).and_then(|bs| Hash::from(&bs)))
    }
    pub fn hex(&self) -> String {
        let s: Vec<String> = self.hash.iter().map(|b| format!("{:02x}", b)).collect();
        s.join("")
    }
}

pub trait Object {
    fn pack(&self) -> Vec<u8>;
    fn hash(&self) -> Hash {
        Hash::sum(&self.pack())
    }
    fn encode(&self) -> Result<Vec<u8>, String>{
        encode(&self.pack())
    }
}

impl Object for Blob {
    fn pack(&self) -> Vec<u8> {
        let mut res = format!("blob {}\0", self.data.len()).as_bytes().to_vec();
        res.append(&mut self.data.clone());
        res
    }
}

impl Object for Tree {
    fn pack(&self) -> Vec<u8> {
        let mut body = Vec::<u8>::new();
        self.children.iter().for_each(|c| {
            body.append(&mut c.mode.as_bytes().to_vec());
            body.push(b' ');
            body.append(&mut c.name.as_bytes().to_vec());
            body.push(0u8);
            body.append(&mut c.hash.hash.to_vec());
        });
        let mut res = format!("tree {}\0", body.len()).as_bytes().to_vec();
        res.append(&mut body);
        res
    }
}

impl Object for Commit {
    fn pack(&self) -> Vec<u8> {
        let mut body = Vec::<u8>::new();
        body.append(&mut format!("tree {}\n", self.tree.hex()).as_bytes().to_vec());
        self.parents.iter().for_each(|p| {
            body.append(&mut format!("parent {}\n", p.hex()).as_bytes().to_vec())
        });
        body.append(&mut format!("author {}\n", self.author.pack_string()).as_bytes().to_vec());
        body.append(&mut format!("committer {}\n", self.committer.pack_string()).as_bytes().to_vec());
        body.append(&mut self.message.as_bytes().to_vec());
        let mut res = format!("commit {}\0", body.len()).as_bytes().to_vec();
        res.append(&mut body);
        res
    }
}

impl Record {
    fn parse(v: &[u8]) -> Result<Record, String> {
        v.iter().position(|&b| b == b'<').ok_or(String::from("unable to locate <")).and_then(|l1| {
            v.iter().position(|&b| b == b'>').ok_or(String::from("unable to locate >")).and_then(|l2| {
                utf8(&v[..l1-1]).and_then(|name| {
                    utf8(&v[l1+1..l2]).and_then(|email| {
                        v[l2+2..].iter().position(|&b| b == b' ').ok_or(String::from("unable to locate space")).and_then(|l3_offset| {
                            let l3 = l3_offset+l2+2;
                            utf8(&v[l2+2..l3]).and_then(|ts_str| {
                                ts_str.parse::<u64>().map_err(|e| e.to_string()).and_then(|ts| {
                                    utf8(&v[l3+1..]).map(|tz| {
                                        Record {
                                            name: name.to_string(),
                                            email: email.to_string(),
                                            time: ts,
                                            time_zone: tz.to_string(),
                                        }
                                    })
                                })
                            })
                        })
                    })
                })
            })
        })
    }
    fn pack_string(&self) -> String {
        format!("{} <{}> {} {}", self.name, self.email, self.time, self.time_zone)
    }
}

impl Blob {
    fn parse(body: &[u8]) -> Result<Blob, String> {
        Ok(Blob{data: body.to_vec()})
    }
}

impl Tree {
    fn parse(body: &[u8]) -> Result<Tree, String> {
        let mut tree_entries: Vec<TreeEntry> = Vec::new();
        let mut tail: &[u8] = body;
        while tail.len() > 0 {
            let entry_res = tail.iter().position(|&b| b == 0u8).ok_or(String::from("unable to locate 0u8")).map(|l| (&tail[..l+21], &tail[l+21..])).and_then(|(h,t)| {
                h.iter().position(|&b| b == b' ').ok_or(String::from("unable to locate space")).and_then(|l1| {
                    h.iter().position(|&b| b == 0u8).ok_or(String::from("unable to locate 0u8")).and_then(|l2| {
                        utf8(&h[..l1]).and_then(|mode| {
                            utf8(&h[l1+1..l2]).and_then(|name| {
                                Hash::from(&h[l2+1..l2+21]).map(|hash| {
                                    (TreeEntry{
                                        mode: mode.to_string(),
                                        name: name.to_string(),
                                        hash: hash,
                                    }, t)
                                })
                            })
                        })
                    })
                })
            });
            match entry_res {
                Ok((entry, t)) => {
                    tree_entries.push(entry);
                    tail = t;
                }
                Err(e) => return Err(e)
            }
        };
        Ok(Tree{children:tree_entries})
    }
}

impl Commit {
    fn parse(body: &[u8]) -> Result<Commit, String> {
        let mut tree_opt: Option<Hash> = None;
        let mut author_opt: Option<Record> = None;
        let mut committer_opt: Option<Record> = None;
        let mut parents: Vec<Hash> = Vec::new();
        let mut msg: String = String::new();
        let mut tail: &[u8] = body;
        while tail.len() > 0 {
            if tail.len() >= 7 && tail[..6] == *b"author" {
                if let None = author_opt {
                    let record_res = tail.iter().position(|&b| b == 10u8).ok_or(String::from("unable to locate 10u8")).and_then(|end_idx| {
                        Record::parse(&tail[7..end_idx]).map(|r| (r, end_idx))
                    });
                    match record_res {
                        Ok((record, end_idx)) => {
                            author_opt = Some(record);
                            tail = &tail[end_idx+1..];
                        }
                        Err(e) => return Err(e)
                    }
                } else {
                    return Err(String::from("duplicate author entry in commit"));
                }
            } else if tail.len() >= 10 && tail[..9] == *b"committer" {
                if let None = committer_opt {
                    let record_res = tail.iter().position(|&b| b == 10u8).ok_or(String::from("unable to locate 10u8")).and_then(|end_idx| {
                        Record::parse(&tail[10..end_idx]).map(|r| (r, end_idx))
                    });
                    match record_res {
                        Ok((record, end_idx)) => {
                            committer_opt = Some(record);
                            tail = &tail[end_idx+1..];
                        }
                        Err(e) => return Err(e)
                    }
                } else {
                    return Err(String::from("duplicate committer entry in commit"));
                }
            } else if tail.len() >= 46 && tail[..4] == *b"tree" {
                if let None = tree_opt {
                    match Hash::from_hex(&tail[5..45]) {
                        Ok(hash) => {
                            tree_opt = Some(hash);
                            tail = &tail[46..];
                        }
                        Err(e) => return Err(e)
                    }
                } else {
                    return Err(String::from("duplicate tree entry in commit"));
                }
            } else if tail.len() >= 48 && tail[..6] == *b"parent" {
                match Hash::from_hex(&tail[7..47]) {
                    Ok(hash) => {
                        parents.push(hash);
                        tail = &tail[48..];
                    }
                    Err(e) => return Err(e)
                }
            } else {
                match from_utf8(tail) {
                    Ok(msg_str) => {
                        msg = msg_str.to_string();
                        tail = &[];
                    }
                    Err(e) => return Err(e.to_string())
                }
            }
        }
        tree_opt.ok_or(String::from("missing tree")).and_then(|t| {
            author_opt.ok_or(String::from("missing author")).and_then(|a| {
                committer_opt.ok_or(String::from("missing committer")).map(|c| {
                    Commit{
                        tree:t,
                        parents: parents,
                        author: a,
                        committer: c,
                        message: msg,
                    }
                })
            })
        })
    }
}

impl Header {
    fn parse(raw_header: &[u8]) -> Result<Header, String> {
        let parse = |x, t| {
            utf8(&raw_header[x..]).and_then(|size_string| {
                size_string.parse::<usize>().map_err(|e| e.to_string()).map(|size_usize| {
                    Header{typp:t, size:size_usize}
                })
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
}

fn parse_object(obj: &[u8]) -> Result<Box<Object>, String> {
    obj.iter().position(|&b| b == 0u8).ok_or(String::from("unable to find 0u8")).and_then(|l| {
        Header::parse(&obj[..l]).and_then(|h| {
            let rb = &obj[l+1..];
            match h.typp {
                Type::Blob => Blob::parse(&rb).map(|b| Box::new(b) as Box<Object>),
                Type::Tree => Tree::parse(&rb).map(|t| Box::new(t) as Box<Object>),
                Type::Commit => Commit::parse(&rb).map(|c| Box::new(c) as Box<Object>),
            }
        })
    })
}

pub fn deflate(compressed: &[u8]) -> Result<Box<Object>, String> {
    decode(compressed).and_then(|v| parse_object(&v))
}