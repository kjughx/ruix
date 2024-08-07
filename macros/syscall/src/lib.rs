use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn, LitInt};

#[proc_macro]
pub fn syscalls(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitInt);
    let num_syscalls = input.base10_parse::<u8>().unwrap();

    let mut syscall_entries = Vec::new();

    for i in 0..num_syscalls {
        let fn_name = syn::Ident::new(
            &format!("__syscall_{}__", i),
            proc_macro2::Span::call_site(),
        );

        syscall_entries.push(quote! {
            #fn_name
        });
    }

    let expanded = quote! {
        #[link_section = ".data"]
        #[no_mangle]
        static SYSCALLS: [fn(&crate::cpu::InterruptFrame) -> usize; #num_syscalls as usize] = [
            #(#syscall_entries),*
        ];
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn syscall(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments (which should be a single integer)
    let attr_args = parse_macro_input!(attr as LitInt);
    let number = attr_args.base10_parse::<u32>().unwrap();

    // Parse the function item
    let mut input_fn = parse_macro_input!(item as ItemFn);

    input_fn.sig.ident = syn::Ident::new(
        &format!("__syscall_{}__", number),
        proc_macro2::Span::call_site(),
    );

    let mut arg_sizes = Vec::new();
    let arg_count = input_fn.sig.inputs.len();

    for arg in &input_fn.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = arg {
            let ty = &*pat_type.ty;
            arg_sizes.push(quote! {core::mem::size_of::<#ty>()});
        }
    }

    let array_name = Ident::new(
        &format!("ARG_SIZES_syscall_{}", number),
        proc_macro2::Span::call_site(),
    );

    let asm_syscall_name = syn::Ident::new(
        &format!("__asm_syscall_{}__", number),
        proc_macro2::Span::call_site(),
    );

    let push_ins: Vec<String> = arg_sizes
        .iter()
        .enumerate()
        .map(|(_, _)| format!("mov ebx, [ebp + 8 + {{}}]; push ebx;"))
        .collect();

    let push_args: Vec<_> = arg_sizes
        .iter()
        .enumerate()
        .map(|(i, _)| quote! { const #array_name[#i]})
        .collect();

    let push_ins_tokens = push_ins
        .iter()
        .map(|ins| quote! { #ins })
        .collect::<Vec<_>>();

    let expanded = if number == 0 {
        quote! {
            #[naked]
            #[no_mangle]
            extern "C" fn #asm_syscall_name() {
                unsafe {core::arch::asm!(
                    "push ebp",
                    "mov ebp, esp",
                    concat!("mov eax, ", #number),
                    "int 0x80",
                    "pop ebp",
                    "ret",
                    options(noreturn),
                )}
            }

            #input_fn
        }
    } else {
        quote! {
            const #array_name: [usize; #arg_count] = [ #(#arg_sizes),* ];

            #[naked]
            #[no_mangle]
            extern "C" fn #asm_syscall_name() {
                unsafe {core::arch::asm!(
                    "push ebp",
                    "mov ebp, esp",
                    #(#push_ins_tokens),*,
                    #(#push_args),*,
                    options(noreturn),
                )}
            }
            #input_fn
        }
    };

    TokenStream::from(expanded)
}
