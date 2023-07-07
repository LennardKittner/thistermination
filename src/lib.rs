use core::panic;

use syn::{DeriveInput, Data, Attribute, parenthesized, LitStr, LitInt};

use proc_macro::TokenStream;
use quote::quote;

#[derive(Debug)]
struct ParsedAttribute {
    exit_code: Option<u8>,
    message: Option<String>,
}

#[proc_macro_derive(Termination, attributes(termination))]
pub fn derive_termination(steam: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(steam).unwrap();
    let name = &ast.ident;
    let variants = match ast.data {
        Data::Enum(ref data) => &data.variants,
        _ => panic!("Termination can only be derived for enums"),
    };
    let termination_impl = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_attributes = parse_helper_attribute_values(&variant.attrs).unwrap();
        let exit_code = variant_attributes.exit_code.unwrap_or(1);
       quote! { #name::#variant_name => #exit_code.into(), }
    });

    let generated = quote! {
        impl std::process::Termination for #name {
            fn report(self) -> std::process::ExitCode {
                match self {
                    #(#termination_impl)*
                }
            }
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "request failed")
            }
        }
    };

    generated.into()
}

fn parse_helper_attribute_values(attributes: &[Attribute]) -> Result<ParsedAttribute, String> {
    let mut parsed_attribute = ParsedAttribute {
        exit_code: None,
        message: None,
    };

    for attribute in attributes {
        if !attribute.path().is_ident("termination") {
            return Err(format!("unknown ident {:?}", attribute.path().get_ident()));
        }
        let a  = attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("exit_code") {
                let content;
                parenthesized!(content in meta.input);
                let lit: LitInt = content.parse()?;
                let n: u8 = lit.base10_parse()?;
                parsed_attribute.exit_code = Some(n as u8);
                return Ok(());
            } else if meta.path.is_ident("msg") {
                let content;
                parenthesized!(content in meta.input);
                let lit: LitStr = content.parse()?;
                parsed_attribute.message = Some(lit.value());
                return Ok(());
            }
            Err(meta.error(format!("unrecognized attribute {}", meta.path.get_ident().unwrap())))
        });
        if let Err(error) = a {
            panic!("parse error {:?}", error);
        }
    }
    Ok(parsed_attribute)
}