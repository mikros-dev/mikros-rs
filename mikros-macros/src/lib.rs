mod env;
mod lifecycle;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// A macro to add a derive functionality that allows to initialize structure
/// members with values from environment variables.
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
