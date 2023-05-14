use proc_macro2::TokenStream;
use quote::format_ident;

use crate::{
    util::{Call, Catch},
    validate::constants::{component_var, event_var, event_variant, global_var},
};

pub type Struct = (TokenStream, syn::Ident);
pub type Event = (TokenStream, syn::Ident, syn::Ident);

pub fn parse_type(path: &str) -> syn::Type {
    syn::parse_str(path).catch(format!("Could not parse type: {path}"))
}
