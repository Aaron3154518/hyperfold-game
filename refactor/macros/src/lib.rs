use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn component(_input: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn global(_input: TokenStream, item: TokenStream) -> TokenStream {
    item
}
