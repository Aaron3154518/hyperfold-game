use std::{array, fs, path::PathBuf};

use quote::format_ident;
use regex::Regex;

use crate::{
    resolve::ast_paths::{EnginePaths, NUM_ENGINE_PATHS},
    util::{Catch, SplitCollect},
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
    EnginePaths,
}

const ENGINE_PATHS: usize = Data::EnginePaths as usize;

#[derive(Debug)]
pub struct Decoder {
    data: [Vec<String>; 6],
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

    fn get_components(&self, cr_idx: usize, cr_alias: String) -> Vec<Component> {
        match self.get_crate_data(Data::Components, cr_idx).as_str() {
            "" => Vec::new(),
            data => data
                .split(",")
                .enumerate()
                .map(|(i, c)| Component {
                    ty: syn::parse_str(format!("{}::{}", cr_alias, c).as_str())
                        .catch(format!("Could not parse type: {}", c)),
                    var: format_ident!("{}", component_var(cr_idx, i)),
                })
                .collect(),
        }
    }

    fn get_dependencies(&self, dir: PathBuf) -> (usize, Vec<Dependency>) {
        let dir = fs::canonicalize(dir.to_owned())
            .catch(format!("Could not canoncialize path: {}", dir.display()));
        let full_regex = Regex::new(r"(?P<file>[^\()]*)\((?P<deps>(\w+:\d+(,\w+:\d+)*)?)\)")
            .expect("Could not create dependency regex");
        let dep_regex =
            Regex::new(r"(?P<alias>\w+):(?P<idx>\d+)").expect("Could not create dependency regex");
        self.data[Data::Dependencies as usize]
            .iter()
            .enumerate()
            .find_map(|(i, deps)| {
                full_regex
                    .captures(deps)
                    .and_then(|c| c.name("file").zip(c.name("deps")))
                    .and_then(|(path, deps)| {
                        (dir == PathBuf::from(path.as_str())).then(|| {
                            (
                                i,
                                deps.as_str()
                                    .split(",")
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
            })
            .catch(format!("Could not locate dependency: {}", dir.display()))
    }

    fn get_engine_paths(&self) -> [Vec<String>; NUM_ENGINE_PATHS] {
        let data = &self.data[Data::EnginePaths as usize];
        EnginePaths::get_variants().map(|v| data[v as usize].split_collect("::"))
    }

    fn codegen_dep(&self, cr_idx: usize, deps: Vec<Dependency>) -> String {
        let components = self.get_components(cr_idx, "crate".to_string());
        let engine_paths = self.get_engine_paths();
        println!("{:#?}", engine_paths);
        // TODO: How to access engine?

        String::new()
    }

    fn codegen_entry(&self) -> String {
        String::new()
    }

    pub fn codegen(&self, dir: PathBuf) -> String {
        let (cr_idx, deps) = self.get_dependencies(dir);
        println!("{:#?}\n{:#?}", cr_idx, deps);
        match cr_idx {
            0 => self.codegen_entry(),
            _ => self.codegen_dep(cr_idx, deps),
        }
    }
}
