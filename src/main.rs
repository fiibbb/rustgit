extern crate rustgit;

use rustgit::repo;
use rustgit::object;

use std::path::Path;
use std::fs;

fn main() {
    test_load();
}

fn test_load() {
    let mut repo = repo::Repo::new();
    repo.load(&Path::new(".git/objects"));
    repo.save(&Path::new("./test_save_target"));
}

fn test_single() {
    let p = Path::new(".git/objects/b7/1c8977c043f1984f4613a397e2d954d285ea26");
    let f = fs::read(&p).unwrap();
    let obj = object::deflate(&f).unwrap();
    let hash = (*obj).hash().hex();
    fs::write(Path::new("./test_save_target/foo"), obj.encode().unwrap());
    println!("path: {:?}", p);
    println!("hash: {}", hash);
}