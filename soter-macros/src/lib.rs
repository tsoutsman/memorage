#![feature(proc_macro_span)]

use std::path::Path;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[cfg(feature = "client")]
#[proc_macro]
pub fn word_list(item: TokenStream) -> TokenStream {
    let word_list_path = parse_macro_input!(item as syn::LitStr);

    let mut rust_file_path = proc_macro::Span::call_site().source_file().path();
    rust_file_path.pop();
    let file_path = rust_file_path.join(Path::new(&word_list_path.value()));

    let file_contents = std::fs::read_to_string(file_path).expect("error reading word list: ");
    let word_list = file_contents.lines().map(|l| quote! { #l });

    quote! {
        [#(#word_list),*]
    }
    .into()
}
