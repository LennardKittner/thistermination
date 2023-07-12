use core::panic;
use syn::{Attribute, parenthesized, LitStr, LitInt, Token, Error, meta::ParseNestedMeta};
use proc_macro2::{TokenStream as TokenStream2, Ident};


pub const MESSAGE: HelperAttribute = HelperAttribute::Message{ 
    label: "msg",
    error_msg: "msg is not allowed non debug macro",
    parse: parse_message,
};

pub const EXIT_CODE: HelperAttribute = HelperAttribute::ExitCode { 
    label: "exit_code",
    error_msg: "exit_code is not allowed on this macro",
    parse: parse_exit_code
};

pub enum HelperAttribute {
    Message{label: &'static str, error_msg: &'static str, parse: fn(&ParseNestedMeta<'_>) -> Result<Message, Error>},
    ExitCode{label: &'static str, error_msg: &'static str, parse: fn(&ParseNestedMeta<'_>) -> Result<u8, Error> }
}

impl HelperAttribute {
    fn label_matches(&self, ident: &Ident) -> bool {
        match self {
            HelperAttribute::Message { label, .. } => ident.to_string() == *label,
            HelperAttribute::ExitCode { label, .. } => ident.to_string() == *label,
        }
    }
}

pub struct ParsedAttribute {
    pub exit_code: Option<u8>,
    pub message: Option<Message>,
}

pub struct Message {
    pub format_string: String,
    pub format_string_arguments: Option<TokenStream2>,
}

//TODO: better error handling e.g. show error at specific attribute
pub fn parse_helper_attribute(attributes: &[Attribute], allowed_helper_attributes: &[HelperAttribute], forbidden_helper_attributes: &[HelperAttribute]) -> Result<ParsedAttribute, String> {
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
            if let Some(ident) = meta.path.get_ident() {
                for helper_attribute in allowed_helper_attributes {
                   if helper_attribute.label_matches(ident) {
                        match helper_attribute {
                            HelperAttribute::Message { parse, .. } => parsed_attribute.message = Some(parse(&meta)?),
                            HelperAttribute::ExitCode { parse, .. } => parsed_attribute.exit_code = Some(parse(&meta)?),
                        }
                        return Ok(());
                   }
                }
                for helper_attribute in forbidden_helper_attributes {
                   if helper_attribute.label_matches(ident) {
                        return Err(meta.error(
                            match helper_attribute {
                                HelperAttribute::Message { error_msg, .. } => error_msg,
                                HelperAttribute::ExitCode { error_msg, .. } => error_msg,
                            }
                        ));
                   }
                }
            } else {
                return Err(meta.error(format!("identifier expected")));
            }
            Err(meta.error(format!("unrecognized attribute {}", meta.path.get_ident().unwrap())))
        });
        if let Err(error) = a {
            panic!("parse error {:?}", error);
        }
    }
    Ok(parsed_attribute)
}

fn parse_exit_code(meta: &ParseNestedMeta<'_>) -> Result<u8, Error> {
   let content;
    parenthesized!(content in meta.input);
    let lit: LitInt = content.parse()?;
    let exit_code: u8 = lit.base10_parse()?;
    return Ok(exit_code);
}

fn parse_message(meta: &ParseNestedMeta<'_>) -> Result<Message, Error> {
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
    return Ok(Message { format_string: lit.value(), format_string_arguments: args });
}