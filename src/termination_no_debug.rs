use syn::{DeriveInput, Data, Error};
use proc_macro::TokenStream;
use crate::{code_generation::generate_termination_trait, parse::{parse_helper_attributes, parse_attributes, Defaults}};

pub fn _derive_termination_no_debug(steam: TokenStream) -> Result<TokenStream, Error> {
    let ast: DeriveInput = syn::parse(steam)?;
    let name = &ast.ident;
    let variants = match ast.data {
        Data::Enum(ref data) => &data.variants,
        _ => return Err(Error::new_spanned(name, "thistermination can currently only be derived on enums"))
    };
    let defaults: Defaults = parse_attributes(&ast.attrs)?.into();
    let termination_attributes = parse_helper_attributes(variants.iter())?;
    for attribute in &termination_attributes {
        if attribute.message.is_some() {
            return Err(Error::new_spanned(&attribute.variant, "unexpected msg(...) on TerminationNoDebug"))
        }
    }
    Ok(generate_termination_trait(name, &termination_attributes, &defaults).into())
}