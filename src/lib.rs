use syn::{DeriveInput, Data};
use proc_macro::TokenStream;
use quote::quote;

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
       quote! { #name::#variant_name => 1u8.into(), }
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