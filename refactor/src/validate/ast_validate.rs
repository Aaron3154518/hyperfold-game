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
        ast_paths::Paths,
        ast_resolve::Path,
    },
    util::{end, Catch, JoinMap, NoneOr, SplitIter},
};

use super::{
    ast_item_list::ItemList,
    constants::{DATA_FILE, SEP},
};

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

        let [components_data, globals_data, events_data, systems_data, dependencies_data] = [
            items
                .components
                .iter()
                .map(|v| v.join_map(|c| c.path.root_path(crates)[1..].join("::"), ","))
                .collect(),
            items
                .globals
                .iter()
                .map(|v| v.join_map(|g| g.path.root_path(crates)[1..].join("::"), ","))
                .collect(),
            items
                .events
                .iter()
                .map(|v| {
                    v.join_map(
                        |e| {
                            format!(
                                "{}({})",
                                e.path.root_path(crates)[1..].join("::"),
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
                    format!(
                        "{}:{}",
                        cr.dir.display(),
                        cr.dependencies
                            .join_map(|d| format!("{}:{}", d.cr_alias, d.cr_idx), ",")
                    )
                })
                .collect(),
        ]
        .map(|v: Vec<_>| {
            if let Some(s) = v.iter().find(|s| s.contains(SEP)) {
                panic!("Found separator \"{}\" in data string: \"{}\"", SEP, s)
            }
            v.join(SEP)
        });

        Self {
            components_data,
            globals_data,
            events_data,
            systems_data,
            dependencies_data,
        }
    }

    pub fn write_to_file(&self) {
        fs::write(
            std::env::temp_dir().join(DATA_FILE),
            format!(
                "{}\n{}\n{}\n{}\n{}",
                &self.components_data,
                &self.globals_data,
                &self.events_data,
                &self.systems_data,
                &self.dependencies_data,
            ),
        )
        .expect("Could not write to data file");
    }
}
