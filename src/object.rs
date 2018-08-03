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
    pub fn from(data: &[u8]) -> Hash {
        Hash {
            hash: sha1::Sha1::from(data).digest().bytes()
        }
    }
    pub fn parse(v: &[u8]) -> Option<Hash> {
        let mut hash: [u8;20] = [0;20];
        if v.len() >= 20 {
            hash.copy_from_slice(&v[..20]);
            Some(Hash{hash})
        } else {
            None
        }
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

fn decode_hex_str(s: &str) -> Option<[u8;20]> {
    hex::decode(s).ok().and_then(|v| {
        let mut res: [u8;20] = [0;20];
        if v.len() == 20 {
            res.copy_from_slice(&v[..]);
            Some(res)
        } else {
            None
        }
    })
}

fn parse_blob(body: &[u8]) -> Result<Blob, String> {
    Ok(Blob{
        data: body.to_vec(),
    })
}

fn parse_tree(body: &[u8]) -> Result<Tree, String> {

    let mut tree_entries: Vec<TreeEntry> = Vec::new();
    let mut tail: &[u8] = body;

    loop {
        if let Some((h, t)) = tail.iter().position(|&b| b == 0u8).map(|l| (&tail[..l+21], &tail[l+21..])) {
            let entry_opt = h.iter().position(|&b| b == b' ').and_then(|l1| {
                h.iter().position(|&b| b == 0u8).and_then(|l2| {
                    from_utf8(&h[..l1]).ok().and_then(|mode| {
                        from_utf8(&h[l1+1..l2]).ok().and_then(|name| {
                            Hash::parse(&h[l2+1..l2+21]).map(|hash| {
                                TreeEntry{
                                    mode: mode.to_string(),
                                    name: name.to_string(),
                                    hash: hash,
                                }
                            })
                        })
                    })
                })
            });
            if let Some(entry) = entry_opt {
                tree_entries.push(entry);
                tail = t;
            } else {
                return Err(String::from("failed to parse tree entry"));
            }
        } else {
            break;
        }
    };

    Ok(Tree{children:tree_entries})
}

fn parse_commit(body: &[u8]) -> Result<Commit, String> {

    let mut tree_opt: Option<Hash> = None;
    let mut author_opt: Option<String> = None;
    let mut commiter_opt: Option<String> = None;
    let mut parents: Vec<Hash> = Vec::new();
    let msg: String;
    let mut tail: &[u8] = body;

    loop {
        if tail.len() >= 46 && tail[..4] == *b"tree" {
            if let None = tree_opt {
                if let Some(hash_str) = from_utf8(&tail[5..45]).ok() {
                    if let Some(hash) = decode_hex_str(hash_str) {
                        tree_opt = Some(Hash{hash});
                        tail = &tail[46..];
                    } else {
                        return Err(String::from("unable to pare tree hash"));
                    }
                } else {
                    return Err(String::from("unable to parse tree hash"));
                }
            } else {
                return Err(String::from("duplicate tree entry in commit"));
            }
        } else if tail.len() >= 7 && tail[..6] == *b"author" {
            if let None = author_opt {
                if let Some(end_idx) = tail.iter().position(|&b| b == 10u8) {
                    if let Some(author_str) = from_utf8(&tail[7..end_idx]).ok() {
                        author_opt = Some(author_str.to_string());
                        tail = &tail[end_idx+1..];
                    } else {
                        return Err(String::from("unable to parse author"));
                    }
                } else {
                    return Err(String::from("unable to parse author"));
                }
            } else {
                return Err(String::from("duplicate author entry in commit"));
            }
        } else if tail.len() >= 10 && tail[..9] == *b"committer" {
            if let None = commiter_opt {
                if let Some(end_idx) = tail.iter().position(|&b| b == 10u8) {
                    if let Some(committer_str) = from_utf8(&tail[10..end_idx]).ok() {
                        commiter_opt = Some(committer_str.to_string());
                        tail = &tail[end_idx+1..];
                    } else {
                        return Err(String::from("unable to parse committer"));
                    }
                } else {
                    return Err(String::from("unable to parse committer"));
                }
            } else {
                return Err(String::from("duplicate committer entry in commit"));
            }
        } else if tail.len() >= 48 && tail[..6] == *b"parent" {
            if let Some(hash_str) = from_utf8(&tail[7..47]).ok() {
                if let Some(hash) = decode_hex_str(hash_str) {
                    parents.push(Hash{hash});
                    tail = &tail[48..];
                } else {
                    return Err(String::from("unable to parse parent hash"));
                }
            } else {
                return Err(String::from("unable to parse parent hash"));
            }
        } else if tail.len() > 0 {
            if let Some(msg_str) = from_utf8(&tail[..]).ok() {
                msg = msg_str.to_string();
                tail = &[];
                break;
            } else {
                return Err(String::from("unable to parse message"));
            }
        }
    }

    let commit_opt = tree_opt.and_then(|t| {
        author_opt.and_then(|a| {
            commiter_opt.map(|c| {
                Commit{
                    tree:t,
                    parents: parents,
                    author: a,
                    committer: c,
                    message: msg,
                }
            })
        })
    });

    commit_opt.ok_or(String::from("missing commit components"))
}

fn parse_header(raw_header: &[u8]) -> Result<Header, String> {
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

fn parse_object(raw: &[u8]) -> Result<FullObject, String> {
    raw.iter().position(|&b| b == 0u8).map(|l| {
        (&raw[..l], &raw[l+1..])
    }).and_then(|(rh, rb)| {
        parse_header(&rh).map(|h| (h, rb)).ok()
    }).and_then(|(h, rb)| {
        match h.typp {
            Type::Blob => parse_blob(&rb).map(|b| FullObject{header:h, object:Box::new(b)}).ok(),
            Type::Tree => parse_tree(&rb).map(|t| FullObject{header:h, object:Box::new(t)}).ok(),
            Type::Commit => parse_commit(&rb).map(|c| FullObject{header:h, object:Box::new(c)}).ok(),
        }
    }).ok_or(String::from("failed to parse object"))
}

pub fn deflate_and_parse_object(compressed: &[u8]) -> Result<(Hash,FullObject), String> {
    let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
    let mut deflated: Vec<u8> = Vec::new();
    if let Err(e) = decoder.read_to_end(&mut deflated) {
        return Err(String::from(e.to_string()));
    }
    parse_object(&deflated).map(|obj| {
        (Hash::from(&deflated), obj)
    })
}