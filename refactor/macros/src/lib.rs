use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn dependency(_input: TokenStream, item: TokenStream) -> TokenStream {
    item
}
