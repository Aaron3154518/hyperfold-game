#![allow(unused)]
use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use parse::ast_crate::Crate;
use regex::Regex;
use resolve::ast_resolve;
use util::SplitCollect;

use crate::{
    decode::decoder::Decoder,
    resolve::{ast_items::ItemsCrate, ast_paths::EnginePaths},
    util::format_code,
    validate::ast_validate::ItemData,
};

mod decode;
mod parse;
mod resolve;
mod util;
mod validate;

// Process:
// 1) Parse - for each crate: traverse AST, extract important items/uses/mods/dependencies
// 2) Resolve - resolve item paths within/across crates
// 3) Validate - convert IR to data format and validate items
// 4) Decode - convert data back to IR

fn test_resolves(crates: &Vec<Crate>) {
    let test = |v: Vec<&str>| {
        println!(
            "{}\n{:#?}",
            v.join("::"),
            ast_resolve::resolve(
                v.iter().map(|s| s.to_string()).collect(),
                &crates[0],
                &crates
            )
        )
    };

    println!("\nOk:\n");
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
        vec!["crate", "c", "e", "DC"],
    ] {
        test(v)
    }

    println!("\nErr:\n");
    for v in [
        vec!["engine", "component"],
        vec!["crate", "component"],
        vec!["crate", "a2", "a3", "mac", "global"],
    ] {
        test(v)
    }
}

fn main() {
    let crates = Crate::parse(PathBuf::from("test/a"));
    // println!("{:#?}", crates);

    let engine_dir =
        fs::canonicalize(PathBuf::from("engine")).expect("Could not canonicalize engine path");
    let paths = crates
        .iter()
        .find_map(|cr| (cr.dir == engine_dir).then_some(EnginePaths::to_paths(cr.idx)))
        .expect("Could not find engine crate. Please include it");

    // test_resolves(&crates);

    let mut items = crates
        .iter()
        .map(|cr| {
            let mut ic = ItemsCrate::new();
            ic.parse_crate(cr, &paths, &crates);
            ic
        })
        .collect::<Vec<_>>();
    // println!("{:#?}", items);

    let data = ItemData::validate(&paths, &mut items);
    data.write_to_file();
    println!("{:#?}", data);

    let decoder = Decoder::new();
    // println!("{:#?}", decoder);
    println!(
        "{}",
        format_code(decoder.codegen(PathBuf::from("./test/a")).to_string())
    );
}
