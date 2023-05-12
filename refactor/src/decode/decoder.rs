use std::{array, fs, path::PathBuf};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use regex::Regex;
use syn::{PathArguments, PathSegment};

use crate::{
    resolve::ast_paths::{EnginePaths, NUM_ENGINE_PATHS},
    util::{Catch, JoinMap, JoinMapInto, SplitCollect, SplitIter},
    validate::constants::{component_var, DATA_FILE, NAMESPACE, SEP},
};

use super::{
    component::Component,
    dependency::Dependency,
    event::{self, Event},
};

#[derive(Copy, Clone, Debug)]
pub enum Data {
    Components,
    Globals,
    Events,
    Systems,
    Dependencies,
    EnginePaths,
    CratePaths,
}

pub const NUM_DATA_LINES: usize = 7;

#[derive(Debug)]
pub struct Decoder {
    data: [Vec<String>; NUM_DATA_LINES],
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

    fn get_events(&self, cr_idx: usize, cr_alias: String) -> Vec<Event> {
        let event_regex = Regex::new(r"(?P<type>\w+(::\w+)*)\((?P<varis>(\w+(,\w+)*)?)\)")
            .expect("Could not parse event regex");
        match self.get_crate_data(Data::Events, cr_idx).as_str() {
            "" => Vec::new(),
            data => event_regex
                .captures_iter(data)
                .map(|c| {
                    c.name("type")
                        .zip(c.name("varis"))
                        .map(|(ty, varis)| Event {
                            ty: syn::parse_str(format!("{}::{}", cr_alias, ty.as_str()).as_str())
                                .catch(format!("Could not parse type: {}", ty.as_str())),
                            variants: match varis.as_str() {
                                "" => vec![],
                                varis => varis.split(",").map(|s| format_ident!("{}", s)).collect(),
                            },
                        })
                        .catch(format!("Could not parse event: {}", data))
                })
                .collect(),
        }
    }

