use crate::{
    resolve::{
        ast_args::GlobalMacroArgs,
        ast_items::{Global, ItemsCrate, System},
        ast_resolve::Path,
    },
    util::JoinMap,
};

impl System {
    pub fn validate_to_data(&self) -> String {
        String::new()
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
    pub fn validate(crates: &Vec<ItemsCrate>) -> Self {
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

        let [components_data, globals_data, events_data, systems_data, dependencies_data] = crates
            .iter()
            .fold(
                [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()],
                |[mut cd, mut gd, mut ed, mut sd, mut dd], cr| {
                    cd.push(format!(
                        "{}({})",
                        cr.cr_idx,
                        cr.components.join_map(|c| c.path.path.join("::"), ",")
                    ));
                    gd.push(format!(
                        "{}({})",
                        cr.cr_idx,
                        cr.globals.join_map(|g| g.path.path.join("::"), ",")
                    ));
                    ed.push(format!(
                        "{}({})",
                        cr.cr_idx,
                        cr.events.join_map(|e| e.path.path.join("::"), ",")
                    ));
                    sd.push(format!(
                        "{}({})",
                        cr.cr_idx,
                        cr.systems.join_map(|s| s.validate_to_data(), ",")
                    ));
                    dd.push(format!(
                        "{}:{}({})",
                        cr.cr_name,
                        cr.cr_idx,
                        cr.dependencies
                            .join_map(|d| format!("{}:{}", d.name, d.cr_idx), ",")
                    ));
                    [cd, gd, ed, sd, dd]
                },
            )
            .map(|v| v.join(" "));

        Self {
            components_data,
            globals_data,
            events_data,
            systems_data,
            dependencies_data,
        }
    }
}
