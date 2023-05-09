use crate::{
    ast_crate::Crate,
    ast_mod::{Mod, Symbol},
    ast_visitor::{Visited, Visitor},
    util::Expect,
};

pub fn resolve(path: &mut Vec<String>, crates: &Vec<Crate>) -> Option<Vec<String>> {
    println!("{}", path.join("::"));
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
    for sym in m.pub_symbols.iter() {
        if sym.ident == name {
            return Some(sym.path.to_vec());
        }
    }
    // Check use statements
    for sym in m.pub_uses.iter() {
        // Glob
        if sym.ident == "*" {
            if let Some(v) = resolve(
                &mut [sym.path.to_vec(), path[idx..].to_vec()].concat(),
                crates,
            ) {
                return Some(v);
            }
        // Use
        } else if sym.ident == name {
            return resolve(
                &mut [sym.path.to_vec(), path[idx + 1..].to_vec()].concat(),
                crates,
            );
        }
    }
    None
}

// pub fn resolve(cr: &Crate, crates: &Vec<Crate>) {
//     resolve_file(&cr.file, crates)
// }

// pub fn resolve_file(f: &FileType, crates: &Vec<Crate>) {
//     match &f {
//         crate::ast_file::FileType::File(f) => resolve_mod(&f.file_mod, crates),
//         crate::ast_file::FileType::Dir(d) => {
//             d.children.iter().for_each(|f| resolve_file(f, crates))
//         }
//     }
// }

// pub fn resolve_mod(m: &Mod, crates: &Vec<Crate>) {
//     for sym in m.pub_uses.iter() {
//         let res_sym = ResolveUsePath {
//             path: sym.alias.to_vec(),
//         }
//         .resolve(crates);
//     }
// }

// impl Visitor for ResolveUsePath {
//     fn visit_crates(&mut self, v: &mut Vec<crate::ast_crate::Crate>) {
//         for cr in v.iter_mut() {
//             // TODO: Check that this crate is a dependency
//             if self.path.last().is_some_and(|s| *s == cr.name) {
//                 self.path.pop();
//                 v.visit(cr);
//                 break;
//             }
//         }
//     }

//     fn visit_mod(&mut self, v: &mut crate::ast_mod::Mod) {}
// }
