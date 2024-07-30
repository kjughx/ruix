extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemStruct};

#[proc_macro_attribute]
pub fn packed(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as ItemStruct);

    // Get the struct name and fields
    let vis = &input.vis;
    let name = &input.ident;
    let fields = &input.fields;

    // Generate the output tokens
    let expanded = quote! {
        #[repr(C, packed)]
        #[derive(Packed, Default, Clone, Copy)]
        #vis struct #name #fields
    };

    // Convert the generated tokens back into a TokenStream
    TokenStream::from(expanded)
}

#[proc_macro_derive(Packed)]
pub fn derive_packed(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the struct name
    let name = &input.ident;

    // Generate the trait implementation
    let expanded = quote! {
        impl #name {
            pub fn size() -> usize {
                core::mem::size_of::<Self>()
            }
        }
        impl crate::Packed for #name {}
        impl crate::FromBytes for #name {
            type Output = Self;
            fn from_bytes(bytes: &[u8]) -> Self {
                assert_eq!(bytes.len(), core::mem::size_of::<#name>(), "Incorrect byte slice length");

                let mut packed = core::mem::MaybeUninit::<#name>::uninit();
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        bytes.as_ptr(),
                        packed.as_mut_ptr() as *mut u8,
                        core::mem::size_of::<#name>(),
                    );
                    packed.assume_init()
                }
            }
        }
        impl crate::ReinterpretBytes for #name {
            type Output = Self;
            fn reinterpret(bytes: &[u8]) -> &Self {
                assert_eq!(bytes.len(), core::mem::size_of::<#name>(), "Incorrect byte slice length");

                unsafe {
                    &*(bytes.as_ptr() as *const Self)
                }
            }
        }
    };

    // Convert the generated tokens back into a TokenStream
    TokenStream::from(expanded)
}
