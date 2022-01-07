#![deny(
    non_ascii_idents,
    // missing_docs,
    rust_2018_idioms,
    rust_2021_compatibility,
    future_incompatible,
    missing_debug_implementations,
    missing_copy_implementations,
    nonstandard_style,
    unreachable_pub,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    rustdoc::broken_intra_doc_links
)]
#![feature(proc_macro_span)]

#[cfg(feature = "client")]
#[proc_macro]
pub fn word_list(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use std::path::Path;

    use quote::quote;
    use syn::parse_macro_input;

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
