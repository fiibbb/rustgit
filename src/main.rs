extern crate rustgit;

use rustgit::object;
use rustgit::pack;
use rustgit::repo;

use std::path::Path;
use std::fs;

fn main() {
    test_idx();
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
}

fn test_idx() {
    let h = String::from("/Users/lingy/Uber/gocode/src/code.uber.internal/go-common.git/.git/objects/pack/pack-115b28b817f10363fddf1f0f9bd48d09ec703791");
    let f_index = fs::read(&Path::new(&format!("{}{}", h, ".idx"))).unwrap();
    let f_pack = fs::read(&Path::new(&format!("{}{}", h, ".pack"))).unwrap();
    pack::parse_pack(&f_index, &f_pack).unwrap();
}