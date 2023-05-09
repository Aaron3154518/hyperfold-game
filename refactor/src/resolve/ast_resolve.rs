use syn::Pat;

use crate::parse::{
    ast_crate::Crate,
    ast_mod::{Mod, Symbol},
};
use crate::util::Expect;

#[derive(Debug)]
pub struct Path {
    pub cr_idx: usize,
    pub path: Vec<String>,
}

pub fn resolve(path: Vec<String>, cr: &Crate, crates: &Vec<Crate>) -> Result<Path, Path> {
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
            return resolve_mod(path, idx + 1, cr, m, crates);
        }
    }
    // Check symbols
    for sym in m.symbols.iter().filter(|sym| sym.public) {
        if sym.ident == name {
            return Ok(Path {
                cr_idx: cr.idx,
                path: sym.path.to_vec(),
            });
        }
    }
    // Check use statements
    for sym in m.uses.iter().filter(|sym| sym.public) {
        // Glob - this is allowed to fail
        if sym.ident == "*" {
            let path = [sym.path.to_vec(), path[idx..].to_vec()].concat();
            if let Ok(v) = resolve_local_path(path.to_vec(), cr, m, crates)
                .or_else(|e| resolve(path, cr, crates))
            {
                return Ok(v);
            }
        // Use
        } else if sym.ident == name {
            let path = [sym.path.to_vec(), path[idx + 1..].to_vec()].concat();
            return resolve_local_path(path.to_vec(), cr, m, crates)
                .or_else(|e| resolve(path, cr, crates));
        }
    }
    Err(Path { cr_idx, path })
}

// Paths that start relative to some mod item
pub fn resolve_local_path(
    path: Vec<String>,
    cr: &Crate,
    m: &Mod,
    crates: &Vec<Crate>,
) -> Result<Path, Path> {
    // Get possible paths
    let name = path
        .first()
        .catch(format!("Empty resolve path: {}", path.join("::")));
    [&m.symbols, &m.uses]
        .iter()
        .find_map(|syns| {
            syns.iter().find_map(|syn| {
                // Get possible path
                if syn.ident == "*" {
                    Some([syn.path.to_vec(), path.to_vec()].concat())
                } else {
                    (name == &syn.ident).then_some([syn.path.to_vec(), path[1..].to_vec()].concat())
                }
                .and_then(|poss_path| {
                    // Test each path
                    Some(resolve(poss_path, cr, crates))
                })
            })
        })
        .unwrap_or_else(|| {
            Err(Path {
                cr_idx: cr.idx,
                path,
            })
        })
}
