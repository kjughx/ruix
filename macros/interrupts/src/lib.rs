use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitInt};

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
        let in_error_code_ins = if ERROR_CODE_EXCEPTIONS.contains(&i) {
            "nop"
        } else {
            "push 0" // We need to add a dummy error code
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
