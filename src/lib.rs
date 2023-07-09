use core::panic;
use regex::Regex;
use syn::{DeriveInput, Data, Attribute, parenthesized, LitStr, LitInt, FieldsNamed, FieldsUnnamed, Meta, Type, Token, parse::Parse};
use proc_macro::TokenStream;
use quote::quote;
use proc_macro2::{TokenStream as TokenStream2, Ident, Span};

struct ParsedAttribute {
    exit_code: Option<u8>,
    message: Option<Message>,
}

struct Message {
    format_string: String,
    format_string_arguments: Option<TokenStream2>,
}

#[proc_macro_derive(Termination, attributes(termination))]
pub fn derive_termination(steam: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(steam).unwrap();
    let name = &ast.ident;
    let variants = match ast.data {
        Data::Enum(ref data) => &data.variants,
        _ => panic!("thistermination can currently only be derived on enums"),
    };
    let termination_impl = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_attributes = parse_helper_attribute_values(&variant.attrs).unwrap();
        match &variant.fields {
            syn::Fields::Named(f) => termination_impl_named(name, variant_name, f, &variant_attributes),
            syn::Fields::Unnamed(f) => termination_impl_unnamed(name, variant_name, f, &variant_attributes),
            syn::Fields::Unit => termination_impl_unit(name, variant_name, &variant_attributes),
        }
    });
    //TODO: maybe another macro that replicates derive Debug
    let debug_impl = variants.iter().map(|variant| {
        let variant_name: &syn::Ident = &variant.ident;
        let variant_attributes = parse_helper_attribute_values(&variant.attrs).unwrap();
        match &variant.fields {
            syn::Fields::Named(f) => debug_impl_named(name, variant_name, f, &variant_attributes),
            syn::Fields::Unnamed(f) => debug_impl_unnamed(name, variant_name, f, &variant_attributes),
            syn::Fields::Unit => debug_impl_unit(name, variant_name, &variant_attributes),
        }
    });

    let generate = quote! {
        impl std::process::Termination for #name {
            fn report(self) -> std::process::ExitCode {
                match self {
                    #(#termination_impl)*
                }
            }
        }
  
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#debug_impl)*
                }
            }
        }
    };

    generate.into()
}

fn termination_impl_named(name: &Ident, variant_name: &Ident, fields: &FieldsNamed, attribute: &ParsedAttribute) -> TokenStream2 {
    let field_names = fields.named.iter().map(|field| &field.ident);
    if let Some(exit_code) = attribute.exit_code {
        quote! { #name::#variant_name { #(ref #field_names),* } => #exit_code.into(), }
    } else { 
        quote! { #name::#variant_name { #(ref #field_names),* } => std::process::ExitCode::FAILURE, }
    }
}

fn termination_impl_unnamed(name: &Ident, variant_name: &Ident, fields: &FieldsUnnamed, attribute: &ParsedAttribute) -> TokenStream2 {
    let field_names = fields.unnamed.iter().enumerate().map(|(i, _)| {
        syn::Ident::new(&format!("field_{}", i), Span::call_site())
    });
    if let Some(exit_code) = attribute.exit_code {
        quote! { #name::#variant_name( #(#field_names),* ) => #exit_code.into(), }
    } else { 
        quote! { #name::#variant_name( #(#field_names),* ) => std::process::ExitCode::FAILURE, }
    }
}

fn termination_impl_unit(name: &Ident, variant_name: &Ident, attribute: &ParsedAttribute) -> TokenStream2 {
    if let Some(exit_code) = attribute.exit_code {
        quote! { #name::#variant_name => #exit_code.into(), }
    } else { 
        quote! { #name::#variant_name => std::process::ExitCode::FAILURE, }
    }
}

fn debug_impl_named(name: &Ident, variant_name: &Ident, fields: &FieldsNamed, attribute: &ParsedAttribute) -> TokenStream2 {
    let field_names = fields.named.iter().map(|field| &field.ident);
    if let Some(Message { format_string, format_string_arguments }) = &attribute.message {
        quote! { #name::#variant_name { #(ref #field_names),* } => write!(f, #format_string, #format_string_arguments), }
    } else { 
        quote! { #name::#variant_name { #(ref #field_names),* } => write!(f, "{}", self), }
    }
}

fn get_formatted_string_with_fields(msg: &str, prefix: &str) -> String {
    let regex = Regex::new(r#"(?:\{(?:(\d+)(?::[^\}]+)?)\})"#).unwrap();
    regex.replace_all(msg, |caps: &regex::Captures| {
        let field = caps.get(1).unwrap().as_str();
        format!("{{{}{}}}",prefix, field)
    }).to_string()
 }

fn debug_impl_unnamed(name: &Ident, variant_name: &Ident, fields: &FieldsUnnamed, attribute: &ParsedAttribute) -> TokenStream2 {
    let field_names = fields.unnamed.iter().enumerate().map(|(i, _)| {
        syn::Ident::new(&format!("field_{}", i), Span::call_site())
    });
    if let Some(Message { format_string, format_string_arguments }) = &attribute.message {
        let format_string = get_formatted_string_with_fields(format_string, "field_");
        quote! { #name::#variant_name(#(#field_names),*) => write!(f, #format_string, #format_string_arguments), }
    } else {
        quote! { #name::#variant_name(#(#field_names),*) => write!(f, "{}", self), }
    }
}

fn debug_impl_unit(name: &Ident, variant_name: &Ident, attribute: &ParsedAttribute) -> TokenStream2 {
    if let Some(Message { format_string, format_string_arguments }) = &attribute.message {
        quote! { #name::#variant_name => write!(f, #format_string, #format_string_arguments), }
    } else {
        quote! { #name::#variant_name => write!(f, "{}", self), }
    }
}

//TODO: better error handling e.g. show error at specific attribute
fn parse_helper_attribute_values(attributes: &[Attribute]) -> Result<ParsedAttribute, String> {
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
                //TODO: why is there no problem if the there is no rest?
                let rest: TokenStream2 = content.parse()?;
                parsed_attribute.message = Some(Message { format_string: lit.value(), format_string_arguments: Some(rest) });
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