    fn get_dependencies(&self, dir: PathBuf) -> Vec<(PathBuf, Vec<Dependency>)> {
        let dir = fs::canonicalize(dir.to_owned())
            .catch(format!("Could not canoncialize path: {}", dir.display()));
        let full_regex = Regex::new(r"(?P<file>[^\()]*)\((?P<deps>(\w+:\d+(,\w+:\d+)*)?)\)")
            .expect("Could not create dependency regex");
        let dep_regex =
            Regex::new(r"(?P<alias>\w+):(?P<idx>\d+)").expect("Could not create dependency regex");

        self.data[Data::Dependencies as usize].map_vec(|deps| {
            full_regex
                .captures(deps)
                .and_then(|c| c.name("file").zip(c.name("deps")))
                .map(|(path, deps)| {
                    (
                        PathBuf::from(path.as_str()),
                        match deps.as_str() {
                            "" => vec![],
                            s => s.split_map(",", |dep| {
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
                            }),
                        },
                    )
                })
                .catch(format!("Could not parse dependency: {}", deps))
        })
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

    fn get_crate_paths(&self) -> Vec<syn::Path> {
        self.data[Data::CratePaths as usize]
            .map_vec(|p| syn::parse_str(&p).catch(format!("Could not parse path: {}", p)))
    }

    fn codegen_dep(&self, cr_idx: usize, deps: &Vec<Vec<Dependency>>) -> TokenStream {
        let engine_paths = self.get_engine_paths(cr_idx);
        let ns = format_ident!("{}", NAMESPACE);
        let dep_aliases = deps[cr_idx]
            .iter()
            .map(|d| format_ident!("{}", d.alias))
            .collect::<Vec<_>>();

        // Aggregate AddComponent traits and dependencies
        let add_comp = format_ident!("{}", EnginePaths::AddComponent.get_type());
        let add_comp_tr = &engine_paths[EnginePaths::AddComponent as usize];
        let mut comp_trs = self
            .get_components(cr_idx, "crate".to_string())
            .into_iter()
            .map(|c| {
                let c_ty = c.ty;
                quote!(#add_comp_tr<#c_ty>)
            })
            .collect::<Vec<_>>();
        comp_trs.append(&mut dep_aliases.map_vec(|da| quote!(#da::#ns::#add_comp)));
        let comp_code = match comp_trs.split_first() {
            Some((first, tail)) => {
                quote!(
                    pub trait #add_comp: #first #(+#tail)* {}
                )
            }
            None => quote!(pub trait #add_comp {}),
        };

        // Aggregate AddEvent traits and dependencies
        let add_event = format_ident!("{}", EnginePaths::AddEvent.get_type());
        let add_event_tr = &engine_paths[EnginePaths::AddEvent as usize];
        let mut event_trs = self
            .get_events(cr_idx, "crate".to_string())
            .into_iter()
            .map(|e| {
                let e_ty = e.ty;
                quote!(#add_event_tr<#e_ty>)
            })
            .collect::<Vec<_>>();
        event_trs.append(&mut dep_aliases.map_vec(|da| quote!(#da::#ns::#add_event)));
        let event_code = match event_trs.split_first() {
            Some((first, tail)) => {
                quote!(
                    pub trait #add_event: #first #(+#tail)* {}
                )
            }
            None => quote!(pub trait #add_event {}),
        };

        quote!(
            #(pub use #dep_aliases;)*
            #comp_code
            #event_code
        )
    }

    fn get_deps_post_order(deps: &Vec<Vec<Dependency>>) -> Vec<usize> {
        let mut v = Vec::new();
        Self::get_deps_post_impl(&mut v, 0, deps);
        v
    }

    fn get_deps_post_impl(list: &mut Vec<usize>, cr_idx: usize, deps: &Vec<Vec<Dependency>>) {
        for d in deps[cr_idx].iter() {
            if !list.contains(&d.cr_idx) {
                Self::get_deps_post_impl(list, d.cr_idx, deps);
            }
        }
        list.push(cr_idx);
    }

    fn codegen_entry(&self, cr_idx: usize, deps: &Vec<Vec<Dependency>>) -> TokenStream {
        // Use statements and traits
        let deps_code = self.codegen_dep(cr_idx, deps);

        let deps_lrn = Self::get_deps_post_order(deps);
        let crate_paths = self.get_crate_paths();
        let crate_paths_post = deps_lrn.map_vec(|i| &crate_paths[*i]);

        let ns = format_ident!("{}", NAMESPACE);
        let engine_paths = self.get_engine_paths(cr_idx);
        let entity_path = &engine_paths[EnginePaths::Entity as usize];

        // Component manager
        let add_comp = format_ident!("{}", EnginePaths::AddComponent.get_type());
        let add_comp_tr = &engine_paths[EnginePaths::AddComponent as usize];
        let (c_vars, c_tys) = crate_paths.iter().enumerate().fold(
            (Vec::new(), Vec::new()),
            |(mut vars, mut tys), (cr_idx, cr_path)| {
                self.get_components(cr_idx, String::new())
                    .into_iter()
                    .map(|c| {
                        let c_ty = c.ty;
                        (c.var, quote!(#cr_path #c_ty))
                    })
                    .unzip()
                    .split_into(|mut vs, mut ts| {
                        vars.append(&mut vs);
                        tys.append(&mut ts);
                    });
                (vars, tys)
            },
        );
        // TODO
        let cfoo_def = quote!();

        // Event manager
        let add_event = format_ident!("{}", EnginePaths::AddEvent.get_type());
        let add_event_tr = &engine_paths[EnginePaths::AddEvent as usize];
        let (e_vars, e_tys) = crate_paths.iter().enumerate().fold(
            (Vec::new(), Vec::new()),
            |(mut vars, mut tys), (cr_idx, cr_path)| {
                for (e_i, e) in self
                    .get_events(cr_idx, String::new())
                    .into_iter()
                    .enumerate()
                {
                    let e_ty = e.ty;
                    for (v_i, v) in e.variants.iter().enumerate() {
                        vars.push(format_ident!("e{}_{}_{}", cr_idx, e_i, v_i));
                        tys.push(quote!(#cr_path #e_ty::#v));
                    }
                }
                (vars, tys)
            },
        );
        let enum_ident = format_ident!("E");
        // TODO
        let efoo_def = quote!();

        quote!(
            #deps_code
            #cfoo_def
            #(
                impl #add_comp_tr<#c_tys> for CFoo {
                    fn add_component(&mut self, e: #entity_path, t: #c_tys) {
                        self.#c_vars.insert(e, t)
                    }
                }
            )*
            #(
                impl #crate_paths_post::#ns::#add_comp for CFoo {}
            )*
            enum #enum_ident {
                #(#e_vars),*
            }
            #efoo_def
            #(
                impl #add_event_tr<#e_tys> for EFoo {
                    fn new_event(&mut self, t: #e_tys) {
                        self.#e_vars.push(t);
                        self.add_event(#enum_ident::#e_vars);
                    }

                    fn get_event<'a>(&'a self) -> Option<&'a #e_tys> {
                        self.#e_vars.last()
                    }
                }
            )*
            #(
                impl #crate_paths_post::#ns::#add_event for EFoo {}
            )*
        )
    }

    pub fn codegen(&self, dir: PathBuf) -> TokenStream {
        let ns: proc_macro2::Ident = format_ident!("{}", NAMESPACE);
        let dir = fs::canonicalize(dir.to_owned())
            .catch(format!("Could not canonicalize path: {}", dir.display()));
        let deps = self.get_dependencies(dir.to_owned());
        let cr_idx = deps
            .iter()
            .position(|(path, deps)| &dir == path)
            .catch(format!("Could not locate dependency: {}", dir.display()));
        let deps = deps.into_iter().map_vec(|(_, d)| d);
        let code = match cr_idx {
            0 => self.codegen_entry(cr_idx, &deps),
            _ => self.codegen_dep(cr_idx, &deps),
        };
        quote!(
            pub mod #ns {
                #code
            }
        )
    }
}
