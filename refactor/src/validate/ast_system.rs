use crate::{
    parse::ast_fn_arg::{FnArg, FnArgType},
    resolve::{
        ast_args::{ComponentMacroArgs, GlobalMacroArgs},
        ast_items::{ItemsCrate, System},
        ast_paths::Paths,
    },
    util::JoinMap,
    validate::constants::{component_var, event_var},
};

use super::{
    ast_item_list::ItemList,
    constants::{global_var, EID},
};

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
    fn validate_ref(&self, should_be_cnt: usize, errs: &mut Vec<String>) {
        if self.ref_cnt != should_be_cnt {
            errs.push(format!(
                "Type should be taken by {}: \"{}\"",
                if should_be_cnt == 0 {
                    "borrow".to_string()
                } else if should_be_cnt == 1 {
                    "single reference".to_string()
                } else {
                    format!("{} references", should_be_cnt)
                },
                self
            ))
        }
    }

    fn validate_mut(&self, should_be_mut: bool, errs: &mut Vec<String>) {
        if self.mutable != should_be_mut {
            errs.push(format!(
                "Type should be taken {}: \"{}\"",
                if should_be_mut {
                    "mutably"
                } else {
                    "immutably"
                },
                self
            ))
        }
    }

    pub fn validate_to_data(
        &self,
        paths: &Paths,
        items: &ItemList,
        errs: &mut Vec<String>,
    ) -> String {
        match self
            // Entity ID
            .to_eid(paths)
            .map(|_| {
                self.validate_ref(1, errs);
                self.validate_mut(false, errs);
                EID.to_string()
            })
            // Component
            .or_else(|| {
                self.to_component(items).map(|(cr_i, c_i, _)| {
                    self.validate_ref(1, errs);
                    component_var(cr_i, c_i)
                })
            })
            // Global
            .or_else(|| {
                self.to_global(items).map(|(cr_i, g_i, g_args)| {
                    self.validate_ref(1, errs);
                    if g_args.is_const {
                        self.validate_mut(false, errs);
                    }
                    global_var(cr_i, g_i)
                })
            })
            // Event
            .or_else(|| {
                self.to_event(items).map(|(cr_i, e_i, v_i)| {
                    self.validate_ref(1, errs);
                    self.validate_mut(false, errs);
                    event_var(cr_i, e_i, v_i)
                })
            })
            // Trait
            .or_else(|| {
                // This assumes the traits are all at the beginning of the globals list
                self.to_trait(items).map(|(cr_i, g_i)| {
                    self.validate_ref(1, errs);
                    global_var(cr_i, g_i)
                })
            })
            // Label
            .or_else(|| {
                self.to_label(paths).map(|(l_ty, args)| {
                    self.validate_ref(0, errs);
                    format!(
                        "l{}{}",
                        l_ty,
                        args.join_map(
                            |a| a.to_component(items).map_or_else(
                                || {
                                    errs.push(format!(
                                        "Label expects Component type, found: {}",
                                        a
                                    ));
                                    String::new()
                                },
                                |(cr_i, c_i, _)| component_var(cr_i, c_i)
                            ),
                            ","
                        )
                    )
                })
            })
            // Container
            .or_else(|| {
                self.to_container(paths).map(|args| {
                    self.validate_ref(0, errs);
                    format!(
                        "v{}",
                        args.join_map(
                            |a| a
                                .to_eid(paths)
                                .map(|_| {
                                    a.validate_ref(1, errs);
                                    a.validate_mut(false, errs);
                                    EID.to_string()
                                })
                                .or_else(|| a.to_component(items).map(|(cr_i, c_i, _)| {
                                    a.validate_ref(1, errs);
                                    format!(
                                        "{}{}",
                                        if a.mutable { "m" } else { "" },
                                        component_var(cr_i, c_i)
                                    )
                                }))
                                .map_or_else(
                                    || {
                                        errs.push(format!(
                                        "Container expects Component or Entity ID type, found: {}",
                                        a
                                    ));
                                        String::new()
                                    },
                                    |s| s
                                ),
                            ":"
                        )
                    )
                })
            }) {
            Some(data) => data,
            None => {
                errs.push(format!("Argument: \"{}\" is not a known type", self));
                String::new()
            }
        }
    }
}

impl System {
    pub fn validate_to_data(
        &self,
        paths: &Paths,
        crates: &Vec<ItemsCrate>,
        items: &ItemList,
    ) -> String {
        let mut errs = Vec::new();
        let data = format!(
            "{}({})",
            self.path.root_path(crates).join("::"),
            self.args
                .join_map(|a| a.validate_to_data(paths, items, &mut errs), ",")
        );
        if !errs.is_empty() {
            panic!(
                "\n\nIn system: \"{}()\"\n{}\n\n",
                self.path.path.join("::"),
                errs.join("\n")
            )
        }
        data
    }
}
