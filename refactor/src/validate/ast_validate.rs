use std::{
    cmp::min,
    fs::{self, File},
    io::Write,
};

use crate::{
    parse::ast_fn_arg::{FnArg, FnArgType},
    resolve::{
        ast_args::{ComponentMacroArgs, GlobalMacroArgs},
        ast_items::{Component, Dependency, Event, Global, ItemsCrate, System},
        ast_paths::{EngineGlobals, EngineTraits, ExpandEnum, Paths},
        ast_resolve::Path,
    },
    util::{end, Catch, JoinMap, JoinMapInto, NoneOr, SplitIter},
};

use super::{
    ast_item_list::ItemList,
    constants::{DATA_FILE, SEP},
};

// Pass 3: Item validation
// Map system arg paths to items
#[macros::expand_enum]
pub enum Data {
    Components,
    Globals,
    Events,
    Systems,
    Dependencies,
    EngineTraits,
    EngineGlobals,
    CratePaths,
}

#[derive(Debug)]
pub struct ItemData {
    pub data: [String; Data::LEN],
}

impl ItemData {
    pub fn validate(paths: &Paths, crates: &mut Vec<ItemsCrate>) -> Self {
        // Sort in order of crate index
        crates.sort_by_key(|cr| cr.cr_idx);

        let traits = [
            paths.get_global(EngineGlobals::CFoo),
            paths.get_global(EngineGlobals::EFoo),
        ]
        .map(|path| Global {
            path: path.to_owned(),
            args: GlobalMacroArgs {
                is_dummy: false,
                is_const: false,
                is_trait: true,
            },
        })
        .to_vec();

        // Collect items
        let items = ItemList::from(crates, &traits);

        let data = Data::VARIANTS
            .map(|dv| match dv {
                Data::Components => items
                    .components
                    .map_vec(|v| v.join_map(|c| c.path.path[1..].join("::"), ",")),
                Data::Globals => items
                    .globals
                    .map_vec(|v| v.join_map(|g| g.path.path[1..].join("::"), ",")),
                Data::Events => items.events.map_vec(|v| {
                    v.join_map(
                        |e| format!("{}({})", e.path.path[1..].join("::"), e.variants.join(",")),
                        ",",
                    )
                }),
                Data::Systems => crates.map_vec(|cr| {
                    cr.systems
                        .join_map(|s| s.validate_to_data(paths, crates, &items), ",")
                }),
                Data::Dependencies => crates.map_vec(|cr| {
                    format!(
                        "{}({})",
                        cr.dir.display(),
                        cr.dependencies
                            .join_map(|d| format!("{}:{}", d.cr_alias, d.cr_idx), ",")
                    )
                }),
                Data::EngineTraits => crates.map_vec(|cr| {
                    paths
                        .traits
                        .join_map(|p| p.path_from(cr.cr_idx, crates).join("::"), ",")
                }),
                Data::EngineGlobals => vec![paths.globals.join_map(
                    |p| {
                        format!(
                            "{}_{}",
                            p.cr_idx,
                            items.globals[p.cr_idx]
                                .iter()
                                .position(|g| &g.path == p)
                                .catch(format!(
                                    "Could not find global: {} in crate {}",
                                    p.path.join("::"),
                                    p.cr_idx
                                ))
                        )
                    },
                    ",",
                )],
                Data::CratePaths => crates.map_vec(|cr| {
                    format!(
                        "{}",
                        Path {
                            cr_idx: cr.cr_idx,
                            path: vec![if cr.cr_idx == 0 {
                                "crate".to_string()
                            } else {
                                cr.cr_name.to_string()
                            }],
                        }
                        .path_from(0, crates)
                        .join("::")
                    )
                }),
            })
            .map(|v| {
                if let Some(s) = v.iter().find(|s| s.contains(SEP)) {
                    panic!("Found separator \"{}\" in data string: \"{}\"", SEP, s)
                }
                v.join(SEP)
            });

        Self { data }
    }

    pub fn write_to_file(&self) {
        fs::write(std::env::temp_dir().join(DATA_FILE), self.data.join("\n"))
            .expect("Could not write to data file");
    }
}
