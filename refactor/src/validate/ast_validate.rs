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

const NAMESPACE: &str = "_engine";

impl Path {
    pub fn root_path(&self, crates: &Vec<ItemsCrate>) -> Vec<String> {
        if self.cr_idx == 0 {
            return self.path.to_vec();
        }

        let start_dep = Dependency {
            cr_idx: 0,
            cr_alias: String::new(),
        };

        // Dijkstra's with weight = length of crate name
        let (mut frontier, mut visited) = (vec![(vec![&start_dep], 0)], vec![0]);
        loop {
            let mut min_path = None;
            for (path, score) in frontier.iter() {
                let dep = *path.last().expect("Empty path in root_path()");
                if dep.cr_idx == self.cr_idx {
                    let mut use_path = Vec::new();
                    // Don't include entry crate
                    for d in path[1..].iter() {
                        use_path.push(d.cr_alias.to_string());
                        use_path.push(NAMESPACE.to_string());
                    }
                    use_path.extend(self.path[1..].to_vec().into_iter());
                    return use_path;
                }
                for d in crates[dep.cr_idx].dependencies.iter() {
                    if !visited.contains(&d.cr_idx)
                        && min_path
                            .is_none_or(|(_, min_score)| *min_score > *score + d.cr_alias.len())
                    {
                        min_path = Some((
                            [path.to_vec(), vec![&d]].concat(),
                            *score + d.cr_alias.len(),
                        ))
                    }
                }
            }
            if let Some((new_path, new_score)) = min_path {
                visited.push(
                    new_path
                        .last()
                        .expect("Empty new_path in root_path()")
                        .cr_idx,
                );
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

// TODO: collect all errors
impl FnArg {
    // Convert to data
    fn to_eid(&self, paths: &Paths) -> Option<()> {
        matches!(&self.ty, FnArgType::Path(p) if p == &paths.eid).then_some(())
    }

    fn to_component<'a>(
        &self,
        items: &'a ItemList,
    ) -> Option<(usize, usize, &'a ComponentMacroArgs)> {
        match &self.ty {
            FnArgType::Path(p) => items.components[p.cr_idx]
                .iter()
                .enumerate()
                .find_map(|(i, c)| (&c.path == p).then_some((p.cr_idx, i, &c.args))),
            _ => None,
        }
    }

    fn to_global<'a>(&self, items: &'a ItemList) -> Option<(usize, usize, &'a GlobalMacroArgs)> {
        match &self.ty {
            FnArgType::Path(p) => items.globals[p.cr_idx]
                .iter()
                .enumerate()
                .find_map(|(i, g)| (&g.path == p).then_some((p.cr_idx, i, &g.args))),
            _ => None,
        }
    }

    fn to_trait(&self, items: &ItemList) -> Option<(usize, usize)> {
        match &self.ty {
            FnArgType::Trait(p) => items
                .traits
                .iter()
                .enumerate()
                .find_map(|(idx, tr)| (tr.path.path == p.path).then_some((0, idx))),
            _ => None,
        }
    }

    fn to_event(&self, items: &ItemList) -> Option<(usize, usize, usize)> {
        match &self.ty {
            FnArgType::Path(p) => items.events[p.cr_idx]
                .iter()
                .enumerate()
                .find_map(|(i, e)| {
                    let mut e_path = p.to_owned();
                    e_path.path.pop();
                    (e.path == e_path)
                        .then(|| i)
                        .zip(
                            e.variants
                                .iter()
                                .position(|v| v == p.path.last().expect("Empty argument path")),
                        )
                        .map(|(e_i, v_i)| (p.cr_idx, e_i, v_i))
                }),
            _ => None,
        }
    }

    fn to_label(&self, paths: &Paths) -> Option<(&str, Vec<&FnArg>)> {
        match &self.ty {
            FnArgType::SContainer(p, a) => (p == &paths.label).then(|| ("&", vec![&**a])),
            FnArgType::Container(p, v) => (p == &paths.and_labels)
                .then_some("&")
                .or_else(|| (p == &paths.or_labels).then_some("|"))
                .or_else(|| (p == &paths.nand_labels).then_some("|&"))
                .or_else(|| (p == &paths.nor_labels).then_some("!|"))
                .map(|l_ty| (l_ty, v.iter().collect())),
            _ => None,
        }
    }

    fn to_container(&self, paths: &Paths) -> Option<Vec<&FnArg>> {
        match &self.ty {
            FnArgType::Container(p, v) => (p == &paths.container).then_some(v.iter().collect()),
            _ => None,
        }
    }

    // Validate conditions
    fn validate_ref(&self, should_be_cnt: usize) {
        if self.ref_cnt != should_be_cnt {
            panic!(
                "Type should be taken by {}: \"{}\"",
                if should_be_cnt == 0 {
                    "borrow".to_string()
                } else if should_be_cnt == 1 {
                    "single reference".to_string()
                } else {
                    format!("{} references", should_be_cnt)
                },
                self
            )
        }
    }

    fn validate_mut(&self, should_be_mut: bool) {
        if self.mutable != should_be_mut {
            panic!(
                "Type should be taken {}: \"{}\"",
                if should_be_mut {
                    "mutably"
                } else {
                    "immutably"
                },
                self
            )
        }
    }

    pub fn validate_to_data(&self, paths: &Paths, items: &ItemList) -> String {
        self
            // Entity ID
            .to_eid(paths)
            .map(|_| {
                self.validate_ref(1);
                self.validate_mut(false);
                "id".to_string()
            })
            // Component
            .or_else(|| {
                self.to_component(items).map(|(cr_i, c_i, _)| {
                    self.validate_ref(1);
                    format!("c{}:{}", cr_i, c_i)
                })
            })
            // Global
            .or_else(|| {
                self.to_global(items).map(|(cr_i, g_i, g_args)| {
                    self.validate_ref(1);
                    if g_args.is_const {
                        self.validate_mut(false);
                    }
                    format!("g{}:{}", cr_i, g_i)
                })
            })
            // Event
            .or_else(|| {
                self.to_event(items).map(|(cr_i, e_i, v_i)| {
                    self.validate_ref(1);
                    self.validate_mut(false);
                    format!("e{}:{}:{}", cr_i, e_i, v_i)
                })
            })
            // Trait
            .or_else(|| {
                // This assumes the traits are all at the beginning of the globals list
                self.to_trait(items).map(|(cr_i, g_i)| {
                    self.validate_ref(1);
                    format!("g{}:{}", cr_i, g_i)
                })
            })
            // Label
            .or_else(|| {
                self.to_label(paths).map(|(l_ty, args)| {
                    self.validate_ref(0);
                    format!(
                        "l{}{}",
                        l_ty,
                        args.join_map(
                            |a| a
                                .to_component(items)
                                .map(|(cr_i, c_i, _)| format!("{}:{}", cr_i, c_i))
                                .catch(format!("Label expects Component type, found: {}", a)),
                            ","
                        )
                    )
                })
            })
            // Container
            .or_else(|| {
                self.to_container(paths).map(|args| {
                    self.validate_ref(0);
                    format!(
                        "v{}",
                        args.join_map(
                            |a| a
                                .to_eid(paths)
                                .map(|_| {
                                    a.validate_ref(1);
                                    a.validate_mut(false);
                                    "id".to_string()
                                })
                                .or_else(|| a.to_component(items).map(|(cr_i, c_i, _)| {
                                    a.validate_ref(1);
                                    format!("{}{}:{}", if a.mutable { "m" } else { "" }, cr_i, c_i)
                                }))
                                .catch(format!(
                                    "Container expects Component or Entity ID type, found: {}",
                                    a
                                )),
                            ":"
                        )
                    )
                })
            })
            .catch(format!("Argument: \"{}\" is not a known type", self))
    }
}

impl System {
    pub fn validate_to_data(
        &self,
        paths: &Paths,
        crates: &Vec<ItemsCrate>,
        items: &ItemList,
    ) -> String {
        format!(
            "{}({})",
            self.path.root_path(crates).join("::"),
            self.args
                .join_map(|a| a.validate_to_data(paths, items), ",")
        )
    }
}

// This struct is used to modify the lists of items
// Traits: hardcodes traits
#[derive(Debug)]
pub struct ItemList<'a> {
    pub components: Vec<Vec<&'a Component>>,
    pub globals: Vec<Vec<&'a Global>>,
    pub events: Vec<Vec<&'a Event>>,
    pub traits: Vec<&'a Global>,
}

impl<'a> ItemList<'a> {
    pub fn from(crates: &'a Vec<ItemsCrate>, traits: &'a Vec<Global>) -> Self {
        let (mut components, mut globals, mut events) = (Vec::new(), Vec::new(), Vec::new());
        for cr in crates.iter() {
            components.push(cr.components.iter().collect());
            if cr.cr_idx == 0 {
                let mut v: Vec<&Global> = traits.iter().collect();
                v.extend(cr.globals.iter());
                globals.push(v);
            } else {
                globals.push(cr.globals.iter().collect());
            }
            events.push(cr.events.iter().collect());
        }

        Self {
            components,
            globals,
            events,
            traits: traits.iter().collect(),
        }
    }
}

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
