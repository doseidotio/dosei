use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Plugin)]
pub fn derive_plugin(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;

  let expanded = quote! {
      // Implement the Plugin trait with the required methods
      #[async_trait::async_trait]
      impl Plugin for #name {
          fn _name(&self) -> &'static str {
              env!("CARGO_PKG_NAME")
          }

          fn _version(&self) -> &'static str {
              env!("CARGO_PKG_VERSION")
          }

          async fn _init(&self, config: PluginConfig) -> Result<(), PluginError> {
              self.init(config).await
          }

          async fn _shutdown(&self) -> Result<(), PluginError> {
              self.shutdown().await
          }

          fn _as_any(&self) -> &dyn std::any::Any {
              self
          }
      }

      #[no_mangle]
      pub extern "C" fn _plugin_create() -> Box<dyn Plugin> {
          Box::new(<#name>::default())
      }
  };

  TokenStream::from(expanded)
}
