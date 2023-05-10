use std::cmp::min;

use crate::{
    parse::ast_fn_arg::FnArg,
    resolve::{
        ast_args::GlobalMacroArgs,
        ast_items::{Global, ItemsCrate, System},
        ast_resolve::Path,
    },
    util::{JoinMap, NoneOr},
};

impl Path {
    pub fn root_path(&self, crates: &Vec<ItemsCrate>) -> Vec<String> {
        if self.cr_idx == 0 {
            return self.path.to_vec();
        }

        // Dijkstra's with weight = length of crate name
        let (mut frontier, mut visited) = (vec![(vec![0], 0)], vec![0]);
        loop {
            let mut min_path: Option<_> = None;
            for (path, score) in frontier.iter() {
                let cr_idx = *path.last().expect("Empty path in root_path()");
                if cr_idx == self.cr_idx {
                    return [
                        // Use crate instead of first crate name
                        vec!["crate".to_string()],
                        path[1..]
                            .iter()
                            .map(|cr_idx| crates[*cr_idx].cr_name.to_string())
                            .collect::<Vec<_>>(),
                        // Trim off crate name
                        self.path[1..].to_vec(),
                    ]
                    .concat();
                }
                for d in crates[cr_idx].dependencies.iter() {
                    if !visited.contains(&d.cr_idx)
                        && min_path
                            .is_none_or(|(_, min_score)| *min_score > *score + d.cr_alias.len())
                    {
                        min_path = Some((
                            [path.to_vec(), vec![d.cr_idx]].concat(),
                            *score + d.cr_alias.len(),
                        ))
                    }
                }
            }
            if let Some((new_path, new_score)) = min_path {
                visited.push(*new_path.last().expect("Empty new path in root_path()"));
                frontier.push((new_path, new_score));
            } else {
                panic!(
                    "Could not find path from entry crate to crate {}",
                    self.cr_idx
                );
            }
        }
    }
}

impl FnArg {
    pub fn validate_to_data(&self) -> String {
        String::new()
    }
}

impl System {
    pub fn validate_to_data(&self, crates: &Vec<ItemsCrate>) -> String {
        format!(
            "{}({})",
            self.path.root_path(crates).join("::"),
            self.args.join_map(FnArg::validate_to_data, ",")
        )
    }
}

// Pass 3: Item validation
// Math system arg paths to items
#[derive(Debug)]
pub struct ItemData {
    pub components_data: String,
    pub globals_data: String,
    pub events_data: String,
    pub systems_data: String,
    pub dependencies_data: String,
}

impl ItemData {
    pub fn validate(crates: &mut Vec<ItemsCrate>) -> Self {
        // Collect items
        let (mut cs, mut gs, mut es, mut ss) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for cr in crates.iter() {
            cs.extend(cr.components.iter());
            gs.extend(cr.globals.iter());
            es.extend(cr.events.iter());
            ss.extend(cr.systems.iter());
        }

        // Add traits
        // TODO: make sure cr_idx matches self
        let traits = [vec!["crate", "CFoo"], vec!["crate", "EFoo"]].map(|path| Global {
            path: Path {
                cr_idx: 0,
                path: path.iter().map(|s| s.to_string()).collect(),
            },
            args: GlobalMacroArgs {
                is_dummy: false,
                is_const: false,
                is_trait: true,
            },
        });
        traits.iter().for_each(|g| gs.push(&g));

        // Sort in order of crate index
        crates.sort_by_key(|cr| cr.cr_idx);

        // Don't use '|' in sub data formats
        let [components_data, globals_data, events_data, systems_data, dependencies_data] = crates
            .iter()
            .fold(
                [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()],
                |[mut cd, mut gd, mut ed, mut sd, mut dd], cr| {
                    cd.push(
                        cr.components
                            .join_map(|c| c.path.root_path(crates).join("::"), ","),
                    );
                    gd.push(
                        cr.globals
                            .join_map(|g| g.path.root_path(crates).join("::"), ","),
                    );
                    ed.push(cr.events.join_map(
                        |e| {
                            format!(
                                "{}({})",
                                e.path.root_path(crates).join("::"),
                                e.variants.join(",")
                            )
                        },
                        ",",
                    ));
                    sd.push(cr.systems.join_map(|s| s.validate_to_data(crates), ","));
                    dd.push(
                        cr.dependencies
                            .join_map(|d| format!("{}:{}", d.cr_alias, d.cr_idx), ","),
                    );
                    [cd, gd, ed, sd, dd]
                },
            )
            .map(|v| v.join("|"));

        Self {
            components_data,
            globals_data,
            events_data,
            systems_data,
            dependencies_data,
        }
    }
}
