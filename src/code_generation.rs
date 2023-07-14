use regex::Regex;
use syn::{FieldsNamed, FieldsUnnamed, punctuated::Punctuated, Variant, token::Comma};
use quote::quote;
use proc_macro2::{TokenStream as TokenStream2, Ident, Span};

use crate::parse::{parse_helper_attribute, Message, ParsedAttribute, HelperAttribute, parse_from_attribute};

//TODO: make the interface better
pub fn generate_debug_trait(name: &Ident, variants: &Punctuated<Variant, Comma>, allowed_attributes: &[HelperAttribute], forbidden_attributes: &[HelperAttribute]) -> TokenStream2 {
    let debug_impl = variants.iter().map(|variant| {
        let variant_name: &syn::Ident = &variant.ident;
        let variant_attributes = parse_helper_attribute(&variant.attrs, allowed_attributes, forbidden_attributes).unwrap();
        match &variant.fields {
            syn::Fields::Named(f) => message_impl_named(name, variant_name, f, &variant_attributes),
            syn::Fields::Unnamed(f) => message_impl_unnamed(name, variant_name, f, &variant_attributes),
            syn::Fields::Unit => message_impl_unit(name, variant_name, &variant_attributes),
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

pub fn generate_termination_trait(name: &Ident, variants: &Punctuated<Variant, Comma>, allowed_attributes: &[HelperAttribute], forbidden_attributes: &[HelperAttribute]) -> TokenStream2 {
    let termination_impl = variants.iter().map(|variant| {
    let variant_name = &variant.ident;
    let variant_attributes = parse_helper_attribute(&variant.attrs, allowed_attributes, forbidden_attributes).unwrap();
    match &variant.fields {
            syn::Fields::Named(f) => termination_impl_named(name, variant_name, f, &variant_attributes),
            syn::Fields::Unnamed(f) => termination_impl_unnamed(name, variant_name, f, &variant_attributes),
            syn::Fields::Unit => termination_impl_unit(name, variant_name, &variant_attributes),
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

pub fn generate_display_trait(name: &Ident, variants: &Punctuated<Variant, Comma>, allowed_attributes: &[HelperAttribute], forbidden_attributes: &[HelperAttribute]) -> TokenStream2 {
    let display_impl = variants.iter().map(|variant| {
        let variant_name: &syn::Ident = &variant.ident;
        let variant_attributes = parse_helper_attribute(&variant.attrs, allowed_attributes, forbidden_attributes).unwrap();
        if variant_attributes.message.is_none() {
            panic!("missing #[termination(msg(\"...\"))] attribute")
        }
        match &variant.fields {
            syn::Fields::Named(f) => message_impl_named(name, variant_name, f, &variant_attributes),
            syn::Fields::Unnamed(f) => message_impl_unnamed(name, variant_name, f, &variant_attributes),
            syn::Fields::Unit => message_impl_unit(name, variant_name, &variant_attributes),
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

pub fn generate_from_traits(name: &Ident, variants: &Punctuated<Variant, Comma>) -> TokenStream2 {
     let from_impl = variants.iter().map(|variant| {
        let variant_name: &syn::Ident = &variant.ident;
        let field_type = parse_from_attribute(&variant.fields).unwrap();
        if let Some(f_type) = field_type {
            let fn_impl = match &variant.fields {
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
        syn::Ident::new(&format!("__{}", i), Span::call_site())
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

fn message_impl_named(name: &Ident, variant_name: &Ident, fields: &FieldsNamed, attribute: &ParsedAttribute) -> TokenStream2 {
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

fn message_impl_unnamed(name: &Ident, variant_name: &Ident, fields: &FieldsUnnamed, attribute: &ParsedAttribute) -> TokenStream2 {
    let field_names = fields.unnamed.iter().enumerate().map(|(i, _)| {
        syn::Ident::new(&format!("__{}", i), Span::call_site())
    });
    if let Some(Message { format_string, format_string_arguments }) = &attribute.message {
        let format_string = get_formatted_string_with_fields(format_string, "__");
        quote! { #name::#variant_name(#(#field_names),*) => write!(f, #format_string, #format_string_arguments), }
    } else {
        quote! { #name::#variant_name(#(#field_names),*) => write!(f, "{}", self), }
    }
}

fn message_impl_unit(name: &Ident, variant_name: &Ident, attribute: &ParsedAttribute) -> TokenStream2 {
    if let Some(Message { format_string, format_string_arguments }) = &attribute.message {
        quote! { #name::#variant_name => write!(f, #format_string, #format_string_arguments), }
    } else {
        quote! { #name::#variant_name => write!(f, "{}", self), }
    }
}