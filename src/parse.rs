use std::collections::HashSet;

use quote::ToTokens;
use syn::{Attribute, parenthesized, LitStr, LitInt, Token, Error, meta::ParseNestedMeta, Type, Variant, Expr};

use crate::pull_up_results;

pub struct Defaults {
    pub exit_code: Option<ExitCodeAttribute>,
    pub message: Option<MessageAttribute>,
}

impl From<(Option<ExitCodeAttribute>, Option<MessageAttribute>)> for Defaults {
    fn from(value: (Option<ExitCodeAttribute>, Option<MessageAttribute>)) -> Self {
        Self { exit_code: value.0, message: value.1 }
    }
}

pub struct ParsedAttribute {
    pub variant: Variant,
    pub exit_code: Option<ExitCodeAttribute>,
    pub message: Option<MessageAttribute>,
}

pub struct MessageAttribute {
    pub format_string_lit: LitStr,
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
        let (exit_code, message) = parse_attributes(&variant.attrs)?;
        Ok(ParsedAttribute { variant: variant.clone(), exit_code, message })
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
    Ok(MessageAttribute { format_string_lit: lit, format_string_arguments: args })
}

pub fn parse_attributes(attributes: &[Attribute]) -> Result<(Option<ExitCodeAttribute>, Option<MessageAttribute>), Error> {
    let mut found_attribute = false;
    let mut exit_code = None;
    let mut message = None;
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
                    if message.is_none() {
                        message = Some(parse_message(&meta)?);
                        return Ok(());
                    } else {
                        return Err(Error::new(ident.span(), "Only one msg per enum variant is allowed."));
                    }
                } else if *ident == "exit_code" {
                    if exit_code.is_none() {
                        exit_code = Some(parse_exit_code(&meta)?);
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
            Err(meta.error(format!("unrecognized attribute {}", meta.path.get_ident().expect("should never happen because of the if before"))))
        })?;
    }
    Ok((exit_code, message))
}

//I want to keep it but it is not needed anymore
// requires syn = { version = "", features = ["full"] }
// fn validate_format_string(lit_string: &LitStr, args: &[Expr]) -> Result<(), Error> {
//     struct Argument<'a> {
//         used: bool,
//         name: String,
//         expr: &'a Expr,
//     }
//     let mut used_args = Vec::new();
//     let format_string = lit_string.value();
//     let mut named_references = HashSet::new();
//     let mut num_pos_references = 0;
//     let mut unnamed_references = HashSet::new();
//     let mut max_unnamed_reference = -1;
//     for expr in args {
//         if let Expr::Assign(ExprAssign { left, .. }) = expr {
//             if let Expr::Path(path) = &**left {
//                 if let Some(segment) = path.path.segments.first() {
//                     used_args.push(Argument { used: false, name: segment.ident.to_string(), expr })
//                 }
//             }
//         } else {
//             used_args.push(Argument { used: false, name: "".to_string(), expr })
//         }
//     }
//     let regex_unnamed = Regex::new(r#"(?:\{(?:(\d+)(?::[^\}]*)?)\})"#).expect("parsing regex");
//     for capture in regex_unnamed.captures_iter(&format_string) {
//         let i: i32 = capture.get(1).expect("the regex always produces one capture group").as_str().parse().expect("safe because of the regex");
//         if i > max_unnamed_reference {
//             max_unnamed_reference = i;
//         }
//         unnamed_references.insert(i);
//     }
//     let regex_pos = Regex::new(r#"(\{(?:(?:)?(?::[^\}]*)?)\})"#).expect("parsing regex");
//     for _ in regex_pos.captures_iter(&format_string) {
//         num_pos_references += 1;
//     }
//     let regex_named = Regex::new(r#"(?:\{(?:([^\}, :]*[a-zA-Z][^\}, :]*)(?::[^\}]*)?)\})"#).expect("parsing regex");
//     for capture in regex_named.captures_iter(&format_string) {
//         named_references.insert(capture.get(1).expect("the regex always produces one capture group").as_str());
//     }
//     if num_pos_references > used_args.len() as i32 {
//         return Err(Error::new_spanned(lit_string, "unused positional argument"));
//     }
//     if max_unnamed_reference > used_args.len() as  i32 -1 {
//         return Err(Error::new_spanned(lit_string, "out of range"));
//     }
//     for arg in used_args.iter_mut() {
//         arg.used = named_references.contains(arg.name.as_str());
//     }
//     for reference in 0..num_pos_references {
//         used_args[reference as usize].used = true;
//     }
//     for reference in unnamed_references.iter() {
//         used_args[*reference as usize].used = true;
//     }
//     if let Some(arg) = used_args.iter().find(|arg| !arg.used) {
//         return Err(Error::new_spanned(arg.expr, "unused argument"));
//     }
//     Ok(())
// }