extern crate proc_macro;
use quote::{format_ident, quote};
use syn::{DeriveInput, Token};

pub fn impl_isnode(input: &DeriveInput) -> proc_macro2::TokenStream {
    let ty = &input.ident;
    let name = format_ident!("{}_node", ty.to_string().to_lowercase());
    match &input.data {
        syn::Data::Struct(data) => impl_isnode_struct(ty, &name, data),
        syn::Data::Enum(data) => impl_isnode_enum(ty, &name, data),
        _ => panic!("IsNode only supports structs and enums"),
    }
}

fn impl_isnode_struct(
    ty: &syn::Ident,
    name: &proc_macro2::Ident,
    struct_data: &syn::DataStruct,
) -> proc_macro2::TokenStream {
    let fields: Vec<&syn::Field> = struct_data.fields.iter().collect();
    let (node_field_name, field_names) = extract_field_names(&fields);
    let extra_fields = fields
        .iter()
        .filter_map(|field| {
            if field.ident == Some(node_field_name.clone()) {
                None
            } else {
                Some(*field)
            }
        })
        .collect::<Vec<_>>();
    let new_signature = make_new_signature(&field_names, &extra_fields);
    let extra_field_names = extra_fields
        .iter()
        .map(|field| field.ident.clone().unwrap())
        .collect::<Vec<_>>();
    let getters = make_getters(&field_names);
    let impls = quote! {
        impl #ty {
            pub fn new(#(#new_signature)*) -> Self {
                Self {
                    #(#extra_field_names,)*
                    #node_field_name: Node::new(
                        parent,
                        crate::ir2::Link::new(vec![#(#field_names),*])
                    ),
                }
            }
            #(#getters)*
        }
        impl crate::ir2::IsParent for #ty {
            fn get_children(&self) -> crate::ir2::Link<Vec<crate::ir2::Link<crate::ir2::NodeType>>> {
                crate::ir2::IsParent::get_children(&self.#node_field_name)
            }
        }
        impl crate::ir2::IsChild for #ty {
            fn get_parent(&self) -> crate::ir2::BackLink<crate::ir2::NodeType> {
                crate::ir2::IsChild::get_parent(&self.#node_field_name)
            }
            fn set_parent(&mut self, parent: crate::ir2::Link<crate::ir2::NodeType>) {
                crate::ir2::IsChild::set_parent(&mut self.#node_field_name, parent);
            }
        }
        impl std::fmt::Debug for #ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}{:?}", stringify!(#ty), &self.#node_field_name)
            }
        }
        impl From<#ty> for crate::ir2::Link<crate::ir2::NodeType> {
            fn from(#name: #ty) -> crate::ir2::Link<crate::ir2::NodeType> {
                crate::ir2::Link::new(crate::ir2::NodeType::MiddleNode(crate::ir2::MiddleNode::#ty(#name)))
            }
        }
    };
    impls
}

fn extract_field_names(fields: &[&syn::Field]) -> (proc_macro2::Ident, Vec<proc_macro2::Ident>) {
    let node_field = fields
        .iter()
        .find(|field| field.attrs.iter().any(|attr| attr.path().is_ident("node")))
        .expect("IsNode requires a node field");
    let node_field_name = node_field.ident.clone().unwrap();
    let field_names = node_field
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path().is_ident("node") {
                let args = attr
                    .parse_args_with(|input: syn::parse::ParseStream| {
                        syn::punctuated::Punctuated::<syn::Ident, Token![,]>::parse_terminated(
                            input,
                        )
                    })
                    .expect("Node field must have a list of field names")
                    .into_iter()
                    .collect::<Vec<_>>();
                Some(args)
            } else {
                None
            }
        })
        .expect("Node field must have a list of field names");
    (node_field_name, field_names)
}

fn make_new_signature(
    field_names: &[proc_macro2::Ident],
    extra_fields: &[&syn::Field],
) -> Vec<proc_macro2::TokenStream> {
    let mut signature = vec![quote! { parent: crate::ir2::BackLink<crate::ir2::NodeType> }];
    signature.extend(extra_fields.iter().map(|field| {
        let field_name = field.ident.clone().unwrap();
        let field_ty = &field.ty;
        quote! {
            ,
            #field_name: #field_ty
        }
    }));
    signature.extend(field_names.iter().map(|field_name| {
        quote! {
            ,
            #field_name: crate::ir2::Link<crate::ir2::NodeType>
        }
    }));
    signature
}

fn make_getters(field_names: &[proc_macro2::Ident]) -> Vec<proc_macro2::TokenStream> {
    field_names
        .iter()
        .enumerate()
        .map(|(field_index, field)| {
            let index = syn::Index::from(field_index);
            quote! {
                pub fn #field(&self) -> crate::ir2::Link<crate::ir2::NodeType> {
                    crate::ir2::IsParent::get_children(self).borrow()[#index].clone()
                }
            }
        })
        .collect()
}

