extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Fields, Ident, Meta,
    NestedMeta, Type,
};

/// Derive macro that automatically implements the `HasStateSpaceData` trait for a struct.
///
/// The `HasStateSpaceData` trait is used to provide access to the `StateSpaceCommonData` struct
/// that is used to store common data for all state spaces.
///
/// The struct must have a member named `state_space_data` of type `StateSpaceCommonData`.
#[proc_macro_derive(WithStateSpaceData)]
pub fn with_state_space_data_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct

    let mut attribute_found = false;

    find_struct_field(
        &input.data,
        &Ident::new("state_space_data", input.span()),
        |field| {
            find_segment_ident(field, "StateSpaceCommonData", |segment| {
                attribute_found = segment.arguments.is_empty();
            });
        },
    );

    // If the arena field with the correct type is not found, generate an error
    if !attribute_found {
        return syn::Error::new_spanned(
            input,
            format!("Struct must have a field `state_space_data` of type `StateSpaceCommonData`"),
        )
        .to_compile_error()
        .into();
    }

    let name = input.ident;

    // Generate the implementation
    let expanded = quote! {
        impl HasStateSpaceData for #name {
            fn state_space_data(&self) -> &StateSpaceCommonData {
                &self.state_space_data
            }

            fn state_space_data_mut(&mut self) -> &mut StateSpaceCommonData {
                &mut self.state_space_data
            }
        }
    };

    // Convert the generated code into a TokenStream and return it
    TokenStream::from(expanded)
}

/// Derive macro that automatically implements the `CanStateAllocateTrait` trait for a struct.
/// The `CanStateAllocateTrait` trait is used to provide access to the `Arena` struct that is used
/// to store the states.
///
/// The struct must have a member named `arena` of type `Arena<#state_type>`.
///
/// Example:
/// ```
/// #[derive(WithArenaAlloc)]
/// #[arena_alloc(state_type = "MyState")]
/// struct MyStruct {
///    arena: Arena<MyState>,
/// }
/// ```
#[proc_macro_derive(WithArenaAlloc, attributes(arena_alloc))]
pub fn with_arena_alloc_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct
    // Try to find the `state_type` attribute in the struct's attributes
    let mut state_type: Option<Ident> = None;
    let mut default_capacity: usize = 100;

    // Iterate through the struct's attributes to find `state_type`
    for meta_name_value in MetaNameValueIterator::new(&input.attrs) {
        if meta_name_value.path.is_ident("state_type") {
            if let syn::Lit::Str(lit_str) = &meta_name_value.lit {
                let ident = syn::Ident::new(&lit_str.value(), lit_str.span());
                state_type = Some(ident);
            }
        }
        else if meta_name_value.path.is_ident("default_capacity") {
            if let syn::Lit::Str(lit_str) = &meta_name_value.lit {
                // the literal should be a positive integer
                match  lit_str.value().parse::<usize>() {
                    Ok(num) => {
                        default_capacity = num;
                    },
                    Err(_) => {
                        return syn::Error::new_spanned(
                            meta_name_value,
                            "default_capacity should be a positive integer",
                        )
                        .to_compile_error()
                        .into();
                    }
                }
            }
        }
        else {
            return syn::Error::new_spanned(
                &meta_name_value,
                format!("Unknown attribute: {:?}", meta_name_value.path.get_ident().unwrap()),
            )
            .to_compile_error()
            .into();
        }
    }
    let state_type = match state_type {
        Some(state_type) => state_type,
        None => {
            return syn::Error::new_spanned(input, "Missing required `arena_alloc` attribute")
                .to_compile_error()
                .into();
        }
    };


    let mut arena_found = false;

    find_struct_field(&input.data, &Ident::new("arena", input.span()), |field| {
        // Ensure that the field is of the correct type `Arena<#state_type>`
        // find_segment_ident(field, "Arena", |segment| {
        //     // Ensure the generic type of Arena matches `#state_type`
        //     if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
        //         if let Some(syn::GenericArgument::Type(syn::Type::Path(type_path))) =
        //             args.args.first()
        //         {
        //             if let Some(segment) = type_path.path.segments.last() {
        //                 if segment.ident == state_type {
        //                     arena_found = true;
        //                 }
        //             }
        //         }
        //     }
        // });
        arena_found = true;
    });

    // If the arena field with the correct type is not found, generate an error
    if !arena_found {
        return syn::Error::new_spanned(
            input,
            format!("Struct must have a field `arena` of type `RefCell<Arena<{state_type}>>`"),
        )
        .to_compile_error()
        .into();
    }

    let name = input.ident;

    // Generate the implementation
    let expanded = quote! {
        impl CanStateAllocateTrait for #name {
            type State = #state_type;

            fn new_arena() -> RefCell<Arena<Self::State>> {
                RefCell::new(Arena::with_capacity(#default_capacity))
            }

            fn get_arena(&self) -> &RefCell<Arena<Self::State>> {
                &self.arena
            }
        }
    };

    // Convert the generated code into a TokenStream and return it
    TokenStream::from(expanded)
}

