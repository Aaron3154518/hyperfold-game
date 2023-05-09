#![allow(unused)]
use std::path::PathBuf;

use ast_crate::Crate;

mod ast_crate;
mod ast_file;
mod ast_mod;
mod ast_use;
mod ast_visitor;
mod util;

fn main() {
    let crates = vec![Crate::new(PathBuf::from("test/a"), true)];
    println!("{:#?}", crates);
    for v in [
        vec!["crate", "T1"],
        vec!["crate", "a1", "A"],
        vec!["crate", "a2", "a5", "HEY"],
        vec!["crate", "a22", "a5", "HEY"],
        vec!["crate", "a22", "a2", "a5", "HEY"],
        vec!["crate", "a2", "a3", "A", "A1"],
        vec!["crate", "a2", "a3", "A", "A2"],
        vec!["crate", "a2", "a3", "B", "A2"],
        vec!["crate", "a2", "a3", "A3", "A1"],
        vec!["crate", "a2", "a2", "A3", "A1"],
    ] {
        println!(
            "{}\n{:#?}",
            v.join("::"),
            ast_use::resolve(&mut v.iter().map(|s| s.to_string()).collect(), &crates)
        )
    }
}
