use core::panic;
use syn::{DeriveInput, Data, Error};
use proc_macro::TokenStream;
use quote::quote;

use crate::{code_generation::{generate_termination_trait, generate_debug_trait, generate_display_trait, generate_error_trait, generate_from_traits}, parse::{parse_from_attribute, parse_helper_attributes}};

pub fn _derive_termination_full(steam: TokenStream) -> Result<TokenStream, Error> {
  let ast: DeriveInput = syn::parse(steam).unwrap();
    let name = &ast.ident;
    let variants = match ast.data {
        Data::Enum(ref data) => &data.variants,
        _ => panic!("thistermination can currently only be derived on enums"),
    };
    let parse_helper_attributes = parse_helper_attributes(variants.iter())?;
    let debug_trait = generate_debug_trait(name, &parse_helper_attributes);
    let display_trait = generate_display_trait(name, &parse_helper_attributes)?;
    let termination_trait = generate_termination_trait(name, &parse_helper_attributes);
    let error_trait = generate_error_trait(name);
    let from_attributes = parse_from_attribute(variants.iter())?;
    let from_traits = generate_from_traits(name, &from_attributes);

    let generate = quote! {
        #debug_trait
        #termination_trait
        #display_trait
        #error_trait
        #from_traits
    };

    Ok(generate.into())
}