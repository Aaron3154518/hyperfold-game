use std::{array, fs, path::PathBuf};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use regex::Regex;
use syn::{PathArguments, PathSegment};

use crate::{
    decode::dependency::get_deps_post_order,
    resolve::ast_paths::{CrateEnginePaths, EntryEnginePaths},
    util::{Catch, JoinMap, JoinMapInto, SplitCollect, SplitIter},
    validate::{
        ast_validate::Data,
        constants::{component_var, event_var, global_var, DATA_FILE, NAMESPACE, SEP},
    },
};

use super::{
    component::Component,
    dependency::Dependency,
    event::{self, Event},
    idents::Idents,
};

#[derive(Debug)]
pub struct Decoder {
    data: [Vec<String>; Data::len()],
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

    fn get_components(&self, cr_idx: usize, cr_alias: String, data_ty: Data) -> Vec<Component> {
        let var_fn = match data_ty {
            Data::Components => component_var,
            Data::Globals => global_var,
            _ => panic!("Expected components or globals data, found: {:#?}", data_ty),
        };
        match self.get_crate_data(data_ty, cr_idx).as_str() {
            "" => Vec::new(),
            data => data
                .split(",")
                .enumerate()
                .map(|(i, c)| Component {
                    ty: syn::parse_str(format!("{}::{}", cr_alias, c).as_str())
                        .catch(format!("Could not parse type: {}", c)),
                    var: format_ident!("{}", var_fn(cr_idx, i)),
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

    fn get_crate_engine_paths(&self, cr_idx: usize) -> [syn::Type; CrateEnginePaths::len()] {
        let mut data = self.data[Data::CrateEnginePaths as usize][cr_idx].split(",");
        array::from_fn(|i| {
            syn::parse_str(
                data.next()
                    .catch(format!("Could not get engine path: {}", i)),
            )
            .catch(format!("Could not parse engine type: {}", i))
        })
    }

    fn get_entry_engine_paths(&self) -> [syn::Type; EntryEnginePaths::len()] {
        let mut data = self.data[Data::EntryEnginePaths as usize][0].split(",");
        array::from_fn(|i| {
            syn::parse_str(
                data.next()
                    .catch(format!("Could not get entry engine path: {}", i)),
            )
            .catch(format!("Could not parse engine type: {}", i))
        })
    }

    fn get_crate_paths(&self) -> Vec<syn::Path> {
        self.data[Data::CratePaths as usize]
            .map_vec(|p| syn::parse_str(&p).catch(format!("Could not parse path: {}", p)))
    }

    fn codegen_dep(&self, cr_idx: usize, deps: &Vec<Vec<Dependency>>) -> TokenStream {
        let engine_paths = self.get_crate_engine_paths(cr_idx);
        let ns = format_ident!("{}", NAMESPACE);
        let dep_aliases = deps[cr_idx]
            .iter()
            .map(|d| format_ident!("{}", d.alias))
            .collect::<Vec<_>>();

        // Aggregate AddComponent traits and dependencies
        let add_comp = Idents::AddComponent.to_ident();
        let add_comp_tr = &engine_paths[CrateEnginePaths::AddComponent as usize];
        let mut comp_trs = self
            .get_components(cr_idx, "crate".to_string(), Data::Components)
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
        let add_event = Idents::AddEvent.to_ident();
        let add_event_tr = &engine_paths[CrateEnginePaths::AddEvent as usize];
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

    fn codegen_entry(&self, cr_idx: usize, deps: &Vec<Vec<Dependency>>) -> TokenStream {
        // Use statements and traits
        let deps_code = self.codegen_dep(cr_idx, deps);

        let deps_lrn = get_deps_post_order(deps);
        let crate_paths = self.get_crate_paths();
        let crate_paths_post = deps_lrn.map_vec(|i| &crate_paths[*i]);

        let ns = Idents::Namespace.to_ident();
        let crate_engine_paths = self.get_crate_engine_paths(cr_idx);
        let entry_engine_paths = self.get_entry_engine_paths();
        let entity = &entry_engine_paths[EntryEnginePaths::Entity as usize];
        let entity_trash = &entry_engine_paths[EntryEnginePaths::EntityTrash as usize];

        // Get all globals and components
        let [(c_vars, c_tys), (g_vars, g_tys)] = [Data::Components, Data::Globals].map(|data_ty| {
            crate_paths.iter().enumerate().fold(
                (Vec::new(), Vec::new()),
                |(mut vars, mut tys), (cr_idx, cr_path)| {
                    self.get_components(cr_idx, String::new(), data_ty)
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
            )
        });

        // Global manager
        let gfoo_ident = Idents::GFoo.to_ident();
        let gfoo_def = quote!(
            pub struct #gfoo_ident {
                #(#g_vars: #g_tys),*
            }

            impl #gfoo_ident {
                pub fn new() -> Self {
                    Self {
                        #(#g_vars: #g_tys::new(),)*
                    }
                }
            }
        );

        // Component manager
        let add_comp = Idents::AddComponent.to_ident();
        let add_comp_tr = &crate_engine_paths[CrateEnginePaths::AddComponent as usize];
        let cfoo_ident = Idents::CFoo.to_ident();
        let cfoo_def = quote!(
            pub struct #cfoo_ident {
                eids: std::collections::HashSet<#entity>,
                #(#c_vars: std::collections::HashMap<#entity, #c_tys>,)*
            }

            impl #cfoo_ident {
                pub fn new() -> Self {
                    Self {
                        eids: std::collections::HashSet::new(),
                        #(#c_vars: std::collections::HashMap::new(),)*
                    }
                }

                pub fn append(&mut self, cm: &mut #cfoo_ident) {
                    self.eids.extend(cm.eids.drain());
                    #(self.#c_vars.extend(cm.#c_vars.drain());)*
                }

                pub fn remove(&mut self, tr: &mut #entity_trash) {
                    for eid in tr.0.drain(..) {
                        self.eids.remove(&eid);
                        #(self.#c_vars.remove(&eid);)*
                    }
                }
            }
        );
        let cfoo_traits = quote!(
            #(
                impl #add_comp_tr<#c_tys> for #cfoo_ident {
                    fn add_component(&mut self, e: #entity, t: #c_tys) {
                        self.#c_vars.insert(e, t)
                    }
                }
            )*
            #(
                impl #crate_paths_post::#ns::#add_comp for #cfoo_ident {}
            )*
        );

