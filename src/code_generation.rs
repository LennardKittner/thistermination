use regex::Regex;
use syn::{FieldsNamed, FieldsUnnamed};
use quote::quote;
use proc_macro2::{TokenStream as TokenStream2, Ident, Span};

use crate::parse::{MessageAttribute, ExitCodeAttribute, FromAttribute, ParsedAttribute};

pub fn generate_debug_trait(name: &Ident, attributes: &[ParsedAttribute]) -> TokenStream2 {
    let debug_impl = attributes.iter().map(|attribute| {
        let variant_name = &attribute.variant.ident;
        match &attribute.variant.fields {
            syn::Fields::Named(f) => message_impl_named(name, variant_name, f, &attribute.message),
            syn::Fields::Unnamed(f) => message_impl_unnamed(name, variant_name, f, &attribute.message),
            syn::Fields::Unit => message_impl_unit(name, variant_name, &attribute.message),
        }
    });
    quote! {
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#debug_impl)*
                }
            }
        }
    }
}

pub fn generate_termination_trait(name: &Ident, attributes: &[ParsedAttribute]) -> TokenStream2 {
    let termination_impl = attributes.iter().map(|attribute| {
    let variant_name = &attribute.variant.ident;
    match &attribute.variant.fields {
            syn::Fields::Named(f) => termination_impl_named(name, variant_name, f, &attribute.exit_code),
            syn::Fields::Unnamed(f) => termination_impl_unnamed(name, variant_name, f, &attribute.exit_code),
            syn::Fields::Unit => termination_impl_unit(name, variant_name, &attribute.exit_code),
        }
    });
    quote! {
        impl std::process::Termination for #name {
            fn report(self) -> std::process::ExitCode {
                match self {
                    #(#termination_impl)*
                }
            }
        }
    }
}

pub fn generate_display_trait(name: &Ident, attributes: &[ParsedAttribute]) -> TokenStream2 {
    let display_impl = attributes.iter().map(|attribute| {
        let variant_name = &attribute.variant.ident;
        match &attribute.variant.fields {
            syn::Fields::Named(f) => message_impl_named(name, variant_name, f, &attribute.message),
            syn::Fields::Unnamed(f) => message_impl_unnamed(name, variant_name, f, &attribute.message),
            syn::Fields::Unit => message_impl_unit(name, variant_name, &attribute.message),
        }
    });
    quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#display_impl)*
                }
            }
        }
    }
}

pub fn generate_error_trait(name: &Ident) -> TokenStream2 {
    quote! {
        impl std::error::Error for #name { }
    }
}

pub fn generate_from_traits(name: &Ident, attributes: &[FromAttribute]) -> TokenStream2 {
     let from_impl = attributes.iter().map(|attribute| {
        let variant_name = &attribute.variant.ident;
        if let Some(f_type) = &attribute.from_type {
            let fn_impl = match &attribute.variant.fields {
                syn::Fields::Named(fields) => {
                    let field_name = fields.named.first().unwrap().ident.as_ref().unwrap();
                    quote! { #name::#variant_name { #field_name: value } }
                }
                syn::Fields::Unnamed(_) => quote! { #name::#variant_name(value) },
                syn::Fields::Unit => panic!("should never happen"),
            };
            quote! {
                impl std::convert::From<#f_type> for #name {
                    fn from(value: #f_type) -> Self {
                        #fn_impl
                    }
                }
            }
        } else {
            quote!{}
        }
    });
    quote! {
        #(#from_impl)*
    }
}

fn termination_impl_named(name: &Ident, variant_name: &Ident, fields: &FieldsNamed, exit_code: &Option<ExitCodeAttribute>) -> TokenStream2 {
    let field_names = fields.named.iter().map(|field| &field.ident);
    if let Some(ExitCodeAttribute { exit_code, .. }) = exit_code {
        quote! { #name::#variant_name { #(ref #field_names),* } => #exit_code.into(), }
    } else { 
        quote! { #name::#variant_name { #(ref #field_names),* } => std::process::ExitCode::FAILURE, }
    }
}

fn termination_impl_unnamed(name: &Ident, variant_name: &Ident, fields: &FieldsUnnamed, exit_code: &Option<ExitCodeAttribute>) -> TokenStream2 {
    let field_names = fields.unnamed.iter().enumerate().map(|(i, _)| {
        syn::Ident::new(&format!("__{}", i), Span::call_site())
    });
    if let Some(ExitCodeAttribute { exit_code, .. }) = exit_code {
        quote! { #name::#variant_name( #(#field_names),* ) => #exit_code.into(), }
    } else { 
        quote! { #name::#variant_name( #(#field_names),* ) => std::process::ExitCode::FAILURE, }
    }
}

fn termination_impl_unit(name: &Ident, variant_name: &Ident, exit_code: &Option<ExitCodeAttribute>) -> TokenStream2 {
    if let Some(ExitCodeAttribute { exit_code, .. }) = exit_code {
        quote! { #name::#variant_name => #exit_code.into(), }
    } else { 
        quote! { #name::#variant_name => std::process::ExitCode::FAILURE, }
    }
}

fn message_impl_named(name: &Ident, variant_name: &Ident, fields: &FieldsNamed, message: &Option<MessageAttribute>) -> TokenStream2 {
    let field_names = fields.named.iter().map(|field| &field.ident);
    if let Some(MessageAttribute { format_string, format_string_arguments, .. }) = message {
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

fn message_impl_unnamed(name: &Ident, variant_name: &Ident, fields: &FieldsUnnamed, message: &Option<MessageAttribute>) -> TokenStream2 {
    let field_names = fields.unnamed.iter().enumerate().map(|(i, _)| {
        syn::Ident::new(&format!("__{}", i), Span::call_site())
    });
    if let Some(MessageAttribute { format_string, format_string_arguments, .. }) = message {
        let format_string = get_formatted_string_with_fields(format_string, "__");
        quote! { #name::#variant_name(#(#field_names),*) => write!(f, #format_string, #format_string_arguments), }
    } else {
        quote! { #name::#variant_name(#(#field_names),*) => write!(f, "{}", self), }
    }
}

fn message_impl_unit(name: &Ident, variant_name: &Ident, message: &Option<MessageAttribute>) -> TokenStream2 {
    if let Some(MessageAttribute { format_string, format_string_arguments, .. }) = message {
        quote! { #name::#variant_name => write!(f, #format_string, #format_string_arguments), }
    } else {
        quote! { #name::#variant_name => write!(f, "{}", self), }
    }
}