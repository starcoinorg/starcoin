use anyhow::Result;
use proc_macro2::TokenStream;
use syn::{parse_quote, ItemTrait, TraitItem, TraitItemMethod};

pub fn generate_server_module(rpc_trait: &mut ItemTrait) -> Result<TokenStream> {
    let delegate_methods: Vec<TokenStream> = rpc_trait
        .items
        .iter()
        .filter_map(|trait_item| {
            if let syn::TraitItem::Method(method) = trait_item {
                Some(generate_to_delegate(method))
            } else {
                None
            }
        })
        .collect();
    let mut rpc_server_trait = rpc_trait.clone();
    let io_delegate_type = quote!(network_rpc_core::delegates::IoDelegate);
    let to_delegate_body = quote! {
        let mut del = #io_delegate_type::new(self.into());
        #(#delegate_methods)*
        del
    };
    let to_delegate_method: syn::TraitItemMethod = parse_quote! {
        /// Create an `IoDelegate`, wiring rpc calls to the trait methods.
        fn to_delegate(self) -> #io_delegate_type<Self> {
            #to_delegate_body
        }
    };
    rpc_server_trait
        .items
        .push(TraitItem::Method(to_delegate_method));

    let rpc_server_module = quote! {
        /// The generated server module.
        pub mod gen_server {
            use super::*;
            use scs::{SCSCodec,from_bytes};
            #rpc_server_trait
        }
    };
    Ok(rpc_server_module)
}

pub fn generate_to_delegate(method: &TraitItemMethod) -> TokenStream {
    let param_types: Vec<_> = method
        .sig
        .inputs
        .iter()
        .cloned()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(ty) => Some(*ty.ty),
            _ => None,
        })
        .collect();

    let rpc_name = method.sig.ident.to_string();
    let method_ident = method.sig.ident.clone();
    let result = &method.sig.output;
    let method_sig = quote! { fn(&Self, #(#param_types), *) #result };
    let param_type = param_types
        .get(1)
        .expect("network rpc method need at least three argument");
    let param_type = quote!(#param_type);
    let closure = quote! {
        move | base, peer_id, params | {
            Box::pin(async move{
                let method = &(Self::#method_ident as #method_sig);
                let params = from_bytes::<#param_type>(&params)?;
                method(&base, peer_id, params).await.and_then(|r| r.encode())
            })
        }
    };
    quote! {
        del.add_method(#rpc_name, #closure);
    }
}
