mod env;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Env, attributes(env))]
pub fn derive_env_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let gen = env::generate(input);
    TokenStream::from(gen)
}
