use std::collections::HashSet;

use quote::ToTokens;
use syn::{Attribute, parenthesized, LitStr, LitInt, Token, Error, meta::ParseNestedMeta, Type, Variant, Expr};

use crate::pull_up_results;

pub struct ParsedAttribute {
    pub variant: Variant,
    pub exit_code: Option<ExitCodeAttribute>,
    pub message: Option<MessageAttribute>,
}

pub struct MessageAttribute {
    pub format_string: String,
    pub format_string_arguments: Vec<Expr>,
}

pub struct ExitCodeAttribute {
    pub exit_code: u8,
}

pub struct FromAttribute {
    pub variant: Variant,
    pub from_type: Option<Type>,
}

pub fn parse_helper_attributes<'a>(variants: impl Iterator<Item = &'a Variant>) -> Result<Vec<ParsedAttribute>, Error> {
    pull_up_results(variants.map(|variant| {
        parse_attributes(variant, &variant.attrs)
    }))
}


pub fn parse_from_attribute<'a>(variants: impl Iterator<Item = &'a Variant>) -> Result<Vec<FromAttribute>, Error> {
    pull_up_results(variants.map(|variant| {
        let mut field_type = None;
        if variant.fields.is_empty() {
            return Ok(FromAttribute { variant: variant.clone(), from_type: None});
        }
        for field in variant.fields.iter() {
            for attribute in &field.attrs {
                if let Some(ident) = attribute.path().get_ident() {
                    if *ident == "from" {
                        // ugly but easy
                        if attribute.to_token_stream().to_string() != "#[from]" {
                            return Err(Error::new_spanned(attribute, "expected #[from]"))
                        }
                        if field_type.is_none() {
                            field_type = Some(field.ty.clone());
                        } else {
                            break;
                        }
                    } else if *ident == "termination" {
                        return Err(Error::new_spanned(attribute, "#[termination(...)] is only allowed on enum variants."))
                    }
                }
            }
        }
        if field_type.is_some() && variant.fields.len() > 1 {
            return Err(Error::new_spanned(&variant.fields, "Only one field is allowed when using #[from]."))
        }
        Ok(FromAttribute { variant: variant.clone(), from_type: field_type})
    }))
}

pub fn check_for_unique_types(attributes: &[FromAttribute]) -> Result<(), Error> {
    let mut types = HashSet::new();
    for attribute in attributes {
        if let Some(from_type) = &attribute.from_type {
            let type_string = from_type.to_token_stream().to_string();
            if !types.contains(&type_string) {
                types.insert(type_string);
            } else {
                return Err(Error::new_spanned(&attribute.variant, "cannot use #[from] because another variant has the same type annotated with #[from]."));
            }
        }
    }
    Ok(())
}

//TODO: also validate attribute string
fn parse_attributes(variant: &Variant, attributes: &[Attribute]) -> Result<ParsedAttribute, Error> {
    let mut found_attribute = false;
    let mut parsed_attribute = ParsedAttribute {
        variant: variant.clone(),
        exit_code: None,
        message: None
    };
    for attribute in attributes {
        if let Some(ident) = attribute.path().get_ident() {
            if *ident == "from" {
                return Err(Error::new_spanned(attribute, "#[from] can only be used on fields and with TerminationFull"));
            }
            if *ident != "termination" {
                continue;
            }
            if found_attribute {
                return Err(Error::new(ident.span(), "only one #[termination(...)] attribute per enum variant is allowed"));
            }
            found_attribute = true;
        } else {
            return Err(Error::new_spanned(attribute, "identifier expected"));
        }
        attribute.parse_nested_meta(|meta| {
            if let Some(ident) = meta.path.get_ident() {
                if *ident == "msg" {
                    if parsed_attribute.message.is_none() {
                        parsed_attribute.message = Some(parse_message(&meta)?);
                        return Ok(());
                    } else {
                        return Err(Error::new(ident.span(), "Only one msg per enum variant is allowed."));
                    }
                } else if *ident == "exit_code" {
                    if parsed_attribute.exit_code.is_none() {
                        parsed_attribute.exit_code = Some(parse_exit_code(&meta)?);
                        return Ok(());
                    } else {
                        return Err(Error::new(ident.span(), "Only one exit_code per enum variant is allowed."));
                    }
                } else if *ident == "from" {
                    return Err(Error::new(ident.span(), "from can only be used on fields and with TerminationFull"));
                }
            } else {
                return Err(meta.error("identifier expected"));
            }
            Err(meta.error(format!("unrecognized attribute {}", meta.path.get_ident().unwrap())))
        })?;
    }
    Ok(parsed_attribute)
}

fn parse_exit_code(meta: &ParseNestedMeta<'_>) -> Result<ExitCodeAttribute, Error> {
    let content;
    parenthesized!(content in meta.input);
    let lit: LitInt = content.parse()?;
    let exit_code: u8 = lit.base10_parse()?;
    Ok(ExitCodeAttribute { exit_code })
}

fn parse_message(meta: &ParseNestedMeta<'_>) -> Result<MessageAttribute, Error> {
    let content;
    parenthesized!(content in meta.input);
    let lit: LitStr = content.parse()?;
    let mut args = Vec::new();
    while !content.is_empty() {
        content.parse::<Token![,]>()?;
        let arg: Expr = content.parse()?;
        args.push(arg);
    }
    Ok(MessageAttribute { format_string: lit.value(), format_string_arguments: args })
}