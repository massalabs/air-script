mod builder;
mod helpers;

use builder::impl_builder;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder, attributes(node))]
pub fn derive_isnode(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_builder(&ast).into()
}
