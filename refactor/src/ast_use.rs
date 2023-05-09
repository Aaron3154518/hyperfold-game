use crate::{
    ast_crate::Crate,
    ast_mod::{Mod, Symbol},
    ast_visitor::{Visited, Visitor},
};

pub fn resolve(path: &mut Vec<String>, crates: &Vec<Crate>) -> Option<Vec<String>> {
    path.reverse();

    for cr in crates.iter() {
        if path.last().is_some_and(|s| *s == cr.name) {
            path.pop();
            return resolve_mod(path, &cr.main);
        }
    }
    None
}

// Assumes we already match with mod
pub fn resolve_mod(path: &mut Vec<String>, m: &Mod) -> Option<Vec<String>> {
    let name = path.last().expect("Resolve path is empty").to_string();
    // Check sub modules
    for m in m.mods.iter() {
        if name == *m.path.last().expect("Mod path is empty") {
            path.pop();
            return resolve_mod(path, m);
        }
    }
    // Check symbols
    if let Some(name) = path.last() {
        for sym in m.pub_symbols.iter() {
            if &sym.ident == name {
                return Some(sym.path.to_vec());
            }
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
