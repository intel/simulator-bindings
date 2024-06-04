// Copyright (C) 2024 Intel Corporation
// SPDX-License-Identifier: Apache-2.0

use darling::{ast::NestedMeta, Error, FromMeta};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, ItemFn, Meta, ReturnType, Type, Visibility};

#[derive(Debug, FromMeta)]
struct VersionedApiOpts {
    #[darling(multiple)]
    /// A list of versions the function is valid for
    pub valid: Vec<String>,
    #[darling(multiple)]
    /// A list of versions the function is invalid for
    pub invalid: Vec<String>,
}

pub trait IsResultType {
    fn is_result_type(&self) -> bool;
}

impl IsResultType for ReturnType {
    fn is_result_type(&self) -> bool {
        match self {
            ReturnType::Default => false,
            ReturnType::Type(_, ty) => match &**ty {
                Type::Path(p) => p
                    .path
                    .segments
                    .last()
                    .map(|l| l.ident == "Result")
                    .unwrap_or(false),
                _ => false,
            },
        }
    }
}

pub fn versioned_api_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(a) => a,
        Err(e) => return TokenStream::from(Error::from(e).write_errors()),
    };

    let mut input = parse_macro_input!(input as ItemFn);

    let args = match VersionedApiOpts::from_list(&attr_args) {
        Ok(a) => a,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    // Get the original ident and visibility before we change them
    let vis = input.vis.clone();
    let mut sig = input.sig.clone();
    let attrs = &input.attrs;
    let doc_attrs = attrs
        .iter()
        .filter(|a| {
            if let Meta::NameValue(attr) = &a.meta {
                if let Some(first) = attr.path.segments.first() {
                    first.ident == "doc"
                } else {
                    false
                }
            } else {
                false
            }
        })
        .collect::<Vec<_>>();

    let inner_ident = format_ident!("versioned_{}", input.sig.ident);
    input.sig.ident = inner_ident.clone();
    let input_name = input.sig.ident.to_string();
    input.vis = Visibility::Inherited;

    let ok_return = sig
        .output
        .is_result_type()
        .then_some(quote!(result))
        .unwrap_or(quote!(Ok(result)));

    sig.output = match sig.output.is_result_type().then_some(&sig.output) {
        Some(o) => o.clone(),
        None => {
            let output = match &sig.output {
                ReturnType::Default => quote!(()),
                ReturnType::Type(_, ty) => quote!(#ty),
            };

            parse_quote!(-> crate::error::Result<#output>)
        }
    };

    let maybe_ty_generics = (!&sig.generics.params.is_empty()).then_some({
        let params = &sig.generics.params;
        quote!(::<#params>)
    });

    let Some(fnargs) = sig
        .inputs
        .iter()
        .map(|i| match i {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(t) => {
                let pat = &t.pat;
                Some(quote!(#pat))
            }
        })
        .collect::<Option<Vec<_>>>()
    else {
        return Error::custom("Methods with a receiver are not supported")
            .write_errors()
            .into();
    };

    let static_name = format_ident!("VERSIONED_ENABLED_{}", input_name.to_ascii_uppercase());
    let const_valid_name = format_ident!("VERSIONED_VALID_{}", input_name.to_ascii_uppercase());
    let const_invalid_name = format_ident!("VERSIONED_INVALID_{}", input_name.to_ascii_uppercase());

    let valid = &args.valid;
    let invalid = &args.invalid;

    let wrapper = quote! {
        static #static_name: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        const #const_valid_name: &[&str] = &[#(#valid),*];
        const #const_invalid_name: &[&str] = &[#(#invalid),*];

        #[allow(non_snake_case)]
        #(#doc_attrs)*
        #vis #sig {

            if !#static_name.get_or_init(|| {
                let version = versions::Versioning::new(
                        &crate::version_base_semver()
                            .expect("Failed to get base version semver")
                    )
                    .ok_or_else(|| crate::error::Error::ParseVersion {
                        version: crate::version_base_semver()
                            .expect("Failed to get base version semver"),
                    }).expect("Failed to parse version");

                let is_valid = #const_valid_name.iter().any(|v| {
                    let requirement = versions::Requirement::new(v)
                        .ok_or_else(|| crate::error::Error::ParseRequirement {
                            version: v.to_string(),
                        }).expect("Failed to parse version");

                    requirement.matches(&version)
                });

                let is_invalid = #const_invalid_name.iter().any(|v| {
                    let requirement = versions::Requirement::new(v)
                        .ok_or_else(|| crate::error::Error::ParseRequirement {
                            version: v.to_string(),
                        }).expect("Failed to parse version");

                    requirement.matches(&version)
                });

                if is_invalid {
                    false
                } else if !is_valid {
                    false
                } else {
                    true
                }
            }) {
                return Err(crate::error::Error::UnsupportedCall {
                    version: crate::version_base_semver()?,
                    invalid: #const_invalid_name.iter().map(|s| s.to_string()).collect(),
                    valid: #const_valid_name.iter().map(|s| s.to_string()).collect(),
                });
            }

            #[allow(deprecated)]
            let result = #inner_ident #maybe_ty_generics(#(#fnargs),*);

            #ok_return
        }
    };

    quote! {
        #input
        #wrapper
    }
    .into()
}
