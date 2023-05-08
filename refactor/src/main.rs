#![allow(unused)]
use std::path::PathBuf;

use ast_crate::Crate;

mod ast_crate;
mod ast_file;
mod ast_mod;
mod util;

fn main() {
    let mut cr = Crate::new(PathBuf::from("test/a/src"), true);
    println!("{:#?}", cr);
}
