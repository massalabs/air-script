extern crate proc_macro;
use quote::{format_ident, quote};
use syn::{DeriveInput, Token};

pub fn impl_isleaf(input: &DeriveInput) -> proc_macro2::TokenStream {
    let ty = &input.ident;
    let name = format_ident!("{}_leaf", ty.to_string().to_lowercase());
    eprintln!("name: {:?}", name);
    match &input.data {
        syn::Data::Enum(data) => impl_isleaf_enum(ty, &name, data),
        _ => panic!("IsLeaf only supports enums"),
    }
}

fn impl_isleaf_enum(
    ty: &syn::Ident,
    name: &proc_macro2::Ident,
    data: &syn::DataEnum,
) -> proc_macro2::TokenStream {
    let variants: Vec<&syn::Variant> = data.variants.iter().collect();
    let variant_names = extract_variant_names(&variants);
    let spec_names = extract_variant_spec(&variants);
    quote! {
        impl crate::ir2::IsChild for #ty {
            fn get_parent(&self) -> crate::ir2::BackLink<crate::ir2::NodeType> {
                match self {
                    #(
                        #ty::#variant_names(#(#spec_names),*) => crate::ir2::IsChild::get_parent(#(#spec_names),*),
                    )*
                }
            }
            fn set_parent(&mut self, parent: crate::ir2::Link<crate::ir2::NodeType>) {
                match self {
                    #(
                        #ty::#variant_names(#(#spec_names),*) => crate::ir2::IsChild::set_parent(#(#spec_names),*, parent),
                    )*
                }
            }
        }
        impl std::fmt::Debug for #ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(
                        #ty::#variant_names(#(#spec_names),*) => write!(f, "{:?}", #(#spec_names),*),
                    )*
                }
            }
        }
        impl From<#ty> for crate::ir2::Link<crate::ir2::NodeType> {
            fn from(#name: #ty) -> crate::ir2::Link<crate::ir2::NodeType> {
                match #name {
                    #(
                        #ty::#variant_names(#(#spec_names),*) => #(#spec_names),*.into(),
                    )*
                }
            }
        }
    }
}

fn extract_variant_names(variants: &[&syn::Variant]) -> Vec<proc_macro2::Ident> {
    variants
        .iter()
        .map(|variant| variant.ident.clone())
        .collect()
}

fn extract_variant_spec<'a>(variants: &[&'a syn::Variant]) -> Vec<Vec<syn::Ident>> {
    let variant_spec: Vec<&syn::Fields> = variants.iter().map(|variant| &variant.fields).collect();
    variant_spec
        .iter()
        .map(|fields| {
            fields
                .iter()
                .map(|field| {
                    let ty = match &field.ty {
                        syn::Type::Path(ty) => &ty.path.segments[0].ident,
                        _ => panic!("IsLeaf only supports Path types"),
                    };
                    eprintln!("ty: {:?}", ty);
                    format_ident!("{}_leaf", quote! {#ty}.to_string().to_lowercase())
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use syn::parse2;

    #[test]
    fn test_derive_is_leaf_enum() {
        let input = quote! {
            #[derive(IsLeaf)]
            enum Test {
                A(ALeaf),
                B(BLeaf),
            }
        };
        let expected = quote! {
            impl crate::ir2::IsChild for Test {
                fn get_parent(&self) -> crate::ir2::BackLink<crate::ir2::NodeType> {
                    match self {
                        Test::A(aleaf_leaf) => crate::ir2::IsChild::get_parent(aleaf_leaf),
                        Test::B(bleaf_leaf) => crate::ir2::IsChild::get_parent(bleaf_leaf),
                    }
                }
                fn set_parent(&mut self, parent: crate::ir2::Link<crate::ir2::NodeType>) {
                    match self {
                        Test::A(aleaf_leaf) => crate::ir2::IsChild::set_parent(aleaf_leaf, parent),
                        Test::B(bleaf_leaf) => crate::ir2::IsChild::set_parent(bleaf_leaf, parent),
                    }
                }
            }
            impl std::fmt::Debug for Test {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        Test::A(aleaf_leaf) => write!(f, "{:?}", aleaf_leaf),
                        Test::B(bleaf_leaf) => write!(f, "{:?}", bleaf_leaf),
                    }
                }
            }
            impl From<Test> for crate::ir2::Link<crate::ir2::NodeType> {
                fn from(test_leaf: Test) -> crate::ir2::Link<crate::ir2::NodeType> {
                    match test_leaf {
                        Test::A(aleaf_leaf) => aleaf_leaf.into(),
                        Test::B(bleaf_leaf) => bleaf_leaf.into(),
                    }
                }
            }
        };
        let ast = parse2(input).unwrap();
        let output = impl_isleaf(&ast);
        assert_eq!(output.to_string(), expected.to_string());
    }
}
