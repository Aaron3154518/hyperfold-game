use std::{array, fs, path::PathBuf};

use quote::format_ident;
use regex::Regex;

use crate::{
    util::Catch,
    validate::constants::{component_var, DATA_FILE, SEP},
};

use super::{component::Component, dependency::Dependency};

#[derive(Copy, Clone, Debug)]
pub enum Data {
    Components,
    Globals,
    Events,
    Systems,
    Dependencies,
}

#[derive(Debug)]
pub struct Decoder {
    data: [Vec<String>; 5],
}

impl Decoder {
    pub fn new() -> Self {
        let data = fs::read_to_string(std::env::temp_dir().join(DATA_FILE))
            .expect("Could not read data file");
        let mut lines = data.split("\n");
        Self {
            data: array::from_fn(|i| {
                lines
                    .next()
                    .catch(format!("Could not get data line: {}", i))
                    .split(SEP)
                    .map(|s| s.to_string())
                    .collect()
            }),
        }
    }

    fn get_crate_data(&self, data: Data, cr_idx: usize) -> &String {
        self.data[data as usize]
            .get(cr_idx)
            .catch(format!("Invalid crate index: {}", cr_idx))
    }

    fn get_dependencies(&self, dir: PathBuf) -> (usize, Vec<Dependency>) {
        let dir = fs::canonicalize(dir.to_owned())
            .catch(format!("Could not canoncialize path: {}", dir.display()));
        let dep_regex =
            Regex::new(r"(?P<alias>\w+):(?P<idx>\d+)").expect("Could not crate dependency regex");
        self.data[Data::Dependencies as usize]
            .iter()
            .enumerate()
            .find_map(|(i, deps)| {
                deps.strip_prefix(format!("{}:", dir.display()).as_str())
                    .map(|deps| {
                        (
                            i,
                            deps.split(",")
                                .map(|dep| {
                                    dep_regex
                                        .captures(dep)
                                        .and_then(|c| c.name("alias").zip(c.name("idx")))
                                        .map(|(alias, idx)| Dependency {
                                            cr_idx: idx
                                                .as_str()
                                                .parse()
                                                .expect("Could not parse dependency index"),
                                            alias: alias.as_str().to_string(),
                                        })
                                        .catch(format!("Could not parse dependency: {}", dep))
                                })
                                .collect(),
                        )
                    })
            })
            .catch(format!("Could not locate dependency: {}", dir.display()))
    }

    fn get_components(&self, cr_idx: usize, cr_alias: String) -> Vec<Component> {
        self.get_crate_data(Data::Components, cr_idx)
            .split(",")
            .enumerate()
            .map(|(i, c)| Component {
                ty: syn::parse_str(format!("{}::{}", cr_alias, c).as_str())
                    .catch(format!("Could not parse type: {}", c)),
                var: format_ident!("{}", component_var(cr_idx, i)),
            })
            .collect()
    }

    fn codegen_dep(&self, cr_idx: usize, deps: Vec<Dependency>) -> String {
        let components = self.get_components(cr_idx, "crate".to_string());
        // TODO: How to access engine?

        String::new()
    }

    fn codegen_entry(&self) -> String {
        String::new()
    }

    pub fn codegen(&self, dir: PathBuf) -> String {
        let (cr_idx, deps) = self.get_dependencies(dir);
        match cr_idx {
            0 => self.codegen_entry(),
            _ => self.codegen_dep(cr_idx, deps),
        }
    }
}
