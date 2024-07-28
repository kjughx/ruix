extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::Expr;
use syn::{parse_macro_input, Ident, LitStr, Token, Type};

struct GlobalInput {
    name: Ident,
    type_: Type,
    value: Expr,
    id: LitStr,
}

impl Parse for GlobalInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let type_: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let value: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let id: LitStr = input.parse()?;

        Ok(GlobalInput {
            name,
            type_,
            value,
            id,
        })
    }
}

#[proc_macro]
pub fn global(input: TokenStream) -> TokenStream {
    let GlobalInput {
        name,
        type_,
        value,
        id,
    } = parse_macro_input!(input as GlobalInput);

    let static_name = name.to_string().to_uppercase();
    let static_ident = Ident::new(&static_name, name.span());

    let expanded = quote! {
        static mut #static_ident: crate::sync::global::Global<#type_> = crate::sync::global::Global::new(
            || #value,
            #id,
        );

        pub struct #name;
        impl #name {
            #[inline]
            pub fn get() -> &'static crate::sync::global::Global<#type_> {
                unsafe {core::ptr::addr_of!(#static_ident).as_ref().unwrap()}
            }
            #[inline]
            pub fn get_mut() -> &'static mut crate::sync::global::Global<#type_> {
                unsafe {core::ptr::addr_of_mut!(#static_ident).as_mut().unwrap()}
            }
        }
    };

    TokenStream::from(expanded)
}
