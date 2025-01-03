extern crate proc_macro;
use std::collections::HashMap;

use quote::{format_ident, quote};
use syn::DeriveInput;

pub fn impl_builder(input: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let (yes_name, no_name) = (
        format_ident!("{}BuilderYes", name),
        format_ident!("{}BuilderNo", name),
    );
    let fields = extract_fields(input);
    let (builder_struct_fields, builder_states, transition_table) =
        make_builder_struct_fields(&fields);
    let builder_struct = make_builder_struct(name, &builder_struct_fields);
    let (empty, full, builder_aliases) =
        make_builder_aliases(name, &builder_states, &yes_name, &no_name);
    let builder_impls = make_builder_impls(
        name,
        &builder_struct_fields,
        &builder_states,
        &transition_table,
    );
    quote! {
        #builder_struct
        #builder_aliases
        //#builder_structs
        impl Builder for #name {
            type Empty = #empty;
            type Full = #full;
            fn builder() -> Self::Empty {
                Self::Empty::default()
            }
        }
        #builder_impls
    }
}

fn extract_fields(data: &syn::DeriveInput) -> Vec<(&syn::Ident, &syn::Type)> {
    match &data.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .map(|field| (field.ident.as_ref().unwrap(), &field.ty))
                .collect(),
            syn::Fields::Unnamed(_) => unimplemented!(),
            syn::Fields::Unit => unimplemented!(),
        },
        syn::Data::Enum(_) => unimplemented!(),
        syn::Data::Union(_) => unimplemented!(),
    }
}

fn next_ty(ty: &syn::PathSegment) -> Option<syn::PathSegment> {
    match &ty.arguments {
        syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            args, ..
        }) => match args.first().unwrap() {
            syn::GenericArgument::Type(syn::Type::Path(syn::TypePath { path, .. })) => {
                Some(path.segments.first().unwrap().clone())
            }
            _ => None,
        },
        syn::PathArguments::None => Some(syn::PathSegment {
            ident: syn::Ident::new("None", ty.ident.span()),
            arguments: syn::PathArguments::None,
        }),
        _ => None,
    }
}

type StructFields<'a> = (
    Vec<(
        // name
        &'a syn::Ident,
        // type
        &'a syn::Type,
        // field declaration
        proc_macro2::TokenStream,
        // field setter argument
        proc_macro2::TokenStream,
        // set field
        proc_macro2::TokenStream,
        // builder
        proc_macro2::TokenStream,
    )>,
    // builder_states, a state is a vec of bools
    Vec<Vec<bool>>,
    // transition_table, rows are states, columns are fields, values are next states
    Vec<Vec<usize>>,
);

