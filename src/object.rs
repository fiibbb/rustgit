extern crate flate2;
extern crate hex;
extern crate sha1;

use std::fmt;
use std::io::Read;
use std::str::from_utf8;


#[derive(PartialEq, Eq, Hash)]
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
}

pub struct FullObject {
    header: Header,
    object: Box<Object>,
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
pub struct Commit {
    tree: Hash,
    parents: Vec<Hash>,
    author: String,
    committer: String,
    message: String,
}

impl Hash {
    pub fn sum(data: &[u8]) -> Hash {
        Hash {
            hash: sha1::Sha1::from(data).digest().bytes()
        }
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
        from_utf8(v).map_err(|e| e.to_string()).and_then(|s| hex::decode(s).map_err(|e| e.to_string()).and_then(|bs| Hash::from(&bs)))
    }
    pub fn hex(&self) -> String {
        let s: Vec<String> = self.hash.iter().map(|b| format!("{:02x}", b)).collect();
        s.join("")
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Hash {{ {:02x?} }}", self.hash)
    }
}

impl fmt::Debug for FullObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FullObject()")
    }
}

impl Object for Blob {
}

impl Object for Tree {
}

impl Object for Commit {
}

fn parse_blob(body: &[u8]) -> Result<Blob, String> {
    Ok(Blob{
        data: body.to_vec(),
    })
}

fn parse_tree(body: &[u8]) -> Result<Tree, String> {

    let mut tree_entries: Vec<TreeEntry> = Vec::new();
    let mut tail: &[u8] = body;

    while tail.len() > 0 {
        let entry_res = tail.iter().position(|&b| b == 0u8).ok_or(String::from("unable to locate 0u8")).map(|l| (&tail[..l+21], &tail[l+21..])).and_then(|(h,t)| {
            h.iter().position(|&b| b == b' ').ok_or(String::from("unable to locate space")).and_then(|l1| {
                h.iter().position(|&b| b == 0u8).ok_or(String::from("unable to locate 0u8")).and_then(|l2| {
                    from_utf8(&h[..l1]).map_err(|e| e.to_string()).and_then(|mode| {
                        from_utf8(&h[l1+1..l2]).map_err(|e| e.to_string()).and_then(|name| {
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

fn parse_commit(body: &[u8]) -> Result<Commit, String> {

    let mut tree_opt: Option<Hash> = None;
    let mut author_opt: Option<String> = None;
    let mut committer_opt: Option<String> = None;
    let mut parents: Vec<Hash> = Vec::new();
    let mut msg: String = String::new();
    let mut tail: &[u8] = body;

    while tail.len() > 0 {
        if tail.len() >= 7 && tail[..6] == *b"author" {
            if let None = author_opt {
                let author_res = tail.iter().position(|&b| b == 10u8).ok_or(String::from("unable to locate 10u8")).and_then(|end_idx| {
                    from_utf8(&tail[7..end_idx]).map_err(|e| e.to_string()).map(|a| (a, end_idx))
                });
                match author_res {
                    Ok((author, end_idx)) => {
                        author_opt = Some(author.to_string());
                        tail = &tail[end_idx+1..];
                    }
                    Err(e) => return Err(e)
                }
            } else {
                return Err(String::from("duplicate author entry in commit"));
            }
        } else if tail.len() >= 10 && tail[..9] == *b"committer" {
            if let None = committer_opt {
                let committer_res = tail.iter().position(|&b| b == 10u8).ok_or(String::from("unable to locate 10u8")).and_then(|end_idx| {
                    from_utf8(&tail[10..end_idx]).map_err(|e| e.to_string()).map(|c| (c, end_idx))
                });
                match committer_res {
                    Ok((committer, end_idx)) => {
                        committer_opt = Some(committer.to_string());
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

fn parse_header(raw_header: &[u8]) -> Result<Header, String> {
    let parse = |x, t| {
        from_utf8(&raw_header[x..]).map_err(|e| e.to_string()).and_then(|size_string| {
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

fn parse_object(raw: &[u8]) -> Result<FullObject, String> {
    raw.iter().position(|&b| b == 0u8).ok_or(String::from("unable to find 0u8")).and_then(|l| {
        parse_header(&raw[..l]).and_then(|h| {
            let rb = &raw[l+1..];
            match h.typp {
                Type::Blob => parse_blob(&rb).map(|b| FullObject{header:h, object:Box::new(b)}),
                Type::Tree => parse_tree(&rb).map(|t| FullObject{header:h, object:Box::new(t)}),
                Type::Commit => parse_commit(&rb).map(|c| FullObject{header:h, object:Box::new(c)}),
            }
        })
    })
}

pub fn deflate_and_parse_object(compressed: &[u8]) -> Result<(Hash,FullObject), String> {
    let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
    let mut deflated: Vec<u8> = Vec::new();
    if let Err(e) = decoder.read_to_end(&mut deflated) {
        return Err(String::from(e.to_string()));
    }
    parse_object(&deflated).map(|obj| {
        (Hash::sum(&deflated), obj)
    })
}