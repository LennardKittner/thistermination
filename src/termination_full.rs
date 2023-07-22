use syn::{DeriveInput, Data, Error};
use proc_macro::TokenStream;
use quote::quote;

use crate::{code_generation::{generate_termination_trait, generate_debug_trait, generate_display_trait, generate_error_trait, generate_from_traits}, parse::{parse_from_attribute, parse_helper_attributes, check_for_unique_types, parse_attributes, Defaults}};

pub fn _derive_termination_full(steam: TokenStream) -> Result<TokenStream, Error> {
    let ast: DeriveInput = syn::parse(steam)?;
    let name = &ast.ident;
    let variants = match ast.data {
        Data::Enum(ref data) => &data.variants,
        _ => return Err(Error::new_spanned(name, "thistermination can currently only be derived on enums"))
    };
    let defaults: Defaults = parse_attributes(&ast.attrs)?.into();
    let parse_helper_attributes = parse_helper_attributes(variants.iter())?;
    let debug_trait = generate_debug_trait(name, &parse_helper_attributes, &defaults);
    let display_trait = generate_display_trait(name, &parse_helper_attributes, &defaults)?;
    let termination_trait = generate_termination_trait(name, &parse_helper_attributes, &defaults);
    let error_trait = generate_error_trait(name);
    let from_attributes = parse_from_attribute(variants.iter())?;
    check_for_unique_types(&from_attributes)?;
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