use std::{array, fs, path::PathBuf};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use regex::Regex;
use syn::{PathArguments, PathSegment};

use crate::{
    decode::dependency::get_deps_post_order,
    resolve::ast_paths::{EngineGlobals, EngineTraits, ExpandEnum, GetPaths},
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
    system::{ContainerArg, FnArg, LabelType, System, SystemRegexes},
};

#[derive(Debug)]
pub struct Decoder {
    data: [Vec<String>; Data::LEN],
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

    fn get_systems(&self, cr_idx: usize, cr_path: &syn::Path) -> Vec<System> {
        let regexes = SystemRegexes::new();

        match self.get_crate_data(Data::Systems, cr_idx).as_str() {
            "" => Vec::new(),
            data => data.split_map(",", |s| System::parse(cr_path, s, &regexes)),
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

    fn get_engine_trait_paths(&self, cr_idx: usize) -> [syn::Type; EngineTraits::LEN] {
        let mut data = self.data[Data::EngineTraits as usize][cr_idx].split(",");
        array::from_fn(|i| {
            syn::parse_str(
                data.next()
                    .catch(format!("Could not get engine path: {}", i)),
            )
            .catch(format!("Could not parse engine type: {}", i))
        })
    }

    fn get_engine_globals(&self) -> [(usize, usize); EngineGlobals::LEN] {
        let mut data = self.data[Data::EngineGlobals as usize][0].split(",");
        array::from_fn(|i| {
            let str = data
                .next()
                .catch(format!("Could not parse engine global data: {}", i));
            str.split_once("_")
                .catch(format!("Could not parse engine global: {}", str))
                .split_into(|cr_idx, g_i| {
                    (
                        cr_idx
                            .parse::<usize>()
                            .catch(format!("Invalid engine global crate index: {}", cr_idx)),
                        g_i.parse::<usize>()
                            .catch(format!("Invalid engine global index: {}", g_i)),
                    )
                })
        })
    }

    fn get_crate_paths(&self) -> Vec<syn::Path> {
        self.data[Data::CratePaths as usize]
            .map_vec(|p| syn::parse_str(&p).catch(format!("Could not parse path: {}", p)))
    }

    fn codegen_dep(&self, cr_idx: usize, deps: &Vec<Vec<Dependency>>) -> TokenStream {
        let engine_paths = self.get_engine_trait_paths(cr_idx);
        let ns = format_ident!("{}", NAMESPACE);
        let dep_aliases = deps[cr_idx]
            .iter()
            .map(|d| format_ident!("{}", d.alias))
            .collect::<Vec<_>>();

        // Aggregate AddComponent traits and dependencies
        let add_comp = Idents::AddComponent.to_ident();
        let add_comp_tr = &engine_paths[EngineTraits::AddComponent as usize];
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
        let add_event_tr = &engine_paths[EngineTraits::AddEvent as usize];
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
        // Paths from entry crate to all other crates
        let crate_paths = self.get_crate_paths();
        let crate_paths_post = deps_lrn.map_vec(|i| &crate_paths[*i]);

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

        // Namespace identifier
        let ns = Idents::Namespace.to_ident();
        // Paths engine traits
        let engine_trait_paths = self.get_engine_trait_paths(cr_idx);

        let engine_globals = self.get_engine_globals();
        // Paths to engine globals
        let [gp_entity, gp_entity_trash] =
            [EngineGlobals::Entity, EngineGlobals::EntityTrash].map(|eg| {
                engine_globals[eg as usize].split(|cr_idx, _| {
                    let path = &crate_paths[*cr_idx];
                    let ident = format_ident!("{}", EngineGlobals::Entity.as_str());
                    quote!(#path::#ident)
                })
            });

        // Component manager
        let add_comp = Idents::AddComponent.to_ident();
        let add_comp_tr = &engine_trait_paths[EngineTraits::AddComponent as usize];
        let cfoo_ident = Idents::CFoo.to_ident();
        let cfoo_def = quote!(
            pub struct #cfoo_ident {
                eids: std::collections::HashSet<#gp_entity>,
                #(#c_vars: std::collections::HashMap<#gp_entity, #c_tys>,)*
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

                pub fn remove(&mut self, tr: &mut #gp_entity_trash) {
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
                    fn add_component(&mut self, e: #gp_entity, t: #c_tys) {
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
        let add_event_tr = &engine_trait_paths[EngineTraits::AddEvent as usize];
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
                        vars.push(format_ident!("{}", event_var(cr_idx, e_i, v_i)));
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
                events: std::collections::VecDeque<(#e_ident, usize)>
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

        // Variables for engine globals
        let engine_globals =
            engine_globals.map(|(cr_idx, g_i)| format_ident!("{}", global_var(cr_idx, g_i)));
        let g_event = &engine_globals[EngineGlobals::Event as usize];
        let g_render_system = &engine_globals[EngineGlobals::RenderSystem as usize];
        let g_entity_trash = &engine_globals[EngineGlobals::EntityTrash as usize];
        let g_screen = &engine_globals[EngineGlobals::Screen as usize];
        let g_camera = &engine_globals[EngineGlobals::Camera as usize];
        let g_efoo = &engine_globals[EngineGlobals::EFoo as usize];
        let g_cfoo = &engine_globals[EngineGlobals::CFoo as usize];

        // Systems
        let systems =
            crate_paths
                .iter()
                .enumerate()
                .fold(Vec::new(), |mut v, (cr_idx, cr_path)| {
                    v.append(&mut self.get_systems(cr_idx, cr_path));
                    v
                });
        let init_systems_code = systems
            .iter()
            .filter(|s| s.is_init)
            .map(|s| s.to_quote())
            .collect::<Vec<_>>();
        let systems_code = systems
            .iter()
            .filter(|s| !s.is_init)
            .map(|s| s.to_quote())
            .collect::<Vec<_>>();

        let [cfoo, gfoo, efoo] =
            [Idents::GenCFoo, Idents::GenGFoo, Idents::GenEFoo].map(|i| i.to_ident());

        // Systems manager
        let sfoo_ident = Idents::SFoo.to_ident();
        let sfoo_def = quote!(
            pub struct #sfoo_ident {
                #cfoo: #gfoo_ident,
                #gfoo: #cfoo_ident,
                #efoo: #efoo_ident,
                stack: Vec<std::collections::VecDeque<(#e_ident, usize)>>,
                services: [Vec<Box<dyn Fn(&mut #cfoo_ident, &mut #gfoo_ident, &mut #efoo_ident)>>; #e_len_ident]
            }

            impl #sfoo_ident {
                pub fn new() -> Self {
                    let mut s = Self {
                        #cfoo: #cfoo_ident::new(),
                        #gfoo: #gfoo_ident::new(),
                        #efoo: #efoo_ident::new(),
                        stack: Vec::new(),
                        services: crate::ecs::shared::array_creator::ArrayCreator::create(|_| Vec::new())
                    };
                    s.init();
                    sevents
                }

                // Init
                fn init(&mut self) {
                    #(#init_systems_code)*
                    self.post_tick();
                    self.add_systems();
                }

                fn add_system(&mut self, e: #e_ident, f: Box<dyn Fn(&mut #cfoo_ident, &mut #gfoo_ident, &mut #efoo_ident)>) {
                    self.services[e as usize].push(f);
                }

                fn add_systems(&mut self) {
                    #(#systems_code)*
                }

                // Tick
                pub fn run(&mut self) {
                    static FPS: u32 = 60;
                    static FRAME_TIME: u32 = 1000 / FPS;

                    let mut t = unsafe { crate::sdl2::SDL_GetTicks() };
                    let mut dt;
                    let mut tsum: u64 = 0;
                    let mut tcnt: u64 = 0;
                    while !self.#gfoo.#g_event.quit {
                        dt = unsafe { crate::sdl2::SDL_GetTicks() } - t;
                        t += dt;

                        self.tick(dt);

                        dt = unsafe { crate::sdl2::SDL_GetTicks() } - t;
                        tsum += dt as u64;
                        tcnt += 1;
                        if dt < FRAME_TIME {
                            unsafe { crate::sdl2::SDL_Delay(FRAME_TIME - dt) };
                        }
                    }

                    println!("Average Frame Time: {}ms", tsum as f64 / tcnt as f64);
                }

                fn tick(&mut self, ts: u32) {
                    // Update #efoo
                    self.#gfoo.#g_event.update(ts, &self.#gfoo.#g_camera.0, &self.#gfoo.#g_screen.0);
                    // Clear the screen
                    self.#gfoo.#g_render_system.r.clear();
                    // Add initial #efoo
                    self.add_events(self.init_events(ts));
                    while !self.stack.is_empty() {
                        // Get element from next queue
                        if let Some((e, i, n)) = self
                            .stack
                            // Get last queue
                            .last_mut()
                            // Get next #efoo
                            .and_then(|queue| queue.front_mut())
                            // Check if the system exists
                            .and_then(|(e, i)| {
                                // Increment the event idx and return the old values
                                let v_s = &self.services[*e as usize];
                                v_s.get(*i).map(|_| {
                                    let vals = (e.clone(), i.clone(), v_s.len());
                                    *i += 1;
                                    vals
                                })
                            })
                        {
                            // This is the last system for this event
                            if i + 1 >= n {
                                self.pop();
                            }
                            // Add a new queue for new #efoo
                            self.#gfoo.#g_efoo = #efoo_ident::new();
                            // Run the system
                            if let Some(s) = self.services[e as usize].get(i) {
                                (s)(&mut self.#cfoo, &mut self.#gfoo, &mut self.#efoo);
                            }
                            // If this is the last system, remove the event
                            if i + 1 >= n {
                                self.#efoo.pop(e);
                            }
                            // Add new #efoo
                            let #efoo = std::mem::replace(&mut self.#gfoo.#g_efoo, #efoo_ident::new());
                            self.add_events(#efoo);
                        } else {
                            // We're done with this event
                            self.pop();
                        }
                    }
                    // Display the screen
                    self.#gfoo.#g_render_system.r.present();

                    self.post_tick();
                }

                fn post_tick(&mut self) {
                    // Remove marked entities
                    self.#cfoo.remove(&mut self.#gfoo.#g_entity_trash);
                    // Add new entities
                    self.#cfoo.append(&mut self.#gfoo.#g_cfoo);
                }

                fn init_events(&self, ts: u32) -> #efoo_ident {
                    let mut #efoo = #efoo_ident::new();
                    #efoo.new_event(CoreEvent::Events);
                    #efoo.new_event(CoreEvent::Update(ts));
                    #efoo.new_event(CoreEvent::Render);
                    #efoo
                }

                fn add_events(&mut self, mut em: #efoo_ident) {
                    if em.has_events() {
                        self.#efoo.append(&mut em);
                        self.stack.push(em.get_events());
                    }
                }

                fn pop(&mut self) {
                    // Remove top element and empty queue
                    if self.stack.last_mut().is_some_and(|queue| {
                        queue.pop_front();
                        queue.is_empty()
                    }) {
                        self.stack.pop();
                    }
                }
            }
        );

        quote!(
            #deps_code

            #gfoo_def

            #cfoo_def
            #cfoo_traits

            #e_def
            #efoo_def
            #efoo_traits

            #sfoo_def
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
