use std::array;
use std::collections::HashSet;
use std::hash::Hash;

use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use regex::Regex;

use crate::codegen::idents::Idents;
use crate::codegen::util::string_to_type;
use crate::codegen::util::type_to_type;
use crate::resolve::ast_paths::EngineGlobals;
use crate::resolve::ast_paths::GetPaths;
use crate::util::Flatten;
use crate::util::JoinMap;
use crate::{
    resolve::ast_paths::ExpandEnum,
    util::{Catch, SplitCollect, SplitIter},
};

#[shared::macros::expand_enum]
pub enum LabelType {
    And,
    Or,
    Nand,
    Nor,
}

impl From<&str> for LabelType {
    fn from(value: &str) -> Self {
        let value = value.trim_start_matches("l");
        if value.starts_with("&") {
            Self::And
        } else if value.starts_with("|") {
            Self::Or
        } else if value.starts_with("!&") {
            Self::Nand
        } else if value.starts_with("!|") {
            Self::Nor
        } else {
            panic!("Invalid label argument: {value}")
        }
    }
}

#[derive(Clone, Debug)]
pub enum ContainerArg {
    EntityId(TokenStream),
    Component(TokenStream, syn::Ident, bool),
}

pub enum FnArg {
    EntityId,
    Component(syn::Ident),
    Global(syn::Ident),
    Event(syn::Ident),
    Label(LabelType, HashSet<syn::Ident>),
    Container(Vec<ContainerArg>),
}

#[derive(Debug)]
pub struct SystemRegexes {
    id: Regex,
    component: Regex,
    global: Regex,
    event: Regex,
    label: Regex,
    vec_comp: Regex,
    vector: Regex,
    system: Regex,
}

impl SystemRegexes {
    pub fn new() -> Self {
        let id = r"id";
        let c = r"c\d+_\d+";
        let g = r"g\d+_\d+";
        let e = r"e\d+_\d+";
        let l = format!(r"l(!)?[\|&]{c}(-{c})*");
        let v_c = format!(r"(m)?{c}");
        let v_i = format!(r"{v_c}|{id}");
        let v = format!(r"v({v_i})(-({v_i}))*");
        let arg = format!(r"{id}|{c}|{g}|{e}|{l}|{v}");

        let [id, component, global, event, label, vec_comp, vector, system] = [
            id,
            c,
            g,
            e,
            &l,
            &v_c,
            &v,
            &format!(r"(?P<name>\w+)\((?P<args>(({arg})(:({arg}))*)?)\)(?P<init>(i)?)"),
        ]
        .map(|r_str| {
            Regex::new(format!(r"^{r_str}$").as_str())
                .catch(format!("Could not create regex: \"^{r_str}$\""))
        });

        Self {
            id,
            component,
            global,
            event,
            label,
            vec_comp,
            vector,
            system,
        }
    }

    pub fn parse_data(&self, sys_str: &str) -> Option<(String, String, bool)> {
        self.system
            .captures(sys_str)
            .and_then(|c| c.name("name").zip(c.name("args")).zip(c.name("init")))
            .map(|((name, args), init)| {
                (
                    name.as_str().to_string(),
                    args.as_str().to_string(),
                    init.as_str() == "i",
                )
            })
    }

