use proc_macro2::TokenStream;

#[derive(Debug)]
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

#[derive(Debug)]
pub enum ContainerArg {
    EntityId,
    Component(bool, syn::Ident),
}

#[derive(Debug)]
pub enum FnArg {
    EntityId,
    Component(syn::Ident),
    Global(syn::Ident),
    Event(syn::Ident),
    Label(LabelType, Vec<syn::Ident>),
    Container(Vec<ContainerArg>),
}

pub struct System {
    pub path: syn::Path,
    pub args: Vec<FnArg>,
}
