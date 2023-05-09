#![allow(unused)]
use std::{fs::File, io::Read, path::PathBuf};

use parse::ast_crate::Crate;
use resolve::ast_resolve;

use crate::resolve::ast_items::ItemsCrate;

mod parse;
mod resolve;
mod util;

fn main() {
    let crates = Crate::parse(PathBuf::from("test/a"));
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
        vec!["macros", "component"],
        vec!["crate", "component"],
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
    let items = crates
        .iter()
        .map(|cr| {
            let mut ic = ItemsCrate::new();
            ic.parse_crate(cr, &crates);
            ic
        })
        .collect::<Vec<_>>();
    println!("{:#?}", items);
}
