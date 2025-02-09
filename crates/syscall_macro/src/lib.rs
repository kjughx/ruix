use proc_macro::TokenStream;
use quote::quote;
use syn::PatType;
use syn::{parse_macro_input, ForeignItem, ForeignItemFn, Ident, ItemFn, ItemForeignMod, LitInt};

#[proc_macro]
#[cfg(feature = "kernel")]
pub fn gen_syscalls(input: TokenStream) -> TokenStream {
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
pub fn syscalls(attr: TokenStream, item: TokenStream) -> TokenStream {
    if cfg!(feature = "kernel") {
        __syscalls_kernel__(attr, item)
    } else if cfg!(feature = "user") {
        __syscalls_user__(attr, item)
    } else {
        panic!("This macro can only be used with features \"kernel\" OR \"user\"");
    }
}

fn uniform_syscall_name(number: usize) -> Ident {
    syn::Ident::new(
        &format!("__syscall_{}__", number),
        proc_macro2::Span::call_site(),
    )
}
fn uniform_syscall_entry_name(number: usize) -> Ident {
    syn::Ident::new(
        &format!("__syscall_entry_{}__", number),
        proc_macro2::Span::call_site(),
    )
}

fn __syscalls_kernel__(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut expanded = quote! {};

    let mut items = parse_macro_input!(item as ItemForeignMod);
    for (i, item) in items.items.iter_mut().enumerate() {
        if let ForeignItem::Fn(ForeignItemFn { ref mut sig, .. }) = item {
            let entry = uniform_syscall_entry_name(i);
            let syscall = uniform_syscall_name(i);
            sig.ident = uniform_syscall_name(i);
            expanded.extend(quote! {
                extern "C" {
                    #sig;
                }

                #[naked]
                #[no_mangle]
                fn #entry() {
                    unsafe {core::arch::naked_asm!("ljmp {}", sym #syscall)}
                }
            })
        }
    }
    TokenStream::from(expanded)
}

fn __syscalls_user__(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut expanded = quote! {};

    let mut items = parse_macro_input!(item as ItemForeignMod);
    for (i, item) in items.items.iter_mut().enumerate() {
        if let ForeignItem::Fn(ForeignItemFn { ref mut sig, .. }) = item {
            let arg_count = sig.inputs.len();
            let args = &sig.inputs;
            let ident = &sig.ident;
            let body = if arg_count == 0 {
                format!(
                    r#"
                        push ebp;
                        mov eax, {i};
                        int 0x80;
                        pop ebp;
                        ret;
                "#
                )
            } else {
                format!(
                    r#"
                        push ebp;
                        mov eax, {i};
                        add esp, {0};
                        int 0x80;
                        sub esp, {0};
                        pop ebp;
                        ret;
                            "#,
                    4 * arg_count
                )
            };

            expanded.extend(quote! {
                #[naked]
                #[no_mangle]
                pub unsafe extern "C" fn #ident(#args) {
                    unsafe {core::arch::naked_asm!( #body)}
                }
            });
        }
    }
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn syscall(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let number = parse_macro_input!(attr as LitInt)
        .base10_parse::<usize>()
        .unwrap();

    let uniform_name = uniform_syscall_name(number);
    let mut decl = Vec::new();

    let args_count = input_fn.sig.inputs.len();
    for (i, arg) in input_fn.sig.inputs.into_iter().enumerate() {
        if let syn::FnArg::Typed(PatType { pat, ty, .. }) = arg {
            if let syn::Pat::Ident(ref ident) = *pat {
                let idx = args_count - i - 1;
                let exp = quote! {
                    let #ident: #ty = crate::task::Task::copy_stack_item(crate::task::CurrentTask::get(), #idx);
                };
                decl.push(exp);
            }
        }
    }

    // TODO: Make sure return type is usize.

    let body = input_fn.block;

    let expanded = quote! {
        fn #uniform_name(frame: &crate::cpu::InterruptFrame) -> usize {
            #(#decl)*

            #body
        }
    };

    TokenStream::from(expanded)
}
