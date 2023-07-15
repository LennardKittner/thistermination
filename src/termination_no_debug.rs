use core::panic;
use syn::{DeriveInput, Data, Error};
use proc_macro::TokenStream;
use crate::{code_generation::generate_termination_trait, parse::parse_helper_attributes};

//TODO: forbid msg on any variant
pub fn _derive_termination_no_debug(steam: TokenStream) -> Result<TokenStream, Error> {
    let ast: DeriveInput = syn::parse(steam).unwrap();
    let name = &ast.ident;
    let variants = match ast.data {
        Data::Enum(ref data) => &data.variants,
        _ => panic!("thistermination can currently only be derived on enums"),
    };
    let termination_attributes = parse_helper_attributes(variants.iter())?;
    Ok(generate_termination_trait(name, &termination_attributes).into())
}