use quote::format_ident;

use crate::{
    resolve::ast_paths::{EngineTraits, ExpandEnum, GetPaths},
    validate::constants::NAMESPACE,
};

#[shared::macros::expand_enum]
pub enum Idents {
    // General
    Namespace,
    // Systems
    SFoo,
    // Globals
    GFoo,
    // Components
    AddComponent,
    CFoo,
    // Events
    AddEvent,
    EFoo,
    E,
    ELen,
    // Code generation
    GenE,
    GenV,
    GenEid,
    GenEids,
    GenCFoo,
    GenGFoo,
    GenEFoo,
}

impl Idents {
    pub fn as_str(&self) -> &str {
        match self {
            Idents::Namespace => NAMESPACE,
            Idents::SFoo => "SFoo",
            Idents::GFoo => "GFoo",
            Idents::AddComponent => EngineTraits::AddComponent.as_ident(),
            Idents::CFoo => "CFoo",
            Idents::AddEvent => EngineTraits::AddEvent.as_ident(),
            Idents::EFoo => "EFoo",
            Idents::E => "E",
            Idents::ELen => "E_LEN",
            Idents::GenE => "e",
            Idents::GenV => "v",
            Idents::GenEid => "eid",
            Idents::GenEids => "eids",
            Idents::GenCFoo => "cfoo",
            Idents::GenGFoo => "gfoo",
            Idents::GenEFoo => "efoo",
        }
    }

    pub fn to_ident(&self) -> syn::Ident {
        format_ident!("{}", self.as_str())
    }
}
