use syn::{DeriveInput, Data, Error};
use proc_macro::TokenStream;
use quote::quote;

use crate::{code_generation::{generate_debug_trait, generate_termination_trait, generate_empty_debug_trait, generate_empty_termination_trait}, parse::{parse_helper_attributes, parse_attributes, Defaults}};

pub fn _derive_termination(steam: TokenStream) -> Result<TokenStream, Error> {
    let ast: DeriveInput = syn::parse(steam)?;
    let name = &ast.ident;
    let variants = match ast.data {
        Data::Enum(ref data) => &data.variants,
        _ => return Err(Error::new_spanned(name, "thistermination can currently only be derived on enums"))
    };

    if variants.is_empty() {
        let debug_trait = generate_empty_debug_trait(name);
        let termination_trait = generate_empty_termination_trait(name);
        let generate = quote! {
            #debug_trait
            #termination_trait
        };
        return Ok(generate.into());
    }

    let defaults: Defaults = parse_attributes(&ast.attrs)?.into();
    let parsed_helper_attributes = parse_helper_attributes(variants.iter())?;
    let debug_trait = generate_debug_trait(name, &parsed_helper_attributes, &defaults);
    let termination_trait = generate_termination_trait(name, &parsed_helper_attributes, &defaults);
    
    let generate = quote! {
        #debug_trait
        #termination_trait
    };

    Ok(generate.into())
}