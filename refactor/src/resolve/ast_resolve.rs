use syn::Pat;

use crate::parse::{
    ast_crate::Crate,
    ast_mod::{Mod, Symbol},
};
use crate::util::Expect;

#[derive(PartialEq, Eq, Debug)]
pub struct Path {
    pub cr_idx: usize,
    pub path: Vec<String>,
}

pub fn resolve(path: Vec<String>, cr: &Crate, crates: &Vec<Crate>) -> Result<Path, Path> {
    // println!("Resolve: {}", path.join("::"));
    let cr_idx = cr.idx;
    match path.first() {
        Some(p) => {
            match match p.as_str() {
                // Match this crate
                "crate" => Some(cr),
                // Match dependency
                _ => cr.deps.iter().find_map(|(idx, name)| {
                    (name == p).then_some(crates.get(*idx).expect("Invalid dependency index"))
                }),
            } {
                Some(cr) => resolve_mod(path, 1, cr, &cr.main, crates),
                None => Err(Path { cr_idx, path }),
            }
        }
        None => Err(Path { cr_idx, path }),
    }
}

// Assumes we already match with mod
pub fn resolve_mod(
    path: Vec<String>,
    idx: usize,
    cr: &Crate,
    m: &Mod,
    crates: &Vec<Crate>,
) -> Result<Path, Path> {
    // println!(
    //     "Resolve Mod: {} at {}",
    //     path.join("::"),
    //     path.get(idx).unwrap_or(&"None".to_string())
    // );
    let cr_idx = cr.idx;

    // Ran out of stuff to parse
    if idx == path.len() {
        return Ok(Path { cr_idx, path });
    }

    let name = path
        .get(idx)
        .catch(format!(
            "Bad resolve path index: {} in path: \"{}\"",
            idx,
            path.join("::")
        ))
        .to_string();
    // Check sub modules
    for m in m.mods.iter() {
        if name == *m.path.last().expect("Mod path is empty") {
            // println!("Found Mod: {}", name);
            return resolve_mod(path, idx + 1, cr, m, crates);
        }
    }
    // Check symbols
    for sym in m.symbols.iter().filter(|sym| sym.public) {
        if sym.ident == name {
            // println!("Found Symbol: {}", sym.path.join("::"));
            return Ok(Path {
                cr_idx: cr.idx,
                path: sym.path.to_vec(),
            });
        }
    }
    // Check use statements
    for sym in m.uses.iter() {
        // Glob - this is allowed to fail
        if sym.ident == "*" {
            let path = [sym.path.to_vec(), path[idx..].to_vec()].concat();
            if let Ok(v) = resolve_path(path.to_vec(), cr, m, crates) {
                return Ok(v);
            }
        // Use
        } else if sym.ident == name {
            let path = [sym.path.to_vec(), path[idx + 1..].to_vec()].concat();
            // println!("Matched Use: {}", sym.path.join("::"));
            return resolve_path(path.to_vec(), cr, m, crates);
        }
    }
    Err(Path { cr_idx, path })
}

// Paths that start relative to some mod item
pub fn resolve_path(
    path: Vec<String>,
    cr: &Crate,
    m: &Mod,
    crates: &Vec<Crate>,
) -> Result<Path, Path> {
    // println!("Local Resolve: {}", path.join("::"));
    let cr_idx = cr.idx;

    let name = path
        .first()
        .catch(format!("Empty resolve path: {}", path.join("::")));

    // Can't be local
    if name == "crate" {
        return resolve(path, cr, crates);
    }

    // Iterate possible paths
    [&m.symbols, &m.uses]
        .iter()
        .find_map(|syns| {
            syns.iter().find_map(|syn| {
                // Get possible path
                if syn.ident == "*" {
                    resolve([syn.path.to_vec(), path.to_vec()].concat(), cr, crates)
                        .map_or(None, |p| Some(Ok(p)))
                } else if name == &syn.ident {
                    Some(resolve(
                        [syn.path.to_vec(), path[1..].to_vec()].concat(),
                        cr,
                        crates,
                    ))
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| resolve(path, cr, crates))
}
