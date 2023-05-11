use std::{array, fs, path::PathBuf};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use regex::Regex;

use crate::{
    resolve::ast_paths::{EnginePaths, NUM_ENGINE_PATHS},
    util::{Catch, SplitCollect},
    validate::constants::{component_var, DATA_FILE, NAMESPACE, SEP},
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

        let crates = self.data[Data::Dependencies as usize]
            .iter()
            .map(|deps| {
                full_regex
                    .captures(deps)
                    .and_then(|c| c.name("file").zip(c.name("deps")))
                    .map(|(path, deps)| {
                        (
                            PathBuf::from(path.as_str()),
                            deps.as_str().split_collect(","),
                        )
                    })
                    .catch(format!("Could not parse dependency: {}", deps))
            })
            .collect::<Vec<_>>();

        crates
            .iter()
            .enumerate()
            .find_map(|(i, (path, deps))| {
                (&dir == path).then(|| {
                    (
                        i,
                        deps.iter()
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

    fn get_engine_paths(&self, cr_idx: usize) -> [syn::Type; NUM_ENGINE_PATHS] {
        let mut data = self.data[Data::EnginePaths as usize][cr_idx].split(",");
        array::from_fn(|i| {
            syn::parse_str(
                data.next()
                    .catch(format!("Could not get engine path: {}", i)),
            )
            .catch(format!("Could not parse engine type: {}", i))
        })
    }

    fn codegen_dep(&self, cr_idx: usize, deps: Vec<Dependency>) -> TokenStream {
        let engine_paths = self.get_engine_paths(cr_idx);
        let ns = format_ident!("{}", NAMESPACE);
        let dep_aliases = deps
            .iter()
            .map(|d| format_ident!("{}", d.alias))
            .collect::<Vec<_>>();

        // Aggregate AddComponent traits and dependencies
        let add_comp = format_ident!("{}", EnginePaths::AddComponent.get_type());
        let add_comp_tr = &engine_paths[EnginePaths::AddComponent as usize];
        let comp_traits = match self
            .get_components(cr_idx, "crate".to_string())
            .into_iter()
            .map(|c| c.ty)
            .collect::<Vec<_>>()
            .split_first()
        {
            Some((first, tail)) => {
                quote!(pub trait #add_comp: #add_comp_tr<#first> #(+#add_comp_tr<#tail>)* #(+#dep_aliases::#ns::#add_comp)* {})
            }
            None => quote!(
                pub trait #add_comp {}
            ),
        };

        // TODO:
        // Aggregate AddEvent traits and dependencies

        quote!(
            pub mod #ns {
                #(pub use #dep_aliases;)*
                #comp_traits
            }
        )
    }

    fn codegen_entry(&self) -> TokenStream {
        quote!()
    }

    pub fn codegen(&self, dir: PathBuf) -> TokenStream {
        let (cr_idx, deps) = self.get_dependencies(dir);
        match cr_idx {
            0 => self.codegen_entry(),
            _ => self.codegen_dep(cr_idx, deps),
        }
    }
}