    pub fn parse_arg(&self, arg_str: &str) -> Option<FnArg> {
        let eid_type = format_ident!("{}", EngineGlobals::Entity.as_ident());

        // Entity id
        self.id
            .find(arg_str)
            .map(|_| FnArg::EntityId)
            // Component
            .or_else(|| {
                self.component
                    .find(arg_str)
                    .map(|_| FnArg::Component(format_ident!("{arg_str}")))
            })
            // Global
            .or_else(|| {
                self.global
                    .find(arg_str)
                    .map(|_| FnArg::Global(format_ident!("{arg_str}")))
            })
            // Event
            .or_else(|| {
                self.event
                    .find(arg_str)
                    .map(|_| FnArg::Event(format_ident!("{arg_str}")))
            })
            // Label
            .or_else(|| {
                self.label.find(arg_str).map(|_| {
                    FnArg::Label(
                        LabelType::from(arg_str),
                        arg_str
                            .trim_start_matches(['l', '!', '&', '|'])
                            .split_map("-", |a| {
                                self.component
                                    .find(a)
                                    .map(|_| format_ident!("{a}"))
                                    .catch(format!("Could not parse label type: {a}"))
                            }),
                    )
                })
            })
            // Container
            .or_else(|| {
                self.vector.find(arg_str).map(|_| {
                    FnArg::Container(arg_str.split_at(1).split_into(|_, args| {
                        args.split_map("-", |a| {
                            self.id
                                .find(a)
                                .map(|_| ContainerArg::EntityId(quote!(&#eid_type)))
                                .or_else(|| {
                                    self.vec_comp.find(a).map(|_| {
                                        let is_mut = a.starts_with("m");
                                        let mut_tok =
                                            if is_mut { (quote!(mut)) } else { (quote!()) };
                                        // TODO:
                                        let ty = quote!();
                                        ContainerArg::Component(
                                            quote!(&#mut_tok #ty),
                                            format_ident!("{}", a.trim_start_matches("m")),
                                            is_mut,
                                        )
                                    })
                                })
                                .catch(format!("Could not parse container item: {a}"))
                        })
                    }))
                })
            })
    }
}

pub struct System {
    pub name: TokenStream,
    args: Vec<TokenStream>,
    c_args: Vec<syn::Ident>,
    and_labels: HashSet<syn::Ident>,
    or_labels: Vec<HashSet<syn::Ident>>,
    nor_labels: HashSet<syn::Ident>,
    nand_labels: Vec<HashSet<syn::Ident>>,
    // Includes reference and mutability
    v_types: Vec<ContainerArg>,
    g_args: Vec<syn::Ident>,
    event: TokenStream,
    is_vec: bool,
    pub is_init: bool,
}

impl System {
    pub fn parse(cr_path: &syn::Path, data: &str, regexes: &SystemRegexes) -> Self {
        let (name, args, is_init) = regexes
            .parse_data(data)
            .catch(format!("Could not parse system: {data}"));
        let name = format_ident!("{name}");
        let mut s = Self {
            name: quote!(#cr_path::#name),
            args: Vec::new(),
            c_args: Vec::new(),
            and_labels: HashSet::new(),
            or_labels: Vec::new(),
            nor_labels: HashSet::new(),
            nand_labels: Vec::new(),
            v_types: Vec::new(),
            g_args: Vec::new(),
            event: quote!(),
            is_vec: false,
            is_init,
        };

        let gfoo = Idents::GFoo.to_ident();

        s.args = match args.as_str() {
            "" => Vec::new(),
            args => args
                .split(":")
                .map(|a| {
                    match regexes
                        .parse_arg(a)
                        .catch(format!("Could not parse system argument: {a}"))
                    {
                        FnArg::EntityId => quote!(id),
                        FnArg::Component(c) => {
                            let tt = quote!(#c);
                            s.c_args.push(c);
                            tt
                        }
                        FnArg::Global(g) => {
                            let tt = quote!(&mut gfoo.#g);
                            s.g_args.push(g);
                            tt
                        }
                        FnArg::Event(e) => {
                            s.event = quote!(e);
                            quote!(e)
                        }
                        FnArg::Label(ty, args) => {
                            match ty {
                                LabelType::And => s.and_labels.extend(args.into_iter()),
                                LabelType::Or => s.or_labels.push(args),
                                LabelType::Nand => s.nand_labels.push(args),
                                LabelType::Nor => s.nor_labels.extend(args.into_iter()),
                            }
                            quote!(std::marker::PhantomData)
                        }
                        FnArg::Container(args) => {
                            s.is_vec = true;
                            (s.c_args, s.v_types) = args
                                .into_iter()
                                .map(|a| match &a {
                                    ContainerArg::EntityId(ty) => (Idents::GenEids.to_ident(), a),
                                    ContainerArg::Component(ty, c, m) => (c.to_owned(), a),
                                })
                                .unzip();
                            quote!(v)
                        }
                    }
                })
                .collect(),
        };

        s.check_labels();

        s
    }

    fn check_labels(&mut self) {
        // Components are implicitly part of AND
        self.and_labels
            .extend(self.c_args.iter().map(|i| i.to_owned()));

        // NOR can't include the label, but AND must include the label
        if !self.and_labels.is_disjoint(&self.nor_labels) {
            panic!(
                "{}\n{}",
                "A label is in both AND and NOR. The label condition cannot be satisfied",
                "Note that all components are implicitly AND labels"
            )
        }

        // AND must have it, so OR is automatically satisfied
        self.or_labels
            .drain_filter(|ors| ors.is_empty() || !self.and_labels.is_disjoint(ors));
        // NOR must not have it, so OR is automatically checked
        for ors in self.or_labels.iter_mut() {
            ors.drain_filter(|c| self.nor_labels.contains(c));
            // NOR must have none, but OR must have at least one
            if ors.is_empty() {
                panic!(
                    "All labels in at least one OR set are also in NOR. The label condition cannot be satisfied"
                )
            }
        }

        // NOR must not have it, so NAND is automatically satisfied
        self.nand_labels
            .drain_filter(|nands| nands.is_empty() || !self.nor_labels.is_disjoint(nands));
        for nands in self.nand_labels.iter_mut() {
            nands.drain_filter(|c| self.and_labels.contains(c));
            // AND must have all, but NAND must not have at least one
            if nands.is_empty() {
                panic!(
                        "{}\n{}",
                        "All labels in at least on NAND set are also in AND. The label condition cannot be satisfied",
                        "Note that all components are implicitly AND labels"
                    )
            }
        }

        // Remove all components from AND
        self.and_labels.drain_filter(|c| self.c_args.contains(c));
    }

    fn quote_labels(&self, body: TokenStream) -> TokenStream {
        let [cfoo, eid] = [Idents::GenCFoo, Idents::GenEid].map(|i| i.to_ident());

        let (ands, nors) = (self.and_labels.iter(), self.nor_labels.iter());

        let mut checks = Vec::new();
        for (i, v) in [&self.and_labels, &self.nor_labels].iter().enumerate() {
            if !v.is_empty() {
                let vs = v.iter();
                let not = if i == 0 { quote!() } else { quote!(!) };
                checks.push(quote!(#(#not #cfoo.#vs.contains_key(#eid))&&*));
            }
        }
        for (i, v) in [&self.or_labels, &self.nand_labels].iter().enumerate() {
            let not = if i == 0 { quote!() } else { quote!(!) };
            checks.append(&mut v.map_vec(|v| {
                let vs = v.iter();
                quote!(#(#not #cfoo.#vs.contains_key(#eid))||*)
            }));
        }

        if checks.is_empty() {
            body
        } else {
            quote!(
                if (#((#checks))&&*) {
                    #body
                }
            )
        }
    }

    pub fn to_quote(&self) -> TokenStream {
        let f = &self.name;
        let args = &self.args;

        let body = if self.c_args.is_empty() {
            quote!(#f(#(#args),*))
        } else if !self.is_vec {
            let c_args = &self.c_args;
            let label_checks = self.quote_labels(quote!(#f(#(#args),*)));

            let eid = Idents::GenEid.to_ident();

            quote!(
                for #eid in crate::ecs::shared::intersect::intersect_keys(&mut [#(crate::ecs::shared::intersect::get_keys(&cfoo.#c_args)),*]).iter() {
                    if let (#(Some(#c_args),)*) = (#(cfoo.#c_args.get_mut(#eid),)*) {
                        #label_checks
                    }
                }
            )
        } else {
            // Container argument types
            let v_types = self
                .v_types
                .iter()
                .map(|a| match a {
                    ContainerArg::EntityId(ty) => ty,
                    ContainerArg::Component(ty, _, _) => ty,
                })
                .collect::<Vec<_>>();
            // Get first argument to initialize the result hashmap
            let arg = self.c_args.first().expect("No first component");
            let nones = ["None"].repeat(self.v_types.len() - 1).join(",");
            let (iter, tuple_init) = match self.v_types.first().expect("No first vector types") {
                ContainerArg::EntityId(_) => ("iter", format!("|k| (k, (None, {}))", nones)),
                ContainerArg::Component(_, _, m) => (
                    if *m { "iter_mut" } else { "iter" },
                    format!("|(k, v)| (k, (Some(v), {}))", nones),
                ),
            };
            let iter = format_ident!("{}", iter);
            let tuple_init = syn::parse_str::<syn::ExprClosure>(tuple_init.as_str())
                .expect("Could not parse tuple init closure");

            let [v, eid] = [Idents::GenV, Idents::GenEid].map(|i| i.to_ident());

            // Intersect with tail args
            let intersect_stmts = self.c_args[1..]
                .iter()
                .zip(self.v_types[1..].iter())
                .enumerate()
                .filter_map(|(i, (a, ty))| match ty {
                    ContainerArg::EntityId(_) => None,
                    ContainerArg::Component(_,_, m) => Some(
                        syn::parse_str::<syn::ExprCall>(
                            format!(
                                "crate::ecs::shared::intersect::intersect{}({v}, &mut cfoo.{a}, |t| &mut t.{})",
                                if *m { "_mut" } else { "" },
                                i + 1
                            )
                            .as_str(),
                        )
                        .expect("Could not parse intersect call"),
                    ),
                })
                .collect::<Vec<_>>();

            // Contsruct final vector
            // v1, v2, ...
            // c_vars only contains v_i where i is not an eid
            let mut c_vars = Vec::new();
            // all_vars contains all v_i
            // all_args replaces eids with "k"
            let (all_vars, all_args) = self
                .c_args
                .iter()
                .zip(self.v_types.iter())
                .enumerate()
                .map(|(i, (_v, ty))| {
                    let v_i = format_ident!("v{}", i);
                    match ty {
                        ContainerArg::EntityId(_) => (v_i, eid.to_owned()),
                        ContainerArg::Component(_, _, _) => {
                            c_vars.push(v_i.to_owned());
                            (v_i.to_owned(), v_i)
                        }
                    }
                })
                .unzip::<_, _, Vec<_>, Vec<_>>();

            let label_checks = self.quote_labels(quote!(return Some((#(#all_args,)*));));

            quote!(
                let mut #v = cfoo.#arg
                    .#iter()
                    .map(#tuple_init)
                    .collect::<std::collections::HashMap<_, (#(Option<#v_types>,)*)>>();
                #(#v = #intersect_stmts;)*
                let #v = #v
                    .into_iter()
                    .filter_map(|(#eid, (#(#all_vars,)*))| {
                        if let (#(Some(#c_vars),)*) = (#(#c_vars,)*) {
                            #label_checks
                        }
                        None
                    })
                    .collect::<Vec<_>>();
                #f(#(#args),*);
            )
        };

        let [cfoo_ty, gfoo_ty, efoo_ty] =
            [Idents::CFoo, Idents::GFoo, Idents::EFoo].map(|i| i.to_ident());
        let [cfoo, gfoo, efoo] =
            [Idents::GenCFoo, Idents::GenGFoo, Idents::GenEFoo].map(|i| i.to_ident());

        if self.is_init {
            quote!(
                (|#cfoo: &mut #cfoo, #gfoo: &mut #gfoo, #efoo: &mut #efoo| {
                      #body
                })(&mut self.cm, &mut self.gm, &mut self.events);
            )
        } else {
            let [e_ident, e] = [Idents::E, Idents::GenE].map(|i| i.to_ident());
            let event = &self.event;
            quote!(
                let f = |#cfoo: &mut #cfoo_ty, #gfoo: &mut #gfoo_ty, #efoo: &mut #efoo_ty| {
                    if let Some(#e) = #efoo.get_event() {
                        #body
                    }
                };
                self.add_system(#e_ident::#event, Box::new(f));
            )
        }
    }
}
