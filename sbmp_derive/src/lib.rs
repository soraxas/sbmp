extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};


#[proc_macro_derive(WithStateSpaceData)]
pub fn extra_data_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct
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