struct MetaNameValueIterator<'a> {
    attributes: &'a Vec<Attribute>,
    attr_idx: usize,
    meta_idx: usize,
}

impl MetaNameValueIterator<'_> {
    fn new(attributes: &Vec<Attribute>) -> MetaNameValueIterator {
        MetaNameValueIterator { attributes, attr_idx: 0, meta_idx: 0 }
    }
}

impl Iterator for MetaNameValueIterator<'_> {
    type Item = syn::MetaNameValue;

    fn next(&mut self) -> Option<Self::Item> {
        while self.attr_idx < self.attributes.len() {
            let attr = &self.attributes[self.attr_idx];
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                while self.meta_idx < meta_list.nested.len() {
                    if let NestedMeta::Meta(Meta::NameValue(name_value)) = &meta_list.nested[self.meta_idx] {
                        self.meta_idx += 1;
                        return Some(name_value.clone());
                    }
                }
            }
            self.attr_idx += 1;
            self.meta_idx = 0;
        }
        None
    }
}

/// helper function to find an attribute with the given name and call a closure with the attribute as an argument
///
/// # Example
/// ```
/// #[derive(WithStateSpaceData)]
/// #[alloc_arena(state_type = "MyState")]
/// struct MyStruct {...}
///
/// find_attribute(&input.attrs, "alloc_arena", |attr| {
///   find_meta(attr, "state_type", |meta| {
///     if let syn::Lit::Str(lit_str) = &meta.lit {
///       let ident = syn::Ident::new(&lit_str.value(), lit_str.span());
///         state_type = Some(ident);
///       }
///   });
/// });
/// ```
fn find_attribute(
    attributes: &Vec<Attribute>,
    attribute_name: &str,
    mut closure: impl FnMut(&Attribute),
) {
    for attr in attributes {
        if attr.path.is_ident(attribute_name) {
            closure(attr);
            break;
        }
    }
}

fn find_meta(attribute: &Attribute, meta_name: &str, mut closure: impl FnMut(&syn::MetaNameValue)) {
    if let Ok(Meta::List(meta_list)) = attribute.parse_meta() {
        for nested_meta in meta_list.nested {
            if let NestedMeta::Meta(Meta::NameValue(name_value)) = nested_meta {
                if name_value.path.is_ident(meta_name) {
                    closure(&name_value);
                    break;
                }
            }
        }
    }
}

/// Find a struct field with the given name and call a closure with the field as an argument.
/// The closure should take a `&syn::Field` as an argument.
/// and if the closure returns `Some`, the function will break
///
/// # Example
/// ```
/// #[derive(WithStateSpaceData)]
/// struct MyStruct {
///    state_space_data: StateSpaceCommonData,
/// }
///
/// find_struct_field(
///     &input.data,
///     &Ident::new("state_space_data", input.span()),
///     |field| {
///         find_segment_ident(field, "StateSpaceCommonData", |segment| {
///             attribute_found = segment.arguments.is_empty();
///         });
///     },
/// );
/// ```
fn find_struct_field(
    input_data: &Data,
    target_field: &Ident,
    mut closure: impl FnMut(&syn::Field),
) {
    if let syn::Data::Struct(data_struct) = &input_data {
        if let Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                if field.ident.as_ref() == Some(target_field) {
                    closure(field);
                    break;
                }
            }
        }
    }
}

/// Find a segment with the given name and call a closure with the segment as an argument.
fn find_segment_ident(
    field: &syn::Field,
    ident_name: &str,
    mut closure: impl FnMut(&syn::PathSegment),
) {
    if let Type::Path(type_path) = &field.ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == ident_name {
                closure(segment);
            }
        }
    }
}
