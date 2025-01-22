use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn generate(input: DeriveInput) -> TokenStream {
    let struct_name = input.ident;

    quote! {
        #[mikros::async_trait::async_trait]
        impl mikros::service::lifecycle::Lifecycle for #struct_name {
            async fn on_start(&mut self, _ctx: std::sync::Arc<mikros::service::context::Context>) -> mikros::errors::Result<()> {
                Ok(())
            }

            async fn on_finish(&self) -> mikros::errors::Result<()> {
                Ok(())
            }
        }
    }
}
