#![allow(unused)]
use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use parse::ast_crate::Crate;
use resolve::ast_resolve;

use crate::resolve::ast_items::ItemsCrate;

mod parse;
mod resolve;
mod util;

fn test_resolves(crates: &Vec<Crate>) {
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
        vec!["macros", "component"],
        vec!["crate", "component"],
        vec!["crate", "a2", "a3", "mac", "global"],
    ] {
        println!(
            "{}\n{:#?}",
            v.join("::"),
            ast_resolve::resolve(
                v.iter().map(|s| s.to_string()).collect(),
                &crates[0],
                &crates
            )
        )
    }
}

fn main() {
    let engine_dir =
        fs::canonicalize(PathBuf::from("macros")).expect("Could not canonicalize engine path");
    let crates = Crate::parse(PathBuf::from("test/a"));
    let engine_crate = crates
        .iter()
        .find(|cr| cr.dir == engine_dir)
        .expect("Could not find engine crate. Please include it");
    println!("{:#?}", crates);
    // test_resolves(&crates);
    let items = crates
        .iter()
        .map(|cr| {
            let mut ic = ItemsCrate::new();
            ic.parse_crate(cr, engine_crate, &crates);
            ic
        })
        .collect::<Vec<_>>();
    println!("{:#?}", items);
}
