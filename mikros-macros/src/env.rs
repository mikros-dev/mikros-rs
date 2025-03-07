mod parser;

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn generate(input: DeriveInput) -> TokenStream {
    let struct_name = input.ident.clone();
    let (field_initializers, default_checks, attributes) = match parser::parse_fields(input) {
        Ok(fields) => fields,
        Err(err) => panic!("{}", err),
    };

    let delimiter = &attributes.delimiter;
    let expanded = quote! {
        impl #struct_name {
            pub fn from_env() -> Self {
                let suffix: Option<String> = None;
                let delimiter = #delimiter;
                let use_defaults = true;

                Self {
                    #(#field_initializers),*
                }
            }

            pub fn from_env_with_suffix(suffix: &str, use_defaults: bool) -> Self {
                let suffix = Some(suffix.to_string());
                let delimiter = #delimiter;

                Self {
                    #(#field_initializers),*
                }
            }

            fn load_env(name: &str, suffix: &Option<String>, delimiter: &str) -> Result<String, std::env::VarError> {
                let key = match suffix {
                    None => name.to_string(),
                    Some(s) => format!("{}{}{}", name, delimiter, s)
                };

                std::env::var(key)
            }

            pub fn check_defaults(&self) -> Vec<(&'static str, bool)> {
                vec![
                    #(#default_checks),*
                ]
            }
        }
    };

    expanded
}
