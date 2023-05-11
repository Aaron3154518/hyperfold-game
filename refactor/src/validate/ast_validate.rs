use std::cmp::min;

use crate::{
    parse::ast_fn_arg::{FnArg, FnArgType},
    resolve::{
        ast_args::{ComponentMacroArgs, GlobalMacroArgs},
        ast_items::{Component, Dependency, Event, Global, ItemsCrate, System},
        ast_paths::Paths,
        ast_resolve::Path,
    },
    util::{end, Catch, JoinMap, NoneOr, SplitIter},
};

use super::ast_item_list::ItemList;

// Pass 3: Item validation
// Map system arg paths to items
#[derive(Debug)]
pub struct ItemData {
    pub components_data: String,
    pub globals_data: String,
    pub events_data: String,
    pub systems_data: String,
    pub dependencies_data: String,
}

impl ItemData {
    pub fn validate(paths: &Paths, crates: &mut Vec<ItemsCrate>) -> Self {
        // Sort in order of crate index
        crates.sort_by_key(|cr| cr.cr_idx);

        let traits = [vec!["crate", "CFoo"], vec!["crate", "EFoo"]]
            .map(|path| Global {
                path: Path {
                    cr_idx: 0,
                    path: path.iter().map(|s| s.to_string()).collect(),
                },
                args: GlobalMacroArgs {
                    is_dummy: false,
                    is_const: false,
                    is_trait: true,
                },
            })
            .to_vec();

        // Collect items
        let items = ItemList::from(crates, &traits);

        let sep = ";";
        let [components_data, globals_data, events_data, systems_data, dependencies_data] = [
            items
                .components
                .iter()
                .map(|v| v.join_map(|c| c.path.root_path(crates).join("::"), ","))
                .collect(),
            items
                .globals
                .iter()
                .map(|v| v.join_map(|g| g.path.root_path(crates).join("::"), ","))
                .collect(),
            items
                .events
                .iter()
                .map(|v| {
                    v.join_map(
                        |e| {
                            format!(
                                "{}({})",
                                e.path.root_path(crates).join("::"),
                                e.variants.join(",")
                            )
                        },
                        ",",
                    )
                })
                .collect(),
            crates
                .iter()
                .map(|cr| {
                    cr.systems
                        .join_map(|s| s.validate_to_data(paths, crates, &items), ",")
                })
                .collect(),
            crates
                .iter()
                .map(|cr| {
                    cr.dependencies
                        .join_map(|d| format!("{}:{}", d.cr_alias, d.cr_idx), ",")
                })
                .collect(),
        ]
        .map(|v: Vec<_>| {
            if let Some(s) = v.iter().find(|s| s.contains(sep)) {
                panic!("Found separator \"{}\" in data string: \"{}\"", sep, s)
            }
            v.join(sep)
        });

        Self {
            components_data,
            globals_data,
            events_data,
            systems_data,
            dependencies_data,
        }
    }
}
