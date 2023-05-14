use proc_macro::TokenStream;
use quote::quote;
use shared::parse_args::{ComponentMacroArgs, GlobalMacroArgs};
use syn::{parse_macro_input, parse_quote};

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
pub fn component(input: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as ComponentMacroArgs);
    if args.is_dummy {
        return quote!().into();
    }

    let input = parse_macro_input!(item as syn::Item);
    match input {
        syn::Item::Struct(mut s) => {
            s.vis = syn::parse_quote!(pub);
            quote!(#s)
        }
        _ => panic!("Only structs can be components"),
    }
    .into()
}

#[proc_macro_attribute]
pub fn global(input: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as GlobalMacroArgs);
    if args.is_dummy {
        return quote!().into();
    }

    let input = parse_macro_input!(item as syn::Item);
    match input {
        syn::Item::Struct(mut s) => {
            s.vis = syn::parse_quote!(pub);
            quote!(#s)
        }
        _ => panic!("Only structs can be globals"),
    }
    .into()
}

#[proc_macro_attribute]
pub fn system(_input: TokenStream, item: TokenStream) -> TokenStream {
    let mut fun = parse_macro_input!(item as syn::ItemFn);
    fun.vis = parse_quote!(pub);

    quote!(#fun).into()
}

#[proc_macro_attribute]
pub fn event(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut ev = parse_macro_input!(item as syn::ItemStruct);
    ev.vis = parse_quote!(pub);

    quote!(#ev).into()
}
