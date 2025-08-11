use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(QEntity)]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = syn::parse(input).unwrap();

    // Build the impl
    impl_qentity(&ast)
}

fn impl_qentity(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let r#gend = quote! {
        impl QEntity for #name {}
    };
    gend.into()
}
