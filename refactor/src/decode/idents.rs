use quote::format_ident;

use crate::{resolve::ast_paths::EnginePaths, validate::constants::NAMESPACE};

#[macros::expand_enum]
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
}

impl Idents {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Namespace => NAMESPACE,
            Self::SFoo => "SFoo",
            Self::GFoo => "GFoo",
            Self::AddComponent => EnginePaths::AddComponent.as_str(),
            Self::CFoo => "CFoo",
            Self::AddEvent => EnginePaths::AddEvent.as_str(),
            Self::EFoo => "EFoo",
            Self::E => "E",
            Self::ELen => "E_LEN",
        }
    }

    pub fn to_ident(&self) -> syn::Ident {
        format_ident!("{}", self.as_str())
    }
}
