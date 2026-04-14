extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, Attribute, Expr, ExprLit, FnArg, GenericArgument, ItemTrait, Lit, Meta, Pat,
    PathArguments, ReturnType, TraitItem, TraitItemFn, Type,
};

#[proc_macro_attribute]
pub fn rpc_schema(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item_trait = parse_macro_input!(input as ItemTrait);
    match expand_rpc_schema(item_trait) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_rpc_schema(item_trait: ItemTrait) -> syn::Result<proc_macro2::TokenStream> {
    let trait_ident = item_trait.ident.clone();
    let module_ident = format_ident!(
        "__openrpc_schema_{}",
        trait_ident.to_string().to_lowercase()
    );
    let (namespace, separator) = parse_trait_rpc_namespace(&item_trait.attrs)?;

    let methods = item_trait
        .items
        .iter()
        .filter_map(|item| match item {
            TraitItem::Fn(method) => Some(parse_method_schema(method, &namespace, &separator)),
            _ => None,
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let schema_methods = methods.iter().map(|method| {
        let method_name = &method.name;
        let return_ty = &method.return_ty;
        let params = method.params.iter().map(|(name, ty)| {
            quote! {
                params.push(ContentDescriptorOrReference::new_content_descriptor::<#ty>(
                    stringify!(#name).to_string(),
                    None,
                ));
            }
        });

        quote! {
            {
                let mut method_object = MethodObject::new(#method_name.to_string(), None);
                method_object.params = {
                    let mut params = Vec::new();
                    #(#params)*
                    params
                };
                method_object.result = ContentDescriptorOrReference::new_content_descriptor::<#return_ty>(
                    stringify!(#return_ty).to_string(),
                    None,
                );
                document.add_object_method(method_object);
            }
        }
    });

    Ok(quote! {
        #item_trait

        mod #module_ident {
            use super::*;
            use openrpc_schema::document::*;

            pub fn gen_schema() -> OpenrpcDocument {
                let mut document = OpenrpcDocument::default();
                #(#schema_methods)*
                document
            }
        }

        pub use self::#module_ident::gen_schema;
    })
}

struct MethodSchema {
    name: String,
    params: Vec<(syn::Ident, Type)>,
    return_ty: Type,
}

fn parse_method_schema(
    method: &TraitItemFn,
    namespace: &Option<String>,
    separator: &str,
) -> syn::Result<MethodSchema> {
    let method_name = parse_method_name(&method.attrs)?.ok_or_else(|| {
        syn::Error::new_spanned(method, "Missing #[method(name = \"...\")] attribute")
    })?;

    let name = match namespace {
        Some(namespace) => format!("{namespace}{separator}{method_name}"),
        None => method_name,
    };

    let params = method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => Some((&*pat_type.pat, &*pat_type.ty)),
        })
        .map(|(pat, ty)| {
            let ident = match pat {
                Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                _ => {
                    return Err(syn::Error::new_spanned(
                        pat,
                        "RPC schema generation requires identifier parameters",
                    ));
                }
            };
            Ok((ident, ty.clone()))
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let return_ty = parse_return_type(&method.sig.output)?;

    Ok(MethodSchema {
        name,
        params,
        return_ty,
    })
}

fn parse_trait_rpc_namespace(attrs: &[Attribute]) -> syn::Result<(Option<String>, String)> {
    let rpc_attr = attrs.iter().find(|attr| attr.path().is_ident("rpc"));
    let Some(rpc_attr) = rpc_attr else {
        return Ok((None, ".".to_string()));
    };

    let metas = parse_attr_metas(rpc_attr)?;
    let mut namespace = None;
    let mut separator = None;
    for meta in &metas {
        if namespace.is_none() {
            namespace = parse_meta_string(meta, "namespace")?;
        }
        if separator.is_none() {
            separator = parse_meta_string(meta, "namespace_separator")?;
        }
    }

    Ok((namespace, separator.unwrap_or_else(|| ".".to_string())))
}

fn parse_method_name(attrs: &[Attribute]) -> syn::Result<Option<String>> {
    let method_attr = attrs.iter().find(|attr| attr.path().is_ident("method"));
    let Some(method_attr) = method_attr else {
        return Ok(None);
    };

    let metas = parse_attr_metas(method_attr)?;
    metas
        .into_iter()
        .find_map(|meta| match parse_meta_string(&meta, "name") {
            Ok(Some(name)) => Some(Ok(name)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        })
        .transpose()
}

fn parse_attr_metas(attr: &Attribute) -> syn::Result<Punctuated<Meta, syn::Token![,]>> {
    attr.parse_args_with(Punctuated::<Meta, syn::Token![,]>::parse_terminated)
}

fn parse_meta_string(meta: &Meta, key: &str) -> syn::Result<Option<String>> {
    match meta {
        Meta::NameValue(name_value) if name_value.path.is_ident(key) => match &name_value.value {
            Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str),
                ..
            }) => Ok(Some(lit_str.value())),
            other => Err(syn::Error::new_spanned(
                other,
                format!("Expected string literal for `{key}`"),
            )),
        },
        _ => Ok(None),
    }
}

fn parse_return_type(output: &ReturnType) -> syn::Result<Type> {
    match output {
        ReturnType::Default => Err(syn::Error::new_spanned(
            output,
            "RPC schema generation requires a return type",
        )),
        ReturnType::Type(_, ty) => {
            Ok(extract_result_type(ty.as_ref()).unwrap_or_else(|| ty.as_ref().clone()))
        }
    }
}

fn extract_result_type(ty: &Type) -> Option<Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let last_segment = type_path.path.segments.last()?;
    if !last_segment.ident.to_string().ends_with("Result") {
        return None;
    }
    first_type_argument(&last_segment.arguments)
}

fn first_type_argument(args: &PathArguments) -> Option<Type> {
    let PathArguments::AngleBracketed(args) = args else {
        return None;
    };
    args.args.iter().find_map(|arg| match arg {
        GenericArgument::Type(ty) => Some(ty.clone()),
        _ => None,
    })
}
