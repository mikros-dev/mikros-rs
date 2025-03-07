use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, Field, Ident, Lit};

#[derive(Default)]
struct FieldAttributes {
    env_name: Option<String>,
    default_value: Option<String>,
}

impl FieldAttributes {
    fn into_token_stream(
        self,
        field_name: &Ident,
        struct_name: &Ident,
        field_type: &syn::Type,
    ) -> proc_macro::TokenStream {
        // Check if we're dealing with an Option<T> member
        let is_option = matches!(
            &field_type,
            syn::Type::Path(type_path) if type_path.path.segments.len() == 1
                && type_path.path.segments[0].ident == "Option"
        );

        let expanded = if let Some(env_name) = self.env_name {
            let default_value = self.default_value.unwrap_or_default();

            if is_option {
                quote! {
                    #field_name: {
                        let env_value = #struct_name::load_env(#env_name, &suffix, delimiter);
                        match env_value {
                            Ok(v) if v.is_empty() || v == "None" => None,
                            Ok(v) => v.parse().ok(),
                            Err(_) if use_defaults => if #default_value == "None" {
                                None
                            } else {
                                #default_value.parse().ok()
                            },
                            Err(_) => None,
                        }
                    }
                }
            } else {
                quote! {
                    #field_name: {
                        let env_value = #struct_name::load_env(#env_name, &suffix, delimiter);
                        match env_value {
                            Ok(v) => v.parse().expect("failed to parse environment variable"),
                            Err(_) if use_defaults => #default_value.parse().expect("failed to parse variable default value"),
                            Err(_) => Default::default(),
                        }
                    }
                }
            }
        } else {
            // Members without attribute will be initialized with their
            // default values (do not mix with 'default' attribute).
            quote! {
                #field_name: Default::default()
            }
        };

        proc_macro::TokenStream::from(expanded)
    }
}

pub(crate) struct StructAttributes {
    pub(crate) delimiter: String,
}

impl Default for StructAttributes {
    fn default() -> Self {
        Self {
            delimiter: "_".to_string(),
        }
    }
}

pub(crate) fn parse_fields(input: DeriveInput) -> Result<(Vec<TokenStream>, Vec<TokenStream>, StructAttributes), String> {
    let struct_name = &input.ident;

    // Parse struct level attributes
    let attributes = parse_struct_attributes(&input)?;

    // Parse fields
    let fields = match input.data {
        syn::Data::Struct(data) => data.fields,
        _ => return Err("Env can only be derived for structs".to_string()),
    };

    let mut field_initializers: Vec<TokenStream> = Vec::new();
    let mut default_checks: Vec<TokenStream> = Vec::new();

    for field in &fields {
        let (initializer, check) = parse_field(field, struct_name)?;

        field_initializers.push(initializer);
        default_checks.push(check);
    }

    Ok((field_initializers, default_checks, attributes))
}

fn parse_struct_attributes(input: &DeriveInput) -> Result<StructAttributes, String> {
    let mut attributes = StructAttributes::default();

    for attr in &input.attrs {
        if attr.path().is_ident("env") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("suffix_delimiter") {
                    if let Ok(Lit::Str(v)) = meta.value()?.parse::<Lit>() {
                        attributes.delimiter = v.value();
                    }
                }

                Ok(())
            });
        }
    }

    Ok(attributes)
}

fn parse_field(field: &Field, struct_name: &Ident) -> Result<(TokenStream, TokenStream), String> {
    let Some(field_name) = field.ident.as_ref() else {
        return Err("expected a field name".to_string());
    };

    let mut attributes = FieldAttributes::default();
    for attr in &field.attrs {
        if attr.path().is_ident("env") {
            attributes = parse_attribute(attr, field_name)?;
        }
    }

    let default_check = generate_default_check(field_name, &attributes.default_value, &field.ty);
    let initializer = attributes
        .into_token_stream(field_name, struct_name, &field.ty)
        .into();


    Ok((initializer, default_check))
}

fn parse_attribute(attr: &Attribute, field_name: &Ident) -> Result<FieldAttributes, String> {
    let mut env_name = None;
    let mut default_value = None;

    let result = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("variable") {
            if let Ok(Lit::Str(v)) = meta.value()?.parse::<Lit>() {
                env_name = Some(v.value());
            }
        } else if meta.path.is_ident("default") {
            if let Ok(Lit::Str(v)) = meta.value()?.parse::<Lit>() {
                default_value = Some(v.value());
            }
        }

        Ok(())
    });

    if result.is_err() {
        return Err(format!(
            "failed to parse 'env' attribute on field '{field_name}'"
        ));
    }

    if env_name.is_some() && default_value.is_none() {
        return Err(format!(
            "'default' attribute is mandatory for env field '{field_name}'"
        ));
    }

    Ok(FieldAttributes {
        env_name,
        default_value,
    })
}

fn generate_default_check(field_name: &Ident, default_value: &Option<String>, field_type: &syn::Type) -> TokenStream {
    let is_option = match field_type {
        syn::Type::Path(type_path) if type_path.path.segments.len() == 1 => {
            type_path.path.segments[0].ident == "Option"
        }
        _ => false,
    };

    let check = if let Some(default) = default_value {
        if is_option {
            if default == "None" {
                quote! { self.#field_name.is_none() }
            } else {
                quote! { self.#field_name.as_ref().map(|v| v.to_string()) == Some(#default.to_string()) }
            }
        } else {
            quote! { self.#field_name.to_string() == #default }
        }
    } else {
        quote! { false } // No default specified
    };

    quote! {
        (stringify!(#field_name), #check)
    }
}