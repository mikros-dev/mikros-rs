mod env;
mod lifecycle;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// A macro to add a derive functionality that allows to initialize structure
/// members with values from environment variables.
///
/// `Env` macro provides a struct member attribute `env` with the following
/// sub-attributes:
///
/// - variable: required attribute which sets the variable name that will be
///             used to set the member.
/// - default: required attribute to set the default value in case the variable
///             is not found.
/// ```ignore
/// use mikros_macros::Env;
///
/// #[derive(Env, Debug)]
/// #[env(suffix_delimiter = "_")]
/// pub struct Example {
///     #[env(variable = "VAR_NAME", default = "value")]
///     name: String,
///
///     #[env(variable = "VAR_AGE", default = "42")]
///     age: i32,
///
///     #[env(variable = "VAR_OPTIONAL", default = "None")]
///     opt: Option<String>,
///
///     // A member that won't be loaded from environment values
///     no_env_loaded: bool,
/// }
///
/// pub fn foo() {
///     let e = Example::from_env();
///     println!("{e}");
/// }
/// ```
#[proc_macro_derive(Env, attributes(env))]
pub fn derive_env_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let gen = env::generate(input);
    TokenStream::from(gen)
}

/// Lifecycle implements the `mikros::service::lifecycle::Lifecycle` trait for
/// a given structure.
#[proc_macro_derive(Lifecycle)]
pub fn derive_lifecycle_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let gen = lifecycle::generate(input);
    TokenStream::from(gen)
}
