use core::panic;
use syn::{DeriveInput, Data};
use proc_macro::TokenStream;
use quote::quote;

use crate::{code_generation::{generate_termination_trait, generate_debug_trait, generate_display_trait, generate_error_trait, generate_from_traits}, parse::{MESSAGE, EXIT_CODE}};

pub fn _derive_termination_full(steam: TokenStream) -> TokenStream {
  let ast: DeriveInput = syn::parse(steam).unwrap();
    let name = &ast.ident;
    let variants = match ast.data {
        Data::Enum(ref data) => &data.variants,
        _ => panic!("thistermination can currently only be derived on enums"),
    };
    let debug_trait = generate_debug_trait(name, variants, &[MESSAGE, EXIT_CODE], &[]);
    let termination_trait = generate_termination_trait(name, variants, &[EXIT_CODE, MESSAGE], &[]);
    let display_trait = generate_display_trait(name, variants, &[MESSAGE, EXIT_CODE], &[]);
    let error_trait = generate_error_trait(name);
    let from_traits = generate_from_traits(name, variants);
    
    let generate = quote! {
        #debug_trait
        #termination_trait
        #display_trait
        #error_trait
        #from_traits
    };

    generate.into()
}