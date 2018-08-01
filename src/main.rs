extern crate rustgit;

use rustgit::repo;
use rustgit::object;

use std::path::Path;

fn main() {
    test_load();
}

fn test_load() {
    let mut repo = repo::Repo::new();
    repo.load(&Path::new(".git/objects"));
}

fn test_iter_2(v: &Vec<u8>) {
    let tmp: Vec<u8> = v.iter().take_while(|b| **b != b'l').map(|b| *b).collect();
    println!("{:?}", std::str::from_utf8(&tmp));
}

fn test_iter() {
    let mut data: Vec<u8> = Vec::new();
    data.push(b'f');
    data.push(b'o');
    data.push(b'o');
    data.push(b'l');
    test_iter_2(&data);
}