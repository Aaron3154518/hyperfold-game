use crate::{
    parse::{ast_crate::Crate, ast_mod::Mod},
    util::{Expect, Get},
};

use super::{
    ast_args::{ComponentMacroArgs, GlobalMacroArgs},
    ast_resolve::resolve_local_path,
};

#[derive(Debug)]
pub struct Component {
    path: Vec<String>,
    args: ComponentMacroArgs,
}

#[derive(Debug)]
pub struct Global {
    path: Vec<String>,
    args: GlobalMacroArgs,
}

#[derive(Debug)]
pub struct Event {
    path: Vec<String>,
    variants: Vec<String>,
}

#[derive(Debug)]
pub struct System {}

#[derive(Debug)]
pub struct Dependency {
    name: String,
}

// Pass 2: use resolving
// Resolve macro paths - convert to engine items
// Resolve system arg paths
#[derive(Debug)]
pub struct ItemsCrate {
    pub dependencies: Vec<Dependency>,
    pub components: Vec<Component>,
    pub globals: Vec<Global>,
    pub events: Vec<Event>,
    pub systems: Vec<System>,
}

impl ItemsCrate {
    pub fn new() -> Self {
        Self {
            dependencies: Vec::new(),
            components: Vec::new(),
            globals: Vec::new(),
            events: Vec::new(),
            systems: Vec::new(),
        }
    }

    pub fn parse_crate(&mut self, cr: &Crate, crates: &Vec<Crate>) {
        self.parse_mod(&cr.main, crates)
    }

    pub fn parse_mod(&mut self, m: &Mod, crates: &Vec<Crate>) {
        let dep_path = vec!["macros".to_string(), "dependency".to_string()];
        for mi in m.marked.iter() {
            match mi.ty {
                crate::parse::ast_mod::MarkType::Struct => {}
                crate::parse::ast_mod::MarkType::Fn => {}
                crate::parse::ast_mod::MarkType::Enum => {}
                crate::parse::ast_mod::MarkType::Use => {
                    for (path, args) in mi.attrs.iter() {
                        println!("{}", path.join("::"));
                        let match_path = resolve_local_path(&mut path.to_vec(), m, crates).get();
                        println!("{}", match_path.join("::"));
                        if match_path == dep_path {
                            println!("Dep: {:#?}", mi.sym.path.last());
                            self.dependencies.push(Dependency {
                                name: mi
                                    .sym
                                    .path
                                    .last()
                                    .expect("Dependency path is empty")
                                    .to_string(),
                            });
                            break;
                        }
                    }
                }
            }
        }
        m.mods.iter().for_each(|m| self.parse_mod(m, crates));
    }
}
