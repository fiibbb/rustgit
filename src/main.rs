extern crate rustgit;

use rustgit::repo;

use std::path::Path;

fn main() {
    test_load();
}

fn test_load() {
    let mut repo = repo::Repo::new();
    repo.load(&Path::new(".git/objects"));
}