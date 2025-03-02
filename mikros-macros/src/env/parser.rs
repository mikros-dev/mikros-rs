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
        is_option: bool,
    ) -> proc_macro::TokenStream {
        let expanded = if let Some(env_name) = self.env_name {
            if is_option {
                quote! {
                    #field_name: #struct_name::load_env(#env_name, &suffix, delimiter)
                        .ok()
                        .and_then(|v| v.parse().ok())
                }
            } else {
                let default_expr = if let Some(default_value) = self.default_value {
                    quote! {
                        .unwrap_or_else(|_| #default_value.to_string())
                    }
                } else {
                    quote! {}
                };

                quote! {
                    #field_name: #struct_name::load_env(#env_name, &suffix, delimiter)
                        #default_expr
                        .parse()
                        .expect("failed to parse environment variable")
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

pub(crate) fn parse_fields(input: DeriveInput) -> Result<(Vec<TokenStream>, StructAttributes), String> {
    let struct_name = &input.ident;

    // Parse struct level attributes
    let attributes = parse_struct_attributes(&input)?;

    // Parse fields
    let fields = match input.data {
        syn::Data::Struct(data) => data.fields,
        _ => return Err("Env can only be derived for structs".to_string()),
    };

    let mut field_initializers: Vec<TokenStream> = Vec::new();
    for field in &fields {
        field_initializers.push(parse_field(field, struct_name)?);
    }

    Ok((field_initializers, attributes))
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

fn parse_field(field: &Field, struct_name: &Ident) -> Result<TokenStream, String> {
    let Some(field_name) = field.ident.as_ref() else {
        return Err("expected a field name".to_string());
    };

    let mut attributes = FieldAttributes::default();
    for attr in &field.attrs {
        if attr.path().is_ident("env") {
            attributes = parse_attribute(attr, field_name)?;
        }
    }

    // Check if we're dealing with an Option<T> member
    let is_option = matches!(
        &field.ty,
        syn::Type::Path(type_path) if type_path.path.segments.len() == 1
            && type_path.path.segments[0].ident == "Option"
    );

    Ok(attributes
        .into_token_stream(field_name, struct_name, is_option)
        .into())
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
