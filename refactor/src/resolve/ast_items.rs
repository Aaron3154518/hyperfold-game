use crate::{
    parse::{
        ast_crate::Crate,
        ast_fn_arg::{FnArg, FnArgType},
        ast_mod::Mod,
    },
    resolve::ast_resolve::resolve_path,
    util::{Catch, Get},
};

use super::{
    ast_args::{ComponentMacroArgs, GlobalMacroArgs},
    ast_resolve::Path,
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
pub struct System {
    path: Vec<String>,
    args: Vec<FnArg>,
}

#[derive(Debug)]
pub struct Dependency {
    name: String,
}

// Pass 2: use resolving
// Resolve macro paths - convert to engine items
#[derive(Debug)]
pub struct ItemsCrate {
    pub cr_idx: usize,
    pub components: Vec<Component>,
    pub globals: Vec<Global>,
    pub events: Vec<Event>,
    pub systems: Vec<System>,
}

impl ItemsCrate {
    pub fn new() -> Self {
        Self {
            cr_idx: 0,
            components: Vec::new(),
            globals: Vec::new(),
            events: Vec::new(),
            systems: Vec::new(),
        }
    }

    pub fn parse_crate(&mut self, cr: &Crate, engine_cr: &Crate, crates: &Vec<Crate>) {
        self.cr_idx = cr.idx;
        self.parse_mod(cr, &cr.main, engine_cr, crates)
    }

    pub fn parse_mod(&mut self, cr: &Crate, m: &Mod, engine_cr: &Crate, crates: &Vec<Crate>) {
        let comp_path = Path {
            cr_idx: engine_cr.idx,
            path: vec!["engine".to_string(), "component".to_string()],
        };
        let glob_path = Path {
            cr_idx: engine_cr.idx,
            path: vec!["engine".to_string(), "global".to_string()],
        };
        let event_path = Path {
            cr_idx: engine_cr.idx,
            path: vec!["engine".to_string(), "event".to_string()],
        };
        let system_path = Path {
            cr_idx: engine_cr.idx,
            path: vec!["engine".to_string(), "system".to_string()],
        };

        for mi in m.marked.iter() {
            for (path, args) in mi.attrs.iter() {
                let match_path = resolve_path(path.to_vec(), cr, m, crates).get();
                match &mi.ty {
                    crate::parse::ast_mod::MarkType::Struct => {
                        if match_path == comp_path {
                            self.components.push(Component {
                                path: mi.sym.path.to_vec(),
                                args: ComponentMacroArgs::from(args.to_vec()),
                            });
                            break;
                        } else if match_path == glob_path {
                            self.globals.push(Global {
                                path: mi.sym.path.to_vec(),
                                args: GlobalMacroArgs::from(args.to_vec()),
                            });
                            break;
                        }
                    }
                    crate::parse::ast_mod::MarkType::Fn { args } => self.systems.push(System {
                        path: mi.sym.path.to_vec(),
                        args: args
                            .iter()
                            .map(|a| {
                                let mut a = a.to_owned();
                                a.resolve_paths(cr, m, crates);
                                a
                            })
                            .collect(),
                    }),
                    crate::parse::ast_mod::MarkType::Enum { variants } => {
                        if match_path == event_path {
                            self.events.push(Event {
                                path: mi.sym.path.to_vec(),
                                variants: variants.to_vec(),
                            })
                        }
                    }
                }
            }
        }
        m.mods
            .iter()
            .for_each(|m| self.parse_mod(cr, m, engine_cr, crates));
    }
}

// Pass 3: Item resolution
// Resolve system arg paths
