extern crate proc_macro2;

use crate::to_client::generate_client_module;
use crate::to_server::generate_server_module;
use anyhow::{Error, Result};
use proc_macro2::TokenStream;

pub fn rpc_impl(input: syn::Item) -> Result<TokenStream> {
    let mut rpc_trait = match input {
        syn::Item::Trait(item_trait) => item_trait,
        item => {
            return Err(Error::from(syn::Error::new_spanned(
                item,
                "The #[net_rpc] custom attribute only works with trait declarations",
            )));
        }
    };
    let client_module = generate_client_module(&rpc_trait)?;
    let server_module = generate_server_module(&mut rpc_trait)?;
    let mut exports = Vec::new();
    exports.push(client_module);
    exports.push(server_module);
    Ok(quote! {
       #(#exports)*
    })
}
