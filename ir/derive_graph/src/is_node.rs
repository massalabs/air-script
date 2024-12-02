extern crate proc_macro;
use quote::{format_ident, quote};
use syn::{DeriveInput, Token};

pub fn impl_node_wrapper(input: &DeriveInput) -> proc_macro2::TokenStream {
    let ty = &input.ident;
    let name = format_ident!("{}_node", ty.to_string().to_lowercase());
    let fields = extract_struct_fields(input);
    let (node_field_name, field_names) = extract_field_names(&fields);
    let new_signature = make_new_signature(&field_names);
    let getters = make_getters(&field_names);
    let impls = quote! {
        impl #ty {
            pub fn new(#(#new_signature)*) -> Self {
                Self {
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

fn extract_struct_fields(input: &DeriveInput) -> Vec<&syn::Field> {
    match &input.data {
        syn::Data::Struct(data) => data.fields.iter().collect(),
        _ => panic!("NodeWrapper only supports structs"),
    }
}

fn extract_field_names(fields: &[&syn::Field]) -> (proc_macro2::Ident, Vec<proc_macro2::Ident>) {
    let node_field = fields
        .iter()
        .find(|field| field.attrs.iter().any(|attr| attr.path().is_ident("node")))
        .expect("NodeWrapper requires a node field");
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

fn make_new_signature(field_names: &[proc_macro2::Ident]) -> Vec<proc_macro2::TokenStream> {
    let mut signature = vec![quote! { parent: crate::ir2::BackLink<crate::ir2::NodeType> }];
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use syn::parse2;

    #[test]
    fn test_derive_node_wrapper() {
        let input = quote! {
            #[derive(IsNode)]
            struct Test {
                #[node(lhs, rhs)]
                node_field: Node,
            }
        };
        let expected = quote! {
            impl Test {
                pub fn new(parent: crate::ir2::BackLink<crate::ir2::NodeType>, lhs: crate::ir2::Link<crate::ir2::NodeType>, rhs: crate::ir2::Link<crate::ir2::NodeType>) -> Self {
                    Self {
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
        let output = impl_node_wrapper(&ast);

        assert_eq!(output.to_string(), expected.to_string());
    }
}