fn impl_isnode_enum(
    ty: &syn::Ident,
    name: &proc_macro2::Ident,
    data: &syn::DataEnum,
) -> proc_macro2::TokenStream {
    let variants: Vec<&syn::Variant> = data.variants.iter().collect();
    let variant_names = extract_variant_names(&variants);
    let spec_names = extract_variant_spec(&variants);
    quote! {
        impl crate::ir2::IsParent for #ty {
            fn get_children(&self) -> crate::ir2::Link<Vec<crate::ir2::Link<crate::ir2::NodeType>>> {
                match self {
                    #(
                        #ty::#variant_names(#(#spec_names),*) => crate::ir2::IsParent::get_children(#(#spec_names),*),
                    )*
                }
            }
        }
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
                    let ty = &field.ty;
                    format_ident!("{}_node", quote! {#ty}.to_string().to_lowercase())
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
    fn test_derive_is_node() {
        let input = quote! {
            #[derive(IsNode)]
            struct Test {
                pub extra: i32,
                #[node(lhs, rhs)]
                node_field: Node,
            }
        };
        let expected = quote! {
            impl Test {
                pub fn new(parent: crate::ir2::BackLink<crate::ir2::NodeType>, extra: i32, lhs: crate::ir2::Link<crate::ir2::NodeType>, rhs: crate::ir2::Link<crate::ir2::NodeType>) -> Self {
                    Self {
                        extra,
                        node_field: Node::new(parent, crate::ir2::Link::new(vec![lhs, rhs])),
                    }
                }
                pub fn lhs(&self) -> crate::ir2::Link<crate::ir2::NodeType> {
                    crate::ir2::IsParent::get_children(self).borrow()[0].clone()
                }
                pub fn rhs(&self) -> crate::ir2::Link<crate::ir2::NodeType> {
                    crate::ir2::IsParent::get_children(self).borrow()[1].clone()
                }
            }
            impl crate::ir2::IsParent for Test {
                fn get_children(&self) -> crate::ir2::Link<Vec<crate::ir2::Link<crate::ir2::NodeType>>> {
                    crate::ir2::IsParent::get_children(&self.node_field)
                }
            }
            impl crate::ir2::IsChild for Test {
                fn get_parent(&self) -> crate::ir2::BackLink<crate::ir2::NodeType> {
                    crate::ir2::IsChild::get_parent(&self.node_field)
                }
                fn set_parent(&mut self, parent: crate::ir2::Link<crate::ir2::NodeType>) {
                    crate::ir2::IsChild::set_parent(&mut self.node_field, parent);
                }
            }
            impl std::fmt::Debug for Test {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}{:?}", stringify!(Test), &self.node_field)
                }
            }
            impl From<Test> for crate::ir2::Link<crate::ir2::NodeType> {
                fn from(test_node: Test) -> crate::ir2::Link<crate::ir2::NodeType> {
                    crate::ir2::Link::new(crate::ir2::NodeType::MiddleNode(crate::ir2::MiddleNode::Test(test_node)))
                }
            }
        };
        let ast = parse2(input).unwrap();
        let output = impl_isnode(&ast);

        assert_eq!(output.to_string(), expected.to_string());
    }

    #[test]
    fn test_derive_is_node_enum() {
        let input = quote! {
            #[derive(IsNode)]
            enum Test {
                A(ANode),
                B(BNode),
            }
        };
        let expected = quote! {
            impl crate::ir2::IsParent for Test {
                fn get_children(&self) -> crate::ir2::Link<Vec<crate::ir2::Link<crate::ir2::NodeType>>> {
                    match self {
                        Test::A(anode_node) => crate::ir2::IsParent::get_children(anode_node),
                        Test::B(bnode_node) => crate::ir2::IsParent::get_children(bnode_node),
                    }
                }
            }
            impl crate::ir2::IsChild for Test {
                fn get_parent(&self) -> crate::ir2::BackLink<crate::ir2::NodeType> {
                    match self {
                        Test::A(anode_node) => crate::ir2::IsChild::get_parent(anode_node),
                        Test::B(bnode_node) => crate::ir2::IsChild::get_parent(bnode_node),
                    }
                }
                fn set_parent(&mut self, parent: crate::ir2::Link<crate::ir2::NodeType>) {
                    match self {
                        Test::A(anode_node) => crate::ir2::IsChild::set_parent(anode_node, parent),
                        Test::B(bnode_node) => crate::ir2::IsChild::set_parent(bnode_node, parent),
                    }
                }
            }
            impl std::fmt::Debug for Test {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        Test::A(anode_node) => write!(f, "{:?}", anode_node),
                        Test::B(bnode_node) => write!(f, "{:?}", bnode_node),
                    }
                }
            }
            impl From<Test> for crate::ir2::Link<crate::ir2::NodeType> {
                fn from(test_node: Test) -> crate::ir2::Link<crate::ir2::NodeType> {
                    match test_node {
                        Test::A(anode_node) => anode_node.into(),
                        Test::B(bnode_node) => bnode_node.into(),
                    }
                }
            }
        };
        let ast = parse2(input).unwrap();
        let output = impl_isnode(&ast);
        assert_eq!(output.to_string(), expected.to_string());
    }
}
