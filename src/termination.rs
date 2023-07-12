use core::panic;
use syn::{DeriveInput, Data};
use proc_macro::TokenStream;
use quote::quote;

use crate::code_generation::{generate_debug_trait, generate_termination_trait};

//TODO: maybe another one containing all traits needed i.e. Display and Error and #[from]
pub fn _derive_termination(steam: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(steam).unwrap();
    let name = &ast.ident;
    let variants = match ast.data {
        Data::Enum(ref data) => &data.variants,
        _ => panic!("thistermination can currently only be derived on enums"),
    };
    let debug_trait = generate_debug_trait(name, variants);
    let termination_trait = generate_termination_trait(name, variants);

    let generate = quote! {
        #debug_trait
        #termination_trait
    };

    generate.into()
}