use core::panic;
use syn::{Attribute, parenthesized, LitStr, LitInt, Token};
use proc_macro2::TokenStream as TokenStream2;

pub struct ParsedAttribute {
    pub exit_code: Option<u8>,
    pub message: Option<Message>,
}

pub struct Message {
    pub format_string: String,
    pub format_string_arguments: Option<TokenStream2>,
}

//TODO: split up to disallow msg in TerminationNoDebug
//TODO: better error handling e.g. show error at specific attribute
pub fn parse_helper_attribute_values(attributes: &[Attribute]) -> Result<ParsedAttribute, String> {
    let mut parsed_attribute = ParsedAttribute {
        exit_code: None,
        message: None,
    };
    let mut found_attribute = false;
    for attribute in attributes {
        if !attribute.path().is_ident("termination") {
            continue;
        }
        if found_attribute {
            panic!("only one #[termination(...)] attribute is allowed")
        }
        found_attribute = true;
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
                let comma_present = content.peek(Token![,]);
                let mut args = None;
                if comma_present {
                    content.parse::<Token![,]>()?; 
                    let rest: TokenStream2 = content.parse()?;
                    args = Some(rest);
                } else if !content.is_empty() {
                    panic!("Missing \",\"");
                }
                parsed_attribute.message = Some(Message { format_string: lit.value(), format_string_arguments: args });
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