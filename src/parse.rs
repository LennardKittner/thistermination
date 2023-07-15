use quote::spanned::Spanned;
use syn::{Attribute, parenthesized, LitStr, LitInt, Token, Error, meta::ParseNestedMeta, Type, Variant};
use proc_macro2::{TokenStream as TokenStream2, Span};

pub struct ParsedAttribute {
    pub variant: Variant,
    pub exit_code: Option<ExitCodeAttribute>,
    pub message: Option<MessageAttribute>,
}

pub struct MessageAttribute {
    pub format_string_span: Span,
    pub format_string: String,
    pub format_string_arguments_span: Option<Span>,
    pub format_string_arguments: Option<TokenStream2>,
}

pub struct ExitCodeAttribute {
    pub span: Span,
    pub exit_code: u8,
}

pub struct FromAttribute {
    pub variant: Variant,
    pub from_type: Option<Type>,
}

fn pull_up_results<T, E, I>(results: I) -> Result<Vec<T>, E> where I: IntoIterator<Item = Result<T, E>> {
    let mut items =  Vec::new();
    for result in results {
        match result {
            Ok(item) => items.push(item),
            Err(error) => return Err(error),
        }
    }
    Ok(items)
}

pub fn parse_helper_attributes<'a>(variants: impl Iterator<Item = &'a Variant>) -> Result<Vec<ParsedAttribute>, Error> {
    pull_up_results(variants.map(|variant| {
        parse_attributes(variant, &variant.attrs)
    }))
}

//TODO: forbid something like #[from(asdf)]
pub fn parse_from_attribute<'a>(variants: impl Iterator<Item = &'a Variant>) -> Result<Vec<FromAttribute>, Error> {
    pull_up_results(variants.map(|variant| {
        let mut field_type = None;
        if variant.fields.is_empty() {
            return Ok(FromAttribute { variant: variant.clone(), from_type: None});
        }
        for field in variant.fields.iter() {
            for attribute in &field.attrs {
                if attribute.path().is_ident("from") {
                    if field_type.is_none() {
                        field_type = Some(field.ty.clone());
                    } else {
                        break;
                    }
                }
            }
        }
        if field_type.is_some() && variant.fields.len() > 1 {
            return Err(Error::new(variant.fields.__span(), "Only one field is allowed when using `#[from]`."))
        }
        Ok(FromAttribute { variant: variant.clone(), from_type: field_type})
    }))
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
            if ident.to_string() != "termination" {
                continue;
            }
            if found_attribute {
                return Err(Error::new(ident.span(), "only one `#[termination(...)]` attribute per enum variant is allowed"));
            }
            found_attribute = true;
        } else {
            return Err(Error::new(attribute.path().__span(), "identifier expected"));
        }
        attribute.parse_nested_meta(|meta| {
            if let Some(ident) = meta.path.get_ident() {
                if ident.to_string() == "msg" {
                    if parsed_attribute.message.is_none() {
                        parsed_attribute.message = Some(parse_message(&meta)?);
                        return Ok(());
                    } else {
                        return Err(Error::new(ident.span(), "Only one `msg` per enum variant is allowed."));
                    }
                } else if ident.to_string() == "exit_code" {
                    if parsed_attribute.exit_code.is_none() {
                        parsed_attribute.exit_code = Some(parse_exit_code(&meta)?);
                        return Ok(());
                    } else {
                        return Err(Error::new(ident.span(), "Only one `exit_code` per enum variant is allowed."));
                    }
                }
            } else {
                return Err(meta.error(format!("identifier expected")));
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
    return Ok(ExitCodeAttribute { span: lit.span(), exit_code });
}

fn parse_message(meta: &ParseNestedMeta<'_>) -> Result<MessageAttribute, Error> {
    let content;
    parenthesized!(content in meta.input);
    let lit: LitStr = content.parse()?;
    let comma_present = content.peek(Token![,]);
    let mut args = None;
    let mut args_span = None;
    if comma_present {
        content.parse::<Token![,]>()?; 
        let rest: TokenStream2 = content.parse()?;
        args = Some(rest);
        args_span = Some(args.__span());
    } else if !content.is_empty() {
        let rest: TokenStream2 = content.parse()?;
        return Err(Error::new(rest.__span(), "expected \",\""));
    }
    return Ok(MessageAttribute { format_string_span: lit.span(), format_string: lit.value(), format_string_arguments_span: args_span, format_string_arguments: args });
}