        // Event manager
        let add_event = Idents::AddEvent.to_ident();
        let add_event_tr = &crate_engine_paths[CrateEnginePaths::AddEvent as usize];
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
                        vars.push(event_var(cr_idx, e_i, v_i));
                        tys.push(quote!(#cr_path #e_ty::#v));
                    }
                }
                (vars, tys)
            },
        );
        let e_ident = Idents::E.to_ident();
        let e_len_ident = Idents::ELen.to_ident();
        let e_len = e_vars.len();
        let e_def = quote!(
            #[derive(Hash, Clone, Copy, Eq, PartialEq, Debug)]
            enum #e_ident {
                #(#e_vars),*
            }
            pub const #e_len_ident: usize = #e_len;
        );
        let efoo_ident = Idents::EFoo.to_ident();
        let efoo_def = quote!(
            #[derive(Debug)]
            pub struct #efoo_ident {
                #(#e_vars: Vec<#e_tys>),*,
                events: std::collections::VecDeque<(E, usize)>
            }

            impl #efoo_ident {
                pub fn new() -> Self {
                    Self {
                        #(#e_vars: Vec::new()),*,
                        events: std::collections::VecDeque::new()
                    }
                }

                pub fn has_events(&self) -> bool {
                    !self.events.is_empty()
                }

                fn add_event(&mut self, e: #e_ident) {
                    self.events.push_back((e, 0));
                }

                pub fn get_events(&mut self) -> std::collections::VecDeque<(#e_ident, usize)> {
                    std::mem::replace(&mut self.events, std::collections::VecDeque::new())
                }

                pub fn append(&mut self, other: &mut Self) {
                    #(
                        other.#e_vars.reverse();
                        self.#e_vars.append(&mut other.#e_vars);
                    )*
                }

                pub fn pop(&mut self, e: #e_ident) {
                    match e {
                        #(
                            #e_ident::#e_vars => {
                                self.#e_vars.pop();
                            }
                        )*
                    }
                }
            }
        );
        let efoo_traits = quote!(
            #(
                impl #add_event_tr<#e_tys> for #efoo_ident {
                    fn new_event(&mut self, t: #e_tys) {
                        self.#e_vars.push(t);
                        self.add_event(#e_ident::#e_vars);
                    }

                    fn get_event<'a>(&'a self) -> Option<&'a #e_tys> {
                        self.#e_vars.last()
                    }
                }
            )*
            #(
                impl #crate_paths_post::#ns::#add_event for #efoo_ident {}
            )*
        );

        quote!(
            #deps_code

            #gfoo_def

            #cfoo_def
            #cfoo_traits

            #e_def
            #efoo_def
            #efoo_traits
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
