use crate::{
    ast_crate::Crate,
    ast_mod::{Mod, Symbol},
    util::Expect,
};

pub fn resolve(path: &mut Vec<String>, crates: &Vec<Crate>) -> Option<Vec<String>> {
    for cr in crates.iter() {
        if path.get(0).is_some_and(|s| *s == cr.name) {
            return resolve_mod(path, 1, &cr.main, crates);
        }
    }
    None
}

// Assumes we already match with mod
pub fn resolve_mod(
    path: &mut Vec<String>,
    idx: usize,
    m: &Mod,
    crates: &Vec<Crate>,
) -> Option<Vec<String>> {
    // Ran out of stuff to parse
    if idx == path.len() {
        return Some(path.to_vec());
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
            return Some(sym.path.to_vec());
        }
    }
    // Check use statements
    for sym in m.uses.iter().filter(|sym| sym.public) {
        // Glob
        if sym.ident == "*" {
            let mut path = [sym.path.to_vec(), path[idx..].to_vec()].concat();
            if let Some(v) = resolve(&mut path.to_vec(), crates)
                .or_else(|| resolve_local_path(&mut path, m, crates))
            {
                return Some(v);
            }
        // Use
        } else if sym.ident == name {
            let mut path = [sym.path.to_vec(), path[idx + 1..].to_vec()].concat();
            return resolve(&mut path.to_vec(), crates)
                .or_else(|| resolve_local_path(&mut path, m, crates));
        }
    }
    None
}

// Paths that start relative to some mod item
pub fn resolve_local_path(
    path: &mut Vec<String>,
    m: &Mod,
    crates: &Vec<Crate>,
) -> Option<Vec<String>> {
    // Get possible paths
    let name = path
        .first()
        .catch(format!("Empty resolve path: {}", path.join("::")));
    [&m.symbols, &m.uses].iter().find_map(|syns| {
        syns.iter().find_map(|syn| {
            // Get possible path
            if syn.ident == "*" {
                Some([syn.path.to_vec(), path.to_vec()].concat())
            } else {
                (name == &syn.ident).then_some([syn.path.to_vec(), path[1..].to_vec()].concat())
            }
            .and_then(|mut poss_path| {
                // Test each path
                resolve(&mut poss_path, crates)
            })
        })
    })
}
