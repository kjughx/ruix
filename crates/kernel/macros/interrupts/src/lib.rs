use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitInt};

const ERROR_CODE_EXCEPTIONS: [u8; 10] = [0x8, 0xa, 0xb, 0xc, 0xd, 0xe, 0x11, 0x15, 0x1d, 0x1e];

/// This generates:
/// ```
/// static INTERRUPT_POINTER_TABLE: [unsafe extern "C" fn(); #num_interrupts];
/// ```
/// and requires this to be defined:
///
/// ```
/// interrupt_handler(i: u16, frame: *const InterruptFrame)
/// ```
#[proc_macro]
pub fn interrupt_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitInt);
    let num_interrupts = input.base10_parse::<u8>().unwrap();

    let mut interrupt_fns = Vec::new();
    let mut interrupt_entries = Vec::new();

    for i in 0..num_interrupts {
        let fn_name = syn::Ident::new(&format!("int{}", i), proc_macro2::Span::call_site());
        let i_str = i.to_string();
        let in_error_code_ins = if !ERROR_CODE_EXCEPTIONS.contains(&i) {
            "push 0" // We need to add a dummy error code
        } else {
            "nop"
        };

        interrupt_fns.push(quote! {
            #[naked]
            #[no_mangle]
            pub extern "C" fn #fn_name() {
                unsafe {
                    core::arch::asm!(
                        "cli",
                        // CPU does:
                        // push ss
                        // push sp
                        // push flags
                        // push cs
                        // push ip
                        #in_error_code_ins,
                        "pushad", // Push general purpose registers
                        "push esp",
                        concat!("push ", #i_str),
                        "call interrupt_handler",
                        "add esp, 12", // Pop the stack, along with the error code
                        "popad", // Restore general purpose registers
                        "iretd", // Return from interrupt
                        options(noreturn)
                    );
                }
            }
        });

        interrupt_entries.push(quote! {
            #fn_name,
        });
    }

    let expanded = quote! {
        extern "C" {
            fn handle_error_code();
        }

        #(#interrupt_fns)*

        #[link_section = ".data"]
        #[no_mangle]
        static INTERRUPT_POINTER_TABLE: [unsafe extern "C" fn(); #num_interrupts as usize] = [
            #(#interrupt_entries)*
        ];
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
/// Usage:
/// ```
/// #[isr(<num>)]
/// fn my_isr(frame: *const InterruptFrame) {
///     ` Service interrupt number <num> `
/// }
///
/// ```
/// The frame passed to the isr is contains the latest and greatest information
/// hot and fresh from the CPU. Please don't sleep here.
pub fn isr(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let number = parse_macro_input!(attr as LitInt)
        .base10_parse::<u8>()
        .unwrap();

    let args_count = input_fn.sig.inputs.len();

    let ident = syn::Ident::new(
        &format!("__isr_{}__", number),
        proc_macro2::Span::call_site(),
    );
    let body = input_fn.block;

    let link_section = format!(".isr.{}", number);

    if args_count != 1 {
        let corrected = format!(
            r#"{}(frame: *const InterruptFrame) {{ `body` }}"#,
            input_fn.sig.ident
        );
        let err = format!(
            "Interrupt service routines should have only one argument: frame: *const InterruptFrame.

        try:

        #[isr({})]
        {}
        ",
            number, corrected
        );
        return TokenStream::from(quote! {compile_error!(#err);});
    }

    let id = syn::Ident::new(
        &format!("__INTERRUPT_HANDLER_{}__", number),
        proc_macro2::Span::call_site(),
    );
    let handler = syn::Ident::new(
        &format!("__interrupt_handler_{}__", number),
        proc_macro2::Span::call_site(),
    );

    let in_error_code_ins = if !ERROR_CODE_EXCEPTIONS.contains(&number) {
        "push 0" // We need to add a dummy error code
    } else {
        "nop"
    };

    let expanded = quote! {
        #[naked]
        #[no_mangle]
        extern "C" fn #handler () {
            unsafe {
                core::arch::asm!(
                    "cli",
                    // CPU does:
                    // push ss
                    // push sp
                    // push flags
                    // push cs
                    // push ip
                    #in_error_code_ins,
                    "pushad", // Push general purpose registers
                    "push esp",
                    "call {}",
                    "add esp, 12", // Pop the stack, along with the error code
                    "popad", // Restore general purpose registers
                    "iretd", // Return from interrupt
                    sym #ident,
                    options(noreturn)
                );
            }
        }

        #[no_mangle]
        #[link_section = #link_section]
        pub static #id: crate::idt::InterruptEntry =  crate::idt::InterruptEntry {
            id: #number as u16, ptr: #handler,
        };

        #[no_mangle]
        unsafe extern "C" fn #ident() {
            crate::io::outb(0x20, 0x20);

            #body

        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
/// Usage:
/// ```
/// #[interrupt_handler(<num>)]
/// fn my_handler() {
///     ` Handle interrupt number <num> `
/// }
/// ```
/// This is the first thing called after the interrupt is triggered,
/// handle with care.
///
/// NOTE: This requires manually acknowledging the interrupt. The recommended
/// way to do that is to use:
///
/// ```
/// outb(0x20, 0x20);
/// ```
pub fn interrupt_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let number = parse_macro_input!(attr as LitInt)
        .base10_parse::<u8>()
        .unwrap();

    let args_count = input_fn.sig.inputs.len();

    if args_count != 0 {
        let corrected = format!(r#"{}() {{ `body` }}"#, input_fn.sig.ident);
        let err = format!(
            "Interrupt handlers should not accept any arguments.

        try:

        #[interrupt_handler({})]
        {}
        ",
            number, corrected
        );
        return TokenStream::from(quote! {compile_error!(#err);});
    }

    let link_section = format!(".isr.{}", number);

    let id = syn::Ident::new(
        &format!("__INTERRUPT_HANDLER_{}__", number),
        proc_macro2::Span::call_site(),
    );
    let handler = syn::Ident::new(
        &format!("__interrupt_handler_{}__", number),
        proc_macro2::Span::call_site(),
    );

    let body = input_fn.block;

    let expanded = quote! {
        #[no_mangle]
        #[link_section = #link_section]
        pub static #id: crate::idt::InterruptEntry =  crate::idt::InterruptEntry {
            id: #number as u16, ptr: #handler,
        };

        #[naked]
        #[no_mangle]
        extern "C" fn #handler () {
            #body
        }
    };

    TokenStream::from(expanded)
}
