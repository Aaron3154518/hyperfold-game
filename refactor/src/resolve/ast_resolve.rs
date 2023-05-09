use crate::parse::{
    ast_crate::Crate,
    ast_mod::{Mod, Symbol},
};
use crate::util::Expect;

pub fn resolve(path: &Vec<String>, crates: &Vec<Crate>) -> Result<Vec<String>, Vec<String>> {
    for cr in crates.iter() {
        if path.get(0).is_some_and(|s| *s == cr.name) {
            return resolve_mod(path, 1, &cr.main, crates);
        }
    }
    Err(path.to_vec())
}

// Assumes we already match with mod
pub fn resolve_mod(
    path: &Vec<String>,
    idx: usize,
    m: &Mod,
    crates: &Vec<Crate>,
) -> Result<Vec<String>, Vec<String>> {
    // Ran out of stuff to parse
    if idx == path.len() {
        return Ok(path.to_vec());
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
            return resolve_mod(path, idx + 1, m, crates);
        }
    }
    // Check symbols
    for sym in m.symbols.iter().filter(|sym| sym.public) {
        if sym.ident == name {
            return Ok(sym.path.to_vec());
        }
    }
    // Check use statements
    for sym in m.uses.iter().filter(|sym| sym.public) {
        // Glob - this is allowed to fail
        if sym.ident == "*" {
            let path = [sym.path.to_vec(), path[idx..].to_vec()].concat();
            if let Ok(v) = resolve_local_path(&path, m, crates).or_else(|e| resolve(&path, crates))
            {
                return Ok(v);
            }
        // Use
        } else if sym.ident == name {
            let path = [sym.path.to_vec(), path[idx + 1..].to_vec()].concat();
            return resolve_local_path(&path, m, crates).or_else(|e| resolve(&path, crates));
        }
    }
    Err(path.to_vec())
}

// Paths that start relative to some mod item
pub fn resolve_local_path(
    path: &Vec<String>,
    m: &Mod,
    crates: &Vec<Crate>,
) -> Result<Vec<String>, Vec<String>> {
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
                .and_then(|mut poss_path| {
                    // Test each path
                    Some(resolve(&mut poss_path, crates))
                })
            })
        })
        .unwrap_or_else(|| Err(path.to_vec()))
}
