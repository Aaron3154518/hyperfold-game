use parse_cfg::Cfg;
use quote::ToTokens;

// Parse attributes from engine components
#[derive(Copy, Clone, Debug)]
pub enum EcsAttribute {
    Component,
    Global,
    System,
    Event,
    Dependency,
}

impl EcsAttribute {
    fn from(value: String) -> Option<Self> {
        match value.as_str() {
            "component" => Some(EcsAttribute::Component),
            "global" => Some(EcsAttribute::Global),
            "system" => Some(EcsAttribute::System),
            "event" => Some(EcsAttribute::Event),
            "dependency" => Some(EcsAttribute::Dependency),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Attribute {
    Ecs(Vec<String>, Vec<String>),
    Cfg(Cfg),
}

impl Attribute {
    fn from(value: String) -> Self {
        match value.as_str() {
            "cfg" => Self::Cfg(Cfg::Is(String::new())),
            s => Self::Ecs(s.split("::").map(|s| s.to_string()).collect(), Vec::new()), // _ => EcsAttribute::from(value).map(|e| Self::Ecs(e, Vec::new())),
        }
    }
}

pub fn get_attributes_if_active(
    attrs: &Vec<syn::Attribute>,
    features: &Vec<String>,
) -> Option<Vec<(Vec<String>, Vec<String>)>> {
    let mut is_active = true;
    let new_attrs = get_attributes(attrs)
        .into_iter()
        .fold(Vec::new(), |mut new_attrs, a| {
            match a {
                Attribute::Ecs(ty, args) => new_attrs.push((ty, args)),
                Attribute::Cfg(cfg) => {
                    is_active = !eval_cfg_args(&cfg, features).is_some_and(|b| !b)
                }
            }
            new_attrs
        });
    is_active.then_some(new_attrs)
}

// Returns list of parsed attributes from ast attributes
pub fn get_attributes(attrs: &Vec<syn::Attribute>) -> Vec<Attribute> {
    attrs
        .iter()
        .filter_map(|a| {
            a.path()
                .segments
                .last()
                .map(|s| parse_attr_args(Attribute::from(s.ident.to_string()), a))
        })
        .collect()
}

// Check cfg args to make sure we are valid
pub fn eval_cfg_args(cfg: &Cfg, features: &Vec<String>) -> Option<bool> {
    match cfg {
        Cfg::Any(cfgs) => Some(
            cfgs.iter()
                .map(|cfg| eval_cfg_args(&cfg, &features).is_some_and(|b| b))
                .collect::<Vec<_>>()
                .contains(&true),
        ),
        Cfg::All(cfgs) => Some(
            !cfgs
                .iter()
                .map(|cfg| !eval_cfg_args(&cfg, &features).is_some_and(|b| !b))
                .collect::<Vec<_>>()
                .contains(&false),
        ),
        Cfg::Not(cfg) => eval_cfg_args(cfg, &features).map(|b| !b),
        Cfg::Equal(k, v) => {
            if k == "feature" {
                Some(features.contains(&v))
            } else {
                None
            }
        }
        Cfg::Is(_) => None,
    }
}

// Parses arguments to a single ast attribute
fn parse_attr_args(mut attr_type: Attribute, attr: &syn::Attribute) -> Attribute {
    match &mut attr_type {
        Attribute::Ecs(_, v) => match &attr.meta {
            syn::Meta::List(l) => {
                for t in l.to_token_stream() {
                    match t {
                        proc_macro2::TokenTree::Group(g) => {
                            *v = g
                                .stream()
                                .into_iter()
                                .filter_map(|tt| match tt {
                                    proc_macro2::TokenTree::Ident(i) => Some(i.to_string()),
                                    _ => None,
                                })
                                .collect();
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        },
        Attribute::Cfg(cfg) => match &attr.meta {
            syn::Meta::List(l) => {
                *cfg = l
                    .to_token_stream()
                    .to_string()
                    .parse()
                    .expect("Could not parse cfg_str");
            }
            _ => (),
        },
    };
    attr_type
}
