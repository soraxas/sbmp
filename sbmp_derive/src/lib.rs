extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, DeriveInput, Fields, Ident, Meta, NestedMeta, Type,
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

    find_struct_field(&input, "state_space_data", |field| {
        // Ensure that the field is of the correct type `Arena<#state_type>`

        find_segment_ident(field, "StateSpaceCommonData", |segment| {
            // Ensure the generic type of Arena matches `#state_type`
            if segment.arguments.is_empty() {
                attribute_found = true;
            }
        });
    });

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
/// #[state_type = "MyState"]
/// struct MyStruct {
///    arena: Arena<MyState>,
/// }
/// ```
#[proc_macro_derive(WithArenaAlloc, attributes(state_type))]
pub fn with_arena_alloc_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct

    // Try to find the `state_type` attribute in the struct's attributes
    let mut state_type: Option<Ident> = None;

    // Iterate through the struct's attributes to find `state_type`
    for attr in &input.attrs {
        if attr.path.is_ident("state_type") {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                for nested_meta in meta_list.nested {
                    if let NestedMeta::Meta(Meta::NameValue(name_value)) = nested_meta {
                        if name_value.path.is_ident("state_type") {
                            if let syn::Lit::Str(lit_str) = name_value.lit {
                                let ident = syn::Ident::new(&lit_str.value(), lit_str.span());
                                state_type = Some(ident);
                            }
                        }
                    }
                }
            }
        }
    }

    // If `state_type` is not provided, error out
    if state_type.is_none() {
        return syn::Error::new_spanned(input, "Missing required `state_type` attribute")
            .to_compile_error()
            .into();
    }

    let state_type = state_type.unwrap(); // Now we are guaranteed to have a state_type

    let mut arena_found = false;

    find_struct_field(&input, "arena", |field| {
        // Ensure that the field is of the correct type `Arena<#state_type>`

        find_segment_ident(field, "Arena", |segment| {
            // Ensure the generic type of Arena matches `#state_type`
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                if let Some(syn::GenericArgument::Type(syn::Type::Path(type_path))) =
                    args.args.first()
                {
                    if let Some(segment) = type_path.path.segments.last() {
                        if segment.ident == state_type {
                            arena_found = true;
                        }
                    }
                }
            }
        });
    });

    // If the arena field with the correct type is not found, generate an error
    if !arena_found {
        return syn::Error::new_spanned(
            input,
            format!("Struct must have a field `arena` of type `Arena<{state_type}>`"),
        )
        .to_compile_error()
        .into();
    }

    let name = input.ident;

    // Generate the implementation
    let expanded = quote! {
        impl CanStateAllocateTrait for #name {
            type State = #state_type;

            fn get_arena_mut(&mut self) -> &mut Arena<Self::State> {
                &mut self.arena
            }

            fn get_arena(&self) -> &Arena<Self::State> {
                &self.arena
            }
        }
    };

    // Convert the generated code into a TokenStream and return it
    TokenStream::from(expanded)
}

/// Find a struct field with the given name and call a closure with the field as an argument.
fn find_struct_field(
    input: &DeriveInput,
    field_name: &str,
    mut closure: impl FnMut(&syn::Field),
) -> Option<syn::Field> {
    if let syn::Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                if field.ident == Some(Ident::new(field_name, field.span())) {
                    // return Some(field.clone());
                    closure(field);
                }
            }
        }
    }
    None
}

/// Find a segment with the given name and call a closure with the segment as an argument.
fn find_segment_ident(
    field: &syn::Field,
    ident_name: &str,
    mut closure: impl FnMut(&syn::PathSegment),
) {
    // Ensure that the field is of the correct type `Arena<#state_type>`
    if let Type::Path(type_path) = &field.ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == ident_name {
                closure(segment);
            }
        }
    }
}
