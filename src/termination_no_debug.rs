use core::panic;
use syn::{DeriveInput, Data};
use proc_macro::TokenStream;

use crate::{code_generation::generate_termination_trait, parse::{MESSAGE, EXIT_CODE}};

pub fn _derive_termination_no_debug(steam: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(steam).unwrap();
    let name = &ast.ident;
    let variants = match ast.data {
        Data::Enum(ref data) => &data.variants,
        _ => panic!("thistermination can currently only be derived on enums"),
    };
    generate_termination_trait(name, variants, &[EXIT_CODE], &[MESSAGE]).into()
}