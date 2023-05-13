use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn expand_enum(_input: TokenStream, item: TokenStream) -> TokenStream {
    let e = parse_macro_input!(item as syn::ItemEnum);

    let e_name = &e.ident;
    let e_len = e.variants.len();
    let e_varis = e.variants.iter().map(|v| &v.ident).collect::<Vec<_>>();

    quote!(
        #[derive(Clone, Copy, Debug)]
        #e

        impl ExpandEnum<#e_len> for  #e_name {
            const VARIANTS: [Self; #e_len] = [#(Self::#e_varis),*];
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn component(_input: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn global(_input: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn event(_input: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn system(_input: TokenStream, item: TokenStream) -> TokenStream {
    item
}
