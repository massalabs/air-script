mod isnode;
use isnode::impl_isnode;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(IsNode, attributes(node))]
pub fn derive_isnode(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_isnode(&ast).into()
}
