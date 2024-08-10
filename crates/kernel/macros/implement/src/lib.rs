extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl};

#[proc_macro_attribute]
pub fn implement(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut input = parse_macro_input!(item as ItemImpl);

    let mut expanded = quote! {};
    let name = &input.self_ty;
    for item in &mut input.items {
        expanded.extend(quote! {pub #item});
    }

    // Generate the output tokens
    let expanded = quote! {
        impl #name {
            #expanded
        }

        #input
    };

    // Convert the generated tokens back into a TokenStream
    TokenStream::from(expanded)
}
