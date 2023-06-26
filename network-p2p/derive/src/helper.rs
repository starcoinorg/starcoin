// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate proc_macro2;

use anyhow::Error;
use syn::punctuated::Punctuated;

pub fn compute_returns(method: &syn::TraitItemMethod) -> anyhow::Result<syn::Type> {
    if let Some(returns) = try_infer_returns(&method.sig.output) {
        return Ok(returns);
    }
    let span = method.attrs[0].pound_token.spans[0];
    let msg = "Missing returns attribute.";
    Err(Error::from(syn::Error::new(span, msg)))
}

pub fn try_infer_help(ty: &syn::Type, match_ident: String) -> Option<syn::Type> {
    match ty {
        syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) => {
            let syn::PathSegment {
                ident, arguments, ..
            } = &segments[0];
            if ident.to_string().eq(&match_ident) {
                get_first_type_argument(arguments)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn try_infer_returns(output: &syn::ReturnType) -> Option<syn::Type> {
    match output {
        syn::ReturnType::Type(_, ty) => {
            let t = try_infer_help(ty, "BoxFuture".into())
                .expect("Network rpc method Must has a BoxFuture return type");
            try_infer_help(&t, "Result".into())
        }
        _ => None,
    }
}

fn get_first_type_argument(args: &syn::PathArguments) -> Option<syn::Type> {
    match args {
        syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            args, ..
        }) => {
            if !args.is_empty() {
                match &args[0] {
                    syn::GenericArgument::Type(ty) => Some(ty.to_owned()),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn compute_args(method: &syn::TraitItemMethod) -> Punctuated<syn::FnArg, syn::token::Comma> {
    let mut args = Punctuated::new();
    for arg in &method.sig.inputs {
        let ty = match arg {
            syn::FnArg::Typed(syn::PatType { ty, .. }) => ty,
            _ => continue,
        };
        let segments = match &**ty {
            syn::Type::Path(syn::TypePath {
                path: syn::Path { ref segments, .. },
                ..
            }) => segments,
            _ => continue,
        };
        let syn::PathSegment { ident, .. } = &segments[0];
        let ident = ident;
        if *ident == "Self" {
            continue;
        }
        args.push(arg.to_owned());
    }
    args
}

pub fn compute_arg_identifiers(
    args: &Punctuated<syn::FnArg, syn::token::Comma>,
) -> anyhow::Result<Vec<&syn::Ident>> {
    let mut arg_names = vec![];
    for arg in args {
        let pat = match arg {
            syn::FnArg::Typed(syn::PatType { pat, .. }) => pat,
            _ => continue,
        };
        let ident = match **pat {
            syn::Pat::Ident(syn::PatIdent { ref ident, .. }) => ident,
            syn::Pat::Wild(ref wild) => {
                let span = wild.underscore_token.spans[0];
                let msg = "No wildcard patterns allowed in rpc trait.";
                return Err(Error::from(syn::Error::new(span, msg)));
            }
            _ => continue,
        };
        arg_names.push(ident);
    }
    Ok(arg_names)
}