/// Generate the fields for the builder struct, and the transition table.
/// The fields are the same as the fields of the original struct,
/// but with an `Option` wrapper, unless the type already has a None variant.
/// Here we hardcode as having a None variant for the types:
/// `BackLink`, `Link<Vec>`, and `Vec`.
/// The transition table is a vec of tuples, where each tuple is a pair of integers.
/// The first integer is the index of the current state, and the second integer is the
/// index of the next state.
///
/// for example, for the struct:
/// ```rust
/// #[derive(Default)]
/// struct BackLink<T: Default>(T);
/// struct Link<T>(T);
/// #[derive(Default)]
/// enum Owner {
///     #[default]
///     None,
/// }
/// enum Node {}
/// enum Op {}
///
/// struct Foo {
///    parent: BackLink<Owner>,
///    a: Link<Node>,
///    bs: Link<Vec<Link<Op>>>,
///    cs: Vec<Link<Op>>,
///    count: i32,
/// }
/// // the builder struct fields would be:
/// struct FooBuilder<State> {
///    _builder_state: std::marker::PhantomData<State>,
///    parent: BackLink<Owner>,
///    a: Option<Link<Node>>,
///    bs: Link<Vec<Link<Op>>>,
///    cs: Vec<Link<Op>>,
///    count: Option<i32>,
/// }
/// ```
/// we generate a vec of the struct's fields, it will be our columns in the transition table.
/// and have the same index as the field in the vec of fields.
///
/// fields:
///   0: parent
///   1: a
///   2: bs
///   3: cs
///   4: count
///
/// We then generate a vec of states, which will be our rows in the transition table.
///
/// Notice that only `a` and `count` have been wrapped in an `Option` type.
/// we call them `transition fields`
/// We create an initial state.
/// Its _builder_state field is a `PhantomData` of a tuple of bools, which are all `true`,
/// except for our `transition fields` which are `false`.
/// We push it to the vec of states.
/// states:
///   0: (true, false, true, true, false) # initial state
/// We generate all combinations of our `transition fields` and push them to the vec of states,
/// from "falsier" to "truer" then from first field to last.
/// states:
///  0: (true, false, true, true, false) # initial state
///  1: (true, true, true, true, false) # a
///  2: (true, false, true, true, true) # count
///  3: (true, true, true, true, true) # a, count
///
/// We then generate the following transition table.
/// | state\\field | parent | a | bs | cs | count
/// |--------------|--------|---|----|----|------
/// | 0: 1 0 1 1 0 | 0      | 1 | 0  | 0  | 2
/// | 1: 1 1 1 1 0 | 1      | 1 | 1  | 1  | 3
/// | 2: 1 0 1 1 1 | 2      | 3 | 2  | 2  | 2
/// | 3: 1 1 1 1 1 | 3      | 3 | 3  | 3  | 3
fn make_builder_struct_fields<'a>(fields: &[(&'a syn::Ident, &'a syn::Type)]) -> StructFields<'a> {
    let initial_state: &mut Vec<bool> = &mut vec![false; fields.len()];
    let fields_info = fields
        .iter()
        .enumerate()
        .map(|(i, (ident, ty))| {
            let first_ty = match ty {
                syn::Type::Path(syn::TypePath { path, .. }) => path.segments.first().unwrap(),
                _ => unimplemented!(),
            };
            let maybe_second_ty = next_ty(first_ty);
            let second_ty = maybe_second_ty.as_ref().unwrap();
            let tys = (first_ty.ident.to_string(), second_ty.ident.to_string());
            let tys_refs = (tys.0.as_str(), tys.1.as_str());

            match tys_refs {
                ("BackLink", _) => {
                    initial_state[i] = true;
                    (
                        *ident,
                        *ty,
                        quote! {#ident: #ty},
                        quote! {#ident: Link<#second_ty>},
                        quote! {self.#ident = value.into();},
                        quote! {#ident: self.#ident.clone()},
                    )
                }
                ("Vec", _) => {
                    initial_state[i] = true;
                    (
                        *ident,
                        *ty,
                        quote! {#ident: #ty},
                        quote! {#ident: #second_ty},
                        quote! {self.#ident.push(value);},
                        quote! {#ident: self.#ident.clone()},
                    )
                }
                ("Link", "Vec") => {
                    let maybe_third_ty = next_ty(second_ty);
                    let third_ty = maybe_third_ty.as_ref().unwrap();
                    initial_state[i] = true;
                    (
                        *ident,
                        *ty,
                        quote! {#ident: #ty},
                        quote! {#ident: #third_ty},
                        quote! {self.#ident.borrow_mut().push(value);},
                        quote! {#ident: self.#ident.clone()},
                    )
                }
                _ => {
                    initial_state[i] = false;
                    (
                        *ident,
                        *ty,
                        quote! {#ident: Option<#ty>},
                        quote! {#ident: #ty},
                        quote! {self.#ident = Some(value);},
                        quote! {#ident: self.#ident.clone().unwrap()},
                    )
                }
            }
        })
        .collect::<Vec<_>>();

    // Enumerate all single field transitions
    let initial_state = initial_state.clone();
    let mut states: Vec<Vec<bool>> = vec![];
    states.push(initial_state.clone());
    for i in 0..fields.len() {
        let mut state = initial_state.clone();
        state[i] = true;
        if !states.contains(&state) {
            states.push(state.clone());
        }
    }
    // Enumerate all combinations of two fields
    let state_len = states.len();
    for i in 1..state_len {
        for j in 1..state_len {
            let mut state = states[i].clone();
            let other_state = states[j].clone();
            // Combine the two states
            state.iter_mut().zip(other_state.iter()).for_each(|(s, o)| {
                *s = *s || *o;
            });
            if !states.contains(&state) {
                states.push(state.clone());
            }
        }
    }
    let reverse_states: HashMap<Vec<bool>, usize> = states
        .iter()
        .enumerate()
        .map(|(i, state)| (state.clone(), i))
        .collect();
    let mut transition_table: Vec<Vec<usize>> = vec![];
    for state in states.iter() {
        let mut row = vec![];
        for col_index in 0..fields.len() {
            let mut next_state = state.clone();
            next_state[col_index] = true;
            let next_state_index = reverse_states[&next_state];
            row.push(next_state_index);
        }
        transition_table.push(row);
    }
    (fields_info, states, transition_table)
}

fn make_builder_struct<'a>(
    name: &syn::Ident,
    fields: &[(
        &'a syn::Ident,
        &'a syn::Type,
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
    )],
) -> proc_macro2::TokenStream {
    let builder_struct_name = format_ident!("{}Builder", name);
    let struct_fields = fields.iter().map(|(_, _, field, _, _, _)| field);
    let builder_struct = quote! {
        struct #builder_struct_name<State> {
            _builder_state: std::marker::PhantomData<State>,
            #(#struct_fields),*
        }
    };
    quote! {
        #builder_struct
    }
}

fn make_builder_aliases<'a>(
    name: &'a syn::Ident,
    states: &'a [Vec<bool>],
    yes_name: &'a syn::Ident,
    no_name: &'a syn::Ident,
) -> (syn::Ident, syn::Ident, proc_macro2::TokenStream) {
    let (yes, no) = (quote! { struct #yes_name; }, quote! { struct #no_name; });
    let builder_struct_name = format_ident!("{}Builder", name);
    let mut alias_names = vec![];
    let mut builder_aliases = vec![yes, no];
    builder_aliases.extend(
        states
            .iter()
            .enumerate()
            .map(|(i, state)| {
                let state_name = format_ident!("State{}", i);
                alias_names.push(state_name.clone());
                let state_fields = state.iter().map(|b| {
                    if *b {
                        quote! { #yes_name }
                    } else {
                        quote! { #no_name }
                    }
                });
                quote! {
                    type #state_name = #builder_struct_name<(#(#state_fields),*)>;
                }
            })
            .collect::<Vec<_>>(),
    );
    let empty_state = alias_names.first().unwrap();
    let full_state = alias_names.last().unwrap();
    (
        empty_state.clone(),
        full_state.clone(),
        quote! {
            #(#builder_aliases)*
        },
    )
}

fn make_builder_impls<'a>(
    name: &syn::Ident,
    fields: &[(
        &'a syn::Ident,
        &'a syn::Type,
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
        proc_macro2::TokenStream,
    )],
    states: &[Vec<bool>],
    transition_table: &[Vec<usize>],
) -> proc_macro2::TokenStream {
    let state_names = states
        .iter()
        .enumerate()
        .map(|(i, _)| format_ident!("State{}", i))
        .collect::<Vec<_>>();
    let empty_state = state_names.first().unwrap();
    let impls = states.iter().enumerate().map(|(i, _)| {
        let state_name = &state_names[i];
        let mut methods = fields
            .iter()
            .enumerate()
            .map(|(j, (ident, _, _, arg, set_field, _))| {
                let (ret, body_ret) = if &state_names[transition_table[i][j]] == state_name {
                    (quote! { Self }, quote! { self })
                } else {
                    (
                        quote! { #state_name },
                        quote! { unsafe { std::mem::transmute(self) } },
                    )
                };
                quote! {
                    fn #ident(&mut self, #arg) -> #ret {
                        #set_field
                        #body_ret
                    }
                }
            })
            .collect::<Vec<_>>();
        if i == states.len() - 1 {
            let builder = fields.iter().map(|(_, _, _, _, _, builder)| builder);
            methods.push(quote! {
                fn build(&self) -> #name {
                    #name {
                        #(#builder),*
                    }
                }
            });
        };
        quote! {
            impl #state_name {
                #(#methods)*
            }
        }
    });
    quote! {
        impl Default for #empty_state {
            fn default() -> Self {
                Self {
                    _builder_state: std::marker::PhantomData,
                    ..Default::default()
                }
            }
        }
        #(#impls)*
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::fmt;
    use pretty_assertions::assert_eq;
    use syn::parse2;

    // #[derive(Default)]
    // struct BackLink<T: Default>(T);
    // struct Link<T>(T);
    // #[derive(Default)]
    // enum Owner {
    //     #[default]
    //     None,
    // }
    // enum Node {}
    // enum Op {}
    #[test]
    fn test_derive_node_wrapper() {
        let (y, n) = (
            format_ident!("FooBuilderYes"),
            format_ident!("FooBuilderNo"),
        );
        let input = quote! {
            #[derive(Builder)]
            struct Foo {
                parent: BackLink<Owner>,
                a: Link<Node>,
                bs: Link<Vec<Link<Op>>>,
                cs: Vec<Link<Op>>,
                count: i32
            }
        };
        let expected = quote! {
            struct FooBuilder<State> {
                _builder_state: std::marker::PhantomData<State>,
                parent: BackLink<Owner>,
                a: Option<Link<Node>>,
                bs: Link<Vec<Link<Op>>>,
                cs: Vec<Link<Op>>,
                count: Option<i32>
            }
            struct #y;
            struct #n;
            type State0 = FooBuilder<(#y, #n, #y, #y, #n)>;
            type State1 = FooBuilder<(#y, #y, #y, #y, #n)>;
            type State2 = FooBuilder<(#y, #n, #y, #y, #y)>;
            type State3 = FooBuilder<(#y, #y, #y, #y, #y)>;
            impl Builder for Foo {
                type Empty = State0;
                type Full = State3;
                fn builder() -> Self::Empty {
                    Self::Empty::default()
                }
            }

            impl Default for State0 {
                fn default() -> Self {
                    Self {
                        _builder_state: std::marker::PhantomData,
                        ..Default::default()
                    }
                }
            }
            impl State0 {
                fn parent(&mut self, parent: Link<Owner>) -> Self {
                    self.parent = value.into();
                    self
                }
                fn a(&mut self, a: Link<Node>) -> State0 {
                    self.a = Some(value);
                    unsafe { std::mem::transmute(self) }
                }
                fn bs(&mut self, bs: Link<Op>) -> Self {
                    self.bs.borrow_mut().push(value);
                    self
                }
                fn cs(&mut self, cs: Link<Op>) -> Self {
                    self.cs.push(value);
                    self
                }
                fn count(&mut self, count: i32) -> State0 {
                    self.count = Some(value);
                    unsafe { std::mem::transmute(self) }
                }
            }
            impl State1 {
                fn parent(&mut self, parent: Link<Owner>) -> Self {
                    self.parent = value.into();
                    self
                }
                fn a(&mut self, a: Link<Node>) -> Self {
                    self.a = Some(value);
                    self
                }
                fn bs(&mut self, bs: Link<Op>) -> Self {
                    self.bs.borrow_mut().push(value);
                    self
                }
                fn cs(&mut self, cs: Link<Op>) -> Self {
                    self.cs.push(value);
                    self
                }
                fn count(&mut self, count: i32) -> State1 {
                    self.count = Some(value);
                    unsafe { std::mem::transmute(self) }
                }
            }
            impl State2 {
                fn parent(&mut self, parent: Link<Owner>) -> Self {
                    self.parent = value.into();
                    self
                }
                fn a(&mut self, a: Link<Node>) -> State2 {
                    self.a = Some(value);
                    unsafe { std::mem::transmute(self) }
                }
                fn bs(&mut self, bs: Link<Op>) -> Self {
                    self.bs.borrow_mut().push(value);
                    self
                }
                fn cs(&mut self, cs: Link<Op>) -> Self {
                    self.cs.push(value);
                    self
                }
                fn count(&mut self, count: i32) -> Self {
                    self.count = Some(value);
                    self
                }
            }
            impl State3 {
                fn parent(&mut self, parent: Link<Owner>) -> Self {
                    self.parent = value.into();
                    self
                }
                fn a(&mut self, a: Link<Node>) -> Self {
                    self.a = Some(value);
                    self
                }
                fn bs(&mut self, bs: Link<Op>) -> Self {
                    self.bs.borrow_mut().push(value);
                    self
                }
                fn cs(&mut self, cs: Link<Op>) -> Self {
                    self.cs.push(value);
                    self
                }
                fn count(&mut self, count: i32) -> Self {
                    self.count = Some(value);
                    self
                }
                fn build(&self) -> Foo {
                    Foo {
                        parent: self.parent.clone(),
                        a: self.a.clone().unwrap(),
                        bs: self.bs.clone(),
                        cs: self.cs.clone(),
                        count: self.count.clone().unwrap(),
                    }
                }
            }
        };
        let ast = parse2(input).unwrap();
        let output = impl_builder(&ast);
        let expected = fmt(expected);
        let output = fmt(output);

        assert_eq!(output, expected);
    }
}
