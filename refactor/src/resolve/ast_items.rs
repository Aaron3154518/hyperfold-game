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
    pub path: Path,
    pub args: ComponentMacroArgs,
}

#[derive(Debug)]
pub struct Global {
    pub path: Path,
    pub args: GlobalMacroArgs,
}

#[derive(Debug)]
pub struct Event {
    pub path: Path,
    pub variants: Vec<String>,
}

#[derive(Debug)]
pub struct System {
    pub path: Path,
    pub args: Vec<FnArg>,
}

#[derive(Debug)]
pub struct Dependency {
    pub cr_idx: usize,
    pub name: String,
}

// Pass 2: use resolving
// Resolve macro paths - convert to engine items
#[derive(Debug)]
pub struct ItemsCrate {
    pub cr_name: String,
    pub cr_idx: usize,
    pub components: Vec<Component>,
    pub globals: Vec<Global>,
    pub events: Vec<Event>,
    pub systems: Vec<System>,
    pub dependencies: Vec<Dependency>,
}

impl ItemsCrate {
    pub fn new() -> Self {
        Self {
            cr_name: String::new(),
            cr_idx: 0,
            components: Vec::new(),
            globals: Vec::new(),
            events: Vec::new(),
            systems: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    pub fn parse_crate(&mut self, cr: &Crate, engine_cr: &Crate, crates: &Vec<Crate>) {
        self.cr_name = cr.name.to_string();
        self.cr_idx = cr.idx;
        self.dependencies = cr
            .deps
            .iter()
            .map(|(&cr_idx, name)| Dependency {
                cr_idx,
                name: name.to_string(),
            })
            .collect::<Vec<_>>();
        self.parse_mod(cr, &cr.main, engine_cr, crates);
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

        let cr_idx = cr.idx;

        for mi in m.marked.iter() {
            for (path, args) in mi.attrs.iter() {
                let match_path = resolve_path(path.to_vec(), cr, m, crates).get();
                match &mi.ty {
                    crate::parse::ast_mod::MarkType::Struct => {
                        if match_path == comp_path {
                            self.components.push(Component {
                                path: Path {
                                    cr_idx,
                                    path: mi.sym.path.to_vec(),
                                },
                                args: ComponentMacroArgs::from(args.to_vec()),
                            });
                            break;
                        } else if match_path == glob_path {
                            self.globals.push(Global {
                                path: Path {
                                    cr_idx,
                                    path: mi.sym.path.to_vec(),
                                },
                                args: GlobalMacroArgs::from(args.to_vec()),
                            });
                            break;
                        }
                    }
                    crate::parse::ast_mod::MarkType::Fn { args } => {
                        if match_path == system_path {
                            self.systems.push(System {
                                path: Path {
                                    cr_idx,
                                    path: mi.sym.path.to_vec(),
                                },
                                args: args
                                    .iter()
                                    .map(|a| {
                                        let mut a = a.to_owned();
                                        a.resolve_paths(cr, m, crates);
                                        a
                                    })
                                    .collect(),
                            });
                            break;
                        }
                    }
                    crate::parse::ast_mod::MarkType::Enum { variants } => {
                        if match_path == event_path {
                            self.events.push(Event {
                                path: Path {
                                    cr_idx,
                                    path: mi.sym.path.to_vec(),
                                },
                                variants: variants.to_vec(),
                            });
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
